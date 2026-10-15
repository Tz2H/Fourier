//! 按界面区域拆分绘制逻辑，共享本轮与轨迹配色。

mod canvas;
mod controls;
mod image_panel;
mod readout;

use eframe::egui::Color32;

pub(super) use canvas::canvas;
pub(super) use controls::controls;
pub(super) use image_panel::image_panel;
pub(super) use readout::readout;

const COLORS: [Color32; 2] = [
    Color32::from_rgb(255, 195, 95),
    Color32::from_rgb(115, 165, 255),
];
const TRACE_COLOR: Color32 = Color32::from_rgb(70, 210, 190);
