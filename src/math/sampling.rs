/// 沿闭合轮廓按弧长等距采样，不重复最后的起点。
pub fn resample_closed(points: &[[f64; 2]], count: usize) -> Result<Vec<[f64; 2]>, String> {
    if points.len() < 3 || count < 3 {
        return Err("Contour needs at least three points.".into());
    }

    let mut cumulative = Vec::with_capacity(points.len() + 1);
    cumulative.push(0.0);
    for index in 0..points.len() {
        let a = points[index];
        let b = points[(index + 1) % points.len()];
        cumulative.push(cumulative[index] + (b[0] - a[0]).hypot(b[1] - a[1]));
    }

    let length = cumulative[points.len()];
    if !length.is_finite() || length <= f64::EPSILON {
        return Err("Contour has no usable length.".into());
    }

    let mut segment = 0;
    let samples = (0..count)
        .map(|index| {
            let distance = length * index as f64 / count as f64;
            while segment + 1 < points.len() && cumulative[segment + 1] <= distance {
                segment += 1;
            }
            let span = cumulative[segment + 1] - cumulative[segment];
            let t = if span > 0.0 {
                (distance - cumulative[segment]) / span
            } else {
                0.0
            };
            let a = points[segment];
            let b = points[(segment + 1) % points.len()];
            [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
        })
        .collect();

    Ok(samples)
}

/// 以采样均值居中、翻转图片 y 轴，并等比缩放到 320 × 320 的区域内。
pub fn normalize_image_points(points: &mut [[f64; 2]]) {
    if points.is_empty() {
        return;
    }

    let count = points.len() as f64;
    let mean = [
        points.iter().map(|p| p[0]).sum::<f64>() / count,
        points.iter().map(|p| p[1]).sum::<f64>() / count,
    ];
    let mut extent = [0.0_f64; 2];
    for point in points.iter_mut() {
        point[0] -= mean[0];
        point[1] = mean[1] - point[1];
        extent[0] = extent[0].max(point[0].abs());
        extent[1] = extent[1].max(point[1].abs());
    }

    let scale = 160.0 / extent[0].max(extent[1]).max(f64::EPSILON);
    for point in points {
        point[0] *= scale;
        point[1] *= scale;
    }
}
