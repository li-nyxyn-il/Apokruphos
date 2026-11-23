//! The painted map drawn under the places and roads.
//!
//! The picture is "The Umbara, roads only": the same map as the City Names
//! map the layout file was measured on, without the names, so the window can
//! draw its own. It is built into the program, so the desktop and web builds
//! show the same map and there is no file to lose.
//!
//! The picture is decoded and sent to the graphics card the first time the
//! map is drawn, not at start-up. If the card cannot hold a texture that
//! big (some phones' browsers cap it at 2048 pixels), it is shrunk to fit.
//! Positions stay in the full picture's pixels either way.

use eframe::egui;
use std::cell::OnceCell;

/// The roads-only map, as a PNG file.
const PICTURE: &[u8] = include_bytes!("../../../data/umbara_roads_only.png");

/// The full picture's size in pixels. The layout file's positions are
/// measured on a picture this size.
pub const MAP_SIZE: egui::Vec2 = egui::vec2(2770.0, 2316.0);

/// The sea colour of the picture, shown past its edges.
pub const SEA: egui::Color32 = egui::Color32::from_rgb(97, 115, 117);

/// The map picture, loaded on first use.
#[derive(Default)]
pub struct MapBackground {
    texture: OnceCell<Option<egui::TextureHandle>>,
}

impl MapBackground {
    /// The picture as a texture, or `None` if it could not be decoded.
    pub fn texture(&self, ctx: &egui::Context) -> Option<&egui::TextureHandle> {
        self.texture.get_or_init(|| load(ctx)).as_ref()
    }
}

fn load(ctx: &egui::Context) -> Option<egui::TextureHandle> {
    let mut picture = match image::load_from_memory_with_format(PICTURE, image::ImageFormat::Png)
    {
        Ok(picture) => picture,
        Err(error) => {
            log_error(&format!("Could not decode the map picture: {error}"));
            return None;
        }
    };

    let max_side = ctx.input(|i| i.max_texture_side) as u32;
    if picture.width() > max_side || picture.height() > max_side {
        // Keeps the shape: resize fits the picture inside the box.
        picture = picture.resize(max_side, max_side, image::imageops::FilterType::Triangle);
    }

    let rgba = picture.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    let colour = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
    Some(ctx.load_texture("umbara_roads_only", colour, egui::TextureOptions::LINEAR))
}

#[cfg(not(target_arch = "wasm32"))]
fn log_error(message: &str) {
    eprintln!("{message}");
}

#[cfg(target_arch = "wasm32")]
fn log_error(message: &str) {
    eframe::web_sys::console::error_1(&message.into());
}
