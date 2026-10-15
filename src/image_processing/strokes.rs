//! 将单像素边缘图追踪成开放笔画或闭合环，不把分叉和断开的边缘硬连。

use super::Contour;
use image::GrayImage;

const OFFSETS: [(i32, i32); 8] = [
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
];

pub(super) fn trace_strokes(image: &GrayImage, min_length: f64) -> Vec<Contour> {
    let width = image.width() as usize;
    let height = image.height() as usize;
    let mut pixels: Vec<bool> = image.as_raw().iter().map(|&p| p != 0).collect();
    thin(&mut pixels, width, height);

    let mut links = vec![0_u8; pixels.len()];
    let at = |x: i32, y: i32| -> bool {
        x >= 0
            && y >= 0
            && x < width as i32
            && y < height as i32
            && pixels[y as usize * width + x as usize]
    };
    for index in 0..pixels.len() {
        if !pixels[index] {
            continue;
        }
        let (x, y) = ((index % width) as i32, (index / width) as i32);
        for (direction, &(dx, dy)) in OFFSETS.iter().enumerate() {
            if !at(x + dx, y + dy) {
                continue;
            }
            // 已有直角连接时不加对角捷径，避免产生无意义的三角形分叉。
            if dx != 0 && dy != 0 && (at(x + dx, y) || at(x, y + dy)) {
                continue;
            }
            links[index] |= 1 << direction;
        }
    }

    let mut visited = vec![0_u8; pixels.len()];
    let mut result = Vec::new();
    // 先处理端点和分叉之间的路径，再处理剩余的闭合环。
    for loops in [false, true] {
        for start in 0..pixels.len() {
            if links[start] == 0 || (!loops && links[start].count_ones() == 2) {
                continue;
            }
            while links[start] & !visited[start] != 0 {
                let mut current = start;
                let mut points = vec![[(start % width) as f64, (start / width) as f64]];

                loop {
                    let available = links[current] & !visited[current];
                    if available == 0 {
                        break;
                    }
                    let direction = available.trailing_zeros() as usize;
                    let (dx, dy) = OFFSETS[direction];
                    let next = ((current / width) as i32 + dy) as usize * width
                        + ((current % width) as i32 + dx) as usize;
                    visited[current] |= 1 << direction;
                    visited[next] |= 1 << ((direction + 4) % 8);
                    current = next;
                    points.push([(current % width) as f64, (current / width) as f64]);
                    if current == start || links[current].count_ones() != 2 {
                        break;
                    }
                }

                let closed = current == start && points.len() > 3;
                let length = points
                    .windows(2)
                    .map(|p| (p[1][0] - p[0][0]).hypot(p[1][1] - p[0][1]))
                    .sum::<f64>();
                if length >= min_length && points.len() >= 3 {
                    if closed {
                        points.pop();
                    }
                    result.push(Contour {
                        points,
                        area: 0.0,
                        touches_edge: false,
                        closed,
                        detail: true,
                    });
                }
            }
        }
    }

    result.sort_by(|a, b| b.length().total_cmp(&a.length()));
    result
}

// Zhang–Suen 细化；每轮同步删除，保留连通性和线条端点。
fn thin(pixels: &mut [bool], width: usize, height: usize) {
    if width < 3 || height < 3 {
        return;
    }

    loop {
        let mut changed = false;
        for second_pass in [false, true] {
            let mut remove = Vec::new();
            for y in 1..height - 1 {
                for x in 1..width - 1 {
                    let index = y * width + x;
                    if !pixels[index] {
                        continue;
                    }
                    let p = OFFSETS.map(|(dx, dy)| {
                        pixels[(y as i32 + dy) as usize * width + (x as i32 + dx) as usize]
                    });
                    let neighbors = p.iter().filter(|&&v| v).count();
                    let transitions = (0..8).filter(|&i| !p[i] && p[(i + 1) % 8]).count();
                    if !(2..=6).contains(&neighbors) || transitions != 1 {
                        continue;
                    }
                    let removable = if second_pass {
                        !(p[0] && p[2] && p[6]) && !(p[0] && p[4] && p[6])
                    } else {
                        !(p[0] && p[2] && p[4]) && !(p[2] && p[4] && p[6])
                    };
                    if removable {
                        remove.push(index);
                    }
                }
            }
            changed |= !remove.is_empty();
            for index in remove {
                pixels[index] = false;
            }
        }
        if !changed {
            break;
        }
    }
}
