//! Loading dialogue trees and sprites from data, and checking them.

use varif_core::{
    check_tree, load_world, unreachable_nodes, Command, Condition, DataError, DialogueTree, Entry,
    Node, PersonId, TreeProblem, World,
};

const FIRST_LEG: &str = include_str!("../../../data/first_leg.ron");

/// First Leg with `extra` person fields added to Hest.
fn with_hest(extra: &str) -> String {
    let marker = "title: \"Gatewarden\",";
    assert_eq!(FIRST_LEG.matches(marker).count(), 1);
    FIRST_LEG.replacen(marker, &format!("{marker}\n{extra}"), 1)
}

fn bad_tree(extra: &str) -> TreeProblem {
    match load_world(&with_hest(extra)) {
        Err(DataError::BadTree { person, problem }) => {
            assert_eq!(person, "hest");
            problem
        }
        other => panic!("expected BadTree, got {other:?}"),
    }
}

/// A tree whose one node, "a", has the given choice text in RON.
fn tree_with(entries: &str, nodes: &str) -> String {
    format!("tree: Some((entries: [{entries}], nodes: [{nodes}])),")
}

const GOOD_ENTRY: &str = "(when: Always, node: \"a\")";

#[test]
fn good_tree_and_sprites_load() {
    let extra = format!(
        "{}\nsprites: [(name: \"portrait\", path: \"p.png\")],",
        tree_with(
            GOOD_ENTRY,
            "(key: \"a\", line: \"hi\", sprite: Some(\"portrait\"), choices: [(text: \"ok\", when: PartyAtSite(\"gate\"), actions: [SetFlag(\"f\")], next: None)])"
        )
    );
    let world = load_world(&with_hest(&extra)).unwrap();
    let hest = world.people.iter().find(|p| p.key == "hest").unwrap();
    assert_eq!(hest.tree.as_ref().unwrap().nodes.len(), 1);
    assert_eq!(hest.sprites.len(), 1);
}

#[test]
fn tree_with_no_entries_is_refused() {
    assert_eq!(
        bad_tree(&tree_with("", "(key: \"a\", line: \"hi\")")),
        TreeProblem::NoEntries
    );
}

#[test]
fn duplicate_node_key_is_refused() {
    assert_eq!(
        bad_tree(&tree_with(
            GOOD_ENTRY,
            "(key: \"a\", line: \"1\"), (key: \"a\", line: \"2\")"
        )),
        TreeProblem::DuplicateNode("a".into())
    );
}

#[test]
fn entry_naming_missing_node_is_refused() {
    assert_eq!(
        bad_tree(&tree_with(
            "(when: Always, node: \"zzz\")",
            "(key: \"a\", line: \"hi\")"
        )),
        TreeProblem::MissingNode("zzz".into())
    );
}

#[test]
fn choice_naming_missing_node_is_refused() {
    assert_eq!(
        bad_tree(&tree_with(
            GOOD_ENTRY,
            "(key: \"a\", line: \"hi\", choices: [(text: \"x\", next: Some(\"nowhere\"))])"
        )),
        TreeProblem::MissingNode("nowhere".into())
    );
}

#[test]
fn hour_past_23_is_refused_even_when_nested() {
    assert_eq!(
        bad_tree(&tree_with(
            "(when: All([Not(HourBetween(from: 4, to: 24))]), node: \"a\")",
            "(key: \"a\", line: \"hi\")"
        )),
        TreeProblem::BadHour(24)
    );
}

#[test]
fn party_at_site_not_in_place_is_refused() {
    assert_eq!(
        bad_tree(&tree_with(
            GOOD_ENTRY,
            "(key: \"a\", line: \"hi\", choices: [(text: \"x\", when: PartyAtSite(\"moat\"))])"
        )),
        TreeProblem::UnknownSite("moat".into())
    );
}

#[test]
fn blank_flag_is_refused_in_conditions_and_actions() {
    assert_eq!(
        bad_tree(&tree_with(
            "(when: Flag(\"  \"), node: \"a\")",
            "(key: \"a\", line: \"hi\")"
        )),
        TreeProblem::BlankFlag
    );
    assert_eq!(
        bad_tree(&tree_with(
            GOOD_ENTRY,
            "(key: \"a\", line: \"hi\", choices: [(text: \"x\", actions: [ClearFlag(\"\")])])"
        )),
        TreeProblem::BlankFlag
    );
}

#[test]
fn unknown_sprite_is_refused() {
    assert_eq!(
        bad_tree(&tree_with(
            GOOD_ENTRY,
            "(key: \"a\", line: \"hi\", sprite: Some(\"ghost\"))"
        )),
        TreeProblem::UnknownSprite("ghost".into())
    );
}

#[test]
fn duplicate_sprite_name_is_refused() {
    let extra =
        "sprites: [(name: \"portrait\", path: \"a.png\"), (name: \"portrait\", path: \"b.png\")],";
    assert_eq!(
        load_world(&with_hest(extra)).unwrap_err(),
        DataError::DuplicateSprite {
            person: "hest".into(),
            sprite: "portrait".into()
        }
    );
}

fn node(key: &str, next: Option<&str>) -> Node {
    use varif_core::Choice;
    Node {
        key: key.into(),
        line: "x".into(),
        sprite: None,
        choices: vec![Choice {
            text: "t".into(),
            when: Condition::Always,
            actions: vec![],
            next: next.map(String::from),
        }],
    }
}

#[test]
fn unreachable_nodes_are_reported() {
    let tree = DialogueTree {
        entries: vec![Entry {
            when: Condition::Always,
            node: "a".into(),
        }],
        nodes: vec![
            node("a", Some("b")),
            node("b", Some("a")),
            node("lost", Some("b")),
        ],
    };
    assert_eq!(unreachable_nodes(&tree), vec!["lost".to_string()]);
    assert_eq!(check_tree(&tree, &[], &[]), Ok(()));
}

fn hest_id(world: &World) -> PersonId {
    PersonId(world.people.iter().position(|p| p.key == "hest").unwrap() as u32)
}

#[test]
fn tree_and_sprites_round_trip_through_save() {
    let extra = format!(
        "{}\nsprites: [(name: \"portrait\", path: \"p.png\", frame: Some((columns: 2, rows: 2, index: 3)))],",
        tree_with(GOOD_ENTRY, "(key: \"a\", line: \"hi\", sprite: Some(\"portrait\"))")
    );
    let mut world = load_world(&with_hest(&extra)).unwrap();
    world.flags.insert("met_hest".to_string());
    let text = world.to_ron().unwrap();
    let back = World::from_ron(&text).unwrap();
    let (a, b) = (hest_id(&world), hest_id(&back));
    assert_eq!(world.people[a.index()].tree, back.people[b.index()].tree);
    assert!(back.people[b.index()].tree.is_some());
    assert_eq!(
        world.people[a.index()].sprites,
        back.people[b.index()].sprites
    );
    assert_eq!(back.people[b.index()].sprites.len(), 1);
    assert_eq!(back.flags, world.flags);
}

#[test]
fn old_save_without_new_fields_loads() {
    let world = load_world(FIRST_LEG).unwrap();
    let text = world.to_ron().unwrap();
    let old: String = text
        .lines()
        .filter(|line| {
            let t = line.trim_start();
            !(t.starts_with("flags:")
                || t.starts_with("conversation_node:")
                || t.starts_with("tree:")
                || t.starts_with("sprites:"))
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        old.len() < text.len(),
        "the new keys should have been stripped"
    );
    let mut back = World::from_ron(&old).expect("an old save should load");
    assert!(back.flags.is_empty());
    let hest = hest_id(&back);
    assert!(back.people[hest.index()].tree.is_none());
    assert!(back.people[hest.index()].sprites.is_empty());
    // Hest still talks, from the old general dialogue.
    let mut at = |stop: &str| {
        let to = back.find_place(stop).unwrap();
        while back.party_at != to || back.journey.is_some() {
            let left = back
                .journey
                .map(|j| j.leagues_left())
                .unwrap_or_else(|| back.road_between(back.party_at, to).unwrap().leagues);
            back.apply(Command::Travel {
                to,
                hours: left.min(8),
            })
            .unwrap();
        }
    };
    at("falls");
    at("yhlgrym");
    assert!(back.apply(Command::Interact { person: hest }).is_ok());
}
