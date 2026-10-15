use std::io::Cursor;

use image::{ImageReader, Rgb, RgbImage};

/// 最长边缩至 768 像素，透明区域合成到白色背景。
#[derive(Clone)]
pub struct SourceImage {
    pub name: String,
    pub original_size: [u32; 2],
    pub rgb: RgbImage,
}

impl SourceImage {
    pub fn decode(name: String, bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > 32 * 1024 * 1024 {
            return Err("Image file is too large (maximum 32 MB).".into());
        }

        let mut reader = ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()
            .map_err(|e| e.to_string())?;
        let mut limits = image::Limits::default();
        limits.max_image_width = Some(12000);
        limits.max_image_height = Some(12000);
        limits.max_alloc = Some(128 * 1024 * 1024);
        reader.limits(limits);

        let decoded = reader
            .decode()
            .map_err(|e| format!("Cannot decode image: {e}"))?;
        let original_size = [decoded.width(), decoded.height()];
        if original_size.contains(&0) {
            return Err("Image is empty.".into());
        }

        let rgba = if decoded.width().max(decoded.height()) > 768 {
            decoded
                .resize(768, 768, image::imageops::FilterType::Triangle)
                .into_rgba8()
        } else {
            decoded.into_rgba8()
        };
        let rgb = RgbImage::from_fn(rgba.width(), rgba.height(), |x, y| {
            let pixel = rgba.get_pixel(x, y).0;
            let alpha = u32::from(pixel[3]);
            Rgb(std::array::from_fn(|i| {
                ((u32::from(pixel[i]) * alpha + 255 * (255 - alpha) + 127) / 255) as u8
            }))
        });

        Ok(Self {
            name,
            original_size,
            rgb,
        })
    }
}
