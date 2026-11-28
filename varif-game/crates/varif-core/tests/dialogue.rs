//! Dialogue conditions: what they read from the world.

use varif_core::{
    load_world, Band, Bucket, Command, Compare, Condition, PersonId, World,
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

/// The party at Yhlgrym's gate, where Hest stands.
fn at_yhlgrym(world: &mut World) {
    walk(world, "falls");
    walk(world, "yhlgrym");
}

fn set_attitude(world: &mut World, who: PersonId, index: u32) {
    world
        .apply(Command::SetAttitude { person: who, index })
        .unwrap();
}

#[test]
fn compare_between_is_inclusive() {
    let between = Compare::Between(30, 70);
    assert!(between.holds(30));
    assert!(between.holds(70));
    assert!(!between.holds(29));
    assert!(!between.holds(71));
    assert!(Compare::AtLeast(5).holds(5));
    assert!(Compare::AtMost(5).holds(5));
    assert!(!Compare::AtMost(5).holds(6));
}

#[test]
fn attitude_condition_reads_the_npc() {
    let mut world = first_leg();
    let hest = person(&world, "hest");
    set_attitude(&mut world, hest, 60);
    assert!(Condition::Attitude(Compare::AtLeast(60)).holds(&world, hest));
    assert!(!Condition::Attitude(Compare::AtLeast(61)).holds(&world, hest));
}

#[test]
fn hostility_condition_reads_primary_faction() {
    let world = first_leg();
    let hest = person(&world, "hest");
    assert!(Condition::Hostility(Compare::AtMost(50)).holds(&world, hest));
    assert!(!Condition::Hostility(Compare::AtMost(49)).holds(&world, hest));
}

#[test]
fn friendliness_and_bucket() {
    let mut world = first_leg();
    let hest = person(&world, "hest");
    set_attitude(&mut world, hest, 50);
    assert!(Condition::Friendliness(Compare::Between(50, 50)).holds(&world, hest));
    assert!(Condition::Bucket(Bucket::Friendliness(Band::Average)).holds(&world, hest));
    assert!(!Condition::Bucket(Bucket::Friendliness(Band::High)).holds(&world, hest));
}

#[test]
fn all_any_not_nest() {
    let mut world = first_leg();
    let hest = person(&world, "hest");
    let cond = Condition::All(vec![
        Condition::Always,
        Condition::Not(Box::new(Condition::Flag("x".into()))),
    ]);
    assert!(cond.holds(&world, hest));
    world.flags.insert("x".into());
    assert!(!cond.holds(&world, hest));
    assert!(!Condition::Any(vec![]).holds(&world, hest));
    assert!(Condition::All(vec![]).holds(&world, hest));
}

#[test]
fn items_and_flags() {
    let mut world = first_leg();
    let hest = person(&world, "hest");
    let has = |count| Condition::PlayerHas {
        item: "Hearth Stone".into(),
        count,
    };
    assert!(has(1).holds(&world, hest));
    assert!(!has(2).holds(&world, hest));
    let npc_has = Condition::NpcHas {
        item: "Hearth Stone".into(),
        count: 1,
    };
    assert!(!npc_has.holds(&world, hest));
    assert!(!Condition::Flag("seen".into()).holds(&world, hest));
    world.flags.insert("seen".into());
    assert!(Condition::Flag("seen".into()).holds(&world, hest));
}

#[test]
fn hour_between_wraps() {
    let mut world = first_leg();
    let hest = person(&world, "hest");
    let night = Condition::HourBetween { from: 22, to: 3 };
    let day = Condition::HourBetween { from: 9, to: 17 };
    for (hour, is_night, is_day) in [
        (23, true, false),
        (1, true, false),
        (12, false, true),
        (9, false, true),
        (17, false, true),
        (18, false, false),
    ] {
        world.hour = hour;
        assert_eq!(night.holds(&world, hest), is_night, "night at {hour}");
        assert_eq!(day.holds(&world, hest), is_day, "day at {hour}");
    }
}

#[test]
fn follower_party_recruitable() {
    let world = first_leg();
    let donold = person(&world, "donold");
    let jonnoj = person(&world, "jonnoj");
    assert!(Condition::IsFollower.holds(&world, donold));
    assert!(Condition::InParty.holds(&world, donold));
    assert!(Condition::Recruitable.holds(&world, jonnoj));
    assert!(!Condition::IsFollower.holds(&world, jonnoj));
    assert!(!Condition::InParty.holds(&world, jonnoj));
}

#[test]
fn party_at_site() {
    let mut world = first_leg();
    at_yhlgrym(&mut world);
    let hest = person(&world, "hest");
    assert!(Condition::PartyAtSite("gate".into()).holds(&world, hest));
    assert!(!Condition::PartyAtSite("market".into()).holds(&world, hest));
}
