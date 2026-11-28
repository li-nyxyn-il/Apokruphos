//! One file per person: how people files override the main data file, how a
//! person is turned back into a file, and the sandbox the editor tests in.

use std::collections::BTreeSet;
use std::path::Path;
use varif_core::{
    check_person_file, load_world, load_world_with_people, person_file_of, person_file_text,
    read_people_dir, sandbox, Action, Choice, Command, Compare, Condition, DataError, DialogueTree,
    Entry, Node, PersonFile, PersonId, SandboxSetup,
};

const FIRST_LEG: &str = include_str!("../../../data/first_leg.ron");

fn index_of(world: &varif_core::World, key: &str) -> usize {
    world
        .people
        .iter()
        .position(|p| p.key == key)
        .expect("the person should exist")
}

fn hest_file(name: &str) -> String {
    format!(
        "(key: \"hest\", name: \"{name}\", title: \"Gatewarden\", at: \"yhlgrym\", site: Some(\"gate\"),
          sheet: Some((role: Npc(follower: false), age: 34, blood: Red, heritage: Shentzi,
                       akstada: Bastion, tongues: [Shentz], background: \"x\", primary: Ecumenci)))"
    )
}

#[test]
fn people_file_overrides_main_entry_in_place() {
    let plain = load_world(FIRST_LEG).unwrap();
    let at = index_of(&plain, "hest");
    let world = load_world_with_people(
        FIRST_LEG,
        &[("hest".to_string(), hest_file("Hest the Elder"))],
    )
    .unwrap();
    assert_eq!(index_of(&world, "hest"), at);
    assert_eq!(world.people[at].name, "Hest the Elder");
    assert_eq!(world.people.len(), plain.people.len());
}

#[test]
fn people_file_adds_new_person() {
    let plain = load_world(FIRST_LEG).unwrap();
    let text = hest_file("Newcomer").replace("\"hest\"", "\"newcomer\"");
    let world = load_world_with_people(FIRST_LEG, &[("newcomer".to_string(), text)]).unwrap();
    assert_eq!(world.people.len(), plain.people.len() + 1);
    assert_eq!(world.people.last().unwrap().key, "newcomer");
}

#[test]
fn file_name_must_match_key() {
    let result = load_world_with_people(FIRST_LEG, &[("other".to_string(), hest_file("Hest"))]);
    assert_eq!(
        result.err(),
        Some(DataError::FileKeyMismatch {
            file: "other".into(),
            key: "hest".into()
        })
    );
}

#[test]
fn two_files_with_one_key_are_refused() {
    let result = load_world_with_people(
        FIRST_LEG,
        &[
            ("hest".to_string(), hest_file("A")),
            ("hest".to_string(), hest_file("B")),
        ],
    );
    assert_eq!(
        result.err(),
        Some(DataError::DuplicatePerson("hest".into()))
    );
}

#[test]
fn bad_people_file_names_the_file() {
    let unreadable = load_world_with_people(FIRST_LEG, &[("hest".to_string(), "(".to_string())]);
    assert!(matches!(
        unreadable.err(),
        Some(DataError::PeopleFile { file, .. }) if file == "hest"
    ));
    let nowhere = hest_file("Hest").replace("yhlgrym", "nowhere");
    let result = load_world_with_people(FIRST_LEG, &[("hest".to_string(), nowhere)]);
    match result.err() {
        Some(DataError::PeopleFile { file, error }) => {
            assert_eq!(file, "hest");
            assert!(matches!(*error, DataError::UnknownPlace { .. }));
        }
        other => panic!("expected PeopleFile, got {other:?}"),
    }
}

#[test]
fn person_file_round_trips() {
    let world = load_world(FIRST_LEG).unwrap();
    for (i, original) in world.people.iter().enumerate() {
        let file = person_file_of(&world, PersonId(i as u32));
        let text = person_file_text(&file);
        let again = load_world_with_people(FIRST_LEG, &[(original.key.clone(), text)])
            .unwrap_or_else(|e| panic!("{} should load back: {e}", original.key));
        assert_eq!(&again.people[i], original, "{} changed", original.key);
    }
}

fn gated_tree() -> DialogueTree {
    let choice = |text: &str, when: Condition| Choice {
        text: text.into(),
        when,
        actions: vec![Action::SetFlag("walked".into())],
        next: None,
    };
    DialogueTree {
        entries: vec![Entry {
            when: Condition::Always,
            node: "a".into(),
        }],
        nodes: vec![Node {
            key: "a".into(),
            line: "Well met.".into(),
            sprite: None,
            choices: vec![
                choice("open", Condition::Always),
                choice("gated", Condition::Attitude(Compare::AtLeast(90))),
            ],
        }],
    }
}

fn setup(attitude: u32, follower: bool) -> SandboxSetup {
    SandboxSetup {
        attitude,
        hostility: 80,
        follower,
        flags: BTreeSet::from(["seen".to_string()]),
        hour: 7,
    }
}

#[test]
fn sandbox_lets_the_tree_be_walked() {
    let before = FIRST_LEG.to_string();
    let world = load_world(FIRST_LEG).unwrap();
    let mut file = person_file_of(&world, PersonId(index_of(&world, "hest") as u32));
    file.tree = Some(gated_tree());
    let (mut world, hest) = sandbox(FIRST_LEG, &[], &file, &setup(95, false)).unwrap();
    assert_eq!(world.hour, 7);
    assert!(world.flags.contains("seen"));
    world.apply(Command::Interact { person: hest }).unwrap();
    assert_eq!(world.dialogue_options(hest).len(), 2);
    world.apply(Command::Say { line: 1 }).unwrap();
    assert!(world.flags.contains("walked"));
    assert_eq!(FIRST_LEG, before);
}

#[test]
fn sandbox_hides_a_gated_choice_at_low_attitude() {
    let world = load_world(FIRST_LEG).unwrap();
    let mut file = person_file_of(&world, PersonId(index_of(&world, "hest") as u32));
    file.tree = Some(gated_tree());
    let (mut world, hest) = sandbox(FIRST_LEG, &[], &file, &setup(60, false)).unwrap();
    world.apply(Command::Interact { person: hest }).unwrap();
    assert_eq!(world.dialogue_options(hest).len(), 1);
}

#[test]
fn sandbox_stands_a_follower_with_the_party() {
    let world = load_world(FIRST_LEG).unwrap();
    let mut file = person_file_of(&world, PersonId(index_of(&world, "syla") as u32));
    file.tree = Some(gated_tree());
    let (mut world, syla) = sandbox(FIRST_LEG, &[], &file, &setup(50, true)).unwrap();
    assert!(world.nearby_npcs().contains(&syla));
    let sheet = world.people[syla.index()].sheet.as_ref().unwrap();
    assert!(sheet.role.is_follower());
    world.apply(Command::Interact { person: syla }).unwrap();
}

#[test]
fn sandbox_needs_a_sheet() {
    let file = PersonFile {
        key: "ghost".into(),
        name: "Ghost".into(),
        at: "yhlgrym".into(),
        site: Some("gate".into()),
        ..PersonFile::default()
    };
    assert_eq!(
        sandbox(FIRST_LEG, &[], &file, &setup(50, false)).err(),
        Some(DataError::NoSheet("ghost".into()))
    );
}

#[test]
fn check_person_file_is_the_save_gate() {
    let world = load_world(FIRST_LEG).unwrap();
    let mut file = person_file_of(&world, PersonId(index_of(&world, "hest") as u32));
    assert_eq!(check_person_file(FIRST_LEG, &[], &file), Ok(()));
    file.at = "nowhere".into();
    assert!(check_person_file(FIRST_LEG, &[], &file).is_err());
}

#[test]
fn read_people_dir_missing_is_empty() {
    let missing = Path::new("no/such/dir/for/people");
    assert_eq!(read_people_dir(missing).unwrap(), Vec::new());
}

#[test]
fn read_people_dir_reads_ron_files_sorted() {
    let dir = std::env::temp_dir().join(format!("varif-people-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("b.ron"), "B").unwrap();
    std::fs::write(dir.join("a.ron"), "A").unwrap();
    std::fs::write(dir.join(".gitkeep"), "").unwrap();
    std::fs::write(dir.join("notes.txt"), "no").unwrap();
    let found = read_people_dir(&dir).unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
    assert_eq!(
        found,
        vec![
            ("a".to_string(), "A".to_string()),
            ("b".to_string(), "B".to_string())
        ]
    );
}

fn first_leg_file(key: &str) -> PersonFile {
    let world = load_world(FIRST_LEG).unwrap();
    person_file_of(&world, PersonId(index_of(&world, key) as u32))
}

#[test]
fn sandbox_person_without_a_site_is_still_nearby() {
    let mut file = first_leg_file("hest");
    file.tree = Some(gated_tree());
    file.site = None;
    file.in_party = true; // an unrelated `at` and in_party must not matter
    let (mut world, hest) = sandbox(FIRST_LEG, &[], &file, &setup(60, false)).unwrap();
    assert!(!world.people[hest.index()].in_party);
    assert!(world.nearby_npcs().contains(&hest));
    world.apply(Command::Interact { person: hest }).unwrap();
}

#[test]
fn sandbox_moves_the_party_to_the_person() {
    let mut file = first_leg_file("hest");
    file.tree = Some(gated_tree());
    file.in_party = true;
    let (world, hest) = sandbox(FIRST_LEG, &[], &file, &setup(60, true)).unwrap();
    let yhlgrym = world.find_place("yhlgrym").unwrap();
    assert_eq!(world.party_at, yhlgrym);
    assert_eq!(world.people[hest.index()].at, yhlgrym);
    assert!(world.nearby_npcs().contains(&hest));
}

#[test]
fn sandbox_applies_the_whole_setup() {
    use varif_core::{FactionId, Role, TimeMode};
    let mut file = first_leg_file("hest");
    file.tree = Some(gated_tree());
    let (world, hest) = sandbox(FIRST_LEG, &[], &file, &setup(42, true)).unwrap();
    let sheet = world.people[hest.index()].sheet.as_ref().unwrap();
    assert_eq!(
        sheet.role,
        Role::Npc {
            follower: true,
            attitude: 42
        }
    );
    assert!(sheet.belongs_to(FactionId::Player));
    assert_eq!(world.hostility_toward(sheet.primary, FactionId::Player), 80);
    assert_eq!(world.time_mode, TimeMode::TurnBased);
    assert!(world.journey.is_none() && !world.camped);
    assert!(world.conversation.is_none());
}

#[test]
fn sandbox_leaves_the_players_own_hostility_alone() {
    use varif_core::FactionId;
    let mut file = first_leg_file("hest");
    file.tree = Some(gated_tree());
    file.sheet.as_mut().unwrap().primary = FactionId::Player;
    let (world, _) = sandbox(FIRST_LEG, &[], &file, &setup(50, false)).unwrap();
    assert_eq!(
        world.hostility_toward(FactionId::Player, FactionId::Player),
        100
    );
}
