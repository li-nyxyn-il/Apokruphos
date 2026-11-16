//! Tests that read like the journey: load the first leg and walk it.

use varif_core::{load_world, Command, CommandError, Date, Event, Item, World, FOOD, WATER, WOOD};

const FIRST_LEG: &str = include_str!("../../../data/first_leg.ron");

fn first_leg() -> World {
    load_world(FIRST_LEG).expect("the first leg data should load")
}

/// Walks the party along the road through each named stop in turn, making
/// camp every night.
fn walk(world: &mut World, stops: &[&str]) -> Vec<Event> {
    let mut events = Vec::new();
    for stop in stops {
        let to = world.find_place(stop).expect("the stop should exist");
        events.extend(
            world
                .apply(Command::Travel { to })
                .expect("the road should exist"),
        );
        events.extend(finish_journey(world, true));
    }
    events
}

/// Answers every night of the journey under way the same way.
fn finish_journey(world: &mut World, camp: bool) -> Vec<Event> {
    let mut events = Vec::new();
    while world.journey.is_some() {
        events.extend(
            world
                .apply(Command::EndDay { camp })
                .expect("the party is on the road"),
        );
    }
    events
}

fn count(events: &[Event], wanted: fn(&Event) -> bool) -> usize {
    events.iter().filter(|e| wanted(e)).count()
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

    // 60 leagues at a camped 9 leagues a day: 7 days.
    assert_eq!(world.date.days_since(start), 7);
    let days_passed = events
        .iter()
        .filter(|e| matches!(e, Event::DayPassed { .. }))
        .count();
    assert_eq!(days_passed, 7);

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

    let result = world.apply(Command::Travel { to });

    assert_eq!(result, Err(CommandError::NoRoad { from, to }));
    assert_eq!(world, before, "a refused command must change nothing");
}

#[test]
fn travelling_to_where_you_are_is_refused() {
    let mut world = first_leg();
    let here = world.party_at;
    assert_eq!(
        world.apply(Command::Travel { to: here }),
        Err(CommandError::AlreadyThere)
    );
}

#[test]
fn making_camp_does_not_pass_the_night() {
    let mut world = first_leg();
    let start = world.date;
    world.nights_without_camp = 2;
    let events = world.apply(Command::MakeCamp).unwrap();
    assert!(world.camped);
    assert_eq!(world.date, start);
    assert_eq!(world.nights_without_camp, 2);
    assert_eq!(events.len(), 1);

    let events = world.apply(Command::Rest).unwrap();
    assert_eq!(world.date, start.next_day());
    assert_eq!(world.nights_without_camp, 0);
    assert!(events
        .iter()
        .any(|event| matches!(event, Event::DayPassed { .. })));
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
    assert_eq!(atzhaim.days, 7, "the same 7 days as walking it");
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
fn a_one_day_walk_does_not_stop_for_the_night() {
    // The one road short enough to walk in a day is Sima to Hila, 6 leagues.
    let mut world = first_leg();
    walk(
        &mut world,
        &["atzhaim", "heedhaim", "atzch-midili", "bahour-ilos", "sima"],
    );
    let hila = world.find_place("hila").unwrap();
    let events = world.apply(Command::Travel { to: hila }).unwrap();
    assert_eq!(world.journey, None);
    assert_eq!(world.party_at, hila);
    assert_eq!(count(&events, |e| matches!(e, Event::NightFalls { .. })), 0);
}

#[test]
fn a_long_walk_stops_each_night_to_ask_about_camp() {
    let mut world = first_leg();
    walk(&mut world, &["falls"]);
    let falls = world.party_at;
    let sthepenhall = world.find_place("sthepenhall").unwrap();

    // The falls to Sthepenhall is 27 leagues, 3 days, so 2 nights on the road.
    let events = world.apply(Command::Travel { to: sthepenhall }).unwrap();
    assert!(matches!(
        events.last(),
        Some(Event::NightFalls { days_left: 2, .. })
    ));
    let journey = world.journey.expect("the party is on the road");
    assert_eq!((journey.leagues, journey.leagues_walked), (27, 9));
    assert_eq!(world.party_at, falls, "the party has not arrived yet");

    let events = world.apply(Command::EndDay { camp: true }).unwrap();
    assert!(matches!(events.first(), Some(Event::CampedOnRoad { .. })));
    assert!(matches!(
        events.last(),
        Some(Event::NightFalls { days_left: 1, .. })
    ));

    let events = world.apply(Command::EndDay { camp: false }).unwrap();
    assert!(matches!(
        events.first(),
        Some(Event::PressedOn {
            nights_without_camp: 1,
            ..
        })
    ));
    assert!(matches!(events.last(), Some(Event::Arrived { .. })));
    assert_eq!(world.journey, None);
    assert_eq!(world.party_at, sthepenhall);
}

#[test]
fn nights_without_camp_are_counted_until_the_party_camps() {
    // The falls to Sthepenhall is 27 leagues, 3 days, so 2 nights.
    let mut world = first_leg();
    walk(&mut world, &["falls"]);
    assert_eq!(world.nights_without_camp, 0);
    let falls = world.party_at;
    let sthepenhall = world.find_place("sthepenhall").unwrap();
    world.apply(Command::Travel { to: sthepenhall }).unwrap();
    finish_journey(&mut world, false);
    assert_eq!(world.nights_without_camp, 2);
    // Two presses cost 10, then 15. Both members started at 100.
    assert_eq!(party_fatigue(&world), [75, 75]);
    assert_eq!(party_debt(&world), [false, false]);

    // The count carries on to the next road, so the next presses are nights
    // 3 and 4 (20, then 25). 30 is below 50, which sets the road-rest debt.
    // Making camp neither clears the streak nor restores fatigue. Sleeping
    // in camp is the night that does both, and the debt keeps the restore
    // from passing 50.
    world.apply(Command::Travel { to: falls }).unwrap();
    finish_journey(&mut world, false);
    assert_eq!(world.nights_without_camp, 4);
    assert_eq!(party_fatigue(&world), [30, 30]);
    assert_eq!(party_debt(&world), [true, true]);
    stock_camp_supplies(&mut world);
    let camped_on = world.date;
    world.apply(Command::MakeCamp).unwrap();
    assert!(world.camped);
    assert_eq!(world.date, camped_on);
    assert_eq!(world.nights_without_camp, 4);
    assert_eq!(party_fatigue(&world), [30, 30]);
    world.apply(Command::Rest).unwrap();
    assert_eq!(world.nights_without_camp, 0);
    assert_eq!(party_fatigue(&world), [35, 35]);
    assert_eq!(party_debt(&world), [true, true]);
    assert_eq!(world.date, camped_on.next_day());
}

fn stock_camp_supplies(world: &mut World) {
    let sheet = world
        .people
        .iter_mut()
        .find(|person| person.key == "pylgrym")
        .and_then(|person| person.sheet.as_mut())
        .expect("pylgrym should have a sheet");
    for (key, name) in [(FOOD, "Food"), (WATER, "Water"), (WOOD, "Wood")] {
        sheet.carry(Item {
            key: key.to_string(),
            name: name.to_string(),
            count: 1,
        });
    }
}

fn party_fatigue(world: &World) -> Vec<u32> {
    world
        .party()
        .filter_map(|person| person.sheet.as_ref().map(|sheet| sheet.fatigue))
        .collect()
}

fn party_debt(world: &World) -> Vec<bool> {
    world
        .party()
        .filter_map(|person| person.sheet.as_ref().map(|sheet| sheet.road_rest_debt))
        .collect()
}

#[test]
fn pressing_on_arrives_sooner_than_camping() {
    let mut camping = first_leg();
    let mut pressing = first_leg();
    let start = camping.date;
    for (world, camp) in [(&mut camping, true), (&mut pressing, false)] {
        for stop in ["atzhaim"] {
            let to = world.find_place(stop).unwrap();
            world.apply(Command::Travel { to }).unwrap();
            finish_journey(world, camp);
        }
    }
    // 60 leagues. Camping is 7 days of 9. Pressing covers 9, then 15, 13, 11,
    // 10 and 10, and arrives on day 6.
    assert_eq!(camping.date.days_since(start), 7);
    assert_eq!(pressing.date.days_since(start), 6);
}

#[test]
fn on_the_road_only_the_night_can_be_answered() {
    let mut world = first_leg();
    let atzhaim = world.find_place("atzhaim").unwrap();
    world.apply(Command::Travel { to: atzhaim }).unwrap();
    let before = world.clone();

    let falls = world.find_place("falls").unwrap();
    assert_eq!(
        world.apply(Command::Travel { to: falls }),
        Err(CommandError::OnTheRoad)
    );
    assert_eq!(world.apply(Command::MakeCamp), Err(CommandError::OnTheRoad));
    assert_eq!(world.apply(Command::Rest), Err(CommandError::OnTheRoad));
    assert_eq!(world.apply(Command::PackUp), Err(CommandError::OnTheRoad));
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

    finish_journey(&mut world, true);
    assert_eq!(
        world.apply(Command::EndDay { camp: true }),
        Err(CommandError::NotOnTheRoad)
    );
}

#[test]
fn a_world_saved_on_the_road_loads_back_on_the_road() {
    let mut world = first_leg();
    let atzhaim = world.find_place("atzhaim").unwrap();
    world.apply(Command::Travel { to: atzhaim }).unwrap();

    let loaded = World::from_ron(&world.to_ron().unwrap()).unwrap();
    assert_eq!(loaded, world);
    let mut a = world;
    let mut b = loaded;
    assert_eq!(finish_journey(&mut a, false), finish_journey(&mut b, false));
}
