//! The world clock, and what an hour of camp, rest, packing, or travel costs.

use varif_core::{load_world, Command, CommandError, Event, ItemType, World};

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
        let events = world
            .apply(Command::Travel {
                to: atzhaim,
                hours,
            })
            .unwrap();
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
