use super::TRACE_COLOR;
use crate::desktop::app::{FourierApp, Mode};
use eframe::egui;

pub(crate) fn readout(ctx: &egui::Context, app: &FourierApp) {
    egui::TopBottomPanel::bottom("readout").show(ctx, |ui| {
        ui.monospace("z(t) = stroke_origin + sum A[k] * exp(i * (2*pi*f[k]*t + phase[k]))");
        ui.horizontal_wrapped(|ui| {
            let point = match app.mode {
                Mode::Image => {
                    if let Some((index, _)) = app.drawing.active() {
                        ui.label(format!(
                            "Stroke {} / {} | {:.1}% | {} terms in this stroke",
                            index + 1,
                            app.drawing.strokes.len(),
                            app.drawing.progress() * 100.0,
                            app.drawing.strokes[index].coefficients.len()
                        ));
                    }
                    if app.drawing.progress() >= 1.0 {
                        ui.label("Complete. Press Play to replay.");
                    }
                    app.drawing
                        .active_chain()
                        .last()
                        .copied()
                        .unwrap_or([0.0, 0.0])
                }
                Mode::Demo => {
                    ui.label(format!("t = {:.2}s", app.simulation.seconds()));
                    app.simulation.endpoint()
                }
            };
            ui.colored_label(
                TRACE_COLOR,
                format!("Tip: ({:.2}, {:.2})", point[0], point[1]),
            );
            if app.paused() {
                ui.label("Paused");
            }
        });
    });
}
