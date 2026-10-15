//! 傅里叶本轮演示。
//!
//! 本地网页服务装配图像处理与数学计算；计算模块不依赖界面类型。

pub mod animation;
#[cfg(feature = "desktop")]
pub mod desktop;
pub mod image_processing;
pub mod math;
mod web;

pub use web::run;
