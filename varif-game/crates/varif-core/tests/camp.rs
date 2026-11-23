//! Making camp, sleeping in it, and packing it up.

use varif_core::{
    load_world, Command, CommandError, Event, Item, ItemType, MemberFatigue, PersonId, World,
};

const FIRST_LEG: &str = include_str!("../../../data/first_leg.ron");

fn first_leg() -> World {
    load_world(FIRST_LEG).expect("the first leg data should load")
}

/// Leaves Sthepenhall onto the road to the falls, where a camp may be pitched.
fn leave_town(world: &mut World) {
    let falls = world.find_place("falls").unwrap();
    world
        .apply(Command::Travel {
            to: falls,
            hours: 1,
        })
        .unwrap();
    assert!(world.journey.is_some());
}

fn clear_packs(world: &mut World) {
    for person in &mut world.people {
        if let Some(sheet) = person.sheet.as_mut() {
            sheet.inventory.clear();
        }
    }
}

fn carry(world: &mut World, who: &str, item_type: ItemType, count: u32) {
    let sheet = world
        .people
        .iter_mut()
        .find(|person| person.key == who)
        .and_then(|person| person.sheet.as_mut())
        .unwrap_or_else(|| panic!("{who} should have a sheet"));
    sheet.carry(Item::supply(item_type, count));
}

fn holding(world: &World, who: &str, item_type: ItemType) -> u32 {
    world
        .people
        .iter()
        .find(|person| person.key == who)
        .and_then(|person| person.sheet.as_ref())
        .unwrap_or_else(|| panic!("{who} should have a sheet"))
        .holding(item_type)
}

fn set_fatigue(world: &mut World, who: &str, fatigue: u32, debt: bool) {
    let sheet = world
        .people
        .iter_mut()
        .find(|person| person.key == who)
        .and_then(|person| person.sheet.as_mut())
        .unwrap_or_else(|| panic!("{who} should have a sheet"));
    sheet.fatigue = fatigue;
    sheet.road_rest_debt = debt;
}

/// Fatigue, health, and resilience of each party member who has a sheet.
fn party_vitals(world: &World) -> Vec<(u32, u32, u32)> {
    world
        .party()
        .filter_map(|person| {
            person
                .sheet
                .as_ref()
                .map(|sheet| (sheet.fatigue, sheet.health, sheet.resilience))
        })
        .collect()
}

#[test]
fn supplies_are_spent_when_the_party_sleeps() {
    let mut world = first_leg();
    clear_packs(&mut world);
    // Water is on the second member, so the spend is the party's, not one pack.
    carry(&mut world, "pylgrym", ItemType::Food, 2);
    carry(&mut world, "donold", ItemType::Water, 1);
    carry(&mut world, "pylgrym", ItemType::Wood, 1);
    leave_town(&mut world);
    let date = world.date;

    let events = world.apply(Command::MakeCamp).unwrap();
    assert!(matches!(events.as_slice(), [Event::CampedOnRoad { .. }]));
    assert!(world.camped);
    assert_eq!(world.date, date);
    assert_eq!(world.hour, 2);
    assert_eq!(holding(&world, "pylgrym", ItemType::Food), 2);
    assert_eq!(holding(&world, "donold", ItemType::Water), 1);
    assert_eq!(holding(&world, "pylgrym", ItemType::Wood), 1);

    let events = world.apply(Command::Rest).unwrap();
    assert!(matches!(
        events.first(),
        Some(Event::Rested {
            spent_food: true,
            spent_water: true,
            spent_wood: true,
            ..
        })
    ));
    assert_eq!(world.date, date);
    assert_eq!(world.hour, 10);
    assert_eq!(holding(&world, "pylgrym", ItemType::Food), 1);
    assert_eq!(holding(&world, "donold", ItemType::Water), 0);
    assert_eq!(holding(&world, "pylgrym", ItemType::Wood), 0);
    assert_eq!(party_vitals(&world), vec![(100, 100, 100), (100, 100, 100)]);

    let pitched = world.clone();
    assert_eq!(
        world.apply(Command::MakeCamp),
        Err(CommandError::AlreadyCamped)
    );
    assert_eq!(world, pitched);
}

#[test]
fn each_missing_supply_applies_only_its_penalty() {
    // Food is also sitting on someone outside the party. It is not the party's.
    let mut hungry = first_leg();
    clear_packs(&mut hungry);
    carry(&mut hungry, "jonnoj", ItemType::Food, 1);
    assert!(
        !hungry
            .people
            .iter()
            .find(|person| person.key == "jonnoj")
            .unwrap()
            .in_party
    );
    carry(&mut hungry, "pylgrym", ItemType::Water, 2);
    carry(&mut hungry, "pylgrym", ItemType::Wood, 2);
    camp_missing(&mut hungry, false, true, true);
    assert_eq!(party_vitals(&hungry), vec![(95, 100, 100), (95, 100, 100)]);
    assert_eq!(holding(&hungry, "jonnoj", ItemType::Food), 1);
    assert_eq!(holding(&hungry, "pylgrym", ItemType::Water), 1);
    assert_eq!(holding(&hungry, "pylgrym", ItemType::Wood), 1);

    let mut dry = first_leg();
    clear_packs(&mut dry);
    carry(&mut dry, "pylgrym", ItemType::Food, 2);
    carry(&mut dry, "pylgrym", ItemType::Wood, 2);
    camp_missing(&mut dry, true, false, true);
    assert_eq!(party_vitals(&dry), vec![(100, 95, 100), (100, 95, 100)]);
    assert_eq!(holding(&dry, "pylgrym", ItemType::Food), 1);
    assert_eq!(holding(&dry, "pylgrym", ItemType::Wood), 1);

    let mut cold = first_leg();
    clear_packs(&mut cold);
    carry(&mut cold, "pylgrym", ItemType::Food, 2);
    carry(&mut cold, "pylgrym", ItemType::Water, 2);
    camp_missing(&mut cold, true, true, false);
    assert_eq!(party_vitals(&cold), vec![(100, 100, 95), (100, 100, 95)]);
    assert_eq!(holding(&cold, "pylgrym", ItemType::Food), 1);
    assert_eq!(holding(&cold, "pylgrym", ItemType::Water), 1);
}

fn camp_missing(world: &mut World, spent_food: bool, spent_water: bool, spent_wood: bool) {
    leave_town(world);
    let date = world.date;
    world.apply(Command::MakeCamp).unwrap();
    assert_eq!(world.date, date);
    assert_eq!(world.hour, 2);
    assert_eq!(party_vitals(world), vec![(100, 100, 100), (100, 100, 100)]);
    let events = world.apply(Command::Rest).unwrap();
    assert!(matches!(
        events.first(),
        Some(Event::Rested {
            spent_food: food,
            spent_water: water,
            spent_wood: wood,
            ..
        }) if *food == spent_food && *water == spent_water && *wood == spent_wood
    ));
    assert!(world.camped);
    assert_eq!(world.date, date);
    assert_eq!(world.hour, 10);
}

#[test]
fn sleeping_still_succeeds_with_an_empty_pack() {
    let mut world = first_leg();
    clear_packs(&mut world);
    leave_town(&mut world);
    let date = world.date;
    world.apply(Command::MakeCamp).unwrap();
    assert_eq!(party_vitals(&world), vec![(100, 100, 100), (100, 100, 100)]);
    let events = world.apply(Command::Rest).unwrap();
    assert!(matches!(
        events.first(),
        Some(Event::Rested {
            spent_food: false,
            spent_water: false,
            spent_wood: false,
            ..
        })
    ));
    assert!(world.camped);
    assert_eq!(world.date, date);
    assert_eq!(world.hour, 10);
    assert_eq!(party_vitals(&world), vec![(95, 95, 95), (95, 95, 95)]);
}

#[test]
fn a_second_night_in_the_same_camp_spends_supplies_again() {
    let mut world = first_leg();
    assert_eq!(holding(&world, "pylgrym", ItemType::Food), 5);
    assert_eq!(holding(&world, "pylgrym", ItemType::Water), 5);
    assert_eq!(holding(&world, "pylgrym", ItemType::Wood), 5);
    leave_town(&mut world);
    world.apply(Command::MakeCamp).unwrap();
    assert_eq!(holding(&world, "pylgrym", ItemType::Food), 5);

    world.apply(Command::Rest).unwrap();
    assert!(world.camped);
    assert_eq!(holding(&world, "pylgrym", ItemType::Food), 4);
    assert_eq!(holding(&world, "pylgrym", ItemType::Water), 4);
    assert_eq!(holding(&world, "pylgrym", ItemType::Wood), 4);

    world.apply(Command::Rest).unwrap();
    assert!(world.camped);
    assert_eq!(holding(&world, "pylgrym", ItemType::Food), 3);
    assert_eq!(holding(&world, "pylgrym", ItemType::Water), 3);
    assert_eq!(holding(&world, "pylgrym", ItemType::Wood), 3);
    assert_eq!(holding(&world, "pylgrym", ItemType::Copper), 10);
}

#[test]
fn the_party_cannot_camp_in_a_town() {
    let mut world = first_leg();
    assert_eq!(world.place_name(world.party_at), "Sthepenhall");
    assert!(world.journey.is_none());

    let before = world.clone();
    assert_eq!(
        world.apply(Command::MakeCamp),
        Err(CommandError::CannotCampInTown)
    );
    assert_eq!(world, before, "a refused camp must change nothing");
}

#[test]
fn make_camp_does_not_sleep() {
    let mut world = first_leg();
    set_fatigue(&mut world, "pylgrym", 80, false);
    set_fatigue(&mut world, "donold", 80, false);
    leave_town(&mut world);
    let date = world.date;

    let events = world.apply(Command::MakeCamp).unwrap();
    assert!(events
        .iter()
        .all(|event| !matches!(event, Event::DayPassed { .. } | Event::Rested { .. })));
    assert!(world.camped);
    assert_eq!(world.date, date);
    assert_eq!(world.hour, 2);
    assert_eq!(party_vitals(&world), vec![(80, 100, 100), (80, 100, 100)]);
}

#[test]
fn sleeping_in_camp_restores_fatigue_like_a_road_camp() {
    let mut world = first_leg();
    // 47 with the road-rest debt stops at 50. 80 with no debt gains 5.
    set_fatigue(&mut world, "pylgrym", 47, true);
    set_fatigue(&mut world, "donold", 80, false);
    leave_town(&mut world);
    let date = world.date;
    world.apply(Command::MakeCamp).unwrap();

    let events = world.apply(Command::Rest).unwrap();
    assert!(matches!(
        events.first(),
        Some(Event::Rested { fatigue, .. }) if fatigue == &vec![
            MemberFatigue { person: PersonId(0), fatigue: 50 },
            MemberFatigue { person: PersonId(1), fatigue: 85 },
        ]
    ));
    assert!(events
        .iter()
        .all(|event| !matches!(event, Event::DayPassed { .. })));
    assert!(world.camped);
    assert_eq!(world.date, date);
    assert_eq!(world.hour, 10);
    // The starting food restores 10 health when the camp consumes it, and
    // never past 100. Donold carries none, so only the Pylgrym would gain,
    // and both are already at the health cap.
    assert_eq!(party_vitals(&world), vec![(50, 100, 100), (85, 100, 100)]);

    // Fatigue caps at 100. A second food still cannot push health past 100.
    set_fatigue(&mut world, "pylgrym", 98, false);
    set_fatigue(&mut world, "donold", 100, false);
    world.apply(Command::Rest).unwrap();
    assert_eq!(party_vitals(&world), vec![(100, 100, 100), (100, 100, 100)]);
}

#[test]
fn sleep_is_refused_without_a_camp() {
    let mut world = first_leg();
    let before = world.clone();
    assert_eq!(world.apply(Command::Rest), Err(CommandError::NotCamped));
    assert_eq!(world, before);
}

#[test]
fn pack_up_is_refused_when_not_camped() {
    let mut world = first_leg();
    let before = world.clone();
    assert_eq!(world.apply(Command::PackUp), Err(CommandError::NotCamped));
    assert_eq!(world, before);
}

#[test]
fn a_journey_cannot_continue_until_pack_up() {
    let mut world = first_leg();
    leave_town(&mut world);
    world.apply(Command::MakeCamp).unwrap();
    let pitched = world.clone();
    let falls = world.find_place("falls").unwrap();

    assert_eq!(
        world.apply(Command::Travel {
            to: falls,
            hours: 8,
        }),
        Err(CommandError::StillCamped)
    );
    assert_eq!(world, pitched);

    let date = world.date;
    let vitals = party_vitals(&world);
    let events = world.apply(Command::PackUp).unwrap();
    assert!(matches!(events.as_slice(), [Event::PackedUp { .. }]));
    assert!(!world.camped);
    assert_eq!(world.date, date);
    assert_eq!(party_vitals(&world), vitals);

    world
        .apply(Command::Travel {
            to: falls,
            hours: 8,
        })
        .unwrap();
    assert!(world.journey.is_some());
}
