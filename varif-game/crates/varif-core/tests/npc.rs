//! NPC behaviour: who is nearby, how friendly they are, and what can be said.

use varif_core::{
    friendliness, load_world, Band, Bucket, Command, CommandError, DataError, Event, FactionId,
    PersonId, Standing, World,
};

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

fn walk(world: &mut World, stop: &str) {
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

/// The sample residents of Yhlgrym live there; the party starts in Sthepenhall.
fn at_yhlgrym(world: &mut World) {
    walk(world, "falls");
    walk(world, "yhlgrym");
}

/// Walks inside the current place to the site `key` stands at.
fn stand_with(world: &mut World, key: &str) {
    let site = world
        .person(person(world, key))
        .unwrap()
        .site
        .expect("they should be at a site");
    if world.party_site != Some(site) {
        world.apply(Command::Walk { to: site }).unwrap();
    }
}

fn texts(world: &World, id: PersonId) -> Vec<String> {
    world
        .dialogue_options(id)
        .iter()
        .map(|l| l.text.clone())
        .collect()
}

#[test]
fn friendliness_is_the_floor_of_the_root_of_attitude_times_hostility() {
    assert_eq!(friendliness(75, 50), 61);
    assert_eq!(friendliness(100, 0), 0);
    assert_eq!(friendliness(0, 100), 0);
    assert_eq!(friendliness(100, 100), 100);
    assert_eq!(friendliness(50, 2), 10);
    assert_eq!(friendliness(50, 1), 7);
}

#[test]
fn scores_fall_into_low_average_and_high() {
    for (score, band) in [
        (0, Band::Low),
        (30, Band::Low),
        (31, Band::Average),
        (70, Band::Average),
        (71, Band::High),
        (100, Band::High),
    ] {
        assert_eq!(Band::of(score), band, "score {score}");
    }
}

#[test]
fn each_score_opens_one_bucket_and_a_follower_one_more() {
    // Hostility 20 and attitude 90 give friendliness 42.
    let standing = Standing::new(90, 20, false);
    assert_eq!(standing.friendliness, 42);
    assert_eq!(
        standing.buckets(),
        [
            Bucket::Friendliness(Band::Average),
            Bucket::Hostility(Band::Low),
            Bucket::Attitude(Band::High),
        ]
    );

    let follower = Standing::new(90, 20, true);
    assert_eq!(follower.buckets().last(), Some(&Bucket::Follower));
    assert_eq!(follower.buckets().len(), 4);
}

#[test]
fn the_npcs_where_the_player_is_are_nearby() {
    let mut world = first_leg();
    let donold = person(&world, "donold");
    let hest = person(&world, "hest");
    let jonnoj = person(&world, "jonnoj");
    let syla = person(&world, "syla");
    let mael = person(&world, "mael");

    // The party starts at Sthepenhall's gate. Only the follower is with them.
    assert_eq!(world.nearby_npcs(), [donold]);

    // In Yhlgrym the warden is at the North Gate. The Kraiark is at the krypt,
    // the merchant at the market, the steward at the palace.
    at_yhlgrym(&mut world);
    assert_eq!(world.nearby_npcs(), [donold, hest]);

    stand_with(&mut world, "jonnoj");
    assert_eq!(world.nearby_npcs(), [donold, jonnoj]);

    stand_with(&mut world, "syla");
    assert_eq!(world.nearby_npcs(), [donold, syla]);

    stand_with(&mut world, "mael");
    assert_eq!(world.nearby_npcs(), [donold, mael]);

    // A follower walks with the player. A place with no sites has everyone there.
    walk(&mut world, "falls");
    assert_eq!(world.nearby_npcs(), [donold]);
}

#[test]
fn someone_at_another_site_cannot_be_talked_to() {
    let mut world = first_leg();
    at_yhlgrym(&mut world);
    let syla = person(&world, "syla");
    let before = world.clone();
    assert_eq!(
        world.apply(Command::Interact { person: syla }),
        Err(CommandError::NotHere)
    );
    assert_eq!(world, before);

    stand_with(&mut world, "syla");
    world.apply(Command::Interact { person: syla }).unwrap();
    assert_eq!(world.conversation, Some(syla));
}

#[test]
fn standing_comes_from_attitude_and_faction_hostility() {
    let world = first_leg();
    // Jonnoj: attitude 50, Ecumenci hostility toward the player 50.
    let jonnoj = world.standing(person(&world, "jonnoj")).unwrap();
    assert_eq!((jonnoj.attitude, jonnoj.hostility), (50, 50));
    assert_eq!(jonnoj.friendliness, 50);
    assert!(!jonnoj.follower);

    // Donold: attitude 50, Audsen hostility toward the player 70.
    let donold = world.standing(person(&world, "donold")).unwrap();
    assert_eq!(donold.friendliness, 59);
    assert!(donold.follower);

    // The player has no standing toward themself.
    let player = PersonId(world.people.iter().position(|p| p.is_player()).unwrap() as u32);
    assert_eq!(world.standing(player), None);
}

#[test]
fn interacting_opens_a_conversation_with_the_lines_on_offer() {
    let mut world = first_leg();
    at_yhlgrym(&mut world);
    let jonnoj = person(&world, "jonnoj");
    stand_with(&mut world, "jonnoj");

    let events = world.apply(Command::Interact { person: jonnoj }).unwrap();
    assert!(matches!(
        events[..],
        [Event::ConversationOpened { person, standing, .. }]
            if person == jonnoj && standing.friendliness == 50
    ));
    assert_eq!(world.conversation, Some(jonnoj));

    // Average friendliness, hostility and attitude, then his own line.
    assert_eq!(
        texts(&world, jonnoj),
        [
            "Good day to you.",
            "What news from your people?",
            "Do you know the road ahead?",
            "What does a Kraiark do in Yhlgrym?",
        ]
    );

    let events = world.apply(Command::Say { line: 3 }).unwrap();
    assert_eq!(
        events,
        [Event::Said {
            person: jonnoj,
            text: "What does a Kraiark do in Yhlgrym?".to_string(),
            reply: "Keeps the Krypt, and keeps an eye on pylgryms like you.".to_string(),
        }]
    );

    // While talking, nothing else can be done.
    let before = world.clone();
    let falls = world.find_place("falls").unwrap();
    assert_eq!(
        world.apply(Command::Travel {
            to: falls,
            hours: 8,
        }),
        Err(CommandError::InConversation)
    );
    assert_eq!(
        world.apply(Command::Interact { person: jonnoj }),
        Err(CommandError::InConversation)
    );
    assert_eq!(
        world.apply(Command::Say { line: 99 }),
        Err(CommandError::UnknownLine(99))
    );
    assert_eq!(world, before, "a refused command must change nothing");

    assert_eq!(
        world.apply(Command::EndConversation).unwrap(),
        [Event::ConversationEnded { person: jonnoj }]
    );
    assert_eq!(world.conversation, None);
    world
        .apply(Command::Travel {
            to: falls,
            hours: 8,
        })
        .unwrap();
}

#[test]
fn only_a_follower_is_offered_follower_lines() {
    let world = first_leg();
    let follower_line = "How are you holding up?".to_string();
    assert!(texts(&world, person(&world, "donold")).contains(&follower_line));
    assert!(!texts(&world, person(&world, "jonnoj")).contains(&follower_line));
}

#[test]
fn a_change_of_attitude_changes_what_can_be_said() {
    let mut world = first_leg();
    let jonnoj = person(&world, "jonnoj");
    world
        .apply(Command::SetAttitude {
            person: jonnoj,
            index: 90,
        })
        .unwrap();
    // Attitude 90 and hostility 50: friendliness 67, still average.
    let lines = texts(&world, jonnoj);
    assert!(lines.contains(&"Will you tell me about yourself?".to_string()));
    assert!(!lines.contains(&"Do you know the road ahead?".to_string()));
    assert!(lines.contains(&"Good day to you.".to_string()));
}

#[test]
fn an_npc_too_hostile_to_talk_to_would_start_combat() {
    let mut world = first_leg();
    at_yhlgrym(&mut world);
    let jonnoj = person(&world, "jonnoj");
    stand_with(&mut world, "jonnoj");
    // Hostility 1 and attitude 50: friendliness 7, below 10.
    world
        .apply(Command::SetHostility {
            from: FactionId::Ecumenci,
            toward: FactionId::Player,
            index: 1,
        })
        .unwrap();
    let before = world.clone();

    let events = world.apply(Command::Interact { person: jonnoj }).unwrap();
    assert_eq!(
        events,
        [Event::TooHostileToTalk {
            person: jonnoj,
            friendliness: 7
        }]
    );
    // Combat is not built yet, so nothing else happens.
    assert_eq!(world, before);
    assert_eq!(world.conversation, None);
}

#[test]
fn friendliness_of_exactly_ten_is_enough_to_talk() {
    let mut world = first_leg();
    at_yhlgrym(&mut world);
    let jonnoj = person(&world, "jonnoj");
    stand_with(&mut world, "jonnoj");
    world
        .apply(Command::SetHostility {
            from: FactionId::Ecumenci,
            toward: FactionId::Player,
            index: 2,
        })
        .unwrap();
    world.apply(Command::Interact { person: jonnoj }).unwrap();
    assert_eq!(world.conversation, Some(jonnoj));
    // Low friendliness and a low hostility index, average attitude.
    assert_eq!(
        texts(&world, jonnoj),
        [
            "I mean you no harm.",
            "Your people and mine need not quarrel.",
            "Do you know the road ahead?",
            "What does a Kraiark do in Yhlgrym?",
        ]
    );
}

#[test]
fn a_bad_interaction_is_refused() {
    let mut world = first_leg();
    let jonnoj = person(&world, "jonnoj");
    let player = PersonId(world.people.iter().position(|p| p.is_player()).unwrap() as u32);
    walk(&mut world, "falls");
    let before = world.clone();

    assert_eq!(
        world.apply(Command::Interact { person: jonnoj }),
        Err(CommandError::NotHere)
    );
    assert_eq!(
        world.apply(Command::Interact { person: player }),
        Err(CommandError::NotAnNpc)
    );
    assert_eq!(
        world.apply(Command::Interact {
            person: PersonId(99)
        }),
        Err(CommandError::UnknownPerson(PersonId(99)))
    );
    assert_eq!(
        world.apply(Command::Say { line: 0 }),
        Err(CommandError::NotInConversation)
    );
    assert_eq!(
        world.apply(Command::EndConversation),
        Err(CommandError::NotInConversation)
    );
    assert_eq!(world, before);
}

#[test]
fn a_world_saved_mid_conversation_loads_back_in_it() {
    let mut world = first_leg();
    at_yhlgrym(&mut world);
    let jonnoj = person(&world, "jonnoj");
    stand_with(&mut world, "jonnoj");
    world.apply(Command::Interact { person: jonnoj }).unwrap();
    let loaded = World::from_ron(&world.to_ron().unwrap()).unwrap();
    assert_eq!(loaded, world);
    assert_eq!(loaded.conversation, Some(jonnoj));
}

#[test]
fn a_person_in_a_city_needs_a_real_site() {
    let missing = r#"(
        start: (year: 192, month: 1, day: 1, seed: 1, party_at: "city"),
        places: [(key: "city", name: "City", kind: Town, sites: [
            (key: "gate", name: "Gate", kind: Gate),
        ])],
        people: [(key: "a", name: "A", title: "T", at: "city")],
    )"#;
    assert_eq!(
        load_world(missing).unwrap_err(),
        DataError::MissingSite {
            person: "A".to_string(),
            place: "City".to_string(),
        }
    );

    let unknown = r#"(
        start: (year: 192, month: 1, day: 1, seed: 1, party_at: "city"),
        places: [(key: "city", name: "City", kind: Town, sites: [
            (key: "gate", name: "Gate", kind: Gate),
        ])],
        people: [(key: "a", name: "A", title: "T", at: "city", site: Some("palace"))],
    )"#;
    assert_eq!(
        load_world(unknown).unwrap_err(),
        DataError::UnknownSite {
            person: "A".to_string(),
            place: "City".to_string(),
            site: "palace".to_string(),
        }
    );
}

#[test]
fn a_blank_dialogue_line_is_reported() {
    let text = r#"(
        start: (year: 192, month: 1, day: 1, seed: 1, party_at: "inn"),
        places: [(key: "inn", name: "Inn", kind: Town)],
        dialogue: [(bucket: Follower, text: "  ")],
    )"#;
    assert_eq!(
        load_world(text).unwrap_err(),
        DataError::BlankDialogue {
            owner: "the general dialogue".to_string()
        }
    );
}
