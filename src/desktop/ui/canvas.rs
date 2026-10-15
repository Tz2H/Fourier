use super::{COLORS, TRACE_COLOR};
use crate::{
    desktop::app::{FourierApp, Mode},
    math::Epicycle,
};
use eframe::egui::{self, Color32, FontId, Pos2, Stroke, Vec2};

pub(crate) fn canvas(ctx: &egui::Context, app: &mut FourierApp) {
    egui::CentralPanel::default().show(ctx, |ui| {
        let (response, painter) =
            ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
        let rect = response.rect;
        app.viewport.interact(&response);
        response.on_hover_cursor(egui::CursorIcon::Grab);
        let bounds = match app.mode {
            Mode::Image => app.drawing.bounds(),
            Mode::Demo => {
                let extent = app
                    .simulation
                    .epicycles()
                    .iter()
                    .map(|e| e.radius)
                    .sum::<f64>()
                    .max(1.0);
                [[-extent; 2], [extent; 2]]
            }
        };
        let (center, scale) = app.viewport.transform(rect, bounds);
        let project = |p: [f64; 2]| center + Vec2::new(p[0] as f32, -p[1] as f32) * scale;
        let axes = Stroke::new(1.0, Color32::from_gray(55));
        painter.line_segment(
            [
                Pos2::new(rect.left() + 15.0, center.y),
                Pos2::new(rect.right() - 15.0, center.y),
            ],
            axes,
        );
        painter.line_segment(
            [
                Pos2::new(center.x, rect.top() + 15.0),
                Pos2::new(center.x, rect.bottom() - 15.0),
            ],
            axes,
        );
        painter.text(
            Pos2::new(rect.right() - 20.0, center.y - 10.0),
            egui::Align2::RIGHT_BOTTOM,
            "+x",
            FontId::proportional(14.0),
            Color32::GRAY,
        );
        painter.text(
            Pos2::new(center.x + 10.0, rect.top() + 15.0),
            egui::Align2::LEFT_TOP,
            "+y",
            FontId::proportional(14.0),
            Color32::GRAY,
        );
        match app.mode {
            Mode::Image => {
                let Some((active, progress)) = app.drawing.active() else {
                    painter.text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "Load an image and extract its strokes",
                        FontId::proportional(18.0),
                        Color32::GRAY,
                    );
                    return;
                };
                let chain = app.drawing.active_chain();
                for (index, stroke) in app.drawing.strokes.iter().enumerate() {
                    if app.show_reference {
                        let mut outline: Vec<_> =
                            stroke.reference.iter().copied().map(project).collect();
                        if stroke.closed && !outline.is_empty() {
                            outline.push(outline[0]);
                        }
                        paint_path(&painter, outline, Color32::from_gray(95), 1.0);
                    }
                    if index < active {
                        paint_path(
                            &painter,
                            stroke.curve.iter().copied().map(project).collect(),
                            TRACE_COLOR,
                            2.0,
                        );
                    } else if index == active {
                        let last = ((stroke.curve.len() - 1) as f64 * progress).floor() as usize;
                        let mut visible: Vec<_> =
                            stroke.curve[..=last].iter().copied().map(project).collect();
                        if let Some(tip) = chain.last() {
                            visible.push(project(*tip));
                        }
                        paint_path(&painter, visible, TRACE_COLOR, 2.0);
                    }
                }
                paint_chain(
                    &painter,
                    &chain,
                    &app.drawing.strokes[active].coefficients,
                    scale,
                    &project,
                    app.show_circles,
                    false,
                );
            }
            Mode::Demo => {
                paint_path(
                    &painter,
                    app.simulation
                        .trace()
                        .iter()
                        .copied()
                        .map(project)
                        .collect(),
                    TRACE_COLOR,
                    2.0,
                );
                paint_chain(
                    &painter,
                    &app.simulation.points(),
                    app.simulation.epicycles(),
                    scale,
                    &project,
                    app.show_circles,
                    true,
                );
            }
        }
    });
}

fn paint_path(painter: &egui::Painter, points: Vec<Pos2>, color: Color32, width: f32) {
    if points.len() >= 2 {
        painter.add(egui::Shape::line(points, Stroke::new(width, color)));
    }
}

fn paint_chain(
    painter: &egui::Painter,
    points: &[[f64; 2]],
    coefficients: &[Epicycle],
    scale: f32,
    project: &impl Fn([f64; 2]) -> Pos2,
    show: bool,
    labels: bool,
) {
    if points.is_empty() {
        return;
    }
    for (index, e) in coefficients.iter().enumerate() {
        let start = project(points[index]);
        let end = project(points[index + 1]);
        let color = COLORS[index % COLORS.len()];
        if show && e.radius as f32 * scale >= 0.6 {
            if e.frequency != 0.0 {
                painter.circle_stroke(
                    start,
                    e.radius as f32 * scale,
                    Stroke::new(1.0, color.gamma_multiply(0.45)),
                );
            }
            painter.arrow(start, end - start, Stroke::new(1.5, color));
        }
        if labels {
            painter.text(
                end + Vec2::new(10.0, -10.0),
                egui::Align2::LEFT_BOTTOM,
                format!("P{}", index + 1),
                FontId::proportional(14.0),
                color,
            );
        }
    }
    painter.circle_filled(project(points[0]), 3.0, Color32::WHITE);
    painter.circle_filled(project(*points.last().unwrap()), 4.0, TRACE_COLOR);
}
