//! Where saves go.
//!
//! On the desktop a save is a file, named by the path in the save box. In a
//! web browser there are no files, so a save is kept in the browser's local
//! storage under that same name instead. It stays there between visits, but
//! only in that browser on that device.

/// Writes the text of a save under the given name.
#[cfg(not(target_arch = "wasm32"))]
pub fn write(name: &str, text: &str) -> Result<(), String> {
    std::fs::write(name, text).map_err(|e| e.to_string())
}

/// Reads back the text of a save.
#[cfg(not(target_arch = "wasm32"))]
pub fn read(name: &str) -> Result<String, String> {
    std::fs::read_to_string(name).map_err(|e| e.to_string())
}

/// Saves are kept under this prefix so they don't mix with anything else
/// the site keeps in the browser.
#[cfg(target_arch = "wasm32")]
const PREFIX: &str = "varif-save:";

#[cfg(target_arch = "wasm32")]
fn local_storage() -> Result<web_sys::Storage, String> {
    web_sys::window()
        .ok_or("there is no browser window")?
        .local_storage()
        .map_err(|_| "the browser refused access to its storage".to_string())?
        .ok_or_else(|| "this browser has no local storage".to_string())
}

#[cfg(target_arch = "wasm32")]
pub fn write(name: &str, text: &str) -> Result<(), String> {
    local_storage()?
        .set_item(&format!("{PREFIX}{name}"), text)
        .map_err(|_| "the browser's storage is full or turned off".to_string())
}

#[cfg(target_arch = "wasm32")]
pub fn read(name: &str) -> Result<String, String> {
    local_storage()?
        .get_item(&format!("{PREFIX}{name}"))
        .map_err(|_| "the browser refused access to its storage".to_string())?
        .ok_or_else(|| format!("there is no save called {name} in this browser"))
}
