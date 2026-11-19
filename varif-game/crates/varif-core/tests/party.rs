//! Tests for followers: who travels with the party, and recruiting more.

use varif_core::{load_world, Command, CommandError, Event, FactionId, PersonId, World};

const FIRST_LEG: &str = include_str!("../../../data/first_leg.ron");

fn first_leg() -> World {
    load_world(FIRST_LEG).expect("the first leg data should load")
}

fn person(world: &World, key: &str) -> PersonId {
    let index = world
        .people
        .iter()
        .position(|p| p.key == key)
        .expect("the person should exist");
    PersonId(index as u32)
}

/// Walks to each stop in turn, eight hours at a time.
fn walk(world: &mut World, stops: &[&str]) -> Vec<Event> {
    let mut events = Vec::new();
    for stop in stops {
        let to = world.find_place(stop).expect("the stop should exist");
        loop {
            let left = if let Some(journey) = world.journey {
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
    }
    events
}

fn offers(events: &[Event]) -> Vec<PersonId> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::RecruitOffered { person, .. } => Some(*person),
            _ => None,
        })
        .collect()
}

#[test]
fn donold_audsen_travels_with_the_party_from_the_start() {
    let mut world = first_leg();
    let donold = person(&world, "donold");
    let sheet = world.person(donold).unwrap().sheet.as_ref().unwrap();
    assert_eq!(world.person(donold).unwrap().name, "Donold Audsen");
    assert!(world.person(donold).unwrap().in_party);
    assert!(sheet.role.is_follower());
    assert_eq!(sheet.primary, FactionId::Audsen);
    assert!(sheet.belongs_to(FactionId::Player));

    walk(&mut world, &["falls"]);
    assert_eq!(world.person(donold).unwrap().at, world.party_at);
}

#[test]
fn kraiark_jonnoj_waits_in_yhlgrym() {
    let world = first_leg();
    let jonnoj = person(&world, "jonnoj");
    let him = world.person(jonnoj).unwrap();
    assert_eq!(him.name, "Kraiark Jonnoj");
    assert!(!him.in_party);
    assert!(him.recruitable);
    assert_eq!(him.at, world.find_place("yhlgrym").unwrap());
    let krypt = world.place(him.at).unwrap().find_site("krypt").unwrap();
    assert_eq!(him.site, Some(krypt));
    assert!(!him.sheet.as_ref().unwrap().role.is_follower());
}

#[test]
fn entering_yhlgrym_asks_about_jonnoj() {
    let mut world = first_leg();
    let jonnoj = person(&world, "jonnoj");

    // Nobody at the falls can join.
    let events = walk(&mut world, &["falls"]);
    assert!(offers(&events).is_empty());

    // Coming back into Yhlgrym, the offer comes after the gate.
    let events = walk(&mut world, &["yhlgrym"]);
    assert_eq!(offers(&events), [jonnoj]);
    assert!(matches!(events.last(), Some(Event::RecruitOffered { .. })));
}

#[test]
fn recruiting_jonnoj_makes_him_a_follower() {
    let mut world = first_leg();
    let jonnoj = person(&world, "jonnoj");
    walk(&mut world, &["falls", "yhlgrym"]);

    let events = world.apply(Command::Recruit { person: jonnoj }).unwrap();
    assert_eq!(
        events,
        [Event::Recruited {
            person: jonnoj,
            at: world.party_at,
            on: world.date
        }]
    );
    let him = world.person(jonnoj).unwrap();
    assert!(him.in_party);
    assert!(!him.recruitable);
    let sheet = him.sheet.as_ref().unwrap();
    assert!(sheet.role.is_follower());
    assert_eq!(sheet.primary, FactionId::Ecumenci);
    assert!(sheet.belongs_to(FactionId::Player));
    assert_eq!(world.party().count(), 3);

    // He walks with the party, and is not offered again.
    let events = walk(&mut world, &["falls", "yhlgrym"]);
    assert!(offers(&events).is_empty());
    assert_eq!(world.person(jonnoj).unwrap().at, world.party_at);

    // A save keeps him in the party.
    let loaded = World::from_ron(&world.to_ron().unwrap()).unwrap();
    assert_eq!(loaded, world);
}

#[test]
fn turning_jonnoj_down_leaves_him_where_he_is() {
    // Answering no needs no command: he stays, and is asked again next time.
    let mut world = first_leg();
    let jonnoj = person(&world, "jonnoj");
    walk(&mut world, &["falls", "yhlgrym"]);
    let events = walk(&mut world, &["falls", "yhlgrym"]);
    assert_eq!(offers(&events), [jonnoj]);
    assert!(!world.person(jonnoj).unwrap().in_party);
}

#[test]
fn a_bad_recruit_is_refused() {
    let mut world = first_leg();
    let jonnoj = person(&world, "jonnoj");
    let donold = person(&world, "donold");

    walk(&mut world, &["falls"]);
    let before = world.clone();
    assert_eq!(
        world.apply(Command::Recruit { person: jonnoj }),
        Err(CommandError::NotHere)
    );
    assert_eq!(
        world.apply(Command::Recruit { person: donold }),
        Err(CommandError::AlreadyInParty)
    );
    assert_eq!(
        world.apply(Command::Recruit {
            person: PersonId(99)
        }),
        Err(CommandError::UnknownPerson(PersonId(99)))
    );
    assert_eq!(world, before, "a refused command must change nothing");
}

#[test]
fn someone_who_will_not_join_is_refused() {
    let text = r#"(
        start: (year: 192, month: 1, day: 1, seed: 1, party_at: "inn"),
        places: [(key: "inn", name: "Inn", kind: Town)],
        people: [(key: "keeper", name: "Keeper", title: "Innkeeper", at: "inn")],
    )"#;
    let mut world = load_world(text).unwrap();
    assert!(world.recruit_offers().is_empty());
    assert_eq!(
        world.apply(Command::Recruit {
            person: PersonId(0)
        }),
        Err(CommandError::NotRecruitable)
    );
}
