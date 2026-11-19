//! Tests that read like the journey: load the first leg and walk it.

use varif_core::{load_world, Command, CommandError, Date, Event, World};

const FIRST_LEG: &str = include_str!("../../../data/first_leg.ron");

fn first_leg() -> World {
    load_world(FIRST_LEG).expect("the first leg data should load")
}

/// Walks the party along the road through each named stop in turn.
/// Each order is at most eight hours, so the march itself adds no fatigue.
fn walk(world: &mut World, stops: &[&str]) -> Vec<Event> {
    let mut events = Vec::new();
    for stop in stops {
        let to = world.find_place(stop).expect("the stop should exist");
        events.extend(travel_to(world, to));
    }
    events
}

/// Covers the road to `to` at one league an hour, eight hours at a time.
fn travel_to(world: &mut World, to: varif_core::PlaceId) -> Vec<Event> {
    let mut events = Vec::new();
    loop {
        let left = if let Some(journey) = world.journey {
            assert_eq!(journey.to, to, "a journey is already bound elsewhere");
            journey.leagues_left()
        } else if world.party_at == to {
            break;
        } else {
            world
                .road_between(world.party_at, to)
                .expect("the road should exist")
                .leagues
        };
        if left == 0 {
            break;
        }
        events.extend(
            world
                .apply(Command::Travel {
                    to,
                    hours: left.min(8),
                })
                .expect("the road should exist"),
        );
        if world.journey.is_none() {
            break;
        }
    }
    events
}

#[test]
fn the_data_loads() {
    let world = first_leg();
    assert_eq!(world.places.len(), 45);
    assert_eq!(world.roads.len(), 54);
    assert_eq!(world.place_name(world.party_at), "Yhlgrym");
    assert_eq!(world.date, Date::new(192, 1, 1).unwrap());
    assert_eq!(world.party().count(), 2, "the Pylgrym and Donold Audsen");
}

#[test]
fn the_party_walks_from_yhlgrym_to_atzhaim() {
    let mut world = first_leg();
    let start = world.date;
    let events = walk(&mut world, &["atzhaim"]);

    let atzhaim = world.find_place("Atzhaim").unwrap();
    assert_eq!(world.party_at, atzhaim);
    let south_gate = world
        .place(atzhaim)
        .unwrap()
        .find_site("south gate")
        .unwrap();
    assert_eq!(
        events[events.len() - 2..],
        [
            Event::Arrived {
                at: atzhaim,
                on: world.date
            },
            Event::EnteredBy {
                at: atzhaim,
                gate: south_gate,
                on: world.date
            },
        ]
    );

    // 60 leagues at one league an hour: 60 hours, which is two days and 12 hours.
    assert_eq!(world.date.days_since(start), 2);
    assert_eq!(world.hour, 12);
    let days_passed = events
        .iter()
        .filter(|e| matches!(e, Event::DayPassed { .. }))
        .count();
    assert_eq!(days_passed, 2);

    // The party's people move with it.
    assert!(world.party().all(|p| p.at == atzhaim));

    // The second temple of the Pylgrymage is a ruin in this era.
    let temple = world.place(atzhaim).unwrap().temple.as_ref().unwrap();
    assert_eq!(temple.krypt_of, "Phionyx");
    assert!(temple.ruined);
}

#[test]
fn there_is_no_road_straight_to_sthepenhall() {
    // Sthepenhall is reached by way of the falls.
    let mut world = first_leg();
    let before = world.clone();
    let from = world.party_at;
    let to = world.find_place("sthepenhall").unwrap();

    let result = world.apply(Command::Travel { to, hours: 1 });

    assert_eq!(result, Err(CommandError::NoRoad { from, to }));
    assert_eq!(world, before, "a refused command must change nothing");
}

#[test]
fn travelling_to_where_you_are_is_refused() {
    let mut world = first_leg();
    let here = world.party_at;
    assert_eq!(
        world.apply(Command::Travel { to: here, hours: 1 }),
        Err(CommandError::AlreadyThere)
    );
}

#[test]
fn making_camp_takes_an_hour_and_does_not_sleep() {
    let mut world = first_leg();
    let start = world.date;
    let events = world.apply(Command::MakeCamp).unwrap();
    assert!(world.camped);
    assert_eq!(world.hour, 1);
    assert_eq!(world.date, start);
    assert_eq!(events.len(), 1);

    let events = world.apply(Command::Rest).unwrap();
    assert_eq!(world.hour, 9);
    assert_eq!(world.date, start);
    assert!(world.camped);
    assert!(events
        .iter()
        .any(|event| matches!(event, Event::Rested { .. })));
    assert!(events
        .iter()
        .all(|event| !matches!(event, Event::DayPassed { .. })));
}

#[test]
fn the_journal_records_where_and_when() {
    let mut world = first_leg();
    walk(&mut world, &["falls"]);
    world
        .apply(Command::WriteJournal {
            text: "  The water is loud here.  ".to_string(),
        })
        .unwrap();

    let entry = world.journal.last().unwrap();
    assert_eq!(entry.text, "The water is loud here.");
    assert_eq!(entry.at, world.party_at);
    assert_eq!(entry.date, world.date);

    let empty = world.apply(Command::WriteJournal {
        text: "   ".to_string(),
    });
    assert_eq!(empty, Err(CommandError::EmptyJournalEntry));
}

#[test]
fn the_same_commands_give_the_same_history() {
    let stops = ["atzhaim"];
    let mut first = first_leg();
    let mut second = first_leg();
    assert_eq!(walk(&mut first, &stops), walk(&mut second, &stops));
    assert_eq!(first, second);
}

#[test]
fn a_saved_world_loads_back_identical() {
    let mut world = first_leg();
    walk(&mut world, &["atzhaim"]);
    world
        .apply(Command::WriteJournal {
            text: "A week on the road.".to_string(),
        })
        .unwrap();

    let saved = world.to_ron().expect("the world should save");
    let loaded = World::from_ron(&saved).expect("the save should load");
    assert_eq!(loaded, world);

    // The random number generator is saved too, so the future matches.
    let mut a = world;
    let mut b = loaded;
    assert_eq!(
        walk(&mut a, &["heedhaim", "hylhaim"]),
        walk(&mut b, &["heedhaim", "hylhaim"])
    );
}

#[test]
fn bad_data_is_reported() {
    let text = r#"(
        start: (year: 192, month: 1, day: 1, seed: 1, party_at: "nowhere"),
        places: [(key: "yhlgrym", name: "Yhlgrym", kind: TempleCity)],
    )"#;
    let error = load_world(text).unwrap_err();
    assert_eq!(
        error.to_string(),
        "the start names a place \"nowhere\" that is not in the file"
    );
}

#[test]
fn the_map_lists_every_place_the_roads_reach() {
    let world = first_leg();
    let routes = world.reachable(world.party_at);
    let names: Vec<&str> = routes.iter().map(|r| world.place_name(r.to)).collect();
    // Nearest first.
    assert_eq!(
        names[..4],
        ["Hylvynhani Falls", "Sthepenhall", "Atzhaim", "Ayin'lil"]
    );

    // The map draws no road to these, or only a road that joins nothing else.
    let cut_off: Vec<&str> = world
        .places
        .iter()
        .map(|p| p.name.as_str())
        .filter(|name| *name != "Yhlgrym" && !names.contains(name))
        .collect();
    assert_eq!(
        cut_off,
        ["Bluestone", "Ulugrozt", "Vhimyst", "Trystzor", "Vinniv'pti"]
    );
    assert_eq!(routes.len(), world.places.len() - 1 - cut_off.len());

    let atzhaim_id = world.find_place("atzhaim").unwrap();
    let atzhaim = routes.iter().find(|r| r.to == atzhaim_id).unwrap();
    assert_eq!(atzhaim.leagues, 60);
    assert_eq!(atzhaim.roads, 1);
    assert_eq!(atzhaim.hours, 60, "sixty leagues is sixty hours");
    assert_eq!(atzhaim.first_step, atzhaim_id);
}

#[test]
fn a_new_place_is_joined_to_where_the_party_stands() {
    let mut world = first_leg();
    walk(&mut world, &["falls"]);
    let falls = world.party_at;

    let events = world
        .apply(Command::AddPlace {
            name: "  Old   Mill Ford ".to_string(),
            leagues: 3,
        })
        .unwrap();

    let ford = world
        .find_place("old-mill-ford")
        .expect("the key comes from the name");
    assert_eq!(world.place_name(ford), "Old Mill Ford");
    assert_eq!(
        world.place(ford).unwrap().region,
        world.place(falls).unwrap().region
    );
    assert_eq!(world.road_between(falls, ford).unwrap().leagues, 3);
    assert_eq!(
        events,
        vec![Event::PlaceAdded {
            place: ford,
            from: falls,
            leagues: 3,
            on: world.date
        }]
    );

    // It is on the map from anywhere, and the party can walk there.
    assert!(world
        .reachable(world.find_place("atzhaim").unwrap())
        .iter()
        .any(|r| r.to == ford));
    walk(&mut world, &["Old Mill Ford"]);
    assert_eq!(world.party_at, ford);

    // A saved world keeps it.
    let loaded = World::from_ron(&world.to_ron().unwrap()).unwrap();
    assert_eq!(loaded, world);
}

#[test]
fn a_bad_new_place_is_refused() {
    let mut world = first_leg();
    let before = world.clone();
    let add = |name: &str, leagues| Command::AddPlace {
        name: name.to_string(),
        leagues,
    };

    assert_eq!(world.apply(add("  ", 2)), Err(CommandError::EmptyPlaceName));
    assert_eq!(world.apply(add("Ford", 0)), Err(CommandError::NoDistance));
    assert_eq!(
        world.apply(add("heedhaim", 2)),
        Err(CommandError::PlaceExists("Heedhaim".to_string()))
    );
    assert_eq!(world, before, "a refused command must change nothing");
}

#[test]
fn a_short_road_is_one_order_of_its_leagues() {
    // Sima to Hila is 6 leagues, so one order of 6 hours arrives.
    let mut world = first_leg();
    walk(
        &mut world,
        &["atzhaim", "heedhaim", "atzch-midili", "bahour-ilos", "sima"],
    );
    let hila = world.find_place("hila").unwrap();
    let events = world.apply(Command::Travel { to: hila, hours: 6 }).unwrap();
    assert_eq!(world.journey, None);
    assert_eq!(world.party_at, hila);
    assert!(events.iter().any(|event| matches!(
        event,
        Event::Traveled {
            leagues: 6,
            leagues_left: 0,
            hours: 6,
            ..
        }
    )));
}

#[test]
fn an_order_stops_after_its_hours_and_camp_does_not_walk() {
    let mut world = first_leg();
    let atzhaim = world.find_place("atzhaim").unwrap();
    let yhlgrym = world.party_at;

    let events = world
        .apply(Command::Travel {
            to: atzhaim,
            hours: 8,
        })
        .unwrap();
    assert!(matches!(
        events.last(),
        Some(Event::Traveled {
            leagues: 8,
            leagues_left: 52,
            hours: 8,
            ..
        })
    ));
    let journey = world.journey.expect("the canyon is not finished");
    assert_eq!((journey.leagues, journey.leagues_walked), (60, 8));
    assert_eq!(world.party_at, yhlgrym, "the party has not arrived yet");
    assert_eq!(world.hour, 8);

    for person in world.people.iter_mut().filter(|person| person.in_party) {
        if let Some(sheet) = person.sheet.as_mut() {
            sheet.fatigue = 80;
        }
    }
    let events = world.apply(Command::MakeCamp).unwrap();
    assert!(matches!(events.as_slice(), [Event::CampedOnRoad { .. }]));
    assert!(world.camped);
    assert_eq!(world.hour, 9);
    assert_eq!(party_fatigue(&world), [80, 80]);
    assert_eq!(world.journey.unwrap().leagues_walked, 8);

    // Sleep is the night. It restores fatigue and does not cover leagues.
    let events = world.apply(Command::Rest).unwrap();
    assert!(matches!(events.first(), Some(Event::Rested { .. })));
    assert!(world.camped);
    assert_eq!(world.hour, 17);
    assert_eq!(party_fatigue(&world), [85, 85]);
    assert_eq!(world.journey.unwrap().leagues_walked, 8);

    world.apply(Command::PackUp).unwrap();
    assert!(!world.camped);
    assert_eq!(world.hour, 18);
    world
        .apply(Command::Travel {
            to: atzhaim,
            hours: 8,
        })
        .unwrap();
    assert_eq!(world.journey.unwrap().leagues_walked, 16);
}

fn party_fatigue(world: &World) -> Vec<u32> {
    world
        .party()
        .filter_map(|person| person.sheet.as_ref().map(|sheet| sheet.fatigue))
        .collect()
}

#[test]
fn the_canyon_is_sixty_hours_of_travel() {
    let mut world = first_leg();
    let start = world.date;
    let atzhaim = world.find_place("atzhaim").unwrap();
    let mut traveled = 0;
    while world.party_at != atzhaim {
        let left = world
            .journey
            .map(|journey| journey.leagues_left())
            .unwrap_or(60);
        let hours = left.min(8);
        world
            .apply(Command::Travel {
                to: atzhaim,
                hours,
            })
            .unwrap();
        traveled += hours;
    }
    assert_eq!(traveled, 60);
    assert_eq!(world.journey, None);
    assert_eq!(world.hour, 12);
    assert_eq!(world.date.days_since(start), 2);
    assert_eq!(party_fatigue(&world), [100, 100]);
}

#[test]
fn on_the_road_only_the_night_can_be_answered() {
    let mut world = first_leg();
    let atzhaim = world.find_place("atzhaim").unwrap();
    world
        .apply(Command::Travel {
            to: atzhaim,
            hours: 8,
        })
        .unwrap();
    let before = world.clone();

    let falls = world.find_place("falls").unwrap();
    assert_eq!(
        world.apply(Command::Travel {
            to: falls,
            hours: 8,
        }),
        Err(CommandError::OnTheRoad)
    );
    assert_eq!(world.apply(Command::Rest), Err(CommandError::NotCamped));
    assert_eq!(world.apply(Command::PackUp), Err(CommandError::NotCamped));
    assert_eq!(world.apply(Command::Sleep), Err(CommandError::OnTheRoad));
    assert_eq!(
        world.apply(Command::WriteJournal {
            text: "A note.".to_string()
        }),
        Err(CommandError::OnTheRoad)
    );
    assert_eq!(
        world.apply(Command::AddPlace {
            name: "Ford".to_string(),
            leagues: 1
        }),
        Err(CommandError::OnTheRoad)
    );
    assert_eq!(world, before, "a refused command must change nothing");

    travel_to(&mut world, atzhaim);
    assert_eq!(
        world.apply(Command::Travel {
            to: atzhaim,
            hours: 0,
        }),
        Err(CommandError::NoHours)
    );
}

#[test]
fn a_world_saved_on_the_road_loads_back_on_the_road() {
    let mut world = first_leg();
    let atzhaim = world.find_place("atzhaim").unwrap();
    world
        .apply(Command::Travel {
            to: atzhaim,
            hours: 8,
        })
        .unwrap();

    let loaded = World::from_ron(&world.to_ron().unwrap()).unwrap();
    assert_eq!(loaded, world);
    assert_eq!(loaded.hour, world.hour);
    let mut a = world;
    let mut b = loaded;
    assert_eq!(
        a.apply(Command::Travel {
            to: atzhaim,
            hours: 8,
        }),
        b.apply(Command::Travel {
            to: atzhaim,
            hours: 8,
        })
    );
}
