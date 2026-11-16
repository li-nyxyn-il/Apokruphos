//! Hostility between the factions, and an NPC's own attitude.
//!
//! Both start from a fixed table and can change as the world goes on.

use varif_core::{
    load_world, Command, CommandError, Event, FactionId, PersonId, Role, SheetError, World,
    BASE_ATTITUDE, MAX_ATTITUDE, MAX_HOSTILITY, MIN_HOSTILITY,
};

const FIRST_LEG: &str = include_str!("../../../data/first_leg.ron");

fn world() -> World {
    load_world(FIRST_LEG).unwrap()
}

#[test]
fn hostility_starts_from_the_table() {
    use FactionId::*;
    let world = world();
    assert_eq!(world.hostility_toward(Ecumenci, Player), 50);
    assert_eq!(world.hostility_toward(Ecumenci, Bandit), 0);
    assert_eq!(world.hostility_toward(Ecumenci, Audsen), 50);

    assert_eq!(world.hostility_toward(Audsen, Player), 70);
    assert_eq!(world.hostility_toward(Audsen, Bandit), 0);
    assert_eq!(world.hostility_toward(Audsen, Ecumenci), 50);

    for other in [Player, Ecumenci, Audsen] {
        assert_eq!(world.hostility_toward(Bandit, other), 0);
    }
    for other in [Ecumenci, Bandit, Audsen] {
        assert_eq!(world.hostility_toward(Player, other), 50);
    }
    for faction in FactionId::ALL {
        assert_eq!(world.hostility_toward(faction, faction), MAX_HOSTILITY);
        let toward_player = world.hostility_toward(faction, Player);
        assert!(toward_player >= MIN_HOSTILITY && toward_player <= MAX_HOSTILITY);
    }
}

#[test]
fn hostility_can_change_and_the_other_direction_stays() {
    use FactionId::*;
    let mut world = world();
    let before = world.clone();

    let events = world
        .apply(Command::SetHostility {
            from: Audsen,
            toward: Player,
            index: 40,
        })
        .unwrap();
    assert_eq!(
        events,
        [Event::HostilityChanged {
            from: Audsen,
            toward: Player,
            index: 40
        }]
    );
    assert_eq!(world.hostility_toward(Audsen, Player), 40);
    // The player's hostility toward the Line is a different number.
    assert_eq!(world.hostility_toward(Player, Audsen), 50);

    let loaded = World::from_ron(&world.to_ron().unwrap()).unwrap();
    assert_eq!(loaded.hostility_toward(Audsen, Player), 40);

    let out_of_range = world.apply(Command::SetHostility {
        from: Audsen,
        toward: Player,
        index: MAX_HOSTILITY + 1,
    });
    assert_eq!(out_of_range, Err(CommandError::IndexOutOfRange));
    assert_eq!(world.hostility_toward(Audsen, Player), 40);

    let itself = world.apply(Command::SetHostility {
        from: Bandit,
        toward: Bandit,
        index: 10,
    });
    assert_eq!(itself, Err(CommandError::HostilityTowardItself));
    assert_eq!(world.hostility_toward(Bandit, Bandit), MAX_HOSTILITY);
    // A refused command leaves every other index as it was, apart from the
    // change that was accepted above.
    assert_eq!(
        before.hostility_toward(Ecumenci, Player),
        world.hostility_toward(Ecumenci, Player)
    );
}

#[test]
fn the_player_faction_is_not_joinable() {
    assert!(!FactionId::Player.joinable());
    assert!(FactionId::Ecumenci.joinable());
    assert!(FactionId::Bandit.joinable());
    assert!(FactionId::Audsen.joinable());
}

#[test]
fn a_followers_attitude_stays_inside_the_scale() {
    let mut sheet = varif_core::Character::new(
        Role::Npc {
            follower: true,
            attitude: BASE_ATTITUDE,
        },
        30,
        varif_core::Blood::Red,
        varif_core::Heritage::Shentzi,
        varif_core::Akstada::Bastion,
        vec![varif_core::Tongue::Shentz],
        "Sellsword",
        FactionId::Audsen,
        Vec::new(),
    )
    .unwrap();
    assert_eq!(sheet.secondary, vec![FactionId::Player]);
    assert_eq!(sheet.role.attitude(), Some(BASE_ATTITUDE));

    sheet.role = Role::Npc {
        follower: true,
        attitude: MAX_ATTITUDE + 1,
    };
    assert_eq!(sheet.check(), Err(SheetError::AttitudeOutOfRange(101)));
}

#[test]
fn an_npcs_attitude_can_change() {
    let mut world = world();
    let donold = PersonId(
        world
            .people
            .iter()
            .position(|person| person.key == "donold")
            .unwrap() as u32,
    );
    let player_id = PersonId(
        world
            .people
            .iter()
            .position(|person| person.is_player())
            .unwrap() as u32,
    );

    let events = world
        .apply(Command::SetAttitude {
            person: donold,
            index: 80,
        })
        .unwrap();
    assert_eq!(
        events,
        [Event::AttitudeChanged {
            person: donold,
            index: 80
        }]
    );
    assert_eq!(
        world
            .person(donold)
            .unwrap()
            .sheet
            .as_ref()
            .unwrap()
            .role
            .attitude(),
        Some(80)
    );

    let loaded = World::from_ron(&world.to_ron().unwrap()).unwrap();
    assert_eq!(
        loaded
            .person(donold)
            .unwrap()
            .sheet
            .as_ref()
            .unwrap()
            .role
            .attitude(),
        Some(80)
    );

    let before = world.clone();
    assert_eq!(
        world.apply(Command::SetAttitude {
            person: donold,
            index: MAX_ATTITUDE + 1,
        }),
        Err(CommandError::IndexOutOfRange)
    );
    assert_eq!(
        world.apply(Command::SetAttitude {
            person: player_id,
            index: 10,
        }),
        Err(CommandError::NoAttitude)
    );
    assert_eq!(world, before);
}
