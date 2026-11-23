//! Tests for walking inside a city and coming in by its gates.

use varif_core::{load_world, Command, CommandError, DataError, Event, World};

const FIRST_LEG: &str = include_str!("../../../data/first_leg.ron");

fn first_leg() -> World {
    load_world(FIRST_LEG).expect("the first leg data should load")
}

/// Travels to each stop in turn, eight hours at a time.
fn travel(world: &mut World, stops: &[&str]) {
    for stop in stops {
        let to = world.find_place(stop).unwrap();
        loop {
            let left = if let Some(journey) = world.journey {
                journey.leagues_left()
            } else if world.party_at == to {
                break;
            } else {
                world.road_between(world.party_at, to).unwrap().leagues
            };
            if left == 0 {
                break;
            }
            world
                .apply(Command::Travel {
                    to,
                    hours: left.min(8),
                })
                .unwrap();
            if world.journey.is_none() {
                break;
            }
        }
    }
}

/// The name of the site the party stands at.
fn standing_at(world: &World) -> Option<&str> {
    let place = world.place(world.party_at)?;
    Some(place.site(world.party_site?)?.name.as_str())
}

#[test]
fn the_party_starts_at_the_gate_of_sthepenhall() {
    let world = first_leg();
    assert_eq!(world.place_name(world.party_at), "Sthepenhall");
    assert_eq!(standing_at(&world), Some("Gate"));
}

#[test]
fn the_party_walks_to_the_palace_and_back() {
    let mut world = first_leg();
    let sthepenhall = world.party_at;
    let place = world.place(sthepenhall).unwrap();
    let (gate, palace) = (
        place.find_site("gate").unwrap(),
        place.find_site("palace").unwrap(),
    );
    let date = world.date;

    let events = world.apply(Command::Walk { to: palace }).unwrap();
    assert_eq!(
        events,
        [Event::Walked {
            at: sthepenhall,
            from: Some(gate),
            to: palace,
            on: date
        }]
    );
    assert_eq!(standing_at(&world), Some("Palace"));
    assert_eq!(world.date, date, "walking inside a city takes no time");

    world.apply(Command::Walk { to: gate }).unwrap();
    assert_eq!(standing_at(&world), Some("Gate"));
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
    world
        .apply(Command::Travel {
            to: falls,
            hours: 1,
        })
        .unwrap();
    assert_eq!(world.party_site, None);
}

#[test]
fn where_the_party_stands_is_saved() {
    let mut world = first_leg();
    let inn = world
        .place(world.party_at)
        .unwrap()
        .find_site("inn")
        .unwrap();
    world.apply(Command::Walk { to: inn }).unwrap();
    let loaded = World::from_ron(&world.to_ron().unwrap()).unwrap();
    assert_eq!(loaded, world);
    assert_eq!(standing_at(&loaded), Some("Inn"));
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
