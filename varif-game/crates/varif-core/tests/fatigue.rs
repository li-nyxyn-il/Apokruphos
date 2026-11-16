//! Fatigue: pressing on, camping on the road, and a night at an inn.

use varif_core::{
    load_world, Command, CommandError, Event, ItemType, MemberFatigue, PersonId, SiteKind, World,
};

const FIRST_LEG: &str = include_str!("../../../data/first_leg.ron");

fn first_leg() -> World {
    load_world(FIRST_LEG).expect("the first leg data should load")
}

fn person(world: &World, key: &str) -> PersonId {
    let index = world
        .people
        .iter()
        .position(|candidate| candidate.key == key)
        .unwrap_or_else(|| panic!("no person {key}"));
    PersonId(index as u32)
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

/// Pitch camp, then sleep. On the road the sleep is the camped march.
fn camp_the_night(world: &mut World) -> Vec<Event> {
    let mut events = world.apply(Command::MakeCamp).unwrap();
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

fn on_the_canyon(world: &mut World) {
    let atzhaim = world.find_place("atzhaim").unwrap();
    world.apply(Command::Travel { to: atzhaim }).unwrap();
    assert!(world.journey.is_some(), "the canyon is more than a day");
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

/// The press at the head of `events`: night of the streak, extra leagues, fatigue.
fn press(events: &[Event]) -> (u32, u32, &[MemberFatigue]) {
    match events.first() {
        Some(Event::PressedOn {
            nights_without_camp,
            extra_leagues,
            fatigue,
            ..
        }) => (*nights_without_camp, *extra_leagues, fatigue),
        other => panic!("expected a press, got {other:?}"),
    }
}

#[test]
fn pressing_on_costs_more_and_covers_less_each_night() {
    let mut world = first_leg();
    let start = world.date;
    let (pylgrym, donold) = (person(&world, "pylgrym"), person(&world, "donold"));
    assert_eq!((pylgrym, donold), (PersonId(0), PersonId(1)));
    on_the_canyon(&mut world);
    assert_eq!(world.journey.unwrap().leagues_walked, 9);

    // Night 1: +6 leagues, 10 fatigue. 2: +4, 15. 3: +2, 20.
    // From the fourth night on: +1 league, and 10 + 5*(n-1) fatigue.
    let curve = [
        (1, 6_u32, 90_u32, 24_u32),
        (2, 4, 75, 37),
        (3, 2, 55, 48),
        (4, 1, 30, 58),
    ];
    for (night, extra, left, walked) in curve {
        let events = world.apply(Command::EndDay { camp: false }).unwrap();
        let (got_night, got_extra, after) = press(&events);
        assert_eq!((got_night, got_extra), (night, extra));
        assert_eq!(after, levels(left, left).as_slice());
        assert_eq!(fatigue_pair(&world), (left, left));
        // 55 stays clear of the debt. 30, the first result below 50, sets it.
        assert_eq!(fatigue(&world, "pylgrym").1, left < 50);
        assert_eq!(fatigue(&world, "donold").1, left < 50);
        assert_eq!(world.journey.unwrap().leagues_walked, walked);
        assert_eq!(world.nights_without_camp, night);
    }

    // Night 5 costs 30 and lands on 0, which is allowed. The road is finished.
    let events = world.apply(Command::EndDay { camp: false }).unwrap();
    let (night, extra, fatigue_after) = press(&events);
    assert_eq!((night, extra), (5, 1));
    assert_eq!(fatigue_after, levels(0, 0).as_slice());
    let atzhaim = world.find_place("atzhaim").unwrap();
    assert!(events
        .iter()
        .any(|event| matches!(event, Event::Arrived { at, .. } if *at == atzhaim)));
    assert_eq!(world.journey, None);
    assert_eq!(world.party_at, atzhaim);
    assert_eq!(world.date.days_since(start), 6);
    assert_eq!(
        fatigue(&world, "jonnoj"),
        (100, false),
        "someone not in the party is untouched"
    );
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
fn camping_clears_the_press_streak() {
    let mut world = first_leg();
    on_the_canyon(&mut world);

    let events = world.apply(Command::EndDay { camp: false }).unwrap();
    assert_eq!(press(&events).0, 1);
    assert_eq!(press(&events).1, 6);
    assert_eq!(fatigue_pair(&world), (90, 90));

    camp_the_night(&mut world);
    assert_eq!(world.nights_without_camp, 0);
    assert_eq!(fatigue_pair(&world), (95, 95));

    // The next press is night 1 again: +6 leagues and 10 fatigue, not night 2.
    let events = world.apply(Command::EndDay { camp: false }).unwrap();
    assert_eq!((press(&events).0, press(&events).1), (1, 6));
    assert_eq!(fatigue_pair(&world), (85, 85));
    assert_eq!(world.journey.unwrap().leagues_walked, 9 + 15 + 9 + 15);
}

#[test]
fn an_inn_night_follows_the_examples_and_lifts_the_ceiling() {
    let mut world = first_leg();
    let yhlgrym = world.party_at;

    let before = world.clone();
    assert_eq!(world.apply(Command::Sleep), Err(CommandError::NotAtAnInn));
    assert_eq!(world, before, "a refused sleep changes nothing");

    let krypt = world.place(yhlgrym).unwrap().find_site("krypt").unwrap();
    world.apply(Command::Walk { to: krypt }).unwrap();
    let before = world.clone();
    assert_eq!(world.apply(Command::Sleep), Err(CommandError::NotAtAnInn));
    assert_eq!(world, before);

    let inn = world.place(yhlgrym).unwrap().find_site("inn").unwrap();
    assert_eq!(
        world.place(yhlgrym).unwrap().site(inn).unwrap().kind,
        SiteKind::Inn
    );
    world.apply(Command::Walk { to: inn }).unwrap();
    // 30 and 40, both in debt, and a streak that the night should clear.
    set_fatigue(&mut world, "pylgrym", 30, true);
    set_fatigue(&mut world, "donold", 40, true);
    world.nights_without_camp = 4;
    let start = world.date;

    assert_eq!(copper(&world), 10);
    let events = world.apply(Command::Sleep).unwrap();
    assert!(matches!(
        events.first(),
        Some(Event::Slept { fatigue, copper: 5, .. }) if fatigue == &levels(50, 55)
    ));
    assert_eq!(copper(&world), 5);
    assert!(matches!(events.last(), Some(Event::DayPassed { .. })));
    assert_eq!(fatigue(&world, "pylgrym"), (50, false));
    assert_eq!(fatigue(&world, "donold"), (55, false));
    assert_eq!(world.nights_without_camp, 0);
    assert!(!world.camped);
    assert_eq!(world.date, start.next_day());

    // The second night adds 15: 50 becomes 65, 55 becomes 70. It spends the rest.
    world.apply(Command::Sleep).unwrap();
    assert_eq!(fatigue(&world, "pylgrym"), (65, false));
    assert_eq!(fatigue(&world, "donold"), (70, false));
    assert_eq!(copper(&world), 0);

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
    assert_eq!(world.date, start.next_day());
    assert_eq!(copper(&world), 10, "a residence does not cost copper");
}

#[test]
fn pressing_on_is_refused_when_anyone_would_drop_below_zero() {
    let mut world = first_leg();
    on_the_canyon(&mut world);
    // One member short of the 10 the first night costs is enough to refuse.
    set_fatigue(&mut world, "pylgrym", 9, false);
    set_fatigue(&mut world, "donold", 100, false);
    let before = world.clone();
    assert_eq!(
        world.apply(Command::EndDay { camp: false }),
        Err(CommandError::TooTired)
    );
    assert_eq!(world, before, "a refused press changes nothing");

    // Exactly the cost is allowed, and lands on 0.
    set_fatigue(&mut world, "pylgrym", 10, false);
    set_fatigue(&mut world, "donold", 10, false);
    world.apply(Command::EndDay { camp: false }).unwrap();
    assert_eq!(fatigue(&world, "pylgrym"), (0, true));
    assert_eq!(fatigue(&world, "donold"), (0, true));

    // The next night costs 15, and 0 cannot pay it.
    let before = world.clone();
    assert_eq!(
        world.apply(Command::EndDay { camp: false }),
        Err(CommandError::TooTired)
    );
    assert_eq!(world, before, "a refused press changes nothing");
}

#[test]
fn fatigue_is_only_for_a_party_member_with_a_sheet() {
    let mut world = first_leg();
    on_the_canyon(&mut world);
    world
        .people
        .iter_mut()
        .find(|candidate| candidate.key == "donold")
        .unwrap()
        .sheet = None;

    let events = world.apply(Command::EndDay { camp: false }).unwrap();
    assert_eq!(press(&events).2, &levels(90, 90)[..1]);
    assert_eq!(fatigue(&world, "pylgrym"), (90, false));
    assert!(world
        .people
        .iter()
        .find(|p| p.key == "donold")
        .unwrap()
        .sheet
        .is_none());

    // With no sheet, Donold cannot block the press. The Pylgrym can.
    set_fatigue(&mut world, "pylgrym", 5, false);
    let before = world.clone();
    assert_eq!(
        world.apply(Command::EndDay { camp: false }),
        Err(CommandError::TooTired)
    );
    assert_eq!(world, before);
}

fn fatigue_pair(world: &World) -> (u32, u32) {
    (fatigue(world, "pylgrym").0, fatigue(world, "donold").0)
}
