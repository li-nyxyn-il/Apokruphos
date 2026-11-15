//! Tests that read like the journey: load the first leg and walk it.

use varif_core::{load_world, Command, CommandError, Date, Event, World};

const FIRST_LEG: &str = include_str!("../../../data/first_leg.ron");

fn first_leg() -> World {
    load_world(FIRST_LEG).expect("the first leg data should load")
}

/// Walks the party along the road through each named stop in turn.
fn walk(world: &mut World, stops: &[&str]) -> Vec<Event> {
    let mut events = Vec::new();
    for stop in stops {
        let to = world.find_place(stop).expect("the stop should exist");
        events.extend(world.apply(Command::Travel { to }).expect("the road should exist"));
    }
    events
}

#[test]
fn the_data_loads() {
    let world = first_leg();
    assert_eq!(world.places.len(), 5);
    assert_eq!(world.roads.len(), 4);
    assert_eq!(world.place_name(world.party_at), "Yhlgrym");
    assert_eq!(world.date, Date::new(192, 1, 1).unwrap());
    assert_eq!(world.party().count(), 1);
}

#[test]
fn the_party_walks_from_yhlgrym_to_atzhaim() {
    let mut world = first_leg();
    let start = world.date;
    let events = walk(&mut world, &["falls", "hylhaim", "heedhaim", "atzhaim"]);

    let atzhaim = world.find_place("Atzhaim").unwrap();
    assert_eq!(world.party_at, atzhaim);
    assert_eq!(events.last(), Some(&Event::Arrived { at: atzhaim, on: world.date }));

    // 5, 14, 11 and 6 leagues at 6 leagues a day: 1 + 3 + 2 + 1 days.
    assert_eq!(world.date.days_since(start), 7);
    let days_passed = events.iter().filter(|e| matches!(e, Event::DayPassed { .. })).count();
    assert_eq!(days_passed, 7);

    // The party's people move with it.
    assert!(world.party().all(|p| p.at == atzhaim));

    // The second temple of the Pylgrymage is a ruin in this era.
    let temple = world.place(atzhaim).unwrap().temple.as_ref().unwrap();
    assert_eq!(temple.krypt_of, "Phionyx");
    assert!(temple.ruined);
}

#[test]
fn there_is_no_road_straight_to_atzhaim() {
    let mut world = first_leg();
    let before = world.clone();
    let from = world.party_at;
    let to = world.find_place("atzhaim").unwrap();

    let result = world.apply(Command::Travel { to });

    assert_eq!(result, Err(CommandError::NoRoad { from, to }));
    assert_eq!(world, before, "a refused command must change nothing");
}

#[test]
fn travelling_to_where_you_are_is_refused() {
    let mut world = first_leg();
    let here = world.party_at;
    assert_eq!(world.apply(Command::Travel { to: here }), Err(CommandError::AlreadyThere));
}

#[test]
fn camping_passes_a_day() {
    let mut world = first_leg();
    let start = world.date;
    let events = world.apply(Command::MakeCamp).unwrap();
    assert!(world.camped);
    assert_eq!(world.date, start.next_day());
    assert_eq!(events.len(), 2);
}

#[test]
fn the_journal_records_where_and_when() {
    let mut world = first_leg();
    walk(&mut world, &["falls"]);
    world
        .apply(Command::WriteJournal { text: "  The water is loud here.  ".to_string() })
        .unwrap();

    let entry = world.journal.last().unwrap();
    assert_eq!(entry.text, "The water is loud here.");
    assert_eq!(entry.at, world.party_at);
    assert_eq!(entry.date, world.date);

    let empty = world.apply(Command::WriteJournal { text: "   ".to_string() });
    assert_eq!(empty, Err(CommandError::EmptyJournalEntry));
}

#[test]
fn the_same_commands_give_the_same_history() {
    let stops = ["falls", "hylhaim", "heedhaim", "atzhaim"];
    let mut first = first_leg();
    let mut second = first_leg();
    assert_eq!(walk(&mut first, &stops), walk(&mut second, &stops));
    assert_eq!(first, second);
}

#[test]
fn a_saved_world_loads_back_identical() {
    let mut world = first_leg();
    walk(&mut world, &["falls", "hylhaim"]);
    world.apply(Command::WriteJournal { text: "Two days on the road.".to_string() }).unwrap();

    let saved = world.to_ron().expect("the world should save");
    let loaded = World::from_ron(&saved).expect("the save should load");
    assert_eq!(loaded, world);

    // The random number generator is saved too, so the future matches.
    let mut a = world;
    let mut b = loaded;
    assert_eq!(walk(&mut a, &["heedhaim", "atzhaim"]), walk(&mut b, &["heedhaim", "atzhaim"]));
}

#[test]
fn bad_data_is_reported() {
    let text = r#"(
        start: (year: 192, month: 1, day: 1, seed: 1, party_at: "nowhere"),
        places: [(key: "yhlgrym", name: "Yhlgrym", kind: TempleCity)],
    )"#;
    let error = load_world(text).unwrap_err();
    assert_eq!(error.to_string(), "the start names a place \"nowhere\" that is not in the file");
}
