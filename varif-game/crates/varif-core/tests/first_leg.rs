//! Tests that read like the journey: load the first leg and walk it.

use varif_core::{load_world, Command, CommandError, Date, Event, World};

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
        events.extend(world.apply(Command::Travel { to }).expect("the road should exist"));
        events.extend(finish_journey(world, true));
    }
    events
}

/// Answers every night of the journey under way the same way.
fn finish_journey(world: &mut World, camp: bool) -> Vec<Event> {
    let mut events = Vec::new();
    while world.journey.is_some() {
        events.extend(world.apply(Command::EndDay { camp }).expect("the party is on the road"));
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
    assert_eq!(world.party().count(), 1);
}

#[test]
fn the_party_walks_from_yhlgrym_to_atzhaim() {
    let mut world = first_leg();
    let start = world.date;
    let events = walk(&mut world, &["falls", "atzhaim"]);

    let atzhaim = world.find_place("Atzhaim").unwrap();
    assert_eq!(world.party_at, atzhaim);
    let south_gate = world.place(atzhaim).unwrap().find_site("south gate").unwrap();
    assert_eq!(
        events[events.len() - 2..],
        [
            Event::Arrived { at: atzhaim, on: world.date },
            Event::EnteredBy { at: atzhaim, gate: south_gate, on: world.date },
        ]
    );

    // 15 and 104 leagues at 9 leagues a day: 2 + 12 days.
    assert_eq!(world.date.days_since(start), 14);
    let days_passed = events.iter().filter(|e| matches!(e, Event::DayPassed { .. })).count();
    assert_eq!(days_passed, 14);

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
    let stops = ["falls", "atzhaim"];
    let mut first = first_leg();
    let mut second = first_leg();
    assert_eq!(walk(&mut first, &stops), walk(&mut second, &stops));
    assert_eq!(first, second);
}

#[test]
fn a_saved_world_loads_back_identical() {
    let mut world = first_leg();
    walk(&mut world, &["falls", "atzhaim"]);
    world.apply(Command::WriteJournal { text: "Two weeks on the road.".to_string() }).unwrap();

    let saved = world.to_ron().expect("the world should save");
    let loaded = World::from_ron(&saved).expect("the save should load");
    assert_eq!(loaded, world);

    // The random number generator is saved too, so the future matches.
    let mut a = world;
    let mut b = loaded;
    assert_eq!(walk(&mut a, &["heedhaim", "hylhaim"]), walk(&mut b, &["heedhaim", "hylhaim"]));
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

#[test]
fn the_map_lists_every_place_the_roads_reach() {
    let world = first_leg();
    let routes = world.reachable(world.party_at);
    let names: Vec<&str> = routes.iter().map(|r| world.place_name(r.to)).collect();
    // Nearest first.
    assert_eq!(names[..4], ["Hylvynhani Falls", "Sthepenhall", "Ayin'lil", "Pheo"]);

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
    assert_eq!(atzhaim.leagues, 15 + 104);
    assert_eq!(atzhaim.roads, 2);
    assert_eq!(atzhaim.days, 14, "the same 2 + 12 days as walking it");
    assert_eq!(atzhaim.first_step, world.find_place("falls").unwrap());
}

#[test]
fn a_new_place_is_joined_to_where_the_party_stands() {
    let mut world = first_leg();
    walk(&mut world, &["falls"]);
    let falls = world.party_at;

    let events = world
        .apply(Command::AddPlace { name: "  Old   Mill Ford ".to_string(), leagues: 3 })
        .unwrap();

    let ford = world.find_place("old-mill-ford").expect("the key comes from the name");
    assert_eq!(world.place_name(ford), "Old Mill Ford");
    assert_eq!(world.place(ford).unwrap().region, world.place(falls).unwrap().region);
    assert_eq!(world.road_between(falls, ford).unwrap().leagues, 3);
    assert_eq!(events, vec![Event::PlaceAdded { place: ford, from: falls, leagues: 3, on: world.date }]);

    // It is on the map from anywhere, and the party can walk there.
    assert!(world.reachable(world.find_place("atzhaim").unwrap()).iter().any(|r| r.to == ford));
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
    let add = |name: &str, leagues| Command::AddPlace { name: name.to_string(), leagues };

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
    walk(&mut world, &["falls", "atzhaim", "heedhaim", "atzch-midili", "bahour-ilos", "sima"]);
    let hila = world.find_place("hila").unwrap();
    let events = world.apply(Command::Travel { to: hila }).unwrap();
    assert_eq!(world.journey, None);
    assert_eq!(world.party_at, hila);
    assert_eq!(count(&events, |e| matches!(e, Event::NightFalls { .. })), 0);
}

#[test]
fn a_long_walk_stops_each_night_to_ask_about_camp() {
    let mut world = first_leg();
    let yhlgrym = world.party_at;
    let sthepenhall = world.find_place("sthepenhall").unwrap();

    // 27 leagues is 3 days, so 2 nights on the road.
    let events = world.apply(Command::Travel { to: sthepenhall }).unwrap();
    assert!(matches!(events.last(), Some(Event::NightFalls { days_left: 2, .. })));
    let journey = world.journey.expect("the party is on the road");
    assert_eq!((journey.days, journey.days_walked), (3, 1));
    assert_eq!(world.party_at, yhlgrym, "the party has not arrived yet");

    let events = world.apply(Command::EndDay { camp: true }).unwrap();
    assert!(matches!(events.first(), Some(Event::CampedOnRoad { .. })));
    assert!(matches!(events.last(), Some(Event::NightFalls { days_left: 1, .. })));

    let events = world.apply(Command::EndDay { camp: false }).unwrap();
    assert!(matches!(events.first(), Some(Event::PressedOn { nights_without_camp: 1, .. })));
    assert!(matches!(events.last(), Some(Event::Arrived { .. })));
    assert_eq!(world.journey, None);
    assert_eq!(world.party_at, sthepenhall);
}

#[test]
fn nights_without_camp_are_counted_until_the_party_camps() {
    // 27 leagues to Sthepenhall is 3 days, so 2 nights.
    let mut world = first_leg();
    let yhlgrym = world.party_at;
    let sthepenhall = world.find_place("sthepenhall").unwrap();
    world.apply(Command::Travel { to: sthepenhall }).unwrap();
    finish_journey(&mut world, false);
    assert_eq!(world.nights_without_camp, 2);

    // The count carries on to the next road, and camping clears it.
    world.apply(Command::Travel { to: yhlgrym }).unwrap();
    finish_journey(&mut world, false);
    assert_eq!(world.nights_without_camp, 4);
    world.apply(Command::MakeCamp).unwrap();
    assert_eq!(world.nights_without_camp, 0);
}

#[test]
fn camping_or_not_the_walk_takes_the_same_days() {
    let mut camping = first_leg();
    let mut pressing = first_leg();
    let start = camping.date;
    for (world, camp) in [(&mut camping, true), (&mut pressing, false)] {
        for stop in ["falls", "atzhaim"] {
            let to = world.find_place(stop).unwrap();
            world.apply(Command::Travel { to }).unwrap();
            finish_journey(world, camp);
        }
    }
    assert_eq!(camping.date.days_since(start), 14);
    assert_eq!(pressing.date, camping.date);
}

#[test]
fn on_the_road_only_the_night_can_be_answered() {
    let mut world = first_leg();
    walk(&mut world, &["falls"]);
    let atzhaim = world.find_place("atzhaim").unwrap();
    world.apply(Command::Travel { to: atzhaim }).unwrap();
    let before = world.clone();

    let yhlgrym = world.find_place("yhlgrym").unwrap();
    assert_eq!(world.apply(Command::Travel { to: yhlgrym }), Err(CommandError::OnTheRoad));
    assert_eq!(world.apply(Command::MakeCamp), Err(CommandError::OnTheRoad));
    assert_eq!(
        world.apply(Command::WriteJournal { text: "A note.".to_string() }),
        Err(CommandError::OnTheRoad)
    );
    assert_eq!(
        world.apply(Command::AddPlace { name: "Ford".to_string(), leagues: 1 }),
        Err(CommandError::OnTheRoad)
    );
    assert_eq!(world, before, "a refused command must change nothing");

    finish_journey(&mut world, true);
    assert_eq!(world.apply(Command::EndDay { camp: true }), Err(CommandError::NotOnTheRoad));
}

#[test]
fn a_world_saved_on_the_road_loads_back_on_the_road() {
    let mut world = first_leg();
    walk(&mut world, &["falls"]);
    let atzhaim = world.find_place("atzhaim").unwrap();
    world.apply(Command::Travel { to: atzhaim }).unwrap();

    let loaded = World::from_ron(&world.to_ron().unwrap()).unwrap();
    assert_eq!(loaded, world);
    let mut a = world;
    let mut b = loaded;
    assert_eq!(finish_journey(&mut a, false), finish_journey(&mut b, false));
}
