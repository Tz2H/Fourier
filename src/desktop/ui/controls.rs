use eframe::egui;

use crate::desktop::app::{FourierApp, Mode};

use super::COLORS;

pub(crate) fn controls(ctx: &egui::Context, app: &mut FourierApp) -> bool {
    let mut changed = false;
    egui::TopBottomPanel::top("controls").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("Fourier / Image to epicycles");
            ui.separator();
            if ui
                .selectable_label(app.mode == Mode::Image, "Image")
                .clicked()
            {
                app.set_mode(Mode::Image);
                changed = true;
            }
            if ui
                .selectable_label(app.mode == Mode::Demo, "Two-vector demo")
                .clicked()
            {
                app.set_mode(Mode::Demo);
                changed = true;
            }
        });
        ui.horizontal_wrapped(|ui| {
            if ui.button("Open image...").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("PNG / JPEG", &["png", "jpg", "jpeg"])
                    .pick_file()
                {
                    app.load_path(ctx, &path);
                }
                changed = true;
            }
            if ui.button("Load sample").clicked() {
                app.load_sample(ctx);
                changed = true;
            }
            if ui
                .button(if app.paused() { "Play" } else { "Pause" })
                .clicked()
            {
                app.toggle_pause();
                changed = true;
            }
            if ui.button("Restart").clicked() {
                app.restart();
                changed = true;
            }
            if app.mode == Mode::Image && ui.button("Show complete").clicked() {
                app.drawing.finish();
                changed = true;
            }
            ui.checkbox(&mut app.show_circles, "Show circles");
            ui.checkbox(&mut app.show_reference, "Show source contour");
        });
        if app.mode == Mode::Image {
            let total = app
                .document
                .as_ref()
                .map_or(0, |document| document.max_terms());
            ui.horizontal_wrapped(|ui| {
                ui.add_enabled_ui(total > 0, |ui| {
                    if ui
                        .add(
                            egui::Slider::new(&mut app.term_count, 1..=total.max(1))
                                .text("Terms / stroke"),
                        )
                        .changed()
                    {
                        app.sync_reconstruction();
                        changed = true;
                    }
                });
                let mut duration = app.drawing.duration();
                if ui
                    .add(egui::Slider::new(&mut duration, 4.0..=120.0).text("Seconds / drawing"))
                    .changed()
                {
                    app.drawing.set_duration(duration);
                    changed = true;
                }
            });
        } else {
            let mut epicycles = app.simulation.epicycles().to_vec();
            let mut parameters_changed = false;
            for (index, epicycle) in epicycles.iter_mut().enumerate() {
                ui.push_id(index, |ui| {
                    ui.horizontal(|ui| {
                        ui.colored_label(
                            COLORS[index % COLORS.len()],
                            format!("Vector {}", index + 1),
                        );
                        parameters_changed |= ui
                            .add(
                                egui::Slider::new(&mut epicycle.radius, 0.0..=180.0).text("Radius"),
                            )
                            .changed();
                        parameters_changed |= ui
                            .add(
                                egui::Slider::new(&mut epicycle.frequency, -1.0..=1.0)
                                    .text("Turns/s"),
                            )
                            .changed();
                    });
                });
            }
            if parameters_changed {
                app.simulation.set_epicycles(epicycles);
                changed = true;
            }
        }
        app.viewport.controls(ui);
        if let Some(error) = &app.error {
            ui.colored_label(egui::Color32::LIGHT_RED, error);
        }
        ui.add_space(4.0);
    });
    changed
}
