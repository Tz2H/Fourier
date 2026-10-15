//! 按笔画调度重建；每条轨迹独立保存，笔画间不画连接线。

use crate::math::{Epicycle, FourierStroke, chain};

pub struct DrawingStroke {
    pub reference: Vec<[f64; 2]>,
    pub closed: bool,
    pub origin: [f64; 2],
    pub coefficients: Vec<Epicycle>,
    pub curve: Vec<[f64; 2]>,
    end_progress: f64,
}

pub struct Drawing {
    pub strokes: Vec<DrawingStroke>,
    progress: f64,
    duration: f64,
    paused: bool,
    extent: f64,
    bounds: [[f64; 2]; 2],
}

impl Default for Drawing {
    fn default() -> Self {
        Self {
            strokes: Vec::new(),
            progress: 0.0,
            duration: 20.0,
            paused: false,
            extent: 160.0,
            bounds: [[-160.0; 2], [160.0; 2]],
        }
    }
}

impl Drawing {
    pub fn load(&mut self, strokes: &[&FourierStroke], terms: usize) {
        let total = strokes.iter().map(|s| s.length.max(1.0)).sum::<f64>();
        let mut cumulative = 0.0;

        self.strokes = strokes
            .iter()
            .map(|stroke| {
                let coefficients: Vec<_> =
                    stroke.coefficients.iter().take(terms).copied().collect();
                let steps = (stroke.coefficients.len() * 2).max(128);
                let span = if stroke.closed { 1.0 } else { 0.5 };
                let curve = (0..=steps)
                    .map(|index| {
                        evaluate(
                            &coefficients,
                            stroke.origin,
                            span * index as f64 / steps as f64,
                        )
                    })
                    .collect();
                cumulative += stroke.length.max(1.0);

                DrawingStroke {
                    reference: stroke.reference.clone(),
                    closed: stroke.closed,
                    origin: stroke.origin,
                    coefficients,
                    curve,
                    end_progress: cumulative / total,
                }
            })
            .collect();

        self.extent = self
            .strokes
            .iter()
            .map(|stroke| {
                stroke.origin[0].abs().max(stroke.origin[1].abs())
                    + stroke.coefficients.iter().map(|e| e.radius).sum::<f64>()
            })
            .fold(160.0, f64::max);

        let mut points = self
            .strokes
            .iter()
            .flat_map(|stroke| stroke.reference.iter().chain(&stroke.curve).copied());
        self.bounds = if let Some(first) = points.next() {
            points.fold([first, first], |[mut min, mut max], point| {
                for axis in 0..2 {
                    min[axis] = min[axis].min(point[axis]);
                    max[axis] = max[axis].max(point[axis]);
                }
                [min, max]
            })
        } else {
            [[-160.0; 2], [160.0; 2]]
        };

        self.reset();
    }

    pub fn extent(&self) -> f64 {
        self.extent
    }

    /// 参考路线与完整重建轨迹的边界，加载时缓存，不随动画进度变化。
    pub fn bounds(&self) -> [[f64; 2]; 2] {
        self.bounds
    }

    pub fn progress(&self) -> f64 {
        self.progress
    }

    pub fn paused(&self) -> bool {
        self.paused
    }

    pub fn duration(&self) -> f64 {
        self.duration
    }

    pub fn set_duration(&mut self, seconds: f64) {
        self.duration = seconds.clamp(4.0, 120.0);
    }

    pub fn reset(&mut self) {
        self.progress = 0.0;
    }

    /// 直接显示完整的傅里叶重建结果，便于检查小笔画。
    pub fn finish(&mut self) {
        self.progress = 1.0;
        self.paused = true;
    }

    pub fn toggle_pause(&mut self) {
        if self.progress >= 1.0 {
            self.progress = 0.0;
            self.paused = false;
        } else {
            self.paused = !self.paused;
        }
    }

    pub fn is_moving(&self) -> bool {
        !self.paused && !self.strokes.is_empty() && self.progress < 1.0
    }

    pub fn advance(&mut self, seconds: f64) {
        if self.is_moving() && seconds.is_finite() && seconds > 0.0 {
            self.progress = (self.progress + seconds.min(0.05) / self.duration).min(1.0);
            if self.progress >= 1.0 {
                self.paused = true;
            }
        }
    }

    /// 当前笔画索引和笔画内进度；按弧长分配整幅图的绘制时间。
    pub fn active(&self) -> Option<(usize, f64)> {
        if self.strokes.is_empty() {
            return None;
        }

        let index = self
            .strokes
            .iter()
            .position(|s| self.progress < s.end_progress)
            .unwrap_or(self.strokes.len() - 1);
        let start = if index == 0 {
            0.0
        } else {
            self.strokes[index - 1].end_progress
        };
        let span = self.strokes[index].end_progress - start;

        Some((
            index,
            ((self.progress - start) / span.max(f64::EPSILON)).clamp(0.0, 1.0),
        ))
    }

    pub fn active_chain(&self) -> Vec<[f64; 2]> {
        let Some((index, progress)) = self.active() else {
            return Vec::new();
        };

        let stroke = &self.strokes[index];
        let time = progress * if stroke.closed { 1.0 } else { 0.5 };

        chain(&stroke.coefficients, time)
            .into_iter()
            .map(|p| [p[0] + stroke.origin[0], p[1] + stroke.origin[1]])
            .collect()
    }
}

fn evaluate(coefficients: &[Epicycle], origin: [f64; 2], time: f64) -> [f64; 2] {
    coefficients.iter().fold(origin, |p, e| {
        let offset = e.offset(time);
        [p[0] + offset[0], p[1] + offset[1]]
    })
}
