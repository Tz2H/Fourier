//! 画布视角：适应图案、以指针为中心缩放，以及拖动平移。

use eframe::egui::{self, Pos2, Rect, Response, Vec2};

pub(super) struct Viewport {
    zoom: f32,
    pan: Vec2,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: Vec2::ZERO,
        }
    }
}

impl Viewport {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    fn zoom_at(&mut self, zoom: f32, anchor: Vec2) {
        let next = zoom.clamp(0.25, 8.0);
        // 保持指针下面的图案坐标不变；按钮和滑块以画布中心为锚点。
        self.pan = anchor - (anchor - self.pan) * (next / self.zoom);
        self.zoom = next;
    }

    pub fn controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label("Canvas zoom");
            if ui.button("- ").on_hover_text("Zoom out").clicked() {
                self.zoom_at(self.zoom / 1.25, Vec2::ZERO);
            }
            let mut percent = self.zoom * 100.0;
            if ui
                .add(
                    egui::Slider::new(&mut percent, 25.0..=800.0)
                        .logarithmic(true)
                        .suffix("%"),
                )
                .changed()
            {
                self.zoom_at(percent / 100.0, Vec2::ZERO);
            }
            if ui.button("+").on_hover_text("Zoom in").clicked() {
                self.zoom_at(self.zoom * 1.25, Vec2::ZERO);
            }
            if ui.button("Fit / reset").clicked() {
                self.reset();
            }
            ui.weak("Wheel: zoom | Drag: pan | Double-click: fit");
        });
    }

    pub fn interact(&mut self, response: &Response) {
        if response.double_clicked() {
            self.reset();
            return;
        }

        self.pan += response.drag_delta();
        if response.hovered() {
            let (factor, pointer) = response.ctx.input_mut(|input| {
                let gesture = input.zoom_delta();
                let factor = if gesture != 1.0 {
                    gesture
                } else {
                    (input.smooth_scroll_delta.y * 0.003).exp()
                };
                input.smooth_scroll_delta = Vec2::ZERO;
                (factor, input.pointer.hover_pos())
            });
            if let Some(pointer) = pointer {
                self.zoom_at(self.zoom * factor, pointer - response.rect.center());
            }
        }
    }

    /// 返回模型原点在屏幕上的位置与单位缩放，所有图元共享同一变换。
    pub fn transform(&self, rect: Rect, bounds: [[f64; 2]; 2]) -> (Pos2, f32) {
        let [min, max] = bounds;
        let size = Vec2::new((max[0] - min[0]) as f32, (max[1] - min[1]) as f32);
        let available = (rect.size() - Vec2::splat(64.0)).max(Vec2::splat(1.0));
        let fit = (available.x / size.x.max(1.0)).min(available.y / size.y.max(1.0));
        let scale = fit * self.zoom;
        let midpoint = Vec2::new((min[0] + max[0]) as f32, -(min[1] + max[1]) as f32) * 0.5;

        (rect.center() + self.pan - midpoint * scale, scale)
    }
}
