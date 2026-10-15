use std::{collections::VecDeque, f64::consts::TAU};

use crate::math::{Epicycle, chain};

pub const MAX_TRACE_POINTS: usize = 4096;
const MAX_FRAME_SECONDS: f64 = 0.05;

/// 动画状态只保存数学坐标，不依赖窗口或绘图库。
pub struct Simulation {
    epicycles: Vec<Epicycle>,
    image_mode: bool,
    rate: f64,
    seconds: f64,
    paused: bool,
    trace: VecDeque<[f64; 2]>,
}

impl Default for Simulation {
    fn default() -> Self {
        Self::new([
            Epicycle {
                radius: 120.0,
                frequency: 0.2,
                phase: 0.0,
            },
            Epicycle {
                radius: 50.0,
                frequency: -0.6,
                phase: 0.0,
            },
        ])
    }
}

impl Simulation {
    pub fn new(epicycles: impl Into<Vec<Epicycle>>) -> Self {
        let mut simulation = Self {
            epicycles: epicycles.into(),
            image_mode: false,
            rate: 1.0,
            seconds: 0.0,
            paused: false,
            trace: VecDeque::new(),
        };

        simulation.reset();
        simulation
    }

    pub fn epicycles(&self) -> &[Epicycle] {
        &self.epicycles
    }

    pub fn seconds(&self) -> f64 {
        self.seconds
    }

    pub fn paused(&self) -> bool {
        self.paused
    }

    pub fn trace(&self) -> &VecDeque<[f64; 2]> {
        &self.trace
    }

    pub fn points(&self) -> Vec<[f64; 2]> {
        chain(&self.epicycles, self.seconds)
    }

    /// 修改参数时从初始位置重新绘制，保留暂停状态。
    pub fn set_epicycles(&mut self, epicycles: impl Into<Vec<Epicycle>>) {
        self.epicycles = epicycles.into();
        self.reset();
    }

    /// 图片模式：频率单位为每轮圈数，默认十秒绘制一轮。
    pub fn load_reconstruction(&mut self, epicycles: Vec<Epicycle>) {
        if !self.image_mode {
            self.rate = 0.1;
        }

        self.image_mode = true;
        self.set_epicycles(epicycles);
    }

    pub fn rate(&self) -> f64 {
        self.rate
    }

    pub fn set_rate(&mut self, rate: f64) {
        self.rate = rate.clamp(0.025, 2.0);
    }

    pub fn image_mode(&self) -> bool {
        self.image_mode
    }

    pub fn endpoint(&self) -> [f64; 2] {
        self.endpoint_at(self.seconds)
    }

    fn endpoint_at(&self, seconds: f64) -> [f64; 2] {
        self.epicycles.iter().fold([0.0, 0.0], |sum, e| {
            let offset = e.offset(seconds);
            [sum[0] + offset[0], sum[1] + offset[1]]
        })
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn reset(&mut self) {
        self.seconds = 0.0;
        self.trace.clear();
        self.trace.push_back(self.endpoint());
    }

    pub fn is_moving(&self) -> bool {
        !self.paused
            && self
                .epicycles
                .iter()
                .any(|e| e.radius != 0.0 && e.frequency != 0.0)
    }

    /// 推进一步动画。长帧最多推进 0.05 秒，避免恢复窗口时突然跳跃。
    pub fn advance(&mut self, seconds: f64) {
        if !self.is_moving() || !seconds.is_finite() || seconds <= 0.0 {
            return;
        }

        let seconds = seconds.min(MAX_FRAME_SECONDS) * self.rate;
        let fastest = self
            .epicycles
            .iter()
            .filter(|e| e.radius != 0.0)
            .map(|e| e.frequency.abs())
            .fold(0.0_f64, f64::max);

        // 图片模式每轮至少约 2048 段；演示模式按最快向量每度细分。
        // 每帧最多 128 段，限制计算量。
        let steps = if self.image_mode {
            (seconds * 2048.0).ceil() as usize
        } else {
            (TAU * fastest * seconds / 1.0_f64.to_radians()).ceil() as usize
        }
        .clamp(1, 128);

        let start = self.seconds;
        for step in 1..=steps {
            let next = start + seconds * step as f64 / steps as f64;
            if self.image_mode
                && next.floor() > (start + seconds * (step - 1) as f64 / steps as f64).floor()
            {
                self.trace.clear();
                self.trace.push_back(self.endpoint_at(0.0));
            }
            self.seconds = if self.image_mode {
                next.rem_euclid(1.0)
            } else {
                next
            };
            self.trace.push_back(self.endpoint());
            if self.trace.len() > MAX_TRACE_POINTS {
                self.trace.pop_front();
            }
        }
    }
}
