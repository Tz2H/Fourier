use image::{GrayImage, Luma};
use imageproc::{
    contours::{BorderType, find_contours},
    contrast::otsu_level,
};

use super::SourceImage;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProcessingSettings {
    pub automatic: bool,
    pub threshold: u8,
    pub invert: bool,
    pub denoise: bool,
    pub include_details: bool,
    pub edge_threshold: f32,
    pub min_stroke_length: f64,
}

impl Default for ProcessingSettings {
    fn default() -> Self {
        Self {
            automatic: true,
            threshold: 220,
            invert: false,
            denoise: true,
            include_details: true,
            edge_threshold: 24.0,
            min_stroke_length: 6.0,
        }
    }
}

pub struct Contour {
    /// 像素坐标，按边界顺序排列，y 向下。
    pub points: Vec<[f64; 2]>,
    pub area: f64,
    pub touches_edge: bool,
    pub closed: bool,
    pub detail: bool,
}

impl Contour {
    pub fn length(&self) -> f64 {
        let length = self
            .points
            .windows(2)
            .map(|p| (p[1][0] - p[0][0]).hypot(p[1][1] - p[0][1]))
            .sum::<f64>();
        if self.closed && self.points.len() > 1 {
            let a = self.points[0];
            let b = self.points[self.points.len() - 1];
            length + (b[0] - a[0]).hypot(b[1] - a[1])
        } else {
            length
        }
    }
}

pub struct Extraction {
    pub gray: GrayImage,
    pub mask: GrayImage,
    pub threshold: u8,
    pub edges: GrayImage,
    pub omitted_details: usize,
    /// 外轮廓按面积排序，随后是按长度排序的独立内部笔画。
    pub contours: Vec<Contour>,
}

pub fn extract(source: &SourceImage, settings: ProcessingSettings) -> Extraction {
    let gray = GrayImage::from_fn(source.rgb.width(), source.rgb.height(), |x, y| {
        let [r, g, b] = source.rgb.get_pixel(x, y).0.map(u32::from);
        Luma([((299 * r + 587 * g + 114 * b + 500) / 1000) as u8])
    });
    let gray = if settings.denoise {
        imageproc::filter::median_filter(&gray, 1, 1)
    } else {
        gray
    };

    let threshold = if settings.automatic {
        otsu_level(&gray)
    } else {
        settings.threshold
    };
    let mask = GrayImage::from_fn(gray.width(), gray.height(), |x, y| {
        let dark = gray.get_pixel(x, y)[0] <= threshold;
        Luma([if dark != settings.invert { 255 } else { 0 }])
    });

    // 补一圈背景，使触及图片边界的前景也有闭合边界。
    let mut padded = GrayImage::new(mask.width() + 2, mask.height() + 2);
    image::imageops::replace(&mut padded, &mask, 1, 1);
    let mut contours: Vec<_> = find_contours::<i32>(&padded)
        .into_iter()
        .filter(|contour| contour.border_type == BorderType::Outer && contour.points.len() >= 3)
        .filter_map(|contour| {
            let mut points: Vec<_> = contour
                .points
                .into_iter()
                .map(|p| [f64::from(p.x - 1), f64::from(p.y - 1)])
                .collect();
            points.dedup();
            if points.first() == points.last() {
                points.pop();
            }
            if points.len() < 3 {
                return None;
            }

            let area = polygon_area(&points);
            if area < 12.0 {
                return None;
            }

            let touches_edge = points.iter().any(|p| {
                p[0] <= 0.0
                    || p[1] <= 0.0
                    || p[0] >= f64::from(mask.width() - 1)
                    || p[1] >= f64::from(mask.height() - 1)
            });

            Some(Contour {
                points,
                area,
                touches_edge,
                closed: true,
                detail: false,
            })
        })
        .collect();
    contours.sort_by(|a, b| b.area.total_cmp(&a.area));

    let mut edges = GrayImage::new(mask.width(), mask.height());
    let mut omitted_details = 0;
    if settings.include_details {
        edges = super::edges::color_edges(&source.rgb, &gray, settings.edge_threshold);

        // 用外轮廓填充主体区域（包括其中亮色孔洞），保留白肚皮和眼睛。
        let mut subject = GrayImage::new(mask.width(), mask.height());
        for contour in &contours {
            let points: Vec<_> = contour
                .points
                .iter()
                .map(|p| imageproc::point::Point::new(p[0] as i32, p[1] as i32))
                .collect();
            imageproc::drawing::draw_polygon_mut(&mut subject, &points, Luma([255]));
        }
        if !contours.is_empty() {
            for (edge, inside) in edges.pixels_mut().zip(subject.pixels()) {
                if inside[0] == 0 {
                    edge[0] = 0;
                }
            }
        }

        // 已有外轮廓单独绘制，去除紧邻它的重复边缘。
        for contour in &contours {
            for point in &contour.points {
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let x = point[0] as i32 + dx;
                        let y = point[1] as i32 + dy;
                        if x >= 0 && y >= 0 && x < edges.width() as i32 && y < edges.height() as i32
                        {
                            edges.put_pixel(x as u32, y as u32, Luma([0]));
                        }
                    }
                }
            }
        }

        let mut details = super::strokes::trace_strokes(&edges, settings.min_stroke_length);
        omitted_details = details.len().saturating_sub(128);
        details.truncate(128);
        contours.extend(details);
    }

    Extraction {
        gray,
        mask,
        threshold,
        edges,
        omitted_details,
        contours,
    }
}

fn polygon_area(points: &[[f64; 2]]) -> f64 {
    points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .take(points.len())
        .map(|(a, b)| a[0] * b[1] - b[0] * a[1])
        .sum::<f64>()
        .abs()
        * 0.5
}
