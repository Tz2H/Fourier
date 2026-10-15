//! 图片解码、灰度分割与有序轮廓提取，不依赖桌面界面。

mod contours;
mod edges;
mod source;
mod strokes;

pub use contours::{Contour, Extraction, ProcessingSettings, extract};
pub use source::SourceImage;
