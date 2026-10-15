use std::f64::consts::TAU;

/// 一个旋转向量。半径表示振幅，频率是每单位模型时间旋转的圈数。
/// 演示模式的模型时间为秒，图片模式的模型时间为绘制一轮的进度。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Epicycle {
    pub radius: f64,
    pub frequency: f64,
    /// 初始相位，单位为弧度。
    pub phase: f64,
}

impl Epicycle {
    /// 返回数学坐标中的向量偏移：x 向右，y 向上。
    pub fn offset(self, seconds: f64) -> [f64; 2] {
        let angle = self.angle(seconds);
        [self.radius * angle.cos(), self.radius * angle.sin()]
    }

    pub fn angle(self, seconds: f64) -> f64 {
        (TAU * self.frequency * seconds + self.phase).rem_euclid(TAU)
    }
}

/// 返回首尾相接的所有节点，第一个节点为原点。
pub fn chain(epicycles: &[Epicycle], seconds: f64) -> Vec<[f64; 2]> {
    let mut points = Vec::with_capacity(epicycles.len() + 1);
    let mut tip = [0.0, 0.0];
    points.push(tip);
    for epicycle in epicycles {
        let offset = epicycle.offset(seconds);
        tip[0] += offset[0];
        tip[1] += offset[1];
        points.push(tip);
    }
    points
}
