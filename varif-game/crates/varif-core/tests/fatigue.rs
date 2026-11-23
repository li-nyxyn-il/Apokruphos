//! Fatigue: a long travel order, camping on the road, and a night at an inn.

use varif_core::{
    load_world, Command, CommandError, Event, ItemType, MemberFatigue, PersonId, SiteKind, World,
};

const FIRST_LEG: &str = include_str!("../../../data/first_leg.ron");

fn first_leg() -> World {
    load_world(FIRST_LEG).expect("the first leg data should load")
}

fn set_fatigue(world: &mut World, key: &str, fatigue: u32, debt: bool) {
    let sheet = world
        .people
        .iter_mut()
        .find(|candidate| candidate.key == key)
        .and_then(|candidate| candidate.sheet.as_mut())
        .unwrap_or_else(|| panic!("{key} should have a sheet"));
    sheet.fatigue = fatigue;
    sheet.road_rest_debt = debt;
}

fn fatigue(world: &World, key: &str) -> (u32, bool) {
    let sheet = world
        .people
        .iter()
        .find(|candidate| candidate.key == key)
        .and_then(|candidate| candidate.sheet.as_ref())
        .unwrap_or_else(|| panic!("{key} should have a sheet"));
    (sheet.fatigue, sheet.road_rest_debt)
}

/// Pitch camp if it is not already up, then sleep. The camp stays pitched.
fn camp_the_night(world: &mut World) -> Vec<Event> {
    let mut events = Vec::new();
    if !world.camped {
        events.extend(world.apply(Command::MakeCamp).unwrap());
    }
    events.extend(world.apply(Command::Rest).unwrap());
    events
}

fn copper(world: &World) -> u32 {
    world
        .party()
        .filter_map(|person| person.sheet.as_ref())
        .map(|sheet| sheet.holding(ItemType::Copper))
        .sum()
}

fn travel_to(world: &mut World, stop: &str) {
    let to = world.find_place(stop).unwrap();
    while world.party_at != to || world.journey.is_some() {
        let left = world
            .journey
            .map(|journey| journey.leagues_left())
            .unwrap_or_else(|| world.road_between(world.party_at, to).unwrap().leagues);
        world
            .apply(Command::Travel {
                to,
                hours: left.min(8),
            })
            .unwrap();
    }
}

fn on_the_canyon(world: &mut World) {
    travel_to(world, "falls");
    travel_to(world, "yhlgrym");
    let atzhaim = world.find_place("atzhaim").unwrap();
    world
        .apply(Command::Travel {
            to: atzhaim,
            hours: 8,
        })
        .unwrap();
    assert!(
        world.journey.is_some(),
        "eight hours does not cross the canyon"
    );
}

fn levels(pylgrym: u32, donold: u32) -> Vec<MemberFatigue> {
    vec![
        MemberFatigue {
            person: PersonId(0),
            fatigue: pylgrym,
        },
        MemberFatigue {
            person: PersonId(1),
            fatigue: donold,
        },
    ]
}

#[test]
fn road_camping_restores_five_and_holds_the_ceiling() {
    let mut world = first_leg();
    on_the_canyon(&mut world);

    // Already at the cap: +5 does not pass 100.
    let events = camp_the_night(&mut world);
    assert!(matches!(
        events.iter().find(|event| matches!(event, Event::Rested { .. })),
        Some(Event::Rested { fatigue, .. }) if fatigue == &levels(100, 100)
    ));
    assert_eq!(fatigue_pair(&world), (100, 100));

    // Above 50, with no debt, a camp may pass 50. At 47 after a drop below 50, it stops at 50.
    set_fatigue(&mut world, "pylgrym", 60, false);
    set_fatigue(&mut world, "donold", 47, true);
    let events = camp_the_night(&mut world);
    assert!(matches!(
        events.iter().find(|event| matches!(event, Event::Rested { .. })),
        Some(Event::Rested { fatigue, .. }) if fatigue == &levels(65, 50)
    ));
    assert_eq!(fatigue(&world, "pylgrym"), (65, false));
    assert_eq!(fatigue(&world, "donold"), (50, true));

    // 30 becomes 35. 98 stops at 100. The debt is not cleared by the road.
    set_fatigue(&mut world, "pylgrym", 98, false);
    set_fatigue(&mut world, "donold", 30, true);
    camp_the_night(&mut world);
    assert_eq!(fatigue(&world, "pylgrym"), (100, false));
    assert_eq!(fatigue(&world, "donold"), (35, true));

    // 47 without that debt is not held at 50.
    set_fatigue(&mut world, "pylgrym", 47, false);
    camp_the_night(&mut world);
    assert_eq!(fatigue(&world, "pylgrym"), (52, false));
}

#[test]
fn an_inn_night_follows_the_examples_and_lifts_the_ceiling() {
    let mut world = first_leg();
    let here = world.party_at;

    let before = world.clone();
    assert_eq!(world.apply(Command::Sleep), Err(CommandError::NotAtAnInn));
    assert_eq!(world, before, "a refused sleep changes nothing");

    let palace = world.place(here).unwrap().find_site("palace").unwrap();
    world.apply(Command::Walk { to: palace }).unwrap();
    let before = world.clone();
    assert_eq!(world.apply(Command::Sleep), Err(CommandError::NotAtAnInn));
    assert_eq!(world, before);

    let inn = world.place(here).unwrap().find_site("inn").unwrap();
    assert_eq!(
        world.place(here).unwrap().site(inn).unwrap().kind,
        SiteKind::Inn
    );
    world.apply(Command::Walk { to: inn }).unwrap();
    // 30 and 40, both in debt. The inn night clears the debt.
    set_fatigue(&mut world, "pylgrym", 30, true);
    set_fatigue(&mut world, "donold", 40, true);
    let start = world.date;

    assert_eq!(copper(&world), 10);
    let events = world.apply(Command::Sleep).unwrap();
    assert!(matches!(
        events.first(),
        Some(Event::Slept { fatigue, copper: 5, .. }) if fatigue == &levels(50, 55)
    ));
    assert_eq!(copper(&world), 5);
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::DayPassed { .. })),
        "eight hours from midnight do not roll the date"
    );
    assert_eq!(fatigue(&world, "pylgrym"), (50, false));
    assert_eq!(fatigue(&world, "donold"), (55, false));
    assert!(!world.camped);
    assert_eq!(world.date, start);
    assert_eq!((world.hour, world.minute), (8, 0));

    // The second night adds 15: 50 becomes 65, 55 becomes 70. It spends the rest.
    world.apply(Command::Sleep).unwrap();
    assert_eq!(fatigue(&world, "pylgrym"), (65, false));
    assert_eq!(fatigue(&world, "donold"), (70, false));
    assert_eq!(copper(&world), 0);
    assert_eq!(world.date, start);
    assert_eq!((world.hour, world.minute), (16, 0));

    // No copper: the night is refused and the world stays as it was.
    let before = world.clone();
    assert_eq!(world.apply(Command::Sleep), Err(CommandError::CannotAfford));
    assert_eq!(world, before);

    // The ceiling is gone, so a later camp on the road may pass 50.
    on_the_canyon(&mut world);
    camp_the_night(&mut world);
    assert_eq!(fatigue(&world, "pylgrym"), (70, false));
    assert_eq!(fatigue(&world, "donold"), (75, false));
}

#[test]
fn a_residence_sleeps_by_the_same_rule() {
    // ASSUMPTION: a bed is the site the party is standing at, when its kind
    // is Inn or Residence. Maphlamar's Pentor Palace is the residence in the
    // data. The party is placed there; the road to Maphlamar is not the rule.
    let mut world = first_leg();
    let maphlamar = world.find_place("maphlamar").unwrap();
    let palace = world.place(maphlamar).unwrap().find_site("estate").unwrap();
    assert_eq!(
        world.place(maphlamar).unwrap().site(palace).unwrap().kind,
        SiteKind::Residence
    );
    world.party_at = maphlamar;
    world.party_site = Some(palace);
    set_fatigue(&mut world, "pylgrym", 30, true);
    set_fatigue(&mut world, "donold", 40, true);
    let start = world.date;
    assert_eq!(copper(&world), 10);

    world.apply(Command::Sleep).unwrap();
    assert_eq!(fatigue(&world, "pylgrym"), (50, false));
    assert_eq!(fatigue(&world, "donold"), (55, false));
    assert_eq!(world.date, start);
    assert_eq!((world.hour, world.minute), (8, 0));
    assert_eq!(copper(&world), 10, "a residence does not cost copper");
}

#[test]
fn travel_fatigue_skips_a_member_without_a_sheet() {
    let mut world = first_leg();
    let falls = world.find_place("falls").unwrap();
    world
        .people
        .iter_mut()
        .find(|candidate| candidate.key == "donold")
        .unwrap()
        .sheet = None;

    // Nine hours is one extra hour, which costs 1, and only the Pylgrym pays.
    let events = world
        .apply(Command::Travel {
            to: falls,
            hours: 9,
        })
        .unwrap();
    let fatigue = events.iter().find_map(|event| match event {
        Event::Traveled { fatigue, .. } => Some(fatigue),
        _ => None,
    });
    assert_eq!(fatigue.unwrap().as_slice(), &levels(99, 99)[..1]);
    assert_eq!(fatigue_pair_pylgrym(&world), (99, false));
    assert!(world
        .people
        .iter()
        .find(|person| person.key == "donold")
        .unwrap()
        .sheet
        .is_none());

    // With no sheet, Donold cannot block the order. The Pylgrym can.
    set_fatigue(&mut world, "pylgrym", 0, false);
    let before = world.clone();
    assert_eq!(
        world.apply(Command::Travel {
            to: falls,
            hours: 9,
        }),
        Err(CommandError::TooTired)
    );
    assert_eq!(world, before);
}

fn fatigue_pair_pylgrym(world: &World) -> (u32, bool) {
    fatigue(world, "pylgrym")
}

fn fatigue_pair(world: &World) -> (u32, u32) {
    (fatigue(world, "pylgrym").0, fatigue(world, "donold").0)
}
