//! A simple desktop window for the world model, built on egui.
//!
//! Run it from the workspace folder with `cargo run -p varif-gui`, or pass a
//! data file: `cargo run -p varif-gui -- data/first_leg.ron`.
//!
//! egui redraws the whole window many times a second. Each redraw reads the
//! `World` and draws it; a click produces an `Action`, which is carried out
//! once drawing is done. The window keeps no copy of the world's state.

mod layout;

use eframe::egui;
use layout::MapLayout;
use std::path::Path;
use std::{env, fs, process};
use varif_core::{
    load_world, Command, Event, Itinerary, Place, PlaceId, PlaceKind, Route, SiteId, World,
};

const DEFAULT_DATA: &str = "data/first_leg.ron";
/// Looked for beside the data file.
const LAYOUT_FILE: &str = "map_layout.ron";
/// A map with many loops has hundreds of ways between two places. Only this
/// many, the quickest, are offered.
const MAX_WAYS: usize = 5;

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
    AddPlace,
    Save,
    Load,
}

/// Everything the window remembers that is not part of the world.
struct App {
    world: World,
    layout: MapLayout,
    /// The event log shown in the middle of the window.
    log: Vec<String>,
    /// The ways on offer to a place no road runs straight to, quickest
    /// first, and how many slower ways were left off the list.
    ways: Option<(PlaceId, Vec<Itinerary>, usize)>,
    /// The stops still to come after the leg the party is walking now.
    heading: Vec<PlaceId>,
    /// The journal entry being typed.
    draft: String,
    /// The name and distance of the place being added.
    new_place: String,
    new_leagues: u32,
    save_path: String,
    /// The result of the last save or load.
    status: String,
}

impl App {
    fn new(world: World, layout: MapLayout, note: Option<String>) -> App {
        let mut log = vec!["The Pylgrymage begins.".to_string()];
        log.extend(note);
        App {
            world,
            layout,
            log,
            ways: None,
            heading: Vec::new(),
            draft: String::new(),
            new_place: String::new(),
            new_leagues: 3,
            save_path: "save.ron".to_string(),
            status: String::new(),
        }
    }

    /// Sends one command to the world and adds what happened to the log.
    /// True if the world accepted it.
    fn send(&mut self, command: Command) -> bool {
        match self.world.apply(command) {
            Ok(events) => {
                for event in &events {
                    self.log.push(describe(&self.world, event));
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
            Action::Load => self.load(),
        }
    }

    fn go_to(&mut self, to: PlaceId) {
        let here = self.world.party_at;
        self.ways = None;
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
            .and_then(|text| fs::write(&self.save_path, text).map_err(|e| e.to_string()));
        self.status = match result {
            Ok(()) => format!("Saved to {}.", self.save_path),
            Err(error) => format!("Could not save: {error}"),
        };
    }

    fn load(&mut self) {
        let result = fs::read_to_string(&self.save_path)
            .map_err(|e| e.to_string())
            .and_then(|text| World::from_ron(&text));
        self.status = match result {
            Ok(world) => {
                self.world = world;
                self.ways = None;
                self.heading.clear();
                self.log.push(format!("Loaded {}.", self.save_path));
                format!("Loaded {}.", self.save_path)
            }
            Err(error) => format!("Could not load: {error}"),
        };
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let mut action: Option<Action> = None;
        // Between days on the road the world accepts only the answer to
        // "make camp?", so everything else is greyed out.
        let on_road = self.world.journey.is_some();

        egui::Panel::top("top_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading(self.world.date.to_string());
                ui.separator();
                ui.label("Save file:");
                ui.add(egui::TextEdit::singleline(&mut self.save_path).desired_width(160.0));
                if ui.button("Save").clicked() {
                    action = Some(Action::Save);
                }
                if ui.button("Load").clicked() {
                    action = Some(Action::Load);
                }
                ui.label(&self.status);
            });
        });

        egui::Panel::left("place_panel")
            .default_size(280.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("place_scroll")
                    .show(ui, |ui| {
                        ui.add_enabled_ui(!on_road, |ui| {
                            place_panel(ui, &self.world, &mut action);
                            ui.separator();
                            ui.strong("Add a place");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.new_place).hint_text("Name"),
                            );
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::DragValue::new(&mut self.new_leagues)
                                        .range(1..=500)
                                        .suffix(" leagues"),
                                );
                                let named = !self.new_place.trim().is_empty();
                                if ui.add_enabled(named, egui::Button::new("Add")).clicked() {
                                    action = Some(Action::AddPlace);
                                }
                            });
                            ui.weak("Joined by a new road to where the party stands.");
                        });
                    });
            });

        egui::Panel::right("party_panel")
            .default_size(300.0)
            .show(ui, |ui| {
                ui.add_enabled_ui(!on_road, |ui| {
                    party_panel(ui, &self.world, &mut self.draft, &mut action);
                });
            });

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Map");
            let height = (ui.available_height() * 0.6).max(240.0);
            ui.add_enabled_ui(!on_road, |ui| {
                map(ui, &self.world, &self.layout, height, &mut action);
            });
            // Below the map, so that the map does not move when a question appears.
            question(ui, &self.world, &self.ways, &self.heading, &mut action);
            ui.separator();
            ui.heading("Events");
            egui::ScrollArea::vertical()
                .id_salt("event_log")
                .auto_shrink(false)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for line in &self.log {
                        ui.label(line);
                    }
                });
        });

        if let Some(action) = action {
            self.carry_out(action);
        }
    }
}

/// Whatever the window is waiting for the player to decide, shown under the
/// map: whether to camp, which way to take, or whether to walk the next leg.
fn question(
    ui: &mut egui::Ui,
    world: &World,
    ways: &Option<(PlaceId, Vec<Itinerary>, usize)>,
    heading: &[PlaceId],
    action: &mut Option<Action>,
) {
    if let Some(journey) = world.journey {
        ui.group(|ui| {
            let left = journey.days - journey.days_walked;
            ui.strong(format!(
                "Night falls on the road, {} from {}.",
                days(left),
                world.place_name(journey.to)
            ));
            ui.horizontal(|ui| {
                if ui.button("Make camp").clicked() {
                    *action = Some(Action::Send(Command::EndDay { camp: true }));
                }
                if ui.button("Press on").clicked() {
                    *action = Some(Action::Send(Command::EndDay { camp: false }));
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

    let mut residents = world.residents(here).peekable();
    if residents.peek().is_some() {
        ui.separator();
        ui.strong("Here");
        for person in residents {
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
            "Health {}    Fatigue {}    Resilience {}",
            sheet.health, sheet.fatigue, sheet.resilience
        ));
        if sheet.inventory.is_empty() {
            ui.weak("Carrying nothing.");
        } else {
            ui.weak("Carrying:");
            for item in &sheet.inventory {
                if item.count == 1 {
                    ui.label(&item.name);
                } else {
                    ui.label(format!("{} ({})", item.name, item.count));
                }
            }
        }
    }
    ui.horizontal(|ui| {
        if ui.button("Make camp").clicked() {
            *action = Some(Action::Send(Command::MakeCamp));
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
    let area = response.rect.shrink2(egui::vec2(80.0, 34.0));

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
            let hit_box = egui::Rect::from_center_size(centre, egui::vec2(26.0, 26.0));
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
        let part = journey.days_walked as f32 / journey.days.max(1) as f32;
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
            ..
        } => match nights_without_camp {
            1 => "The party presses on through the night.".to_string(),
            n => format!("The party presses on through the night, {n} nights without camp."),
        },
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
        Event::Camped { at, .. } => format!("The party makes camp at {}.", world.place_name(*at)),
        Event::JournalWritten { .. } => "The entry is written.".to_string(),
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
