use crate::desktop::app::FourierApp;
use eframe::egui::{self, Color32, Stroke, Vec2};

pub(crate) fn image_panel(ctx: &egui::Context, app: &mut FourierApp) -> bool {
    let mut changed = false;
    egui::SidePanel::left("image_pipeline")
        .resizable(true)
        .default_width(330.0)
        .width_range(280.0..=440.0)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("1 / Extract outlines and details");
                ui.checkbox(
                    &mut app.settings.include_details,
                    "Include internal details (color edges)",
                );

                if app.settings.include_details {
                    let mut sensitivity = 101.0 - app.settings.edge_threshold;
                    if ui
                        .add(
                            egui::Slider::new(&mut sensitivity, 1.0..=95.0)
                                .text("Detail sensitivity"),
                        )
                        .changed()
                    {
                        app.settings.edge_threshold = 101.0 - sensitivity;
                    }
                    ui.add(
                        egui::Slider::new(&mut app.settings.min_stroke_length, 3.0..=40.0)
                            .text("Min stroke length (px)"),
                    );
                    ui.small(
                        "Higher sensitivity captures softer color boundaries; longer minimum strokes remove small marks.",
                    );
                }

                ui.collapsing("Outer silhouette settings", |ui| {
                    ui.checkbox(&mut app.settings.automatic, "Automatic threshold (Otsu)");
                    ui.add_enabled_ui(!app.settings.automatic, |ui| {
                        ui.add(
                            egui::Slider::new(&mut app.settings.threshold, 0..=255)
                                .text("Threshold"),
                        );
                    });
                    ui.checkbox(&mut app.settings.invert, "Light subject on dark background");
                    ui.checkbox(&mut app.settings.denoise, "Median denoise for silhouette");
                });
                egui::ComboBox::from_label("Sampling quality")
                    .selected_text(app.sample_count.to_string())
                    .show_ui(ui, |ui| {
                        for count in [128, 256, 512, 1024] {
                            ui.selectable_value(&mut app.sample_count, count, count.to_string());
                        }
                    });

                let pending = app
                    .document
                    .as_ref()
                    .is_some_and(|doc| doc.settings != app.settings || doc.sample_count != app.sample_count);
                if ui
                    .add_enabled(
                        app.document.is_some(),
                        egui::Button::new("Apply image settings"),
                    )
                    .clicked()
                {
                    app.apply_processing(ctx);
                    changed = true;
                }
                if pending {
                    ui.label("Apply to update extraction and drawing.");
                }

                ui.separator();

                if ui.checkbox(&mut app.draw_all, "Draw all strokes").changed() {
                    app.sync_reconstruction();
                    changed = true;
                }

                let mut selection = None;
                if let Some(doc) = &app.document {
                    ui.label(&doc.source.name);
                    ui.small(format!(
                        "Original {} x {} | Working {} x {} | Threshold {}",
                        doc.source.original_size[0],
                        doc.source.original_size[1],
                        doc.source.rgb.width(),
                        doc.source.rgb.height(),
                        doc.extraction.threshold
                    ));
                    ui.horizontal_wrapped(|ui| {
                        for (index, label) in ["Original", "Gray", "Mask", "Edges", "Strokes"]
                            .iter()
                            .enumerate()
                        {
                            ui.selectable_value(&mut app.preview_kind, index, *label);
                        }
                    });

                    let texture = match app.preview_kind {
                        1 => &doc.gray_texture,
                        2 => &doc.mask_texture,
                        3 => &doc.edges_texture,
                        _ => &doc.original_texture,
                    };
                    let size = texture.size_vec2();
                    let factor = (ui.available_width() / size.x).min(220.0 / size.y);
                    let response = ui.add(egui::Image::new(texture).fit_to_exact_size(size * factor));

                    if app.preview_kind == 4 {
                        for (index, contour) in doc.extraction.contours.iter().enumerate() {
                            if !app.draw_all && index != doc.selected {
                                continue;
                            }

                            let mut points: Vec<_> = contour
                                .points
                                .iter()
                                .map(
                                    |p| response.rect.min
                                        + Vec2::new(p[0] as f32, p[1] as f32) * factor,
                                )
                                .collect();
                            if contour.closed && !points.is_empty() {
                                points.push(points[0]);
                            }

                            let color = if contour.detail {
                                Color32::from_rgb(235, 50, 110)
                            } else {
                                Color32::from_rgb(30, 160, 220)
                            };
                            if points.len() > 1 {
                                ui.painter()
                                    .add(egui::Shape::line(points, Stroke::new(1.5, color)));
                            }
                        }
                    }

                    ui.separator();
                    ui.heading("2 / Inspect strokes");

                    let details = doc.extraction.contours.iter().filter(|c| c.detail).count();
                    ui.label(format!(
                        "{} outer outlines + {} internal strokes",
                        doc.extraction.contours.len() - details,
                        details
                    ));

                    if doc.strokes.is_empty() {
                        ui.colored_label(
                            Color32::LIGHT_RED,
                            "No usable stroke. Adjust the extraction settings and Apply.",
                        );
                    } else {
                        let mut selected = doc.selected;
                        egui::ComboBox::from_id_salt("stroke_selector")
                            .selected_text(format!("Stroke {} / {}", selected + 1, doc.strokes.len()))
                            .show_ui(ui, |ui| {
                                for (index, contour) in doc.extraction.contours.iter().enumerate() {
                                    let kind = if contour.detail { "detail" } else { "outline" };
                                    let shape = if contour.closed { "closed" } else { "open" };
                                    ui.selectable_value(
                                        &mut selected,
                                        index,
                                        format!(
                                            "{}: {} / {}, {:.0} px",
                                            index + 1,
                                            kind,
                                            shape,
                                            contour.length()
                                        ),
                                    );
                                }
                            });

                        if selected != doc.selected {
                            selection = Some(selected);
                        }
                        ui.small(
                            "Disable Draw all strokes to inspect one stroke in the main canvas.",
                        );

                        if doc.extraction.contours.iter().any(|c| c.touches_edge) {
                            ui.colored_label(
                                Color32::from_rgb(255, 195, 95),
                                "The silhouette touches the crop boundary.",
                            );
                        }
                        if doc.extraction.omitted_details > 0 {
                            ui.label(format!(
                                "{} extra short detail strokes omitted (128 detail limit). Increase Min stroke length to reduce noise.",
                                doc.extraction.omitted_details
                            ));
                        }
                    }
                }

                if let Some(index) = selection {
                    app.select_contour(index);
                    changed = true;
                }

                ui.separator();
                ui.heading("3 / Draw independent strokes");
                ui.label(
                    "Each stroke keeps its position. The pen lifts between strokes without connecting lines.",
                );
                ui.small(
                    "Open routes use an out-and-back extension; only the forward half is drawn. Completed drawings stay visible.",
                );
            });
        });

    changed
}
