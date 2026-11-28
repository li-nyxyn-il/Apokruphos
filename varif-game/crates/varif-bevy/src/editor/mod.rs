//! The NPC editor: an egui screen over the Bevy window, started with
//! `cargo bevy -- --edit [data file]`.
//!
//! The layout is a people list on the left with its problems beneath, and
//! on the right a tab per part of a person. Each tab is a module with a
//! `ui(ui, file, context)` function; to add one, add a [`Tab`] variant,
//! a module, and a line in [`tab_ui`].

mod factions;
mod logic;
mod sheet;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use std::{
    fs,
    path::{Path, PathBuf},
};
use varif_core::{load_world_with_people, read_people_dir, Hostility, PersonFile};

use logic::{
    blank_person, copy_key, delete_person, find_problems, is_saved, people_from, person_from_main,
    person_path, unique_key, write_person, NEW_KEY,
};

/// The tabs of the editor, in the order they are shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    Sheet,
    Inventory,
    Location,
    Factions,
    Sprites,
    Dialogue,
}

impl Tab {
    pub const ALL: [Tab; 6] = [
        Tab::Sheet,
        Tab::Inventory,
        Tab::Location,
        Tab::Factions,
        Tab::Sprites,
        Tab::Dialogue,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Tab::Sheet => "Sheet",
            Tab::Inventory => "Inventory",
            Tab::Location => "Location & schedule",
            Tab::Factions => "Factions",
            Tab::Sprites => "Sprites",
            Tab::Dialogue => "Dialogue",
        }
    }
}

/// What the tabs need to know about the world, built once from the loaded
/// data file.
#[derive(Resource, Clone, Debug)]
pub struct EditorContext {
    /// Each place's key and name, with its sites' keys and names.
    pub places: Vec<(String, String, Vec<(String, String)>)>,
    /// The live hostility table, for showing how factions stand.
    pub hostility: Hostility,
}

/// Everything the editor is holding.
#[derive(Resource)]
pub struct EditorState {
    /// The text of the main data file, which every check loads against.
    pub main_text: String,
    /// The folder the data file is in. People are saved to `people/` in it.
    pub data_dir: PathBuf,
    /// Everyone in the list, with their unsaved edits.
    pub people: Vec<PersonFile>,
    /// True for a person who came from the main data file. Such a person
    /// is marked until they are saved to `data/people/`.
    pub from_file: Vec<bool>,
    /// True for a person edited since they were loaded or last saved.
    pub edited: Vec<bool>,
    /// The key each person had when loaded or last saved, to notice a
    /// rename.
    pub saved_key: Vec<String>,
    /// Keys in the main data file, to know when a save shadows one.
    pub main_keys: Vec<String>,
    /// The people files on disk now, as (file stem, text), for the checks.
    pub disk: Vec<(String, String)>,
    pub selected: Option<usize>,
    pub tab: Tab,
    /// True when the selected person's problems need working out again.
    /// The check reloads the whole data file, so it only runs after a
    /// change.
    pub dirty: bool,
    /// The selected person's problems, errors first, ready to show.
    pub problems: Vec<String>,
    /// True when `problems` holds an error. That stops a save.
    pub has_error: bool,
    pub status: String,
    /// True while the delete confirmation is open.
    pub confirm_delete: bool,
}

impl EditorState {
    /// Reads the data file and the people files beside it.
    pub fn load(path: &Path) -> Result<(EditorState, EditorContext), String> {
        let main_text = fs::read_to_string(path)
            .map_err(|error| format!("Could not open {}: {error}", path.display()))?;
        let data_dir = path
            .parent()
            .map(Path::to_path_buf)
            .filter(|dir| !dir.as_os_str().is_empty())
            .unwrap_or_else(|| PathBuf::from("."));
        let disk = read_people_dir(&logic::people_dir(&data_dir)).map_err(|error| {
            format!(
                "Could not read the people files beside {}: {error}",
                path.display()
            )
        })?;
        let world = load_world_with_people(&main_text, &disk)
            .map_err(|error| format!("Could not load {}: {error}", path.display()))?;
        let main_keys: Vec<String> = people_from(&main_text, &[])?
            .into_iter()
            .map(|file| file.key)
            .collect();
        let people = people_from(&main_text, &disk)?;

        let context = EditorContext {
            places: world
                .places
                .iter()
                .map(|place| {
                    let sites = place
                        .sites
                        .iter()
                        .map(|site| (site.key.clone(), site.name.clone()))
                        .collect();
                    (place.key.clone(), place.name.clone(), sites)
                })
                .collect(),
            hostility: world.hostility.clone(),
        };
        let state = EditorState {
            from_file: people
                .iter()
                .map(|file| main_keys.contains(&file.key))
                .collect(),
            edited: vec![false; people.len()],
            saved_key: people.iter().map(|file| file.key.clone()).collect(),
            main_keys,
            people,
            main_text,
            data_dir,
            disk,
            selected: None,
            tab: Tab::Sheet,
            dirty: false,
            problems: Vec::new(),
            has_error: false,
            status: String::new(),
            confirm_delete: false,
        };
        Ok((state, context))
    }

    fn keys_except(&self, skip: usize) -> Vec<&str> {
        self.people
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != skip)
            .map(|(_, file)| file.key.as_str())
            .collect()
    }

    fn all_keys(&self) -> Vec<&str> {
        self.people.iter().map(|file| file.key.as_str()).collect()
    }

    /// True when the person at `index` should be marked: from the main
    /// data file and not yet in `data/people/`.
    fn is_unsaved_main_person(&self, index: usize) -> bool {
        self.from_file[index] && !is_saved(&self.disk, &self.people[index].key)
    }

    /// Picks someone, or no one.
    fn select(&mut self, index: Option<usize>) {
        self.selected = index;
        self.confirm_delete = false;
        self.dirty = true;
    }

    /// Works out the selected person's problems again, if they are stale.
    fn refresh_problems(&mut self) {
        if !self.dirty {
            return;
        }
        self.dirty = false;
        self.problems.clear();
        self.has_error = false;
        let Some(index) = self.selected else {
            return;
        };
        let found = find_problems(
            &self.main_text,
            &self.disk,
            &self.people[index],
            &self.keys_except(index),
        );
        self.has_error = !found.errors.is_empty();
        self.problems = found
            .errors
            .into_iter()
            .map(|message| format!("Error: {message}"))
            .chain(
                found
                    .warnings
                    .into_iter()
                    .map(|message| format!("Warning: {message}")),
            )
            .collect();
    }

    fn reload_disk(&mut self) {
        match read_people_dir(&logic::people_dir(&self.data_dir)) {
            Ok(disk) => self.disk = disk,
            Err(error) => self.status = format!("Could not re-read data/people/: {error}"),
        }
    }

    fn add_person(&mut self, file: PersonFile, from_file: bool) {
        self.saved_key.push(file.key.clone());
        self.people.push(file);
        self.from_file.push(from_file);
        self.edited.push(true);
        self.select(Some(self.people.len() - 1));
    }

    fn remove_person(&mut self, index: usize) {
        self.people.remove(index);
        self.from_file.remove(index);
        self.edited.remove(index);
        self.saved_key.remove(index);
        let next = if self.people.is_empty() {
            None
        } else {
            Some(index.min(self.people.len() - 1))
        };
        self.select(next);
    }

    fn new_person(&mut self, first_place: &str, first_site: Option<&str>) {
        let key = unique_key(NEW_KEY, &self.all_keys());
        self.add_person(blank_person(&key, first_place, first_site), false);
        self.status = format!("Started {key}. It is not saved yet.");
    }

    fn duplicate(&mut self) {
        let Some(index) = self.selected else {
            return;
        };
        let mut file = self.people[index].clone();
        file.key = copy_key(&file.key, &self.all_keys());
        self.status = format!("Copied to {}. It is not saved yet.", file.key);
        self.add_person(file, false);
    }

    fn save(&mut self) {
        let Some(index) = self.selected else {
            return;
        };
        if self.has_error {
            return;
        }
        let file = &self.people[index];
        match write_person(&self.data_dir, file) {
            Ok(path) => {
                let key = file.key.clone();
                let old_key = std::mem::replace(&mut self.saved_key[index], key.clone());
                let mut status = format!("Saved {}.", path.display());
                if self.main_keys.contains(&key) {
                    status = format!(
                        "Saved. The copy of {key} in first_leg.ron is now shadowed and can be deleted by hand."
                    );
                }
                if old_key != key && is_saved(&self.disk, &old_key) {
                    status.push_str(&format!(
                        " The key changed from {old_key}, so {old_key}.ron is still in data/people/ until you Delete it."
                    ));
                }
                self.status = status;
                self.edited[index] = false;
                self.reload_disk();
                self.dirty = true;
            }
            Err(error) => self.status = format!("Could not save: {error}"),
        }
    }

    fn delete(&mut self) {
        self.confirm_delete = false;
        let Some(index) = self.selected else {
            return;
        };
        let key = self.people[index].key.clone();
        if !is_saved(&self.disk, &key) {
            if self.from_file[index] {
                self.status = "Not in data/people/ \u{2014} nothing to delete".to_string();
            } else {
                self.remove_person(index);
                self.status = format!("Dropped the unsaved person {key}.");
            }
            return;
        }
        match delete_person(&self.data_dir, &key) {
            Ok(_) => {
                self.reload_disk();
                self.status = format!("Deleted {}.", person_path(&self.data_dir, &key).display());
                match self.from_file[index].then(|| person_from_main(&self.main_text, &key)) {
                    Some(Some(original)) => {
                        self.people[index] = original;
                        self.saved_key[index] = key.clone();
                        self.edited[index] = false;
                        self.status
                            .push_str(" The first_leg.ron copy is back in the list.");
                        self.dirty = true;
                    }
                    _ => self.remove_person(index),
                }
            }
            Err(error) => self.status = format!("Could not delete: {error}"),
        }
    }
}

/// Starts the editor: the resources and the egui system.
pub struct EditorPlugin {
    pub data_path: PathBuf,
}

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        match EditorState::load(&self.data_path) {
            Ok((state, context)) => {
                app.insert_resource(state)
                    .insert_resource(context)
                    .add_systems(Startup, spawn_camera)
                    .add_systems(EguiPrimaryContextPass, editor_ui);
            }
            Err(message) => {
                eprintln!("{message}");
                std::process::exit(1);
            }
        }
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// What a click in the people panel asks for. Collected while drawing and
/// carried out afterwards, so the drawing code does not mutate the list it
/// is walking.
enum Action {
    Select(usize),
    New,
    Duplicate,
    AskDelete,
    ConfirmDelete,
    CancelDelete,
    Save,
}

fn editor_ui(
    mut contexts: EguiContexts,
    mut state: ResMut<EditorState>,
    context: Res<EditorContext>,
) -> Result {
    let egui_ctx = contexts.ctx_mut()?.clone();
    let state = &mut *state;
    state.refresh_problems();

    let mut actions: Vec<Action> = Vec::new();
    let mut root = egui::Ui::new(
        egui_ctx.clone(),
        "editor_root".into(),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(egui_ctx.viewport_rect()),
    );

    egui::Panel::left("people_panel")
        .default_size(280.0)
        .show(&mut root, |ui| people_panel(ui, state, &mut actions));
    egui::Panel::bottom("status_panel").show(&mut root, |ui| {
        ui.label(if state.status.is_empty() {
            "Ready."
        } else {
            &state.status
        });
    });
    egui::CentralPanel::default().show(&mut root, |ui| central_panel(ui, state, &context));

    if state.confirm_delete {
        if let Some(index) = state.selected {
            let key = state.people[index].key.clone();
            egui::Window::new("Delete person")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(&egui_ctx, |ui| {
                    ui.label(format!(
                        "Delete {key}.ron from data/people/? first_leg.ron is not touched."
                    ));
                    ui.horizontal(|ui| {
                        if ui.button("Delete").clicked() {
                            actions.push(Action::ConfirmDelete);
                        }
                        if ui.button("Cancel").clicked() {
                            actions.push(Action::CancelDelete);
                        }
                    });
                });
        }
    }

    // A place with sites needs one named, so a new person stands at the
    // first place's first site.
    let (first_place, first_site) = context
        .places
        .first()
        .map(|(key, _, sites)| (key.clone(), sites.first().map(|(site, _)| site.clone())))
        .unwrap_or_default();
    for action in actions {
        match action {
            Action::Select(index) => state.select(Some(index)),
            Action::New => state.new_person(&first_place, first_site.as_deref()),
            Action::Duplicate => state.duplicate(),
            Action::AskDelete => state.confirm_delete = true,
            Action::ConfirmDelete => state.delete(),
            Action::CancelDelete => state.confirm_delete = false,
            Action::Save => state.save(),
        }
    }
    Ok(())
}

fn people_panel(ui: &mut egui::Ui, state: &EditorState, actions: &mut Vec<Action>) {
    ui.heading("People");
    ui.horizontal_wrapped(|ui| {
        if ui.button("New").clicked() {
            actions.push(Action::New);
        }
        let selected = state.selected.is_some();
        if ui
            .add_enabled(selected, egui::Button::new("Duplicate"))
            .clicked()
        {
            actions.push(Action::Duplicate);
        }
        if ui
            .add_enabled(selected, egui::Button::new("Delete"))
            .clicked()
        {
            actions.push(Action::AskDelete);
        }
        let can_save = selected && !state.has_error;
        if ui
            .add_enabled(can_save, egui::Button::new("Save"))
            .clicked()
        {
            actions.push(Action::Save);
        }
    });
    ui.small("* from first_leg.ron, not yet saved to data/people/");
    ui.separator();

    let list_height = (ui.available_height() * 0.6).max(120.0);
    egui::ScrollArea::vertical()
        .id_salt("people_list")
        .max_height(list_height)
        .show(ui, |ui| {
            for (index, file) in state.people.iter().enumerate() {
                let mut label = format!("{} ({})", file.name, file.key);
                if state.is_unsaved_main_person(index) {
                    label.push_str(" *");
                }
                if state.edited[index] {
                    label.push_str(" \u{2022}");
                }
                let chosen = state.selected == Some(index);
                if ui.selectable_label(chosen, label).clicked() {
                    actions.push(Action::Select(index));
                }
            }
        });

    ui.separator();
    ui.strong("Problems");
    egui::ScrollArea::vertical()
        .id_salt("problems")
        .show(ui, |ui| {
            if state.selected.is_none() {
                ui.weak("Nobody selected.");
            } else if state.problems.is_empty() {
                ui.label("None.");
            }
            for problem in &state.problems {
                let colour = if problem.starts_with("Error") {
                    egui::Color32::LIGHT_RED
                } else {
                    egui::Color32::YELLOW
                };
                ui.add(egui::Label::new(egui::RichText::new(problem).color(colour)).wrap());
            }
        });
}

fn central_panel(ui: &mut egui::Ui, state: &mut EditorState, context: &EditorContext) {
    let EditorState {
        people,
        selected,
        tab,
        dirty,
        edited,
        ..
    } = state;
    let Some(index) = *selected else {
        ui.label("Pick someone from the list, or start a New person.");
        return;
    };
    let file = &mut people[index];
    let before = file.clone();

    egui::Grid::new("person_header")
        .num_columns(2)
        .show(ui, |ui| {
            ui.label("Key");
            ui.text_edit_singleline(&mut file.key);
            ui.end_row();
            ui.label("Name");
            ui.text_edit_singleline(&mut file.name);
            ui.end_row();
            ui.label("Title");
            ui.text_edit_singleline(&mut file.title);
            ui.end_row();
        });
    ui.separator();
    ui.horizontal_wrapped(|ui| {
        for option in Tab::ALL {
            ui.selectable_value(tab, option, option.label());
        }
    });
    ui.separator();
    egui::ScrollArea::vertical()
        .id_salt("tab_body")
        .show(ui, |ui| tab_ui(*tab, ui, file, context));

    if *file != before {
        edited[index] = true;
        *dirty = true;
    }
}

/// Draws the body of one tab. Tabs not built yet say so.
fn tab_ui(tab: Tab, ui: &mut egui::Ui, file: &mut PersonFile, context: &EditorContext) {
    match tab {
        Tab::Sheet => sheet::ui(ui, file, context),
        Tab::Factions => factions::ui(ui, file, context),
        Tab::Inventory | Tab::Location | Tab::Sprites | Tab::Dialogue => {
            ui.label("Coming in a later task");
        }
    }
}

/// A dropdown over a fixed list of choices, for the tabs to share.
pub fn combo<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    id: &str,
    value: &mut T,
    all: &[T],
    name: fn(T) -> &'static str,
) {
    egui::ComboBox::from_id_salt(id)
        .selected_text(name(*value))
        .show_ui(ui, |ui| {
            for &option in all {
                ui.selectable_value(value, option, name(option));
            }
        });
}
