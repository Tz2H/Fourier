//! 纯数学计算，不依赖动画状态或桌面界面。

mod epicycle;
mod fourier;
mod reconstruction;
mod sampling;
mod stroke_order;

pub use epicycle::{Epicycle, chain};

pub use fourier::dft;
pub use sampling::{normalize_image_points, resample_closed};

pub use reconstruction::{FourierStroke, InputStroke, fit_strokes};

pub use stroke_order::order_strokes;
