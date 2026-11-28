//! The Sprites tab: a named set of pictures for one person, each with a
//! path under `assets/`, an optional frame grid, and a live preview.
//!
//! Preview textures need Bevy resources the plain tab functions do not
//! have, so the tab only talks to [`SpritePreviews`]: it asks for a path
//! to be loaded, and the editor's system loads it and registers it with
//! egui before the next frame.

use bevy::prelude::*;
use bevy_egui::egui;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};
use varif_core::{Frame, PersonFile, Sprite};

/// The biggest the preview is drawn, in either direction.
pub const PREVIEW_MAX: f32 = 256.0;

/// The folder sprites live in, under the assets root.
const SPRITES_DIR: &str = "sprites";

/// The assets root, the same one the `AssetPlugin` is given.
pub fn assets_root() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets"))
}

/// A loaded preview image: its egui texture and its size in pixels.
#[derive(Clone, Copy, Debug)]
pub struct Preview {
    pub texture: egui::TextureId,
    pub size: [f32; 2],
}

/// The PNG files to pick from, and the preview images loaded so far.
#[derive(Resource, Default)]
pub struct SpritePreviews {
    /// Paths relative to `assets/`, sorted: `sprites/foo.png`.
    pub files: Vec<String>,
    /// Image handles by path. Held so the images stay loaded.
    pub handles: HashMap<String, Handle<Image>>,
    /// Paths that have finished loading and been given a texture.
    pub ready: HashMap<String, Preview>,
    /// Paths the tab wants loaded; the system loads them next frame.
    pub wanted: HashSet<String>,
    /// Set by the Refresh button; the system rescans next frame.
    pub rescan: bool,
}

impl SpritePreviews {
    /// A resource with the files under `assets/sprites/` already listed.
    pub fn scanned() -> SpritePreviews {
        SpritePreviews {
            files: scan_pngs(assets_root()),
            ..default()
        }
    }

    /// The preview for a path, once it has loaded.
    pub fn texture_for(&self, path: &str) -> Option<Preview> {
        self.ready.get(path).copied()
    }

    /// Asks for a path to be loaded. Cheap to call every frame.
    pub fn request(&mut self, path: &str) {
        if !path.is_empty() && !self.handles.contains_key(path) {
            self.wanted.insert(path.to_string());
        }
    }

    /// True when the path is a file the scan found.
    pub fn knows(&self, path: &str) -> bool {
        self.files.iter().any(|file| file == path)
    }
}

/// Every `.png` under `root/sprites/`, as paths relative to `root` with
/// forward slashes, sorted. A missing folder gives an empty list.
pub fn scan_pngs(root: &Path) -> Vec<String> {
    fn walk(dir: &Path, relative: &str, found: &mut Vec<String>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let path = entry.path();
            let here = format!("{relative}/{name}");
            if path.is_dir() {
                walk(&path, &here, found);
            } else if name.to_ascii_lowercase().ends_with(".png") {
                found.push(here);
            }
        }
    }
    let mut found = Vec::new();
    walk(&root.join(SPRITES_DIR), SPRITES_DIR, &mut found);
    found.sort();
    found
}

/// The UV rectangle of one cell of a `columns` by `rows` sheet, as
/// (min, max) in 0..1. An index past the end wraps to the last cell
/// instead of panicking; zero columns or rows count as one.
pub fn cell_uv(frame: Frame) -> ([f32; 2], [f32; 2]) {
    let columns = frame.columns.max(1);
    let rows = frame.rows.max(1);
    let index = frame.index.min(columns * rows - 1);
    let (col, row) = (index % columns, index / columns);
    (
        [col as f32 / columns as f32, row as f32 / rows as f32],
        [
            (col + 1) as f32 / columns as f32,
            (row + 1) as f32 / rows as f32,
        ],
    )
}

/// The size to draw something `width` by `height` so it fits within
/// `max` by `max`, keeping its shape. It is never made bigger than it is.
pub fn fit_size(width: f32, height: f32, max: f32) -> [f32; 2] {
    if width <= 0.0 || height <= 0.0 {
        return [0.0, 0.0];
    }
    let scale = (max / width).min(max / height).min(1.0);
    [width * scale, height * scale]
}

/// What the tab's buttons ask for, done after the list is drawn.
enum Change {
    Remove(usize),
    Up(usize),
    Down(usize),
}

pub fn ui(ui: &mut egui::Ui, file: &mut PersonFile, previews: &mut SpritePreviews) {
    ui.horizontal(|ui| {
        if ui.button("Add sprite").clicked() {
            file.sprites.push(Sprite {
                name: next_name(&file.sprites),
                path: String::new(),
                frame: None,
            });
        }
        if ui.button("Refresh files").clicked() {
            previews.rescan = true;
        }
        ui.weak(format!(
            "{} PNG files in assets/sprites/",
            previews.files.len()
        ));
    });
    if file.sprites.is_empty() {
        ui.weak("No sprites yet.");
    }

    let names: Vec<String> = file.sprites.iter().map(|s| s.name.clone()).collect();
    let count = file.sprites.len();
    let mut change = None;
    for (at, sprite) in file.sprites.iter_mut().enumerate() {
        ui.separator();
        let duplicate = names.iter().filter(|name| **name == sprite.name).count() > 1;
        ui.push_id(at, |ui| {
            sprite_ui(ui, sprite, duplicate, at, count, previews, &mut change)
        });
    }
    match change {
        Some(Change::Remove(at)) => {
            file.sprites.remove(at);
        }
        Some(Change::Up(at)) if at > 0 => file.sprites.swap(at, at - 1),
        Some(Change::Down(at)) if at + 1 < count => file.sprites.swap(at, at + 1),
        _ => {}
    }
}

/// A name no sprite of the person has yet.
fn next_name(sprites: &[Sprite]) -> String {
    let mut number = sprites.len() + 1;
    loop {
        let name = format!("sprite-{number}");
        if sprites.iter().all(|s| s.name != name) {
            return name;
        }
        number += 1;
    }
}

fn sprite_ui(
    ui: &mut egui::Ui,
    sprite: &mut Sprite,
    duplicate: bool,
    at: usize,
    count: usize,
    previews: &mut SpritePreviews,
    change: &mut Option<Change>,
) {
    ui.horizontal(|ui| {
        ui.label("Name");
        ui.text_edit_singleline(&mut sprite.name);
        if duplicate {
            ui.colored_label(egui::Color32::LIGHT_RED, "Another sprite has this name");
        }
        if ui.add_enabled(at > 0, egui::Button::new("Up")).clicked() {
            *change = Some(Change::Up(at));
        }
        if ui
            .add_enabled(at + 1 < count, egui::Button::new("Down"))
            .clicked()
        {
            *change = Some(Change::Down(at));
        }
        if ui.button("Remove").clicked() {
            *change = Some(Change::Remove(at));
        }
    });

    ui.horizontal(|ui| {
        ui.label("Path");
        let shown = if sprite.path.is_empty() {
            "(none)"
        } else {
            sprite.path.as_str()
        };
        egui::ComboBox::from_id_salt("path")
            .selected_text(shown)
            .width(260.0)
            .show_ui(ui, |ui| {
                for file in &previews.files {
                    ui.selectable_value(&mut sprite.path, file.clone(), file);
                }
            });
        if !sprite.path.is_empty() && !previews.knows(&sprite.path) {
            ui.colored_label(egui::Color32::YELLOW, "file not found in assets/sprites/");
        }
    });

    let mut sheet = sprite.frame.is_some();
    if ui.checkbox(&mut sheet, "Sprite sheet").changed() {
        sprite.frame = sheet.then_some(Frame {
            columns: 1,
            rows: 1,
            index: 0,
        });
    }
    if let Some(frame) = sprite.frame.as_mut() {
        ui.horizontal(|ui| {
            ui.label("Columns");
            ui.add(egui::DragValue::new(&mut frame.columns).range(1..=256));
            ui.label("Rows");
            ui.add(egui::DragValue::new(&mut frame.rows).range(1..=256));
            let cells = frame.columns.max(1) * frame.rows.max(1);
            ui.label("Frame");
            ui.add(egui::DragValue::new(&mut frame.index));
            if frame.index >= cells {
                ui.colored_label(
                    egui::Color32::LIGHT_RED,
                    format!("Frame must be below {cells}"),
                );
            }
        });
    }

    previews.request(&sprite.path);
    if sprite.path.is_empty() {
        return;
    }
    match previews.texture_for(&sprite.path) {
        Some(preview) => {
            let (min, max) = match sprite.frame {
                Some(frame) => cell_uv(frame),
                None => ([0.0, 0.0], [1.0, 1.0]),
            };
            let (width, height) = (
                preview.size[0] * (max[0] - min[0]),
                preview.size[1] * (max[1] - min[1]),
            );
            let size = fit_size(width, height, PREVIEW_MAX);
            ui.add(
                egui::Image::new(egui::load::SizedTexture::new(
                    preview.texture,
                    egui::vec2(size[0], size[1]),
                ))
                .uv(egui::Rect::from_min_max(
                    egui::pos2(min[0], min[1]),
                    egui::pos2(max[0], max[1]),
                )),
            );
        }
        None if previews.knows(&sprite.path) => {
            ui.weak("Loading preview...");
        }
        None => {}
    }
}

/// Runs each frame before the editor draws: rescans if asked, starts the
/// loads the tab requested, and registers finished images with egui.
/// `register` turns a loaded image's handle into an egui texture.
pub fn update_previews(
    previews: &mut SpritePreviews,
    server: &AssetServer,
    images: &Assets<Image>,
    mut register: impl FnMut(Handle<Image>) -> egui::TextureId,
) {
    if previews.rescan {
        previews.rescan = false;
        previews.files = scan_pngs(assets_root());
        // A changed file on disk is read again.
        for path in previews.handles.keys() {
            server.reload(path.clone());
        }
    }
    for path in std::mem::take(&mut previews.wanted) {
        let handle = server.load(path.clone());
        previews.handles.insert(path, handle);
    }
    let pending: Vec<(String, Handle<Image>)> = previews
        .handles
        .iter()
        .filter(|(path, _)| !previews.ready.contains_key(*path))
        .map(|(path, handle)| (path.clone(), handle.clone()))
        .collect();
    for (path, handle) in pending {
        if let Some(image) = images.get(&handle) {
            let size = [image.width() as f32, image.height() as f32];
            let texture = register(handle);
            previews.ready.insert(path, Preview { texture, size });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::logic::{people_from, scratch_dir, TEST_MAIN};
    use super::*;
    use varif_core::check_person_file;

    fn frame(columns: u32, rows: u32, index: u32) -> Frame {
        Frame {
            columns,
            rows,
            index,
        }
    }

    #[test]
    fn cell_uv_picks_the_cell_by_row_and_column() {
        // 4 x 2 sheet, index 5 is column 1 of row 1.
        let (min, max) = cell_uv(frame(4, 2, 5));
        assert_eq!(min, [0.25, 0.5]);
        assert_eq!(max, [0.5, 1.0]);
        let (min, max) = cell_uv(frame(4, 2, 0));
        assert_eq!(min, [0.0, 0.0]);
        assert_eq!(max, [0.25, 0.5]);
        let (min, max) = cell_uv(frame(1, 1, 0));
        assert_eq!((min, max), ([0.0, 0.0], [1.0, 1.0]));
    }

    #[test]
    fn cell_uv_does_not_panic_on_bad_frames() {
        let (min, max) = cell_uv(frame(2, 2, 99));
        assert_eq!((min, max), ([0.5, 0.5], [1.0, 1.0]));
        let (min, max) = cell_uv(frame(0, 0, 3));
        assert_eq!((min, max), ([0.0, 0.0], [1.0, 1.0]));
    }

    #[test]
    fn fit_size_keeps_the_shape_and_never_grows() {
        assert_eq!(fit_size(512.0, 256.0, 256.0), [256.0, 128.0]);
        assert_eq!(fit_size(100.0, 400.0, 256.0), [64.0, 256.0]);
        assert_eq!(fit_size(32.0, 16.0, 256.0), [32.0, 16.0]);
        assert_eq!(fit_size(0.0, 16.0, 256.0), [0.0, 0.0]);
    }

    #[test]
    fn scan_finds_pngs_sorted_with_relative_paths() {
        let root = scratch_dir();
        let sprites = root.join("sprites");
        fs::create_dir_all(sprites.join("npc")).unwrap();
        fs::write(sprites.join("b.png"), b"").unwrap();
        fs::write(sprites.join("A.PNG"), b"").unwrap();
        fs::write(sprites.join("npc").join("c.png"), b"").unwrap();
        fs::write(sprites.join("notes.txt"), b"").unwrap();
        fs::write(sprites.join(".gitkeep"), b"").unwrap();
        fs::write(root.join("outside.png"), b"").unwrap();

        assert_eq!(
            scan_pngs(&root),
            vec!["sprites/A.PNG", "sprites/b.png", "sprites/npc/c.png"]
        );
    }

    #[test]
    fn scan_of_a_missing_folder_is_empty() {
        let root = scratch_dir();
        assert!(scan_pngs(&root).is_empty());
    }

    #[test]
    fn next_name_skips_taken_names() {
        let sprite = |name: &str| Sprite {
            name: name.to_string(),
            path: String::new(),
            frame: None,
        };
        assert_eq!(next_name(&[]), "sprite-1");
        assert_eq!(next_name(&[sprite("sprite-2")]), "sprite-3");
        assert_eq!(next_name(&[sprite("sprite-2"), sprite("x")]), "sprite-3");
    }

    #[test]
    fn jonnoj_with_a_portrait_and_a_token_passes_the_check() {
        let mut jonnoj = people_from(TEST_MAIN, &[])
            .unwrap()
            .into_iter()
            .find(|person| person.key == "jonnoj")
            .unwrap();
        jonnoj.sprites = vec![
            Sprite {
                name: "portrait".to_string(),
                path: "sprites/jonnoj.png".to_string(),
                frame: None,
            },
            Sprite {
                name: "token".to_string(),
                path: "sprites/tokens.png".to_string(),
                frame: Some(frame(4, 2, 5)),
            },
        ];
        check_person_file(TEST_MAIN, &[], &jonnoj).unwrap();

        // Two sprites with one name are refused by the loader.
        jonnoj.sprites[1].name = "portrait".to_string();
        assert!(check_person_file(TEST_MAIN, &[], &jonnoj).is_err());
    }
}
