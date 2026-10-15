//! 把图像处理结果组织成可预览、可重建的文档。

use eframe::egui::{self, TextureHandle, TextureOptions};

use crate::{
    image_processing::{self, Extraction, ProcessingSettings, SourceImage},
    math::{FourierStroke, InputStroke, fit_strokes},
};

pub(super) struct Document {
    pub source: SourceImage,
    pub extraction: Extraction,
    pub settings: ProcessingSettings,
    pub selected: usize,
    pub sample_count: usize,
    pub strokes: Vec<FourierStroke>,
    pub original_texture: TextureHandle,
    pub gray_texture: TextureHandle,
    pub mask_texture: TextureHandle,
    pub edges_texture: TextureHandle,
}

impl Document {
    pub fn new(
        ctx: &egui::Context,
        source: SourceImage,
        settings: ProcessingSettings,
        sample_count: usize,
    ) -> Result<Self, String> {
        let extraction = image_processing::extract(&source, settings);
        let size = [source.rgb.width() as usize, source.rgb.height() as usize];
        let original_texture = ctx.load_texture(
            "original",
            egui::ColorImage::from_rgb(size, source.rgb.as_raw()),
            TextureOptions::LINEAR,
        );
        let gray_texture = ctx.load_texture(
            "grayscale",
            egui::ColorImage::from_gray(size, extraction.gray.as_raw()),
            TextureOptions::LINEAR,
        );
        let mask_texture = ctx.load_texture(
            "mask",
            egui::ColorImage::from_gray(size, extraction.mask.as_raw()),
            TextureOptions::NEAREST,
        );
        let edges_texture = ctx.load_texture(
            "edges",
            egui::ColorImage::from_gray(size, extraction.edges.as_raw()),
            TextureOptions::NEAREST,
        );

        let strokes = fit_strokes(
            extraction
                .contours
                .iter()
                .map(|contour| InputStroke {
                    points: contour.points.clone(),
                    closed: contour.closed,
                })
                .collect(),
            sample_count,
        )?;

        let document = Self {
            source,
            extraction,
            settings,
            selected: 0,
            sample_count,
            strokes,
            original_texture,
            gray_texture,
            mask_texture,
            edges_texture,
        };

        Ok(document)
    }

    pub fn max_terms(&self) -> usize {
        self.strokes
            .iter()
            .map(|s| s.coefficients.len())
            .max()
            .unwrap_or(0)
    }
}
