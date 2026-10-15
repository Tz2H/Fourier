//! 多通道梯度边缘：保留亮度相近但颜色不同的区域边界。

use image::{GrayImage, Luma, RgbImage};
use imageproc::{
    filter::gaussian_blur_f32,
    gradients::{horizontal_sobel, vertical_sobel},
};

pub(super) fn color_edges(rgb: &RgbImage, gray: &GrayImage, high: f32) -> GrayImage {
    let (width, height) = gray.dimensions();
    let mut output = GrayImage::new(width, height);
    if width < 3 || height < 3 {
        return output;
    }

    let size = (width * height) as usize;
    let mut strength = vec![0.0_f32; size];
    let mut direction = vec![0.0_f32; size];

    // 在每个像素保留灰度/R/G/B 中最强的梯度，而非只对二值剪影取边界。
    for channel in 0..4 {
        let input = if channel == 3 {
            gray.clone()
        } else {
            GrayImage::from_fn(width, height, |x, y| Luma([rgb.get_pixel(x, y)[channel]]))
        };
        let blurred = gaussian_blur_f32(&input, 0.8);
        let gx = horizontal_sobel(&blurred);
        let gy = vertical_sobel(&blurred);
        for (index, (x, y)) in gx.pixels().zip(gy.pixels()).enumerate() {
            let dx = f32::from(x[0]);
            let dy = f32::from(y[0]);
            let magnitude = dx.hypot(dy);
            if magnitude > strength[index] {
                strength[index] = magnitude;
                direction[index] = dy.atan2(dx).to_degrees().rem_euclid(180.0);
            }
        }
    }

    let stride = width as usize;
    let mut thinned = vec![0.0_f32; size];
    for y in 1..height as usize - 1 {
        for x in 1..stride - 1 {
            let index = y * stride + x;
            let angle = direction[index];
            let delta = if !(22.5..157.5).contains(&angle) {
                1
            } else if angle < 67.5 {
                stride + 1
            } else if angle < 112.5 {
                stride
            } else {
                stride - 1
            };
            if strength[index] >= strength[index - delta]
                && strength[index] >= strength[index + delta]
            {
                thinned[index] = strength[index];
            }
        }
    }

    // 八邻域滞后连接：弱边缘必须与强边缘相连，避免全图低阈值噪声。
    let high = high.max(1.0);
    let low = high * 0.4;
    let mut stack = Vec::new();
    for (index, &value) in thinned.iter().enumerate() {
        if value >= high {
            output.as_mut()[index] = 255;
            stack.push(index);
        }
    }

    while let Some(index) = stack.pop() {
        let x = index % stride;
        let y = index / stride;
        for dy in -1_i32..=1 {
            for dx in -1_i32..=1 {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                    continue;
                }
                let next = ny as usize * stride + nx as usize;
                if output.as_raw()[next] == 0 && thinned[next] >= low {
                    output.as_mut()[next] = 255;
                    stack.push(next);
                }
            }
        }
    }

    output
}
