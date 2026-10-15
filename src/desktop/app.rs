use std::{path::Path, time::Duration};

use eframe::egui;

use crate::{
    animation::{Drawing, Simulation},
    image_processing::{ProcessingSettings, SourceImage},
};

use super::{document::Document, ui, viewport::Viewport};

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Mode {
    Image,
    Demo,
}

pub(super) struct FourierApp {
    pub simulation: Simulation,
    pub drawing: Drawing,
    pub viewport: Viewport,
    pub draw_all: bool,
    pub document: Option<Document>,
    pub mode: Mode,
    pub settings: ProcessingSettings,
    pub sample_count: usize,
    pub term_count: usize,
    pub show_circles: bool,
    pub show_reference: bool,
    pub preview_kind: usize,
    pub error: Option<String>,
}

impl FourierApp {
    pub fn new(ctx: &egui::Context) -> Self {
        let mut app = Self {
            simulation: Simulation::default(),
            drawing: Drawing::default(),
            viewport: Viewport::default(),
            draw_all: true,
            document: None,
            mode: Mode::Image,
            settings: ProcessingSettings::default(),
            sample_count: 512,
            term_count: 256,
            show_circles: true,
            show_reference: true,
            preview_kind: 0,
            error: None,
        };

        app.load_sample(ctx);
        app
    }

    pub fn load_sample(&mut self, ctx: &egui::Context) {
        self.load_bytes(
            ctx,
            "images.jpeg".into(),
            include_bytes!("../../images.jpeg"),
        );
    }

    pub fn load_path(&mut self, ctx: &egui::Context, path: &Path) {
        let result = (|| {
            let size = std::fs::metadata(path).map_err(|e| e.to_string())?.len();
            if size > 32 * 1024 * 1024 {
                return Err("Image file is too large (maximum 32 MB).".to_owned());
            }
            std::fs::read(path).map_err(|e| e.to_string())
        })();

        match result {
            Ok(bytes) => self.load_bytes(
                ctx,
                path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into(),
                &bytes,
            ),
            Err(error) => self.error = Some(error),
        }
    }

    fn load_bytes(&mut self, ctx: &egui::Context, name: String, bytes: &[u8]) {
        let result = SourceImage::decode(name, bytes)
            .and_then(|source| Document::new(ctx, source, self.settings, self.sample_count));
        self.install_document(result);
    }

    fn install_document(&mut self, result: Result<Document, String>) {
        match result {
            Ok(document) => {
                self.document = Some(document);
                self.mode = Mode::Image;
                self.error = None;
                self.sync_reconstruction();
                self.viewport.reset();
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub fn apply_processing(&mut self, ctx: &egui::Context) {
        if let Some(document) = &self.document {
            let result = Document::new(
                ctx,
                document.source.clone(),
                self.settings,
                self.sample_count,
            );
            self.install_document(result);
        }
    }

    pub fn select_contour(&mut self, index: usize) {
        if let Some(document) = &mut self.document {
            if index < document.strokes.len() {
                document.selected = index;
            }
        }

        if !self.draw_all {
            self.sync_reconstruction();
        }
    }

    pub fn sync_reconstruction(&mut self) {
        if let Some(document) = &self.document {
            if document.max_terms() > 0 {
                self.term_count = self.term_count.clamp(1, document.max_terms());
            }

            let strokes: Vec<_> = document
                .strokes
                .iter()
                .enumerate()
                .filter(|(index, _)| self.draw_all || *index == document.selected)
                .map(|(_, stroke)| stroke)
                .collect();
            self.drawing.load(&strokes, self.term_count);
        } else {
            self.drawing.load(&[], self.term_count);
        }
    }

    pub fn paused(&self) -> bool {
        match self.mode {
            Mode::Image => self.drawing.paused(),
            Mode::Demo => self.simulation.paused(),
        }
    }

    pub fn toggle_pause(&mut self) {
        match self.mode {
            Mode::Image => self.drawing.toggle_pause(),
            Mode::Demo => self.simulation.toggle_pause(),
        }
    }

    pub fn restart(&mut self) {
        match self.mode {
            Mode::Image => self.drawing.reset(),
            Mode::Demo => self.simulation.reset(),
        }
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        self.viewport.reset();

        match mode {
            Mode::Image => self.sync_reconstruction(),
            Mode::Demo => self.simulation = Simulation::default(),
        }
    }
}

impl eframe::App for FourierApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let dropped = ctx.input(|input| {
            input
                .raw
                .dropped_files
                .iter()
                .find_map(|file| file.path.clone())
        });
        let mut changed = dropped.is_some();
        if let Some(path) = dropped {
            self.load_path(ctx, &path);
        }

        changed |= ui::controls(ctx, self);
        if self.mode == Mode::Image {
            changed |= ui::image_panel(ctx, self);
        }

        if !changed {
            let delta = f64::from(ctx.input(|input| input.stable_dt));
            match self.mode {
                Mode::Image => self.drawing.advance(delta),
                Mode::Demo => self.simulation.advance(delta),
            }
        }

        ui::readout(ctx, self);
        ui::canvas(ctx, self);

        let moving = match self.mode {
            Mode::Image => self.drawing.is_moving(),
            Mode::Demo => self.simulation.is_moving(),
        };
        if moving {
            ctx.request_repaint_after(Duration::from_millis(16));
        }
    }
}
