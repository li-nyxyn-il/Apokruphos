//! The player sheet: what a valid character is, and what the data file may say.

use varif_core::{
    load_world, Akstada, Blood, DataError, Heritage, Item, Role, SheetError, Tongue, World,
    BASE_FATIGUE, BASE_HEALTH, BASE_LEVEL, BASE_RESILIENCE,
};

const FIRST_LEG: &str = include_str!("../../../data/first_leg.ron");

fn with_sheet(
    role: &str,
    age: u32,
    tongues: &str,
    background: &str,
    extra: &str,
) -> Result<World, DataError> {
    let text = format!(
        r#"(
            start: (year: 192, month: 1, day: 1, seed: 1, party_at: "a"),
            places: [(key: "a", name: "A", kind: Town)],
            people: [(
                key: "pylgrym",
                name: "The Pylgrym",
                title: "Pylgrym",
                at: "a",
                in_party: true,
                sheet: Some((
                    role: {role},
                    age: {age},
                    blood: Red,
                    heritage: Maphlian,
                    akstada: Hare,
                    tongues: [{tongues}],
                    background: "{background}",
                    {extra}
                )),
            )],
        )"#
    );
    load_world(&text)
}

fn bad_sheet(result: Result<World, DataError>) -> SheetError {
    match result.expect_err("the sheet should be refused") {
        DataError::BadSheet { person, error } => {
            assert_eq!(person, "The Pylgrym");
            error
        }
        other => panic!("expected a bad sheet, got {other}"),
    }
}

#[test]
fn the_pylgrym_starts_from_the_base() {
    let world = load_world(FIRST_LEG).expect("the first leg data should load");
    let player = world.player().expect("the party should have a player");
    assert_eq!(player.name, "The Pylgrym");

    let sheet = player.sheet.as_ref().unwrap();
    assert_eq!(sheet.role, Role::Player);
    assert_eq!(sheet.primary, varif_core::FactionId::Player);
    assert!(sheet.secondary.is_empty());
    assert_eq!(sheet.faction_label(), "Player");
    assert_eq!(sheet.age, 20);
    assert_eq!(sheet.blood, Blood::Red);
    assert_eq!(sheet.heritage, Heritage::Shentzi);
    assert_eq!(sheet.akstada, Akstada::Pylgrym);
    assert_eq!(sheet.tongues, vec![Tongue::Shentz, Tongue::Maphlian]);
    assert_eq!(sheet.tongue_list(), "Shentz, Maphlian");
    assert_eq!(sheet.background, "Pylgrym");
    assert_eq!(sheet.health, BASE_HEALTH);
    assert_eq!(sheet.fatigue, BASE_FATIGUE);
    assert_eq!(sheet.resilience, BASE_RESILIENCE);
    assert_eq!(sheet.level, BASE_LEVEL);
    assert!(sheet.inventory.is_empty());
    assert_eq!(sheet.holding("knife"), 0);
}

#[test]
fn age_must_fall_between_sixteen_and_forty() {
    assert!(with_sheet("Player", 16, "Maphlian", "Farmer", "").is_ok());
    assert!(with_sheet("Player", 40, "Maphlian", "Farmer", "").is_ok());
    assert_eq!(
        bad_sheet(with_sheet("Player", 15, "Maphlian", "Farmer", "")),
        SheetError::AgeOutOfRange(15)
    );
    assert_eq!(
        bad_sheet(with_sheet("Player", 41, "Maphlian", "Farmer", "")),
        SheetError::AgeOutOfRange(41)
    );
    // An NPC is not held to the player's age.
    assert!(with_sheet("Npc(follower: false)", 8, "Maphlian", "Farmer", "").is_ok());
    assert!(with_sheet("Npc(follower: false)", 80, "Maphlian", "Farmer", "").is_ok());
}

#[test]
fn a_sheet_needs_a_tongue_and_a_background() {
    assert_eq!(
        bad_sheet(with_sheet("Player", 20, "", "Farmer", "")),
        SheetError::NoTongue
    );
    assert_eq!(
        bad_sheet(with_sheet("Player", 20, "Maphlian, Maphlian", "Farmer", "")),
        SheetError::DuplicateTongue(Tongue::Maphlian)
    );
    assert_eq!(
        bad_sheet(with_sheet("Player", 20, "Maphlian", "   ", "")),
        SheetError::BlankBackground
    );
}

#[test]
fn the_inventory_stacks_by_key() {
    let mut world = with_sheet(
        "Player",
        20,
        "Aphlian",
        "Carter",
        r#"inventory: [(key: "knife", name: "Knife", count: 1)]"#,
    )
    .expect("one knife is a valid inventory");
    let sheet = world.people[0].sheet.as_mut().unwrap();
    assert_eq!(sheet.holding("knife"), 1);

    sheet.carry(Item {
        key: "knife".to_string(),
        name: "Knife".to_string(),
        count: 2,
    });
    sheet.carry(Item {
        key: "rope".to_string(),
        name: "Rope".to_string(),
        count: 1,
    });
    sheet.carry(Item {
        key: "rope".to_string(),
        name: "Rope".to_string(),
        count: 0,
    });
    assert_eq!(sheet.holding("knife"), 3);
    assert_eq!(sheet.holding("rope"), 1);
    assert_eq!(sheet.inventory.len(), 2);

    let saved = World::from_ron(&world.to_ron().unwrap()).unwrap();
    assert_eq!(saved, world);
}

#[test]
fn a_bad_inventory_is_reported() {
    assert_eq!(
        bad_sheet(with_sheet(
            "Player",
            20,
            "Shentz",
            "Farmer",
            r#"inventory: [(key: "knife", name: "Knife", count: 0)]"#,
        )),
        SheetError::EmptyStack("Knife".to_string())
    );
    assert_eq!(
        bad_sheet(with_sheet(
            "Player",
            20,
            "Shentz",
            "Farmer",
            r#"inventory: [(key: "", name: "Knife", count: 1)]"#,
        )),
        SheetError::BlankItem
    );
    assert_eq!(
        bad_sheet(with_sheet(
            "Player",
            20,
            "Shentz",
            "Farmer",
            r#"inventory: [
                (key: "knife", name: "Knife", count: 1),
                (key: "knife", name: "Other knife", count: 1),
            ]"#,
        )),
        SheetError::DuplicateItem("knife".to_string())
    );
}

#[test]
fn a_person_with_no_sheet_still_loads() {
    let text = r#"(
        start: (year: 192, month: 1, day: 1, seed: 1, party_at: "a"),
        places: [(key: "a", name: "A", kind: Town)],
        people: [(key: "host", name: "The Host", title: "Innkeeper", at: "a")],
    )"#;
    let world = load_world(text).unwrap();
    assert!(world.player().is_none());
    assert!(world.people[0].sheet.is_none());
}

#[test]
fn an_npc_keeps_follower_status_and_a_faction() {
    let text = r#"(
        start: (year: 192, month: 1, day: 1, seed: 1, party_at: "a"),
        places: [(key: "a", name: "A", kind: Town)],
        people: [
            (
                key: "pylgrym",
                name: "The Pylgrym",
                title: "Pylgrym",
                at: "a",
                in_party: true,
                sheet: Some((
                    role: Player,
                    age: 20,
                    blood: Red,
                    heritage: Shentzi,
                    akstada: Pylgrym,
                    tongues: [Shentz],
                    background: "Pylgrym",
                )),
            ),
            (
                key: "guide",
                name: "The Guide",
                title: "Guide",
                at: "a",
                in_party: true,
                sheet: Some((
                    role: Npc(follower: true),
                    age: 54,
                    blood: Blue,
                    heritage: Aphlian,
                    akstada: Fox,
                    tongues: [Aphlian, Ecumene],
                    background: "Prelate",
                    primary: Ecumenci,
                )),
            ),
        ],
    )"#;
    let world = load_world(text).unwrap();
    assert_eq!(world.player().unwrap().key, "pylgrym");
    let followers: Vec<_> = world
        .followers()
        .map(|person| person.key.as_str())
        .collect();
    assert_eq!(followers, ["guide"]);
    let guide = world.followers().next().unwrap().sheet.as_ref().unwrap();
    assert_eq!(guide.role.attitude(), Some(varif_core::BASE_ATTITUDE));
    assert!(guide.role.is_follower());
    assert_eq!(guide.primary, varif_core::FactionId::Ecumenci);
    assert_eq!(guide.secondary, vec![varif_core::FactionId::Player]);
    assert_eq!(guide.faction_label(), "Ecumenci");
}

#[test]
fn two_players_are_refused() {
    let text = r#"(
        start: (year: 192, month: 1, day: 1, seed: 1, party_at: "a"),
        places: [(key: "a", name: "A", kind: Town)],
        people: [
            (
                key: "one",
                name: "One",
                title: "Pylgrym",
                at: "a",
                in_party: true,
                sheet: Some((
                    role: Player,
                    age: 20,
                    blood: Red,
                    heritage: Maphlian,
                    akstada: Hare,
                    tongues: [Maphlian],
                    background: "Pylgrym",
                )),
            ),
            (
                key: "two",
                name: "Two",
                title: "Pylgrym",
                at: "a",
                in_party: true,
                sheet: Some((
                    role: Player,
                    age: 21,
                    blood: Red,
                    heritage: Maphlian,
                    akstada: Hare,
                    tongues: [Maphlian],
                    background: "Pylgrym",
                )),
            ),
        ],
    )"#;
    assert_eq!(load_world(text).unwrap_err(), DataError::TwoPlayers);
}

#[test]
fn the_choices_a_new_character_is_offered() {
    assert_eq!(Blood::ALL.len(), 3);
    assert_eq!(Heritage::ALL.len(), 9);
    assert_eq!(Akstada::ALL.len(), 21);
    assert_eq!(Tongue::ALL.len(), 10);
    for blood in Blood::ALL {
        assert!(!blood.name().is_empty());
    }
    for sign in Akstada::ALL {
        assert!(sign.name().starts_with("the "));
    }
}
