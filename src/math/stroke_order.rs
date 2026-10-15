//! 按邻近端点排序独立笔画；仅改变顺序和走向，不添加连接线。

use super::InputStroke;

pub fn order_strokes(mut remaining: Vec<InputStroke>) -> Vec<InputStroke> {
    remaining.retain(|stroke| stroke.points.len() >= 2);
    let mut ordered = Vec::with_capacity(remaining.len());
    let Some(first) = remaining
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| length(a).total_cmp(&length(b)))
        .map(|(index, _)| index)
    else {
        return ordered;
    };

    let first = remaining.remove(first);
    let mut tip = exit(&first);
    ordered.push(first);

    while !remaining.is_empty() {
        let mut best = (f64::INFINITY, 0, 0);
        for (index, stroke) in remaining.iter().enumerate() {
            for (entry, point) in stroke.points.iter().enumerate() {
                if !stroke.closed && entry != 0 && entry != stroke.points.len() - 1 {
                    continue;
                }
                let distance = (point[0] - tip[0]).hypot(point[1] - tip[1]);
                if distance < best.0 {
                    best = (distance, index, entry);
                }
            }
        }

        let mut stroke = remaining.remove(best.1);
        if stroke.closed {
            stroke.points.rotate_left(best.2);
        } else if best.2 != 0 {
            stroke.points.reverse();
        }

        tip = exit(&stroke);
        ordered.push(stroke);
    }

    ordered
}

fn exit(stroke: &InputStroke) -> [f64; 2] {
    if stroke.closed {
        stroke.points[0]
    } else {
        *stroke.points.last().unwrap()
    }
}

fn length(stroke: &InputStroke) -> f64 {
    let length = stroke
        .points
        .windows(2)
        .map(|p| (p[1][0] - p[0][0]).hypot(p[1][1] - p[0][1]))
        .sum::<f64>();
    length
        + if stroke.closed {
            let first = stroke.points[0];
            let last = stroke.points.last().unwrap();
            (first[0] - last[0]).hypot(first[1] - last[1])
        } else {
            0.0
        }
}
