//! 原生桌面入口，负责装配窗口、应用状态与界面。

mod app;
mod document;
mod ui;
mod viewport;

use eframe::egui;

use app::FourierApp;

/// 启动原生桌面演示。
pub fn run() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1240.0, 840.0])
            .with_min_inner_size([960.0, 680.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Fourier - Image to epicycles",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_theme(egui::ThemePreference::Dark);
            Ok(Box::new(FourierApp::new(&cc.egui_ctx)))
        }),
    )
}
