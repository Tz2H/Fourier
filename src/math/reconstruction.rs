//! 多笔画共用坐标变换；开放路径通过原路折返成为周期信号。

use super::{Epicycle, dft, resample_closed};

pub struct InputStroke {
    pub points: Vec<[f64; 2]>,
    pub closed: bool,
}

pub struct FourierStroke {
    pub reference: Vec<[f64; 2]>,
    pub closed: bool,
    pub origin: [f64; 2],
    pub coefficients: Vec<Epicycle>,
    pub length: f64,
}

pub fn fit_strokes(
    strokes: Vec<InputStroke>,
    max_samples: usize,
) -> Result<Vec<FourierStroke>, String> {
    if strokes.is_empty() {
        return Ok(Vec::new());
    }

    let mut min = [f64::INFINITY; 2];
    let mut max = [f64::NEG_INFINITY; 2];
    for point in strokes.iter().flat_map(|stroke| &stroke.points) {
        for axis in 0..2 {
            min[axis] = min[axis].min(point[axis]);
            max[axis] = max[axis].max(point[axis]);
        }
    }

    let center = [(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5];
    let scale = 320.0 / (max[0] - min[0]).max(max[1] - min[1]).max(1.0);

    let length = |stroke: &InputStroke| {
        let mut value = stroke
            .points
            .windows(2)
            .map(|p| (p[1][0] - p[0][0]).hypot(p[1][1] - p[0][1]))
            .sum::<f64>();
        if stroke.closed && stroke.points.len() > 1 {
            let a = stroke.points[0];
            let b = stroke.points[stroke.points.len() - 1];
            value += (b[0] - a[0]).hypot(b[1] - a[1]);
        }
        value
    };
    let longest = strokes.iter().map(length).fold(1.0_f64, f64::max);

    strokes
        .into_iter()
        .map(|stroke| {
            let arc_length = length(&stroke);
            let reference: Vec<_> = stroke
                .points
                .iter()
                .map(|p| [(p[0] - center[0]) * scale, (center[1] - p[1]) * scale])
                .collect();

            // 开放笔画折返后长度翻倍；短细节少采样，避免每条小线段都做 1024² 次运算。
            let factor = if stroke.closed { 1.0 } else { 2.0 };
            let count = ((max_samples as f64 * arc_length * factor / longest).ceil() as usize)
                .clamp(32, max_samples.max(32))
                .next_multiple_of(2);
            fit_normalized(reference, stroke.closed, count)
        })
        .collect()
}

/// 输入已在整幅图的统一坐标系内，不再单独居中缩放。
pub(super) fn fit_normalized(
    reference: Vec<[f64; 2]>,
    closed: bool,
    count: usize,
) -> Result<FourierStroke, String> {
    let mut periodic = reference.clone();
    if !closed && reference.len() > 2 {
        periodic.extend(reference[1..reference.len() - 1].iter().rev().copied());
    }

    let mut samples = resample_closed(&periodic, count)?;
    let origin = [
        samples.iter().map(|p| p[0]).sum::<f64>() / count as f64,
        samples.iter().map(|p| p[1]).sum::<f64>() / count as f64,
    ];
    for point in &mut samples {
        point[0] -= origin[0];
        point[1] -= origin[1];
    }

    let coefficients = dft(&samples)
        .into_iter()
        .filter(|e| e.frequency != 0.0)
        .collect();

    let mut length = reference
        .windows(2)
        .map(|pair| (pair[1][0] - pair[0][0]).hypot(pair[1][1] - pair[0][1]))
        .sum::<f64>();
    if closed && reference.len() > 1 {
        let a = reference[0];
        let b = reference[reference.len() - 1];
        length += (a[0] - b[0]).hypot(a[1] - b[1]);
    }

    Ok(FourierStroke {
        reference,
        closed,
        origin,
        coefficients,
        length,
    })
}
