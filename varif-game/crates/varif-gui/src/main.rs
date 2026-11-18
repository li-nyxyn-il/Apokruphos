//! A simple desktop window for the world model, built on egui.
//!
//! Run it from the workspace folder with `cargo run -p varif-gui`, or pass a
//! data file: `cargo run -p varif-gui -- data/first_leg.ron`.
//!
//! The same window also runs in a web browser: see `index.html` and the
//! README for the web build. There the data file is built in and saves go to
//! the browser's own storage instead of to files.
//!
//! On a narrow screen (a phone, or a thin window) the play screen shows one
//! panel at a time, with tabs along the bottom: map, place, party, events.
//! On a touch screen the buttons are also made bigger, to fit a finger.
//!
//! The window opens on a choice: start a new game, or load a save. A new game
//! asks for a character before the map is shown.
//!
//! egui redraws the whole window many times a second. Each redraw reads the
//! `World` and draws it; a click produces an `Action`, which is carried out
//! once drawing is done. The window keeps no copy of the world's state.

mod layout;
mod storage;

use eframe::egui;
use layout::MapLayout;
#[cfg(not(target_arch = "wasm32"))]
use std::{env, fs, path::Path, process};
use varif_core::{
    load_world, Akstada, Blood, Command, EffectKind, Event, FactionId, Heritage, Itinerary,
    MemberFatigue, PersonId, Place, PlaceId, PlaceKind, Route, SiteId, SiteKind, Tongue, World,
    BASE_FATIGUE, BASE_HEALTH, BASE_LEVEL, BASE_RESILIENCE, MAX_AGE, MIN_AGE,
};

#[cfg(not(target_arch = "wasm32"))]
const DEFAULT_DATA: &str = "data/first_leg.ron";
/// Looked for beside the data file.
#[cfg(not(target_arch = "wasm32"))]
const LAYOUT_FILE: &str = "map_layout.ron";
/// A map with many loops has hundreds of ways between two places. Only this
/// many, the quickest, are offered.
const MAX_WAYS: usize = 5;
/// Narrower than this, in egui points, and the play screen shows one panel
/// at a time instead of three side by side.
const COMPACT_WIDTH: f32 = 760.0;
/// How many of the latest events are shown under the map on a narrow screen.
const EVENTS_UNDER_MAP: usize = 3;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    let path = env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_DATA.to_string());
    let world = match read_data(&path) {
        Ok(world) => world,
        Err(error) => {
            eprintln!("Could not load {path}: {error}");
            process::exit(1);
        }
    };

    let layout_path = Path::new(&path).with_file_name(LAYOUT_FILE);
    let (layout, layout_note) = match MapLayout::read(&layout_path) {
        Ok(layout) => (layout, None),
        Err(error) => (
            MapLayout::default(),
            Some(format!("Could not read {}: {error}", layout_path.display())),
        ),
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1240.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Varif",
        options,
        Box::new(move |_creation| Ok(Box::new(App::new(world, layout, layout_note)))),
    )
}

/// The web build: a browser page has no files to read, so the data and the
/// layout are built into the program, and the window draws into the canvas
/// that `index.html` provides.
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    const WEB_DATA: &str = include_str!("../../../data/first_leg.ron");
    const WEB_LAYOUT: &str = include_str!("../../../data/map_layout.ron");

    // Panics show up in the browser's console instead of vanishing.
    std::panic::set_hook(Box::new(|info| {
        eframe::web_sys::console::error_1(&info.to_string().into());
    }));

    let world = load_world(WEB_DATA).expect("the built-in data file should load");
    let (layout, layout_note) = match MapLayout::from_text(WEB_LAYOUT) {
        Ok(layout) => (layout, None),
        Err(error) => (
            MapLayout::default(),
            Some(format!("Could not read the built-in map layout: {error}")),
        ),
    };

    // A phone or tablet says it has touch points. Knowing this from the start
    // means the bigger buttons are there before the first tap, rather than
    // the layout jumping under the player's finger.
    let touch = eframe::web_sys::window()
        .map(|window| window.navigator().max_touch_points() > 0)
        .unwrap_or(false);

    wasm_bindgen_futures::spawn_local(async move {
        let document = eframe::web_sys::window()
            .and_then(|window| window.document())
            .expect("the page should have a document");
        let canvas = document
            .get_element_by_id("varif_canvas")
            .expect("index.html should have a canvas called varif_canvas")
            .dyn_into::<eframe::web_sys::HtmlCanvasElement>()
            .expect("varif_canvas should be a canvas");

        let started = eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(move |_creation| {
                    let mut app = App::new(world, layout, layout_note);
                    app.touch = touch;
                    Ok(Box::new(app))
                }),
            )
            .await;

        // Swap the "Loading" text for the window, or for what went wrong.
        if let Some(loading) = document.get_element_by_id("loading") {
            match started {
                Ok(()) => loading.remove(),
                Err(error) => loading.set_inner_html(&format!(
                    "<p>Varif could not start in this browser.</p><p>{error:?}</p>"
                )),
            }
        }
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn read_data(path: &str) -> Result<World, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    load_world(&text).map_err(|e| e.to_string())
}

/// Something a click asks for. Drawing only collects one of these; it is
/// carried out afterwards, so that drawing never changes anything.
enum Action {
    /// Send a command to the world.
    Send(Command),
    /// Go to a place: straight there if a road joins it, otherwise offer the
    /// ways that lead there.
    GoTo(PlaceId),
    /// Take the numbered way from the list on offer.
    TakeWay(usize),
    /// Close the list of ways without going anywhere.
    ForgetWays,
    /// Walk the next leg of the way the party is following.
    Continue,
    /// Stop following the way and stay here.
    Stay,
    /// Turn down the offer of someone joining the party.
    Decline(PersonId),
    AddPlace,
    Save,
    Load,
}

/// Where the window is in the opening of a game.
enum Stage {
    /// New game, or load a save.
    Title { load_open: bool },
    /// The player is choosing the stats of a new character.
    Create,
    /// The map and the road.
    Play,
}

/// The editable part of a new player. Health, fatigue, resilience, level
/// and the inventory stay at their bases.
struct Creator {
    name: String,
    age: u32,
    blood: Blood,
    heritage: Heritage,
    akstada: Akstada,
    tongues: Vec<Tongue>,
    background: String,
    primary: FactionId,
    secondary: Vec<FactionId>,
    error: String,
}

impl Creator {
    fn from_player(world: &World) -> Creator {
        let Some(person) = world.player() else {
            return Creator::empty();
        };
        let Some(sheet) = person.sheet.as_ref() else {
            return Creator {
                name: person.name.clone(),
                ..Creator::empty()
            };
        };
        Creator {
            name: person.name.clone(),
            age: sheet.age.clamp(MIN_AGE, MAX_AGE),
            blood: sheet.blood,
            heritage: sheet.heritage,
            akstada: sheet.akstada,
            tongues: sheet.tongues.clone(),
            background: sheet.background.clone(),
            primary: sheet.primary,
            secondary: sheet.secondary.clone(),
            error: String::new(),
        }
    }

    fn empty() -> Creator {
        Creator {
            name: String::new(),
            age: 20,
            blood: Blood::Red,
            heritage: Heritage::Shentzi,
            akstada: Akstada::Pylgrym,
            tongues: vec![Tongue::Shentz],
            background: String::new(),
            primary: FactionId::Player,
            secondary: Vec::new(),
            error: String::new(),
        }
    }
}

/// Everything the window remembers that is not part of the world.
struct App {
    stage: Stage,
    creator: Creator,
    world: World,
    layout: MapLayout,
    /// The event log shown in the middle of the window.
    log: Vec<String>,
    /// The ways on offer to a place no road runs straight to, quickest
    /// first, and how many slower ways were left off the list.
    ways: Option<(PlaceId, Vec<Itinerary>, usize)>,
    /// The stops still to come after the leg the party is walking now.
    heading: Vec<PlaceId>,
    /// People offered to the party on arrival whom the player has not yet
    /// answered about. The first is asked about.
    offers: Vec<PersonId>,
    /// The journal entry being typed.
    draft: String,
    /// The name and distance of the place being added.
    new_place: String,
    new_leagues: u32,
    save_path: String,
    /// The result of the last save or load.
    status: String,
    /// Which panel a narrow screen is showing.
    tab: Tab,
    /// True on a touch screen. Set at start-up on the web; otherwise turned
    /// on by the first touch.
    touch: bool,
    /// Whether the bigger touch style has been put in place yet.
    touch_styled: bool,
}

/// The panels a narrow screen switches between.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Map,
    Place,
    Party,
    Events,
}

impl Tab {
    const ALL: [Tab; 4] = [Tab::Map, Tab::Place, Tab::Party, Tab::Events];

    fn name(self) -> &'static str {
        match self {
            Tab::Map => "Map",
            Tab::Place => "Place",
            Tab::Party => "Party",
            Tab::Events => "Events",
        }
    }
}

impl App {
    fn new(world: World, layout: MapLayout, note: Option<String>) -> App {
        let mut log = vec!["The Pylgrymage begins.".to_string()];
        log.extend(note);
        App {
            stage: Stage::Title { load_open: false },
            creator: Creator::from_player(&world),
            world,
            layout,
            log,
            ways: None,
            heading: Vec::new(),
            offers: Vec::new(),
            draft: String::new(),
            new_place: String::new(),
            new_leagues: 3,
            save_path: "save.ron".to_string(),
            status: String::new(),
            tab: Tab::Map,
            touch: false,
            touch_styled: false,
        }
    }

    /// Sends one command to the world and adds what happened to the log.
    /// True if the world accepted it.
    fn send(&mut self, command: Command) -> bool {
        match self.world.apply(command) {
            Ok(events) => {
                for event in &events {
                    self.log.push(describe(&self.world, event));
                    if let Event::RecruitOffered { person, .. } = event {
                        self.offers.push(*person);
                    }
                }
                true
            }
            Err(error) => {
                self.log.push(format!("Refused: {error}."));
                false
            }
        }
    }

    fn carry_out(&mut self, action: Action) {
        match action {
            Action::Send(command) => {
                let was_journal = matches!(command, Command::WriteJournal { .. });
                // Setting out by hand drops any way the party was following.
                if matches!(command, Command::Travel { .. }) {
                    self.heading.clear();
                    self.ways = None;
                    self.offers.clear();
                }
                if let Command::Recruit { person } = command {
                    self.offers.retain(|&p| p != person);
                }
                if self.send(command) && was_journal {
                    self.draft.clear();
                }
            }
            Action::GoTo(to) => self.go_to(to),
            Action::TakeWay(index) => {
                if let Some((_, mut routes, _)) = self.ways.take() {
                    if index < routes.len() {
                        let stops = routes.swap_remove(index).stops;
                        self.log.push(format!(
                            "The party takes the way through {}.",
                            through(&self.world, &stops)
                        ));
                        self.set_out(stops);
                    }
                }
            }
            Action::ForgetWays => self.ways = None,
            Action::Continue => {
                let stops = std::mem::take(&mut self.heading);
                self.set_out(stops);
            }
            Action::Decline(person) => {
                self.offers.retain(|&p| p != person);
                let name = self
                    .world
                    .person(person)
                    .map_or("They", |p| p.name.as_str());
                self.log.push(format!("{name} stays behind."));
            }
            Action::Stay => {
                self.heading.clear();
                self.log.push(format!(
                    "The party stays at {}.",
                    self.world.place_name(self.world.party_at)
                ));
            }
            Action::AddPlace => {
                let command = Command::AddPlace {
                    name: self.new_place.clone(),
                    leagues: self.new_leagues,
                };
                if self.send(command) {
                    self.new_place.clear();
                }
            }
            Action::Save => self.save(),
            Action::Load => {
                self.load();
            }
        }
    }

    fn go_to(&mut self, to: PlaceId) {
        let here = self.world.party_at;
        self.ways = None;
        self.offers.clear();
        if to == here || self.world.road_between(here, to).is_some() {
            self.heading.clear();
            self.send(Command::Travel { to });
            return;
        }
        let mut routes = self.world.routes_between(here, to);
        let hidden = routes.len().saturating_sub(MAX_WAYS);
        routes.truncate(MAX_WAYS);
        if routes.is_empty() {
            self.log.push(format!(
                "No road leads from {} to {}, by any way.",
                self.world.place_name(here),
                self.world.place_name(to)
            ));
        } else {
            self.ways = Some((to, routes, hidden));
        }
    }

    /// Walks the first leg of a way and remembers the stops still to come.
    fn set_out(&mut self, stops: Vec<PlaceId>) {
        self.heading.clear();
        // An offer not answered before leaving is left behind.
        self.offers.clear();
        if let Some((&first, rest)) = stops.split_first() {
            if self.send(Command::Travel { to: first }) {
                self.heading = rest.to_vec();
            }
        }
    }

    fn save(&mut self) {
        let result = self
            .world
            .to_ron()
            .and_then(|text| storage::write(&self.save_path, &text));
        self.status = match result {
            Ok(()) => format!("Saved to {}.", self.save_path),
            Err(error) => format!("Could not save: {error}"),
        };
    }

    /// Reads a save into the window. True when the file loaded.
    fn load(&mut self) -> bool {
        let result = storage::read(&self.save_path).and_then(|text| World::from_ron(&text));
        match result {
            Ok(world) => {
                self.world = world;
                self.ways = None;
                self.heading.clear();
                self.offers.clear();
                self.log.push(format!("Loaded {}.", self.save_path));
                self.status = format!("Loaded {}.", self.save_path);
                true
            }
            Err(error) => {
                self.status = format!("Could not load: {error}");
                false
            }
        }
    }

    /// Writes the creator onto the player. Stays on the creation screen when
    /// the sheet is not valid, and leaves the world untouched in that case.
    fn begin(&mut self) {
        let name = self.creator.name.trim().to_string();
        if name.is_empty() {
            self.creator.error = "The character needs a name.".to_string();
            return;
        }
        let Some(index) = self
            .world
            .people
            .iter()
            .position(|person| person.is_player())
        else {
            self.creator.error = "This world has no player.".to_string();
            return;
        };
        let Some(mut sheet) = self.world.people[index].sheet.clone() else {
            self.creator.error = "This world has no player.".to_string();
            return;
        };
        sheet.age = self.creator.age;
        sheet.blood = self.creator.blood;
        sheet.heritage = self.creator.heritage;
        sheet.akstada = self.creator.akstada;
        sheet.tongues = self.creator.tongues.clone();
        sheet.background = self.creator.background.trim().to_string();
        sheet.primary = self.creator.primary;
        sheet.secondary = self.creator.secondary.clone();
        if let Err(error) = sheet.check() {
            self.creator.error = error.to_string();
            return;
        }
        self.world.people[index].name = name.clone();
        self.world.people[index].sheet = Some(sheet);
        self.creator.error.clear();
        self.status.clear();
        self.log = vec![format!("{name} sets out on the Pylgrymage.")];
        self.stage = Stage::Play;
    }

    fn title_screen(&mut self, ui: &mut egui::Ui) {
        let load_open = matches!(self.stage, Stage::Title { load_open: true });
        egui::CentralPanel::default().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                // Less room above the title on a short screen.
                ui.add_space((ui.available_height() * 0.15).min(120.0));
                ui.heading("Varif");
                ui.label("The Pylgrymage");
                ui.add_space(28.0);
                if ui
                    .add_sized([220.0, 32.0], egui::Button::new("New game"))
                    .clicked()
                {
                    self.creator = Creator::from_player(&self.world);
                    self.status.clear();
                    self.stage = Stage::Create;
                }
                ui.add_space(8.0);
                if ui
                    .add_sized([220.0, 32.0], egui::Button::new("Load game"))
                    .clicked()
                {
                    self.status.clear();
                    self.stage = Stage::Title {
                        load_open: !load_open,
                    };
                }
                if load_open {
                    ui.add_space(12.0);
                    ui.add(
                        egui::TextEdit::singleline(&mut self.save_path)
                            .desired_width(ui.available_width().min(220.0)),
                    );
                    ui.add_space(8.0);
                    if ui
                        .add_sized([220.0, 32.0], egui::Button::new("Load"))
                        .clicked()
                        && self.load()
                    {
                        self.stage = Stage::Play;
                    }
                }
                if !self.status.is_empty() {
                    ui.add_space(12.0);
                    ui.colored_label(ui.visuals().error_fg_color, &self.status);
                }
            });
        });
    }

    fn create_screen(&mut self, ui: &mut egui::Ui) {
        egui::Panel::bottom("create_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Back").clicked() {
                    self.creator.error.clear();
                    self.stage = Stage::Title { load_open: false };
                }
                if ui.button("Begin").clicked() {
                    self.begin();
                }
                if !self.creator.error.is_empty() {
                    ui.colored_label(ui.visuals().error_fg_color, &self.creator.error);
                }
            });
        });

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("New character");
            ui.weak(format!(
                "Health {BASE_HEALTH}, fatigue {BASE_FATIGUE}, resilience {BASE_RESILIENCE}. Level {BASE_LEVEL}. These stay as they are, and the inventory starts empty."
            ));
            ui.add_space(8.0);
            egui::ScrollArea::vertical().id_salt("create").show(ui, |ui| {
                ui.set_max_width(460.0);
                ui.label("Name");
                ui.add(
                    egui::TextEdit::singleline(&mut self.creator.name)
                        .desired_width(ui.available_width().min(280.0)),
                );
                ui.add_space(8.0);
                ui.label("Age");
                ui.add(egui::Slider::new(&mut self.creator.age, MIN_AGE..=MAX_AGE).text("years"));
                ui.add_space(8.0);
                choice(ui, "blood", "Blood", self.creator.blood.name(), |ui| {
                    for blood in Blood::ALL {
                        ui.selectable_value(&mut self.creator.blood, blood, blood.name());
                    }
                });
                choice(ui, "heritage", "Heritage", self.creator.heritage.name(), |ui| {
                    for heritage in Heritage::ALL {
                        ui.selectable_value(&mut self.creator.heritage, heritage, heritage.name());
                    }
                });
                choice(ui, "akstada", "Akstada", self.creator.akstada.name(), |ui| {
                    for sign in Akstada::ALL {
                        ui.selectable_value(&mut self.creator.akstada, sign, sign.name());
                    }
                });
                ui.add_space(8.0);
                ui.label("Tongues");
                for tongue in Tongue::ALL {
                    let mut on = self.creator.tongues.contains(&tongue);
                    if ui.checkbox(&mut on, tongue.name()).changed() {
                        if on {
                            self.creator.tongues.push(tongue);
                        } else {
                            self.creator.tongues.retain(|known| *known != tongue);
                        }
                    }
                }
                ui.add_space(8.0);
                ui.label("Background");
                ui.add(
                    egui::TextEdit::multiline(&mut self.creator.background)
                        .desired_rows(3)
                        .desired_width(ui.available_width().min(360.0)),
                );
                choice(ui, "primary_faction", "Primary faction", self.creator.primary.name(), |ui| {
                    for faction in FactionId::ALL {
                        ui.selectable_value(&mut self.creator.primary, faction, faction.name());
                    }
                });
                self.creator.secondary.retain(|faction| *faction != self.creator.primary);
                ui.add_space(8.0);
                ui.label("Secondary factions");
                for faction in FactionId::ALL {
                    if faction == self.creator.primary {
                        continue;
                    }
                    let mut on = self.creator.secondary.contains(&faction);
                    let label = if faction.joinable() {
                        faction.name().to_string()
                    } else {
                        format!("{} (not open)", faction.name())
                    };
                    if ui.checkbox(&mut on, label).changed() {
                        if on {
                            self.creator.secondary.push(faction);
                        } else {
                            self.creator.secondary.retain(|held| *held != faction);
                        }
                    }
                }
            });
        });
    }
}

/// Bigger buttons, more space between them and slightly larger text, so that
/// a finger can hit what it means to.
fn touch_style(style: &mut egui::Style) {
    style.spacing.interact_size.y = style.spacing.interact_size.y.max(36.0);
    style.spacing.button_padding = egui::vec2(12.0, 8.0);
    style.spacing.item_spacing = egui::vec2(10.0, 10.0);
    style.spacing.combo_width = style.spacing.combo_width.max(200.0);
    for font in style.text_styles.values_mut() {
        font.size += 2.0;
    }
}

/// True when the screen is too narrow for three panels side by side.
fn is_compact(ui: &egui::Ui) -> bool {
    ui.ctx().content_rect().width() < COMPACT_WIDTH
}

fn choice(
    ui: &mut egui::Ui,
    salt: &str,
    label: &str,
    selected: &str,
    body: impl FnOnce(&mut egui::Ui),
) {
    ui.add_space(8.0);
    ui.label(label);
    egui::ComboBox::from_id_salt(salt)
        .selected_text(selected)
        .width(220.0)
        .show_ui(ui, body);
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if !self.touch && ui.ctx().input(|i| i.has_touch_screen()) {
            self.touch = true;
        }
        if self.touch && !self.touch_styled {
            ui.ctx().all_styles_mut(touch_style);
            self.touch_styled = true;
        }
        match self.stage {
            Stage::Title { .. } => self.title_screen(ui),
            Stage::Create => self.create_screen(ui),
            Stage::Play => self.play_screen(ui),
        }
    }
}

impl App {
    /// True while the world accepts only the question under the map: the
    /// answer to "make camp?", or what is said in a conversation. Everything
    /// else is greyed out.
    fn world_locked(&self) -> bool {
        self.world.journey.is_some() || self.world.conversation.is_some()
    }

    fn play_screen(&mut self, ui: &mut egui::Ui) {
        let mut action: Option<Action> = None;
        if is_compact(ui) {
            self.play_compact(ui, &mut action);
        } else {
            self.play_wide(ui, &mut action);
        }
        if let Some(action) = action {
            self.carry_out(action);
        }
    }

    /// The desktop layout: places on the left, the party on the right, and
    /// the map, questions and events in the middle.
    fn play_wide(&mut self, ui: &mut egui::Ui, action: &mut Option<Action>) {
        egui::Panel::top("top_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading(self.world.date.to_string());
                ui.separator();
                self.save_controls(ui, action);
                ui.label(&self.status);
            });
        });

        egui::Panel::left("place_panel")
            .default_size(280.0)
            .show(ui, |ui| self.places(ui, action));

        egui::Panel::right("party_panel")
            .default_size(300.0)
            .show(ui, |ui| {
                ui.add_enabled_ui(!self.world_locked(), |ui| {
                    party_panel(ui, &self.world, &mut self.draft, action);
                });
            });

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Map");
            let height = (ui.available_height() * 0.6).max(240.0);
            ui.add_enabled_ui(!self.world_locked(), |ui| {
                map(ui, &self.world, &self.layout, height, action);
            });
            // Below the map, so that the map does not move when a question appears.
            self.ask(ui, action);
            ui.separator();
            ui.heading("Events");
            self.events(ui);
        });
    }

    /// The phone layout: one panel at a time, chosen by tabs along the
    /// bottom. A question waiting for an answer sits just above the tabs,
    /// so it is never hidden behind another tab.
    fn play_compact(&mut self, ui: &mut egui::Ui, action: &mut Option<Action>) {
        egui::Panel::top("top_bar_compact").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.strong(self.world.date.to_string());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.menu_button("Save / Load", |ui| {
                        ui.set_min_width(220.0);
                        ui.label("Save name");
                        self.save_controls(ui, action);
                    });
                });
            });
            if !self.status.is_empty() {
                ui.weak(&self.status);
            }
        });

        egui::Panel::bottom("tab_bar").show(ui, |ui| {
            ui.add_space(4.0);
            ui.columns(Tab::ALL.len(), |columns| {
                for (column, tab) in columns.iter_mut().zip(Tab::ALL) {
                    let button = egui::Button::selectable(self.tab == tab, tab.name());
                    if column
                        .add_sized([column.available_width(), 40.0], button)
                        .clicked()
                    {
                        self.tab = tab;
                    }
                }
            });
            ui.add_space(4.0);
        });

        if self.has_question() {
            egui::Panel::bottom("question_bar").show(ui, |ui| {
                ui.add_space(4.0);
                self.ask(ui, action);
                ui.add_space(4.0);
            });
        }

        egui::CentralPanel::default().show(ui, |ui| match self.tab {
            Tab::Map => {
                // Keep room under the map for the latest few events.
                let room_for_events = 24.0 * EVENTS_UNDER_MAP as f32 + 16.0;
                let height = (ui.available_height() - room_for_events).max(200.0);
                ui.add_enabled_ui(!self.world_locked(), |ui| {
                    map(ui, &self.world, &self.layout, height, action);
                });
                ui.separator();
                let skip = self.log.len().saturating_sub(EVENTS_UNDER_MAP);
                for line in &self.log[skip..] {
                    ui.label(line);
                }
            }
            Tab::Place => self.places(ui, action),
            Tab::Party => {
                egui::ScrollArea::vertical()
                    .id_salt("party_scroll")
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        ui.add_enabled_ui(!self.world_locked(), |ui| {
                            party_panel(ui, &self.world, &mut self.draft, action);
                        });
                    });
            }
            Tab::Events => self.events(ui),
        });
    }

    /// The save name and the Save and Load buttons.
    fn save_controls(&mut self, ui: &mut egui::Ui, action: &mut Option<Action>) {
        ui.add(
            egui::TextEdit::singleline(&mut self.save_path)
                .desired_width(ui.available_width().min(160.0)),
        );
        if ui.button("Save").clicked() {
            *action = Some(Action::Save);
        }
        if ui.button("Load").clicked() {
            *action = Some(Action::Load);
        }
    }

    /// Where the party is, the roads out, and adding a place.
    fn places(&mut self, ui: &mut egui::Ui, action: &mut Option<Action>) {
        let locked = self.world_locked();
        egui::ScrollArea::vertical()
            .id_salt("place_scroll")
            .show(ui, |ui| {
                ui.add_enabled_ui(!locked, |ui| {
                    place_panel(ui, &self.world, action);
                    ui.separator();
                    ui.strong("Add a place");
                    ui.add(egui::TextEdit::singleline(&mut self.new_place).hint_text("Name"));
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::DragValue::new(&mut self.new_leagues)
                                .range(1..=500)
                                .suffix(" leagues"),
                        );
                        let named = !self.new_place.trim().is_empty();
                        if ui.add_enabled(named, egui::Button::new("Add")).clicked() {
                            *action = Some(Action::AddPlace);
                        }
                    });
                    ui.weak("Joined by a new road to where the party stands.");
                });
            });
    }

    /// The whole event log, newest at the bottom.
    fn events(&self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .id_salt("event_log")
            .auto_shrink(false)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for line in &self.log {
                    ui.label(line);
                }
            });
    }

    /// True when the window is waiting for the player to answer something.
    fn has_question(&self) -> bool {
        self.world_locked()
            || !self.offers.is_empty()
            || self.ways.is_some()
            || !self.heading.is_empty()
    }

    fn ask(&self, ui: &mut egui::Ui, action: &mut Option<Action>) {
        question(
            ui,
            &self.world,
            &self.offers,
            &self.ways,
            &self.heading,
            action,
        );
    }
}

/// Whatever the window is waiting for the player to decide, shown under the
/// map: whether to camp, what to say in a conversation, whether to recruit
/// someone, which way to take, or whether to walk the next leg.
fn question(
    ui: &mut egui::Ui,
    world: &World,
    offers: &[PersonId],
    ways: &Option<(PlaceId, Vec<Itinerary>, usize)>,
    heading: &[PlaceId],
    action: &mut Option<Action>,
) {
    if let Some(journey) = world.journey {
        ui.group(|ui| {
            if world.camped {
                ui.strong("The party is camped beside the road.");
                ui.horizontal(|ui| {
                    if ui.button("Sleep").clicked() {
                        *action = Some(Action::Send(Command::Rest));
                    }
                    if ui.button("Pack up").clicked() {
                        *action = Some(Action::Send(Command::PackUp));
                    }
                });
            } else {
                let left = journey.days_left();
                ui.strong(format!(
                    "Night falls on the road, {} from {}.",
                    days(left),
                    world.place_name(journey.to)
                ));
                ui.horizontal(|ui| {
                    if ui.button("Make camp").clicked() {
                        *action = Some(Action::Send(Command::MakeCamp));
                    }
                    if ui.button("Press on").clicked() {
                        *action = Some(Action::Send(Command::EndDay { camp: false }));
                    }
                });
            }
        });
    } else if let Some(person) = world.conversation {
        ui.group(|ui| {
            let name = world.person(person).map_or("Someone", |p| p.name.as_str());
            match world.standing(person) {
                Some(standing) => ui.strong(format!(
                    "Talking with {name} (friendliness {}).",
                    standing.friendliness
                )),
                None => ui.strong(format!("Talking with {name}.")),
            };
            let options = world.dialogue_options(person);
            if options.is_empty() {
                ui.weak("There is nothing to say.");
            }
            for (index, line) in options.iter().enumerate() {
                let button = ui
                    .button(format!("\"{}\"", line.text))
                    .on_hover_text(line.bucket.name());
                if button.clicked() {
                    *action = Some(Action::Send(Command::Say { line: index }));
                }
            }
            if ui.button("Leave").clicked() {
                *action = Some(Action::Send(Command::EndConversation));
            }
        });
    } else if let Some(&person) = offers.first() {
        ui.group(|ui| {
            let name = world.person(person).map_or("Someone", |p| p.name.as_str());
            ui.strong(format!("{name} could join the party. Recruit them?"));
            ui.horizontal(|ui| {
                if ui.button("Recruit").clicked() {
                    *action = Some(Action::Send(Command::Recruit { person }));
                }
                if ui.button("Not now").clicked() {
                    *action = Some(Action::Decline(person));
                }
            });
        });
    } else if let Some((to, routes, hidden)) = ways {
        ui.group(|ui| {
            ui.strong(format!(
                "No road runs straight to {}. Which way?",
                world.place_name(*to)
            ));
            for (index, route) in routes.iter().enumerate() {
                let passing = through(world, &route.stops);
                let label = format!(
                    "Through {passing}: {} leagues, {}",
                    route.leagues,
                    days(route.days)
                );
                if ui.button(label).clicked() {
                    *action = Some(Action::TakeWay(index));
                }
            }
            if *hidden > 0 {
                ui.weak(format!("{hidden} slower ways are not shown."));
            }
            if ui.button("Stay here").clicked() {
                *action = Some(Action::ForgetWays);
            }
        });
    } else if let Some(&next) = heading.first() {
        ui.group(|ui| {
            ui.strong(format!("Continue on to {}?", world.place_name(next)));
            ui.horizontal(|ui| {
                if ui.button("Continue").clicked() {
                    *action = Some(Action::Continue);
                }
                if ui.button("Stay here").clicked() {
                    *action = Some(Action::Stay);
                }
            });
        });
    }
}

/// The left panel: where the party is, the sites inside the place, and the
/// roads out.
fn place_panel(ui: &mut egui::Ui, world: &World, action: &mut Option<Action>) {
    let here = world.party_at;
    let Some(place) = world.place(here) else {
        ui.label("The party is nowhere.");
        return;
    };

    ui.heading(&place.name);
    ui.label(match place.kind {
        PlaceKind::TempleCity => "Temple city",
        PlaceKind::Town => "Town",
        PlaceKind::Landmark => "Landmark",
    });
    if let Some(region) = place.region {
        ui.label(format!("Region: {region:?}"));
    }

    if let Some(temple) = &place.temple {
        ui.separator();
        let state = if temple.ruined { " (in ruins)" } else { "" };
        ui.strong(format!("Krypt of {}{}", temple.krypt_of, state));
        if let Some(rite) = &temple.rite {
            ui.label(format!("Rite: {rite}"));
        }
        if !temple.nedia.is_empty() {
            ui.label(format!("Nedia of {}", temple.nedia.join(", ")));
        }
    }

    let nearby = world.nearby_npcs();
    if !nearby.is_empty() {
        ui.separator();
        ui.strong("Nearby");
        for id in nearby {
            let Some(person) = world.person(id) else {
                continue;
            };
            ui.horizontal(|ui| {
                let follower = if person.is_follower() {
                    ", follower"
                } else {
                    ""
                };
                ui.label(format!("{} ({}{follower})", person.name, person.title));
                if ui.button("Interact").clicked() {
                    *action = Some(Action::Send(Command::Interact { person: id }));
                }
            });
            if let Some(standing) = world.standing(id) {
                if standing.will_talk() {
                    ui.weak(format!("Friendliness {}", standing.friendliness));
                } else {
                    ui.weak("Hostile");
                }
            }
        }
    }
    // People named only as cast have no sheet, so they cannot be talked to.
    let mut cast = world
        .residents(here)
        .filter(|p| p.sheet.is_none())
        .peekable();
    if cast.peek().is_some() {
        ui.separator();
        ui.strong("Here");
        for person in cast {
            ui.label(format!("{} ({})", person.name, person.title));
        }
    }

    if !place.sites.is_empty() {
        ui.separator();
        ui.strong(format!("Inside {}", place.name));
        for (index, site) in place.sites.iter().enumerate() {
            let id = SiteId(index as u32);
            let standing_here = world.party_site == Some(id);
            let label = format!("{} ({:?})", site.name, site.kind);
            // The site the party stands at is shown pressed in.
            if ui.selectable_label(standing_here, label).clicked() && !standing_here {
                *action = Some(Action::Send(Command::Walk { to: id }));
            }
        }
    }

    ui.separator();
    ui.strong("Roads");
    let neighbours = world.neighbours(here);
    if neighbours.is_empty() {
        ui.weak("No road leads anywhere from here.");
    }
    for (to, road) in neighbours {
        let label = format!(
            "{}: {} leagues, {}",
            world.place_name(to),
            road.leagues,
            days(road.days())
        );
        if ui.button(label).clicked() {
            *action = Some(Action::Send(Command::Travel { to }));
        }
    }

    let further: Vec<Route> = world
        .reachable(here)
        .into_iter()
        .filter(|r| r.roads > 1)
        .collect();
    if !further.is_empty() {
        ui.separator();
        ui.strong("Further afield");
        for route in further {
            let label = format!(
                "{}: {} leagues, {}",
                world.place_name(route.to),
                route.leagues,
                days(route.days)
            );
            let button = ui
                .button(label)
                .on_hover_text(format!("First to {}", world.place_name(route.first_step)));
            if button.clicked() {
                *action = Some(Action::GoTo(route.to));
            }
        }
    }
}

/// The right panel: the party, the camp and the journal.
fn party_panel(ui: &mut egui::Ui, world: &World, draft: &mut String, action: &mut Option<Action>) {
    ui.heading("Party");
    for person in world.party() {
        ui.label(format!("{} ({})", person.name, person.title));
        let Some(sheet) = &person.sheet else { continue };
        let follower = if sheet.role.is_follower() {
            ", follower"
        } else {
            ""
        };
        ui.weak(format!("Role: {}{follower}", sheet.role.name()));
        ui.weak(format!("Faction: {}", sheet.faction_label()));
        if !sheet.secondary.is_empty() {
            ui.weak(format!("Also: {}", sheet.secondary_list()));
        }
        if let Some(attitude) = sheet.role.attitude() {
            ui.weak(format!("Attitude: {attitude}"));
        }
        ui.weak(format!(
            "{} years, {} blood, {}",
            sheet.age,
            sheet.blood.name(),
            sheet.heritage.name()
        ));
        ui.weak(format!("Akstada: {}", sheet.akstada.name()));
        ui.weak(format!("Tongues: {}", sheet.tongue_list()));
        ui.weak(format!("Background: {}", sheet.background));
        ui.label(format!("Level {}", sheet.level));
        ui.label(format!(
            "Health {}    Fatigue {}    Resilience {} (max {})",
            sheet.health,
            sheet.fatigue,
            sheet.resilience,
            sheet.max_resilience()
        ));
        if sheet.effective_health() != sheet.health {
            ui.label(format!(
                "Health while items are carried or worn: {}",
                sheet.effective_health()
            ));
        }
        ui.label(format!(
            "Vitality {}    Willpower {}    Fortitude {}",
            sheet.vitality(),
            sheet.willpower(),
            sheet.fortitude()
        ));
        if sheet.inventory.is_empty() {
            ui.weak("Carrying nothing.");
        } else {
            ui.weak("Carrying:");
            for item in &sheet.inventory {
                ui.horizontal(|ui| {
                    if item.count == 1 {
                        ui.label(format!("{} {}", item.rarity.name(), item.name));
                    } else {
                        ui.label(format!(
                            "{} {} ({})",
                            item.rarity.name(),
                            item.name,
                            item.count
                        ));
                    }
                    if person.is_player() {
                        if let Some(slot) = item.slot {
                            if ui.button("Equip").clicked() {
                                *action = Some(Action::Send(Command::Equip {
                                    item: item.name.clone(),
                                    slot,
                                }));
                            }
                        }
                        if item
                            .effects
                            .iter()
                            .any(|effect| effect.kind == EffectKind::Use)
                            && ui.button("Use").clicked()
                        {
                            *action = Some(Action::Send(Command::Use {
                                item: item.name.clone(),
                            }));
                        }
                    }
                });
            }
        }
        if sheet.equipped.is_empty() {
            ui.weak("Wearing nothing.");
        } else {
            ui.weak("Wearing:");
            for worn in &sheet.equipped {
                ui.horizontal(|ui| {
                    ui.label(format!("{} on the {}", worn.item.name, worn.slot.name()));
                    if person.is_player() && ui.button("Unequip").clicked() {
                        *action = Some(Action::Send(Command::Unequip { slot: worn.slot }));
                    }
                });
            }
        }
    }
    ui.horizontal(|ui| {
        if ui.button("Make camp").clicked() {
            *action = Some(Action::Send(Command::MakeCamp));
        }
        if ui.button("Sleep").clicked() {
            *action = Some(Action::Send(Command::Rest));
        }
        if ui.button("Pack up").clicked() {
            *action = Some(Action::Send(Command::PackUp));
        }
        if at_a_bed(world) && ui.button("Sleep at the inn").clicked() {
            *action = Some(Action::Send(Command::Sleep));
        }
        if world.camped {
            ui.label("Camped.");
        }
    });
    if world.nights_without_camp > 0 {
        ui.label(format!(
            "{} without camp.",
            nights(world.nights_without_camp)
        ));
    }

    ui.separator();
    ui.heading("Journal");
    egui::ScrollArea::vertical()
        .id_salt("journal")
        .max_height((ui.available_height() - 150.0).max(60.0))
        .auto_shrink([false, true])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            if world.journal.is_empty() {
                ui.weak("No entries yet.");
            }
            for entry in &world.journal {
                ui.strong(format!("{}, at {}", entry.date, world.place_name(entry.at)));
                ui.label(&entry.text);
                ui.add_space(6.0);
            }
        });

    ui.separator();
    ui.add(
        egui::TextEdit::multiline(draft)
            .hint_text("Write an entry")
            .desired_rows(4)
            .desired_width(f32::INFINITY),
    );
    let has_words = !draft.trim().is_empty();
    if ui
        .add_enabled(has_words, egui::Button::new("Write entry"))
        .clicked()
    {
        *action = Some(Action::Send(Command::WriteJournal {
            text: draft.clone(),
        }));
    }
}

/// A map of every place and road. Clicking a place goes there.
fn map(
    ui: &mut egui::Ui,
    world: &World,
    layout: &MapLayout,
    height: f32,
    action: &mut Option<Action>,
) {
    let size = egui::vec2(ui.available_width(), height);
    let (response, painter) = ui.allocate_painter(size, egui::Sense::hover());
    // Room around the edge for names. A narrow map gives up some of it.
    let margin_x = (response.rect.width() * 0.15).min(80.0);
    let area = response.rect.shrink2(egui::vec2(margin_x, 34.0));
    // A fingertip is bigger than a mouse pointer.
    let hit_size = if ui.ctx().input(|i| i.has_touch_screen()) {
        44.0
    } else {
        26.0
    };

    // Where every place sits, in the map image's pixels.
    let spots = layout.positions(world);
    let Some((min, max)) = layout::bounds(&spots) else {
        return;
    };

    // One scale for both directions, so that the map keeps its shape.
    let span = egui::vec2((max.0 - min.0).max(1.0), (max.1 - min.1).max(1.0));
    let scale = (area.width() / span.x).min(area.height() / span.y);
    let middle = egui::vec2((min.0 + max.0) / 2.0, (min.1 + max.1) / 2.0);
    let on_screen: Vec<egui::Pos2> = spots
        .iter()
        .map(|&(x, y)| area.center() + (egui::vec2(x, y) - middle) * scale)
        .collect();

    let text_colour = ui.visuals().text_color();
    let weak_colour = ui.visuals().weak_text_color();
    let here_colour = ui.visuals().selection.bg_fill;

    for road in &world.roads {
        let (a, b) = (on_screen[road.a.index()], on_screen[road.b.index()]);
        painter.line_segment([a, b], egui::Stroke::new(2.0, weak_colour));
    }

    let here = world.party_at;
    let routes = world.reachable(here);
    let font = egui::FontId::proportional(13.0);
    // The space each place's dot takes up, which names must keep clear of.
    let dots: Vec<egui::Rect> = world
        .places
        .iter()
        .zip(&on_screen)
        .map(|(place, &centre)| {
            egui::Rect::from_center_size(centre, egui::Vec2::splat(2.0 * radius_of(place) + 4.0))
        })
        .collect();
    let mut names: Vec<egui::Rect> = Vec::new();

    for (index, place) in world.places.iter().enumerate() {
        let id = PlaceId(index as u32);
        let centre = on_screen[index];
        let is_here = id == here && world.journey.is_none();
        let route = routes.iter().find(|r| r.to == id);
        let radius = radius_of(place);

        if let Some(route) = route {
            let hit_box = egui::Rect::from_center_size(centre, egui::Vec2::splat(hit_size));
            let node = ui
                .interact(
                    hit_box,
                    ui.scope_id().with(("map_node", index)),
                    egui::Sense::click(),
                )
                .on_hover_text(format!(
                    "Go to {}: {} leagues, {}",
                    place.name,
                    route.leagues,
                    days(route.days)
                ));
            if node.clicked() {
                *action = Some(Action::GoTo(id));
            }
        }

        let colour = match route {
            _ if is_here => here_colour,
            Some(route) if route.roads == 1 => ui.visuals().strong_text_color(),
            Some(_) => text_colour,
            None => weak_colour,
        };
        painter.circle_filled(centre, radius, colour);
        if is_here {
            painter.circle_stroke(centre, radius + 4.0, egui::Stroke::new(2.0, here_colour));
        }

        // Try the name below, above, right and left of the dot, and keep the
        // spot where it covers the least of the dots and the names already
        // drawn. Places are crowded in parts of the map.
        let label_colour = if route.is_some() || is_here {
            text_colour
        } else {
            weak_colour
        };
        let galley = painter.layout_no_wrap(place.name.clone(), font.clone(), label_colour);
        let gap = radius + 5.0;
        let spots = [
            (egui::vec2(0.0, gap), egui::Align2::CENTER_TOP),
            (egui::vec2(0.0, -gap), egui::Align2::CENTER_BOTTOM),
            (egui::vec2(gap, 0.0), egui::Align2::LEFT_CENTER),
            (egui::vec2(-gap, 0.0), egui::Align2::RIGHT_CENTER),
        ];
        let best = spots
            .iter()
            .map(|&(offset, anchor)| anchor.anchor_size(centre + offset, galley.size()))
            .min_by(|a, b| {
                let covered = |rect: &egui::Rect| -> f32 {
                    let on_dots = dots.iter().enumerate().filter(|(other, _)| *other != index);
                    on_dots.map(|(_, dot)| overlap(rect, dot)).sum::<f32>()
                        + names.iter().map(|name| overlap(rect, name)).sum::<f32>()
                };
                covered(a).total_cmp(&covered(b))
            })
            .expect("there are four spots");
        painter.galley(best.min, galley, label_colour);
        names.push(best);
    }

    // On the road, mark how far along it the party has come.
    if let Some(journey) = world.journey {
        let (from, to) = (
            on_screen[journey.from.index()],
            on_screen[journey.to.index()],
        );
        let part = journey.leagues_walked as f32 / journey.leagues.max(1) as f32;
        let party = from + (to - from) * part;
        painter.circle_filled(party, 5.0, here_colour);
        painter.circle_stroke(party, 9.0, egui::Stroke::new(2.0, here_colour));
    }
}

/// How big a place's dot is drawn.
fn radius_of(place: &Place) -> f32 {
    if place.kind == PlaceKind::TempleCity {
        9.0
    } else {
        6.0
    }
}

/// The area two rectangles share, or zero if they do not touch.
fn overlap(a: &egui::Rect, b: &egui::Rect) -> f32 {
    let shared = a.intersect(*b);
    if shared.is_positive() {
        shared.area()
    } else {
        0.0
    }
}

/// The same wording as the command-line program, so the two logs read alike.
fn describe(world: &World, event: &Event) -> String {
    match event {
        Event::Departed { from, to, .. } => format!(
            "The party leaves {} for {}.",
            world.place_name(*from),
            world.place_name(*to)
        ),
        Event::DayPassed { date } => format!("    {date}"),
        Event::RoadEncounter { .. } => "    Something meets the party on the road.".to_string(),
        Event::NightFalls { to, days_left, .. } => format!(
            "Night falls on the road, {} from {}.",
            days(*days_left),
            world.place_name(*to)
        ),
        Event::CampedOnRoad { .. } => "The party makes camp beside the road.".to_string(),
        Event::PressedOn {
            nights_without_camp,
            extra_leagues,
            fatigue,
            ..
        } => {
            let night = match nights_without_camp {
                1 => "The party presses on through the night".to_string(),
                n => format!("The party presses on through the night, {n} nights without camp"),
            };
            format!(
                "{night}, covering {extra_leagues} extra leagues.{}",
                fatigue_note(world, fatigue)
            )
        }
        Event::Arrived { at, .. } => format!("The party arrives at {}.", world.place_name(*at)),
        Event::EnteredBy { at, gate, .. } => {
            format!("It comes in by the {}.", site_name(world.place(*at), *gate))
        }
        Event::Walked { at, from, to, .. } => {
            let place = world.place(*at);
            match from {
                Some(from) => format!(
                    "The party walks from the {} to the {}.",
                    site_name(place, *from),
                    site_name(place, *to)
                ),
                None => format!("The party walks to the {}.", site_name(place, *to)),
            }
        }
        Event::Camped { at, .. } => {
            format!("The party makes camp at {}.", world.place_name(*at))
        }
        Event::Rested {
            at,
            fatigue,
            spent_food,
            spent_water,
            spent_wood,
            ..
        } => format!(
            "The party sleeps in camp at {}.{}{}",
            world.place_name(*at),
            camp_note(*spent_food, *spent_water, *spent_wood),
            fatigue_note(world, fatigue)
        ),
        Event::PackedUp { at, .. } => {
            format!("The party packs up the camp at {}.", world.place_name(*at))
        }
        Event::Slept {
            at,
            site,
            fatigue,
            copper,
            ..
        } => {
            let paid = if *copper > 0 {
                format!(" Pays {copper} copper.")
            } else {
                String::new()
            };
            format!(
                "The party sleeps at the {} in {}.{}{}",
                site_name(world.place(*at), *site),
                world.place_name(*at),
                paid,
                fatigue_note(world, fatigue)
            )
        }
        Event::JournalWritten { .. } => "The entry is written.".to_string(),
        Event::RecruitOffered { person, .. } => format!(
            "{} is here, and could join the party.",
            person_name(world, *person)
        ),
        Event::ConversationOpened { person, standing } => format!(
            "You speak with {} (friendliness {}).",
            person_name(world, *person),
            standing.friendliness
        ),
        Event::Said {
            person,
            text,
            reply,
        } => {
            if reply.trim().is_empty() {
                format!("You: \"{text}\"  {} says nothing.", person_name(world, *person))
            } else {
                format!("You: \"{text}\"  {}: \"{reply}\"", person_name(world, *person))
            }
        }
        Event::ConversationEnded { person } => {
            format!("You leave {}.", person_name(world, *person))
        }
        Event::TooHostileToTalk {
            person,
            friendliness,
        } => format!(
            "{} is too hostile to talk to (friendliness {friendliness}). Combat would start here, but it is not built yet.",
            person_name(world, *person)
        ),
        Event::Recruited { person, .. } => {
            format!("{} joins the party.", person_name(world, *person))
        }
        Event::HostilityChanged {
            from,
            toward,
            index,
        } => format!(
            "The {}'s hostility toward {} is now {index}.",
            from.name(),
            toward.name()
        ),
        Event::AttitudeChanged { person, index } => format!(
            "{}'s attitude toward the player is now {index}.",
            person_name(world, *person)
        ),
        Event::Equipped { person, item, slot } => format!(
            "{} equips the {item} on the {slot}.",
            person_name(world, *person)
        ),
        Event::Unequipped { person, item, slot } => format!(
            "{} takes the {item} off the {slot} and puts it back in the pack.",
            person_name(world, *person)
        ),
        Event::Used { person, item } => {
            format!("{} uses the {item}.", person_name(world, *person))
        }
        Event::EffectFired {
            person,
            item,
            kind,
            payload,
        } => format!(
            "{}'s {item}, {}, {}.",
            person_name(world, *person),
            kind.name(),
            payload.note()
        ),
        Event::PlaceAdded {
            place,
            from,
            leagues,
            ..
        } => format!(
            "{} is added to the map, {} leagues from {}.",
            world.place_name(*place),
            leagues,
            world.place_name(*from)
        ),
    }
}

fn person_name(world: &World, id: PersonId) -> &str {
    world.person(id).map_or("someone", |p| p.name.as_str())
}

fn camp_note(spent_food: bool, spent_water: bool, spent_wood: bool) -> String {
    let mut note = String::new();
    let mut used = Vec::new();
    if spent_food {
        used.push("food");
    }
    if spent_water {
        used.push("water");
    }
    if spent_wood {
        used.push("wood");
    }
    if !used.is_empty() {
        note.push_str(&format!(" Uses {}.", used.join(", ")));
    }
    if !spent_food {
        note.push_str(" No food: 5 fatigue.");
    }
    if !spent_water {
        note.push_str(" No water: 5 health.");
    }
    if !spent_wood {
        note.push_str(" No wood for a fire: 5 resilience.");
    }
    note
}

fn fatigue_note(world: &World, fatigue: &[MemberFatigue]) -> String {
    if fatigue.is_empty() {
        return String::new();
    }
    let parts = fatigue
        .iter()
        .map(|entry| format!("{} {}", person_name(world, entry.person), entry.fatigue))
        .collect::<Vec<_>>()
        .join(", ");
    format!(" Fatigue: {parts}.")
}

/// True when the party is standing at an inn or a residence.
fn at_a_bed(world: &World) -> bool {
    let Some(place) = world.place(world.party_at) else {
        return false;
    };
    let Some(site) = world.party_site.and_then(|id| place.site(id)) else {
        return false;
    };
    matches!(site.kind, SiteKind::Inn | SiteKind::Residence)
}

fn site_name(place: Option<&Place>, id: SiteId) -> &str {
    place
        .and_then(|p| p.site(id))
        .map_or("an unknown spot", |s| s.name.as_str())
}

/// The places a way passes, without its destination.
fn through(world: &World, stops: &[PlaceId]) -> String {
    let passed = &stops[..stops.len().saturating_sub(1)];
    passed
        .iter()
        .map(|&id| world.place_name(id))
        .collect::<Vec<_>>()
        .join(", ")
}

fn days(count: u32) -> String {
    if count == 1 {
        "1 day".to_string()
    } else {
        format!("{count} days")
    }
}

fn nights(count: u32) -> String {
    if count == 1 {
        "1 night".to_string()
    } else {
        format!("{count} nights")
    }
}
