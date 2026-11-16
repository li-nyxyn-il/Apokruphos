//! Tests for walking inside a city and coming in by its gates.

use varif_core::{load_world, Command, CommandError, DataError, Event, World};

const FIRST_LEG: &str = include_str!("../../../data/first_leg.ron");

fn first_leg() -> World {
    load_world(FIRST_LEG).expect("the first leg data should load")
}

/// Travels to each stop in turn, camping every night.
fn travel(world: &mut World, stops: &[&str]) {
    for stop in stops {
        let to = world.find_place(stop).unwrap();
        world.apply(Command::Travel { to }).unwrap();
        while world.journey.is_some() {
            world.apply(Command::MakeCamp).unwrap();
            world.apply(Command::Rest).unwrap();
        }
    }
}

/// The name of the site the party stands at.
fn standing_at(world: &World) -> Option<&str> {
    let place = world.place(world.party_at)?;
    Some(place.site(world.party_site?)?.name.as_str())
}

#[test]
fn the_party_starts_at_the_gate_of_yhlgrym() {
    let world = first_leg();
    assert_eq!(standing_at(&world), Some("North Gate"));
}

#[test]
fn the_party_walks_to_the_krypt_and_back() {
    let mut world = first_leg();
    let yhlgrym = world.party_at;
    let place = world.place(yhlgrym).unwrap();
    let (gate, krypt) = (
        place.find_site("gate").unwrap(),
        place.find_site("the krypt of grymsiere").unwrap(),
    );
    let date = world.date;

    let events = world.apply(Command::Walk { to: krypt }).unwrap();
    assert_eq!(
        events,
        [Event::Walked {
            at: yhlgrym,
            from: Some(gate),
            to: krypt,
            on: date
        }]
    );
    assert_eq!(standing_at(&world), Some("Krypt of Grymsiere"));
    assert_eq!(world.date, date, "walking inside a city takes no time");

    world.apply(Command::Walk { to: gate }).unwrap();
    assert_eq!(standing_at(&world), Some("North Gate"));
    assert_eq!(
        world.apply(Command::Walk { to: gate }),
        Err(CommandError::AlreadyThere)
    );
}

#[test]
fn a_place_without_sites_has_nowhere_to_walk() {
    let mut world = first_leg();
    travel(&mut world, &["falls"]);
    assert_eq!(world.party_site, None);
    let before = world.clone();
    assert_eq!(
        world.apply(Command::Walk {
            to: varif_core::SiteId(0)
        }),
        Err(CommandError::NowhereToWalk)
    );
    assert_eq!(world, before);
}

#[test]
fn an_unknown_site_is_refused() {
    let mut world = first_leg();
    let before = world.clone();
    let to = varif_core::SiteId(99);
    assert_eq!(
        world.apply(Command::Walk { to }),
        Err(CommandError::UnknownSite(to))
    );
    assert_eq!(world, before);
}

#[test]
fn leaving_a_city_leaves_its_sites_behind() {
    let mut world = first_leg();
    let falls = world.find_place("falls").unwrap();
    world.apply(Command::Travel { to: falls }).unwrap();
    assert_eq!(world.party_site, None);
}

#[test]
fn where_the_party_stands_is_saved() {
    let mut world = first_leg();
    let krypt = world
        .place(world.party_at)
        .unwrap()
        .find_site("krypt")
        .unwrap();
    world.apply(Command::Walk { to: krypt }).unwrap();
    let loaded = World::from_ron(&world.to_ron().unwrap()).unwrap();
    assert_eq!(loaded, world);
    assert_eq!(standing_at(&loaded), Some("Krypt of Grymsiere"));
}

#[test]
fn bad_sites_are_reported() {
    let no_gate = r#"(
        start: (year: 192, month: 1, day: 1, seed: 1, party_at: "a"),
        places: [(key: "a", name: "A", kind: Town, sites: [(key: "inn", name: "Inn", kind: Inn)])],
    )"#;
    assert_eq!(
        load_world(no_gate).unwrap_err(),
        DataError::NoGate("a".to_string())
    );

    let twice = r#"(
        start: (year: 192, month: 1, day: 1, seed: 1, party_at: "a"),
        places: [(key: "a", name: "A", kind: Town, sites: [
            (key: "gate", name: "Gate", kind: Gate),
            (key: "gate", name: "Other Gate", kind: Gate),
        ])],
    )"#;
    assert_eq!(
        load_world(twice).unwrap_err(),
        DataError::DuplicateSite {
            place: "a".to_string(),
            site: "gate".to_string()
        }
    );

    let faces_nowhere = r#"(
        start: (year: 192, month: 1, day: 1, seed: 1, party_at: "a"),
        places: [(key: "a", name: "A", kind: Town, sites: [
            (key: "gate", name: "Gate", kind: Gate, faces: ["nowhere"]),
        ])],
    )"#;
    assert_eq!(
        load_world(faces_nowhere).unwrap_err().to_string(),
        "the Gate of A names a place \"nowhere\" that is not in the file"
    );
}
