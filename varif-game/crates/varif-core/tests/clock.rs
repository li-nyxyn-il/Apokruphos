//! The world clock, and what an hour of camp, rest, packing, or travel costs.

use varif_core::{
    game_minutes, load_world, real_ms_until_next_game_minute, AdvanceKind, Command, CommandError,
    Event, ItemType, TimeMode, World,
};

const FIRST_LEG: &str = include_str!("../../../data/first_leg.ron");

fn first_leg() -> World {
    load_world(FIRST_LEG).expect("the first leg data should load")
}

fn fatigue(world: &World, key: &str) -> (u32, bool) {
    let sheet = world
        .people
        .iter()
        .find(|person| person.key == key)
        .and_then(|person| person.sheet.as_ref())
        .unwrap_or_else(|| panic!("{key} should have a sheet"));
    (sheet.fatigue, sheet.road_rest_debt)
}

fn set_fatigue(world: &mut World, key: &str, value: u32) {
    let sheet = world
        .people
        .iter_mut()
        .find(|person| person.key == key)
        .and_then(|person| person.sheet.as_mut())
        .unwrap_or_else(|| panic!("{key} should have a sheet"));
    sheet.fatigue = value;
    sheet.road_rest_debt = false;
}

fn holding(world: &World, key: &str, item_type: ItemType) -> u32 {
    world
        .people
        .iter()
        .find(|person| person.key == key)
        .and_then(|person| person.sheet.as_ref())
        .unwrap_or_else(|| panic!("{key} should have a sheet"))
        .holding(item_type)
}

#[test]
fn the_clock_rolls_the_date_at_midnight() {
    let mut world = first_leg();
    let start = world.date;
    assert_eq!(world.hour, 0);
    world.hour = 22;
    let atzhaim = world.find_place("atzhaim").unwrap();
    // This order is about the date, so the road does not roll an encounter.
    // A wait that would pass one stops early in its own test.
    let here = world.party_at;
    for road in &mut world.roads {
        if road.joins(here, atzhaim) {
            road.encounter_chance = 0;
        }
    }

    let events = world
        .apply(Command::Travel {
            to: atzhaim,
            hours: 4,
        })
        .unwrap();

    assert_eq!(world.hour, 2);
    assert_eq!(world.date, start.next_day());
    assert_eq!(world.journey.unwrap().leagues_walked, 4);
    assert!(events.iter().any(|event| matches!(
        event,
        Event::DayPassed { date } if *date == world.date
    )));
}

#[test]
fn camp_rest_and_pack_up_take_one_eight_and_one_hours() {
    let mut world = first_leg();
    let start = world.date;
    let food = holding(&world, "pylgrym", ItemType::Food);
    let water = holding(&world, "pylgrym", ItemType::Water);
    let wood = holding(&world, "pylgrym", ItemType::Wood);
    set_fatigue(&mut world, "pylgrym", 80);
    set_fatigue(&mut world, "donold", 80);

    let events = world.apply(Command::MakeCamp).unwrap();
    assert!(matches!(events.as_slice(), [Event::Camped { .. }]));
    assert!(world.camped);
    assert_eq!(world.hour, 1);
    assert_eq!(world.date, start);
    assert_eq!(holding(&world, "pylgrym", ItemType::Food), food);
    assert_eq!(holding(&world, "pylgrym", ItemType::Water), water);
    assert_eq!(holding(&world, "pylgrym", ItemType::Wood), wood);
    assert_eq!(fatigue(&world, "pylgrym"), (80, false));
    assert_eq!(fatigue(&world, "donold"), (80, false));

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
    assert!(world.camped);
    assert_eq!(world.hour, 9);
    assert_eq!(world.date, start);
    assert_eq!(holding(&world, "pylgrym", ItemType::Food), food - 1);
    assert_eq!(holding(&world, "pylgrym", ItemType::Water), water - 1);
    assert_eq!(holding(&world, "pylgrym", ItemType::Wood), wood - 1);
    // The road-camp restore is 5, and 80 is above the ceiling with no debt.
    assert_eq!(fatigue(&world, "pylgrym"), (85, false));
    assert_eq!(fatigue(&world, "donold"), (85, false));

    let events = world.apply(Command::PackUp).unwrap();
    assert!(matches!(events.as_slice(), [Event::PackedUp { .. }]));
    assert!(!world.camped);
    assert_eq!(world.hour, 10);
    assert_eq!(world.date, start);
    assert_eq!(holding(&world, "pylgrym", ItemType::Food), food - 1);
    assert_eq!(fatigue(&world, "pylgrym"), (85, false));
}

#[test]
fn one_hour_of_travel_is_one_league() {
    let mut world = first_leg();
    let atzhaim = world.find_place("atzhaim").unwrap();

    world
        .apply(Command::Travel {
            to: atzhaim,
            hours: 1,
        })
        .unwrap();
    assert_eq!(world.journey.unwrap().leagues_walked, 1);
    assert_eq!(world.hour, 1);

    world
        .apply(Command::Travel {
            to: atzhaim,
            hours: 8,
        })
        .unwrap();
    assert_eq!(world.journey.unwrap().leagues_walked, 9);
    assert_eq!(world.hour, 9);
    assert_eq!(fatigue(&world, "pylgrym").0, 100);
    assert_eq!(fatigue(&world, "donold").0, 100);
}

#[test]
fn extra_hours_cost_the_triangular_sum_once() {
    let atzhaim_hours = |hours: u32| {
        let mut world = first_leg();
        let atzhaim = world.find_place("atzhaim").unwrap();
        let events = world.apply(Command::Travel { to: atzhaim, hours }).unwrap();
        let traveled = events.iter().find_map(|event| match event {
            Event::Traveled {
                leagues,
                hours: spent,
                fatigue,
                ..
            } => Some((*leagues, *spent, fatigue.clone())),
            _ => None,
        });
        (world, traveled)
    };

    // Eight hours or fewer: no fatigue from this rule.
    let (world, traveled) = atzhaim_hours(8);
    assert_eq!(traveled.unwrap().0, 8);
    assert_eq!(fatigue(&world, "pylgrym"), (100, false));
    assert_eq!(fatigue(&world, "donold"), (100, false));

    // One extra hour costs 1, once.
    let (world, traveled) = atzhaim_hours(9);
    let (leagues, spent, after) = traveled.unwrap();
    assert_eq!((leagues, spent), (9, 9));
    assert_eq!(after.len(), 2);
    assert!(after.iter().all(|entry| entry.fatigue == 99));
    assert_eq!(fatigue(&world, "pylgrym"), (99, false));
    assert_eq!(fatigue(&world, "donold"), (99, false));

    // Three extra hours cost 1 + 2 + 3 = 6, once, not per league.
    let (world, traveled) = atzhaim_hours(11);
    let (leagues, _, after) = traveled.unwrap();
    assert_eq!(leagues, 11);
    assert!(after.iter().all(|entry| entry.fatigue == 94));
    assert_eq!(fatigue(&world, "pylgrym"), (94, false));
    assert_eq!(fatigue(&world, "donold"), (94, false));
    assert_eq!(world.journey.unwrap().leagues_walked, 11);
}

#[test]
fn a_travel_order_that_would_pass_zero_fatigue_is_refused() {
    let mut world = first_leg();
    let atzhaim = world.find_place("atzhaim").unwrap();
    set_fatigue(&mut world, "pylgrym", 0);
    set_fatigue(&mut world, "donold", 100);

    // One extra hour costs 1, and 0 cannot pay it.
    let before = world.clone();
    assert_eq!(
        world.apply(Command::Travel {
            to: atzhaim,
            hours: 9,
        }),
        Err(CommandError::TooTired)
    );
    assert_eq!(world, before, "a refused order changes nothing");

    // Three extra hours cost 6. Five cannot pay it.
    set_fatigue(&mut world, "pylgrym", 5);
    set_fatigue(&mut world, "donold", 5);
    let before = world.clone();
    assert_eq!(
        world.apply(Command::Travel {
            to: atzhaim,
            hours: 11,
        }),
        Err(CommandError::TooTired)
    );
    assert_eq!(world, before);

    // Eight hours cost nothing, so fatigue of 0 is still allowed.
    set_fatigue(&mut world, "pylgrym", 0);
    set_fatigue(&mut world, "donold", 0);
    world
        .apply(Command::Travel {
            to: atzhaim,
            hours: 8,
        })
        .unwrap();
    assert_eq!(fatigue(&world, "pylgrym"), (0, false));
    assert_eq!(fatigue(&world, "donold"), (0, false));
    assert_eq!(world.journey.unwrap().leagues_walked, 8);
}

#[test]
fn camp_rest_pack_travel_and_wait_move_the_same_clock() {
    let mut world = first_leg();
    let start = world.absolute_hour();
    world
        .apply(Command::SetTimeMode {
            mode: TimeMode::RealTime,
        })
        .unwrap();
    let moved = world.advance(30, AdvanceKind::Elapsed);
    assert_eq!(moved.minutes, 30);
    assert_eq!((world.hour, world.minute), (0, 30));

    let food = holding(&world, "pylgrym", ItemType::Food);
    world.apply(Command::MakeCamp).unwrap();
    assert_eq!((world.hour, world.minute), (1, 30));
    assert!(world.camped);

    world.apply(Command::Rest).unwrap();
    assert_eq!((world.hour, world.minute), (9, 30));
    assert_eq!(holding(&world, "pylgrym", ItemType::Food), food - 1);

    world.apply(Command::PackUp).unwrap();
    assert!(!world.camped);
    assert_eq!((world.hour, world.minute), (10, 30));

    let atzhaim = world.find_place("atzhaim").unwrap();
    world
        .apply(Command::Travel {
            to: atzhaim,
            hours: 1,
        })
        .unwrap();
    assert_eq!(world.journey.unwrap().leagues_walked, 1);
    assert_eq!((world.hour, world.minute), (11, 30));

    world.apply(Command::Wait { hours: 1 }).unwrap();
    assert_eq!(world.journey.unwrap().leagues_walked, 1);
    assert_eq!((world.hour, world.minute), (12, 30));
    assert_eq!(world.absolute_hour(), start + 12);
}

#[test]
fn wait_rejects_zero_and_twenty_five() {
    let mut world = first_leg();
    let before = world.clone();
    assert_eq!(
        world.apply(Command::Wait { hours: 0 }),
        Err(CommandError::BadWait)
    );
    assert_eq!(world, before, "a refused wait changes nothing");
    assert_eq!(
        world.apply(Command::Wait { hours: 25 }),
        Err(CommandError::BadWait)
    );
    assert_eq!(world, before, "a refused wait changes nothing");
}

#[test]
fn the_time_scale_defaults_to_one_and_saves() {
    let mut world = first_leg();
    assert_eq!(world.time_scale, 1);

    let events = world.apply(Command::SetTimeScale { scale: 10 }).unwrap();
    assert_eq!(events, vec![Event::TimeScaleChanged { scale: 10 }]);
    assert_eq!(world.time_scale, 10);

    let saved = world.to_ron().unwrap();
    assert!(saved.contains("time_scale: 10"));
    let loaded = World::from_ron(&saved).unwrap();
    assert_eq!(loaded.time_scale, 10);

    let old = saved.replacen("time_scale: 10,", "", 1);
    let from_before = World::from_ron(&old).unwrap();
    assert_eq!(from_before.time_scale, 1);

    let again = world.apply(Command::SetTimeScale { scale: 10 }).unwrap();
    assert!(again.is_empty());
}

#[test]
fn a_time_scale_outside_one_to_sixty_changes_nothing() {
    let mut world = first_leg();
    let before = world.clone();
    assert_eq!(
        world.apply(Command::SetTimeScale { scale: 0 }),
        Err(CommandError::BadTimeScale)
    );
    assert_eq!(world, before);
    assert_eq!(
        world.apply(Command::SetTimeScale { scale: 61 }),
        Err(CommandError::BadTimeScale)
    );
    assert_eq!(world, before);
}

#[test]
fn camp_and_step_ignore_the_time_scale() {
    let mut world = first_leg();
    world.apply(Command::SetTimeScale { scale: 60 }).unwrap();
    let atzhaim = world.find_place("atzhaim").unwrap();
    world
        .apply(Command::Travel {
            to: atzhaim,
            hours: 1,
        })
        .unwrap();
    assert!(world.journey.is_some());
    world.apply(Command::SetTimeScale { scale: 4 }).unwrap();
    assert_eq!(world.time_scale, 4);

    let mut camped = first_leg();
    camped.apply(Command::SetTimeScale { scale: 60 }).unwrap();
    camped.apply(Command::MakeCamp).unwrap();
    assert!(camped.camped);
    assert_eq!((camped.hour, camped.minute), (1, 0));
    camped.apply(Command::Step).unwrap();
    assert_eq!((camped.hour, camped.minute), (2, 0));
}

#[test]
fn game_minutes_follow_the_scale_and_keep_the_remainder() {
    assert_eq!(game_minutes(60_000, 1), (1, 0));
    assert_eq!(game_minutes(1_000, 60), (1, 0));
    assert_eq!(game_minutes(999, 60), (0, 999));
    assert_eq!(game_minutes(30_000, 1), (0, 30_000));
    assert_eq!(game_minutes(45_000, 2), (1, 15_000));
    assert_eq!(game_minutes(1_000, 0), (0, 1_000));
    assert_eq!(real_ms_until_next_game_minute(0, 1), 60_000);
    assert_eq!(real_ms_until_next_game_minute(30_000, 1), 30_000);
    assert_eq!(real_ms_until_next_game_minute(0, 60), 1_000);
}

#[test]
fn ninety_minutes_of_real_time_rolls_the_hour() {
    let mut world = first_leg();
    let start = world.absolute_hour();
    world
        .apply(Command::SetTimeMode {
            mode: TimeMode::RealTime,
        })
        .unwrap();

    let moved = world.advance(90, AdvanceKind::Elapsed);
    assert_eq!(moved.minutes, 90);
    assert_eq!(world.hour, 1);
    assert_eq!(world.minute, 30);
    assert_eq!(world.absolute_hour(), start + 1);

    let loaded = World::from_ron(&world.to_ron().unwrap()).unwrap();
    assert_eq!(loaded, world);
}

#[test]
fn turn_based_mode_ignores_a_passive_elapsed_call() {
    let mut world = first_leg();
    assert_eq!(world.time_mode, TimeMode::TurnBased);
    let before = world.clone();

    let moved = world.advance(90, AdvanceKind::Elapsed);
    assert_eq!(moved.minutes, 0);
    assert!(moved.events.is_empty());
    assert_eq!(world, before);

    world.apply(Command::Step).unwrap();
    assert_eq!((world.hour, world.minute), (1, 0));
    assert_eq!(world.date, before.date);
}

#[test]
fn a_wait_stops_early_at_a_road_encounter() {
    let mut world = first_leg();
    let atzhaim = world.find_place("atzhaim").unwrap();
    let here = world.party_at;
    for road in &mut world.roads {
        if road.joins(here, atzhaim) {
            road.encounter_chance = 0;
        }
    }
    world
        .apply(Command::Travel {
            to: atzhaim,
            hours: 1,
        })
        .unwrap();
    world.journey.as_mut().unwrap().encounter_chance = 100;
    world.hour = 20;
    world.minute = 0;
    let date = world.date;
    let walked = world.journey.unwrap().leagues_walked;

    let events = world.apply(Command::Wait { hours: 6 }).unwrap();

    assert_eq!(world.date, date.next_day());
    assert_eq!((world.hour, world.minute), (0, 0));
    assert!(world.clock_paused);
    assert_eq!(world.journey.unwrap().leagues_walked, walked);
    assert!(events
        .iter()
        .any(|event| matches!(event, Event::RoadEncounter { .. })));
    assert!(events
        .iter()
        .any(|event| matches!(event, Event::DayPassed { .. })));
    assert!(events
        .iter()
        .any(|event| matches!(event, Event::Waited { hours: 4, .. })));
}

#[test]
fn the_date_rolls_at_twenty_four_hours() {
    let mut world = first_leg();
    let start = world.date;
    let events = world.apply(Command::Wait { hours: 24 }).unwrap();

    assert_eq!(world.date, start.next_day());
    assert_eq!((world.hour, world.minute), (0, 0));
    assert!(events.iter().any(|event| matches!(
        event,
        Event::DayPassed { date } if *date == world.date
    )));
    assert!(events
        .iter()
        .any(|event| matches!(event, Event::Waited { hours: 24, .. })));
}
