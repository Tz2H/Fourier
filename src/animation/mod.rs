//! 管理动画时间与轨迹，基于数学模型生成当前帧状态。

mod drawing;
mod simulation;

pub use simulation::{MAX_TRACE_POINTS, Simulation};

pub use drawing::Drawing;
