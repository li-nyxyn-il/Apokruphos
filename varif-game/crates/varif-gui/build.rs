//! Embeds the people files in `data/people` for the web build.
//!
//! A browser page has no files to read, so every `*.ron` file in that folder
//! is written into `people.rs` as one expression, a slice of (key, text) pairs,
//! so the web build can use it as the value of a constant. Only the web build
//! includes it; native builds read the folder when they start.

use std::path::Path;
use std::{env, fs};

fn main() {
    let manifest = env::var("CARGO_MANIFEST_DIR").expect("cargo sets CARGO_MANIFEST_DIR");
    let out = env::var("OUT_DIR").expect("cargo sets OUT_DIR");
    let dir = Path::new(&manifest).join("../../data/people");
    // Cargo scans a watched folder for changes to the files inside it, so the
    // folder alone covers added, removed and edited people files.
    println!("cargo:rerun-if-changed={}", dir.display());

    let mut files: Vec<(String, String)> = Vec::new();
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("ron") {
                continue;
            }
            let Some(key) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            let absolute = path.canonicalize().unwrap_or(path.clone());
            // include_str! takes a Rust string, so write the path as a debug
            // string, which escapes backslashes. The `\\?\` prefix Windows adds
            // to canonical paths is dropped.
            let text = absolute.display().to_string();
            let text = text.strip_prefix(r"\\?\").unwrap_or(&text).to_string();
            files.push((key.to_string(), text));
        }
    }
    files.sort();

    let mut source = String::from("&[\n");
    for (key, path) in &files {
        source.push_str(&format!("    ({key:?}, include_str!({path:?})),\n"));
    }
    source.push_str("]\n");
    fs::write(Path::new(&out).join("people.rs"), source).expect("write people.rs");
}
