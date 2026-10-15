use std::f64::consts::TAU;

use super::Epicycle;

/// 复数 DFT，输入 x + iy。返回按振幅降序排列的全部频率分量。
/// 每条笔画最多 1024 点；仅在路径改变时计算。
pub fn dft(points: &[[f64; 2]]) -> Vec<Epicycle> {
    let count = points.len();
    let mut coefficients = Vec::with_capacity(count);

    for bin in 0..count {
        let frequency = if bin <= count / 2 {
            bin as f64
        } else {
            bin as f64 - count as f64
        };

        let mut re = 0.0;
        let mut im = 0.0;

        for (index, &[x, y]) in points.iter().enumerate() {
            let angle = -TAU * frequency * index as f64 / count as f64;
            let (sin, cos) = angle.sin_cos();
            re += x * cos - y * sin;
            im += x * sin + y * cos;
        }

        re /= count as f64;
        im /= count as f64;
        coefficients.push(Epicycle {
            radius: re.hypot(im),
            frequency,
            phase: im.atan2(re),
        });
    }

    coefficients.sort_by(|a, b| b.radius.total_cmp(&a.radius));
    coefficients
}
