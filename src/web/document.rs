use serde_json::{Value, json};

use crate::{
    animation::Drawing,
    image_processing::{self, ProcessingSettings, SourceImage},
    math::{InputStroke, fit_strokes, order_strokes},
};

/// 将技术参数收敛为三种预设，输出独立笔画与平滑抬笔时间轴。
pub(super) fn process(bytes: &[u8], detail: &str) -> Result<Value, String> {
    let (threshold, minimum, samples, terms) = match detail {
        "simple" => (32.0, 10.0, 256, 128),
        "balanced" => (24.0, 6.0, 512, 256),
        "fine" => (18.0, 4.0, 512, 512),
        _ => return Err("未知的细节预设。".into()),
    };

    let source = SourceImage::decode("image".into(), bytes)?;
    let extraction = image_processing::extract(
        &source,
        ProcessingSettings {
            edge_threshold: threshold,
            min_stroke_length: minimum,
            ..ProcessingSettings::default()
        },
    );

    let strokes = fit_strokes(
        order_strokes(
            extraction
                .contours
                .iter()
                .map(|contour| InputStroke {
                    points: contour.points.clone(),
                    closed: contour.closed,
                })
                .collect(),
        ),
        samples,
    )?;
    if strokes.is_empty() {
        return Err("没有找到清晰轮廓，请换一张主体与背景对比更明显的图片。".into());
    }

    let mut drawing = Drawing::default();
    drawing.load(&strokes.iter().collect::<Vec<_>>(), terms);
    let (scheduled, duration) = super::playback::schedule(&drawing);

    Ok(json!({
        "strokes": scheduled,
        "duration": duration,
        "bounds": drawing.bounds(),
        "strokeCount": strokes.len(),
        "size": source.original_size,
        "clipped": extraction.contours.iter().any(|c| c.touches_edge),
        "omitted": extraction.omitted_details,
    }))
}
