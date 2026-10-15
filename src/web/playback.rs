//! 独立笔画的时间轴与抬笔曲线，端点来自实际重建结果。

use crate::animation::Drawing;
use serde_json::{Value, json};

pub(super) fn schedule(drawing: &Drawing) -> (Vec<Value>, f64) {
    let distances: Vec<Vec<f64>> = drawing
        .strokes
        .iter()
        .map(|stroke| {
            let mut cumulative = vec![0.0];
            for pair in stroke.curve.windows(2) {
                cumulative.push(cumulative.last().unwrap() + distance(pair[0], pair[1]));
            }
            cumulative
        })
        .collect();
    let total = distances
        .iter()
        .map(|s| s.last().unwrap().max(1.0))
        .sum::<f64>();

    let mut clock = 0.0;
    let strokes = drawing
        .strokes
        .iter()
        .enumerate()
        .map(|(index, stroke)| {
            let start = clock;
            // 每笔保留最小绘制时间，避免细小轮廓在一帧内闪过。
            clock += 0.08 + 18.0 * distances[index].last().unwrap().max(1.0) / total;
            let end = clock;

            let travel = drawing.strokes.get(index + 1).map(|next| {
                let a = *stroke.curve.last().unwrap();
                let d = next.curve[0];
                let gap = distance(a, d);
                let handle = (gap * 0.3).min(32.0);
                let outgoing = direction(stroke.curve[stroke.curve.len() - 2], a);
                let incoming = direction(d, next.curve[1]);
                let b = [a[0] + outgoing[0] * handle, a[1] + outgoing[1] * handle];
                let c = [d[0] - incoming[0] * handle, d[1] - incoming[1] * handle];
                clock += 0.12 + 0.18 * (gap / 320.0).clamp(0.0, 1.0).sqrt();
                json!({"end": clock, "controls": [a, b, c, d]})
            });

            let coefficients: Vec<_> = stroke
                .coefficients
                .iter()
                .map(|e| [e.radius, e.frequency, e.phase])
                .collect();

            json!({
                "origin": stroke.origin,
                "coefficients": coefficients,
                "curve": stroke.curve,
                "reference": stroke.reference,
                "closed": stroke.closed,
                "arcLengths": distances[index],
                "start": start, "end": end, "travel": travel,
            })
        })
        .collect();

    (strokes, clock)
}

fn distance(a: [f64; 2], b: [f64; 2]) -> f64 {
    (b[0] - a[0]).hypot(b[1] - a[1])
}

fn direction(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    let length = distance(a, b).max(f64::EPSILON);
    [(b[0] - a[0]) / length, (b[1] - a[1]) / length]
}
