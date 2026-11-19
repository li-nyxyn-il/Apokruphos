//! NPC schedules: a repeating cycle, looked up from the clock.

use varif_core::{
    load_world, Command, Event, Person, PersonId, PlaceId, RoadLeg, Schedule, SiteId, Span,
    Whereabouts, World,
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

fn site(world: &World, place: &str, key: &str) -> SiteId {
    let place = world.find_place(place).unwrap();
    world.place(place).unwrap().find_site(key).unwrap()
}

#[test]
fn the_farmer_moves_through_one_day_of_sites() {
    let mut world = first_leg();
    let bren = person(&world, "bren");
    let yhlgrym = world.find_place("yhlgrym").unwrap();
    let farms = site(&world, "yhlgrym", "farms");
    let inn = site(&world, "yhlgrym", "inn");
    let market = site(&world, "yhlgrym", "market");
    let start = world.date;
    let schedule = world.person(bren).unwrap().schedule.clone();
    assert_eq!(schedule.cycle_hours(), 24);

    // The cycle is one day, so the hour of the day is the place in the round.
    let dawn = world.whereabouts(bren).unwrap();
    assert_eq!(
        dawn,
        Whereabouts {
            place: yhlgrym,
            site: Some(farms),
            on_the_road: None,
        }
    );

    world.hour = 12;
    let midday = world.whereabouts(bren).unwrap();
    assert_eq!(midday.place, yhlgrym);
    assert_eq!(midday.site, Some(inn));

    world.hour = 18;
    let evening = world.whereabouts(bren).unwrap();
    assert_eq!(evening.place, yhlgrym);
    assert_eq!(evening.site, Some(market));

    assert_eq!(world.date, start, "the three lookups are the same day");
    assert_ne!(dawn.site, midday.site);
    assert_ne!(midday.site, evening.site);
    // The stored post does not move. The schedule is the lookup.
    assert_eq!(world.person(bren).unwrap().site, Some(farms));
    assert_eq!(schedule.at(world.absolute_hour()), Some(evening));
}

#[test]
fn the_courier_crosses_the_neck_and_returns_within_the_month() {
    let world = first_leg();
    let iorvyn = person(&world, "iorvyn");
    let yhlgrym = world.find_place("yhlgrym").unwrap();
    let atzhaim = world.find_place("atzhaim").unwrap();
    let inn = site(&world, "yhlgrym", "inn");
    let market = site(&world, "atzhaim", "market");
    let schedule = world.person(iorvyn).unwrap().schedule.clone();
    // Fifty days of twenty-four hours.
    assert_eq!(schedule.cycle_hours(), 50 * 24);

    let base = world.absolute_hour();
    let at = |offset: u64| schedule.at(base + offset).unwrap();

    let home = at(0);
    assert_eq!(home.place, yhlgrym);
    assert_eq!(home.site, Some(inn));
    assert!(home.on_the_road.is_none());

    let outward = at(540);
    assert_eq!(
        outward.on_the_road,
        Some(RoadLeg {
            from: yhlgrym,
            to: atzhaim,
        })
    );

    let there = at(600);
    assert_eq!(there.place, atzhaim);
    assert_eq!(there.site, Some(market));
    assert!(there.on_the_road.is_none());

    let back = at(1140);
    assert_eq!(
        back.on_the_road,
        Some(RoadLeg {
            from: atzhaim,
            to: yhlgrym,
        })
    );

    let returned = at(1200);
    assert_eq!(returned, home);

    assert_ne!(home.place, there.place);
    assert_eq!(world.whereabouts(iorvyn), Some(home));
}

#[test]
fn replacing_a_schedule_is_what_the_next_lookup_reads() {
    let mut world = first_leg();
    let bren = person(&world, "bren");
    let atzhaim = world.find_place("atzhaim").unwrap();
    let market = site(&world, "atzhaim", "market");
    let schedule = Schedule {
        spans: vec![Span {
            hours: 24,
            whereabouts: Whereabouts {
                place: atzhaim,
                site: Some(market),
                on_the_road: None,
            },
        }],
    };

    let events = world
        .apply(Command::SetSchedule {
            person: bren,
            schedule: schedule.clone(),
        })
        .unwrap();
    assert_eq!(events, vec![Event::ScheduleChanged { person: bren }]);
    assert_eq!(world.person(bren).unwrap().schedule, schedule);

    assert_eq!(world.whereabouts(bren).unwrap().place, atzhaim);
    world.hour = 15;
    assert_eq!(world.whereabouts(bren).unwrap().site, Some(market));
    assert_eq!(
        world.whereabouts(bren),
        schedule.at(world.absolute_hour())
    );
}

#[test]
fn a_lookup_does_not_simulate_the_other_npcs() {
    let world = first_leg();
    let bren = person(&world, "bren");
    let schedule = world.person(bren).unwrap().schedule.clone();
    let hour = world.absolute_hour();
    let expected = schedule.at(hour);

    // A crowd of other schedules. The farmer's answer is still the function
    // of the farmer's own cycle, and nobody's stored place is rewritten.
    let mut crowded = world.clone();
    let yhlgrym = crowded.find_place("yhlgrym").unwrap();
    let filler = Schedule {
        spans: vec![Span {
            hours: 3,
            whereabouts: Whereabouts {
                place: yhlgrym,
                site: None,
                on_the_road: None,
            },
        }],
    };
    for index in 0..300 {
        let mut extra: Person = crowded.person(person(&crowded, "iorvyn")).unwrap().clone();
        extra.key = format!("crowd-{index}");
        extra.name = extra.key.clone();
        extra.at = PlaceId(0);
        extra.schedule = filler.clone();
        crowded.people.push(extra);
    }
    let before = crowded.people.clone();

    assert_eq!(crowded.whereabouts(bren), expected);
    assert_eq!(crowded.people, before);
    assert_eq!(schedule.at(hour), expected);
}
