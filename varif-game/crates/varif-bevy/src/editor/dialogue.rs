//! The Dialogue tab: the entries that open the conversation, the nodes of
//! the tree, and for each node its line, sprite and choices.
//!
//! Every change to the tree's shape goes through the plain functions at the
//! top of this file, which are tested without a screen.

use super::{
    action,
    condition::{self, ConditionContext},
    widgets::key_combo,
    EditorContext,
};
use bevy_egui::egui;
use std::fmt;
use varif_core::{
    unreachable_nodes, Action, Choice, Condition, DialogueTree, Entry, Node, PersonFile,
};

/// The key of the first node of a new tree.
const START_KEY: &str = "start";
/// The key a new node starts with, made unique if it is taken.
const NODE_KEY: &str = "node";

/// A tree with one node, "start", and an entry that always opens it.
pub fn new_tree() -> DialogueTree {
    DialogueTree {
        entries: vec![Entry {
            when: Condition::Always,
            node: START_KEY.to_string(),
        }],
        nodes: vec![Node {
            key: START_KEY.to_string(),
            line: String::new(),
            sprite: None,
            choices: Vec::new(),
        }],
    }
}

/// `base` if no node has it, otherwise `base-2`, `base-3` and so on.
pub fn unique_node_key(tree: &DialogueTree, base: &str) -> String {
    if tree.node(base).is_none() {
        return base.to_string();
    }
    (2..)
        .map(|n| format!("{base}-{n}"))
        .find(|key| tree.node(key).is_none())
        .expect("there are always more numbers")
}

/// Adds an empty node with a fresh key and returns the key.
pub fn add_node(tree: &mut DialogueTree) -> String {
    let key = unique_node_key(tree, NODE_KEY);
    tree.nodes.push(Node {
        key: key.clone(),
        line: String::new(),
        sprite: None,
        choices: Vec::new(),
    });
    key
}

/// Why a rename was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenameError {
    Empty,
    Taken,
    Missing,
}

impl fmt::Display for RenameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenameError::Empty => write!(f, "A node key cannot be empty."),
            RenameError::Taken => write!(f, "Another node already has that key."),
            RenameError::Missing => write!(f, "That node is gone."),
        }
    }
}

/// Gives the node `old` the key `new` (spaces at the ends are dropped) and
/// rewrites every entry and every choice that led to it. Returns the key
/// now in use. Renaming to the same key does nothing.
pub fn rename_node(tree: &mut DialogueTree, old: &str, new: &str) -> Result<String, RenameError> {
    let new = new.trim();
    if tree.node(old).is_none() {
        return Err(RenameError::Missing);
    }
    if new.is_empty() {
        return Err(RenameError::Empty);
    }
    if new == old {
        return Ok(new.to_string());
    }
    if tree.node(new).is_some() {
        return Err(RenameError::Taken);
    }
    for node in &mut tree.nodes {
        if node.key == old {
            node.key = new.to_string();
        }
        for choice in &mut node.choices {
            if choice.next.as_deref() == Some(old) {
                choice.next = Some(new.to_string());
            }
        }
    }
    for entry in &mut tree.entries {
        if entry.node == old {
            entry.node = new.to_string();
        }
    }
    Ok(new.to_string())
}

/// What deleting a node does to the rest of the tree.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Fallout {
    /// Choices elsewhere that led to the node and now end the conversation.
    pub links_ended: usize,
    /// Entries that opened at the node and are removed.
    pub entries_removed: usize,
}

/// What deleting `key` would do, without doing it.
pub fn fallout_of_delete(tree: &DialogueTree, key: &str) -> Fallout {
    Fallout {
        links_ended: tree
            .nodes
            .iter()
            .filter(|node| node.key != key)
            .flat_map(|node| &node.choices)
            .filter(|choice| choice.next.as_deref() == Some(key))
            .count(),
        entries_removed: tree.entries.iter().filter(|e| e.node == key).count(),
    }
}

/// Removes the node. Choices elsewhere that led to it become "End" and
/// entries that opened at it are removed. Returns what that cost.
pub fn delete_node(tree: &mut DialogueTree, key: &str) -> Fallout {
    let fallout = fallout_of_delete(tree, key);
    tree.nodes.retain(|node| node.key != key);
    for node in &mut tree.nodes {
        for choice in &mut node.choices {
            if choice.next.as_deref() == Some(key) {
                choice.next = None;
            }
        }
    }
    tree.entries.retain(|entry| entry.node != key);
    fallout
}

/// Adds an entry that always holds and opens at the first node. Does
/// nothing, and returns false, when the tree has no nodes.
pub fn add_entry(tree: &mut DialogueTree) -> bool {
    let Some(first) = tree.nodes.first() else {
        return false;
    };
    tree.entries.push(Entry {
        when: Condition::Always,
        node: first.key.clone(),
    });
    true
}

pub fn remove_entry(tree: &mut DialogueTree, index: usize) {
    if index < tree.entries.len() {
        tree.entries.remove(index);
    }
}

/// Moves an entry one place earlier (`-1`) or later (`1`). Returns false
/// at either end.
pub fn move_entry(tree: &mut DialogueTree, index: usize, by: isize) -> bool {
    let Some(target) = index.checked_add_signed(by) else {
        return false;
    };
    if index >= tree.entries.len() || target >= tree.entries.len() {
        return false;
    }
    tree.entries.swap(index, target);
    true
}

/// A new choice: always offered, saying nothing, ending the conversation.
pub fn add_choice(node: &mut Node) {
    node.choices.push(Choice {
        text: String::new(),
        when: Condition::Always,
        actions: Vec::new(),
        next: None,
    });
}

pub fn remove_choice(node: &mut Node, index: usize) {
    if index < node.choices.len() {
        node.choices.remove(index);
    }
}

/// What the tab remembers between frames, per person.
#[derive(Clone, Default)]
struct UiState {
    selected: Option<String>,
    /// The key being typed in the rename box, and the node it was started for.
    rename: String,
    rename_for: String,
    /// The node whose delete is waiting for a yes.
    confirm_delete: Option<String>,
    message: String,
}

/// A change to the tree's shape, collected while drawing and made after.
enum Op {
    Select(String),
    AddNode,
    Rename(String, String),
    AskDelete(String),
    ConfirmDelete(String),
    CancelDelete,
    AddEntry,
    RemoveEntry(usize),
    MoveEntry(usize, isize),
}

pub fn ui(ui: &mut egui::Ui, file: &mut PersonFile, context: &EditorContext) {
    let Some(tree) = file.tree.as_mut() else {
        ui.label("This person has no dialogue tree, so they use the general lines.");
        if ui.button("Create tree").clicked() {
            file.tree = Some(new_tree());
        }
        return;
    };

    let sites = context
        .places
        .iter()
        .find(|(key, _, _)| key == &file.at)
        .map(|(_, _, sites)| sites.clone())
        .unwrap_or_default();
    let cctx = ConditionContext { sites };
    let sprites: Vec<(String, String)> = file
        .sprites
        .iter()
        .map(|sprite| (sprite.name.clone(), sprite.name.clone()))
        .collect();

    let state_id = egui::Id::new(("dialogue_ui", file.key.as_str()));
    let mut state: UiState = ui
        .data_mut(|data| data.get_temp(state_id))
        .unwrap_or_default();
    if state
        .selected
        .as_deref()
        .is_some_and(|key| tree.node(key).is_none())
    {
        state.selected = None;
    }

    let keys: Vec<String> = tree.nodes.iter().map(|node| node.key.clone()).collect();
    let unreachable = unreachable_nodes(tree);
    let mut ops: Vec<Op> = Vec::new();

    entries_section(ui, tree, &keys, &cctx, &mut ops);
    ui.separator();
    ui.heading("Nodes");
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            ui.set_width(190.0);
            node_list(ui, &keys, &unreachable, &state, &mut ops);
        });
        ui.separator();
        ui.vertical(|ui| {
            let selected = state.selected.clone();
            match selected.and_then(|key| tree.nodes.iter().position(|n| n.key == key)) {
                Some(at) => {
                    let fallout = fallout_of_delete(tree, &tree.nodes[at].key);
                    let node = &mut tree.nodes[at];
                    ui.push_id(node.key.clone(), |ui| {
                        node_editor(
                            ui, node, &keys, &sprites, &cctx, &mut state, fallout, &mut ops,
                        );
                    });
                }
                None => {
                    ui.weak("Pick a node on the left, or add one.");
                }
            }
        });
    });

    for op in ops {
        apply(op, tree, &mut state);
    }
    ui.data_mut(|data| data.insert_temp(state_id, state));
}

fn apply(op: Op, tree: &mut DialogueTree, state: &mut UiState) {
    match op {
        Op::Select(key) => {
            state.message.clear();
            state.confirm_delete = None;
            state.selected = Some(key);
        }
        Op::AddNode => {
            state.message.clear();
            state.selected = Some(add_node(tree));
        }
        Op::Rename(old, new) => match rename_node(tree, &old, &new) {
            Ok(key) => {
                state.message.clear();
                state.rename = key.clone();
                state.rename_for = key.clone();
                state.selected = Some(key);
            }
            Err(error) => state.message = error.to_string(),
        },
        Op::AskDelete(key) => state.confirm_delete = Some(key),
        Op::ConfirmDelete(key) => {
            let fallout = delete_node(tree, &key);
            state.confirm_delete = None;
            state.selected = None;
            state.message = format!(
                "Deleted {key}: {} link(s) now end the conversation, {} entry(ies) removed.",
                fallout.links_ended, fallout.entries_removed
            );
        }
        Op::CancelDelete => state.confirm_delete = None,
        Op::AddEntry => {
            add_entry(tree);
        }
        Op::RemoveEntry(index) => remove_entry(tree, index),
        Op::MoveEntry(index, by) => {
            move_entry(tree, index, by);
        }
    }
}

fn entries_section(
    ui: &mut egui::Ui,
    tree: &mut DialogueTree,
    keys: &[String],
    cctx: &ConditionContext,
    ops: &mut Vec<Op>,
) {
    ui.heading("Entries");
    ui.label("First that holds wins.");
    let count = tree.entries.len();
    for (index, entry) in tree.entries.iter_mut().enumerate() {
        ui.push_id(("entry", index), |ui| {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("{}.", index + 1));
                    if ui.add_enabled(index > 0, egui::Button::new("Up")).clicked() {
                        ops.push(Op::MoveEntry(index, -1));
                    }
                    if ui
                        .add_enabled(index + 1 < count, egui::Button::new("Down"))
                        .clicked()
                    {
                        ops.push(Op::MoveEntry(index, 1));
                    }
                    if ui.button("Remove").clicked() {
                        ops.push(Op::RemoveEntry(index));
                    }
                    ui.label("opens at");
                    node_picker(ui, "entry_node", &mut entry.node, keys);
                });
                ui.horizontal_top(|ui| {
                    ui.label("when");
                    ui.vertical(|ui| condition::ui(ui, "when", &mut entry.when, cctx));
                });
            });
        });
    }
    if ui
        .add_enabled(!keys.is_empty(), egui::Button::new("Add entry"))
        .clicked()
    {
        ops.push(Op::AddEntry);
    }
    if count == 0 {
        ui.colored_label(
            egui::Color32::YELLOW,
            "With no entries nobody can start this conversation.",
        );
    }
}

fn node_list(
    ui: &mut egui::Ui,
    keys: &[String],
    unreachable: &[String],
    state: &UiState,
    ops: &mut Vec<Op>,
) {
    for key in keys {
        let mut label = key.clone();
        if unreachable.contains(key) {
            label.push_str(" (unreachable)");
        }
        let chosen = state.selected.as_deref() == Some(key.as_str());
        if ui.selectable_label(chosen, label).clicked() {
            ops.push(Op::Select(key.clone()));
        }
    }
    if ui.button("Add node").clicked() {
        ops.push(Op::AddNode);
    }
}

/// A dropdown over the node keys, for a key that must be one of them.
fn node_picker(ui: &mut egui::Ui, id: &str, value: &mut String, keys: &[String]) {
    egui::ComboBox::from_id_salt(id)
        .selected_text(value.as_str())
        .show_ui(ui, |ui| {
            for key in keys {
                ui.selectable_value(value, key.clone(), key);
            }
        });
}

/// A dropdown over the node keys, or "End" for none.
fn next_picker(ui: &mut egui::Ui, id: &str, value: &mut Option<String>, keys: &[String]) {
    egui::ComboBox::from_id_salt(id)
        .selected_text(value.as_deref().unwrap_or("End"))
        .show_ui(ui, |ui| {
            ui.selectable_value(value, None, "End");
            for key in keys {
                ui.selectable_value(value, Some(key.clone()), key);
            }
        });
}

#[allow(clippy::too_many_arguments)]
fn node_editor(
    ui: &mut egui::Ui,
    node: &mut Node,
    keys: &[String],
    sprites: &[(String, String)],
    cctx: &ConditionContext,
    state: &mut UiState,
    fallout: Fallout,
    ops: &mut Vec<Op>,
) {
    if state.rename_for != node.key {
        state.rename = node.key.clone();
        state.rename_for = node.key.clone();
    }
    ui.horizontal(|ui| {
        ui.label("Key");
        ui.text_edit_singleline(&mut state.rename);
        if ui.button("Rename").clicked() {
            ops.push(Op::Rename(node.key.clone(), state.rename.clone()));
        }
        if ui.button("Delete node").clicked() {
            ops.push(Op::AskDelete(node.key.clone()));
        }
    });
    if state.confirm_delete.as_deref() == Some(node.key.as_str()) {
        ui.group(|ui| {
            ui.label(format!(
                "Delete {}? {} links will become End, and {} entries will be removed.",
                node.key, fallout.links_ended, fallout.entries_removed
            ));
            ui.horizontal(|ui| {
                if ui.button("Delete").clicked() {
                    ops.push(Op::ConfirmDelete(node.key.clone()));
                }
                if ui.button("Cancel").clicked() {
                    ops.push(Op::CancelDelete);
                }
            });
        });
    }
    if !state.message.is_empty() {
        ui.colored_label(egui::Color32::YELLOW, &state.message);
    }

    ui.label("NPC says");
    ui.add(
        egui::TextEdit::multiline(&mut node.line)
            .desired_rows(3)
            .desired_width(f32::INFINITY),
    );
    ui.horizontal(|ui| {
        ui.label("Sprite");
        key_combo(ui, "node_sprite", &mut node.sprite, sprites);
    });

    ui.separator();
    ui.strong("Choices");
    let mut remove = None;
    for (index, choice) in node.choices.iter_mut().enumerate() {
        ui.push_id(("choice", index), |ui| {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("{}.", index + 1));
                    ui.add(
                        egui::TextEdit::singleline(&mut choice.text)
                            .hint_text("what the player says"),
                    );
                    if ui.button("Remove choice").clicked() {
                        remove = Some(index);
                    }
                });
                ui.horizontal_top(|ui| {
                    ui.label("offered when");
                    ui.vertical(|ui| condition::ui(ui, "when", &mut choice.when, cctx));
                });
                actions_editor(ui, &mut choice.actions);
                ui.horizontal(|ui| {
                    ui.label("then go to");
                    next_picker(ui, "next", &mut choice.next, keys);
                });
            });
        });
    }
    if let Some(index) = remove {
        remove_choice(node, index);
    }
    if ui.button("Add choice").clicked() {
        add_choice(node);
    }
}

fn actions_editor(ui: &mut egui::Ui, actions: &mut Vec<Action>) {
    ui.label("Actions, in order");
    let mut remove = None;
    for (index, act) in actions.iter_mut().enumerate() {
        ui.horizontal_top(|ui| {
            if ui.small_button("Remove").clicked() {
                remove = Some(index);
            }
            ui.vertical(|ui| action::ui(ui, ("action", index), act));
        });
    }
    if let Some(index) = remove {
        actions.remove(index);
    }
    if ui.small_button("Add action").clicked() {
        actions.push(action::default_of(0));
    }
}

#[cfg(test)]
mod tests {
    use super::super::logic::{people_from, TEST_MAIN};
    use super::*;
    use varif_core::{check_person_file, sandbox, Command, Compare, Event, SandboxSetup};

    fn node(key: &str, choices: Vec<Choice>) -> Node {
        Node {
            key: key.to_string(),
            line: format!("line of {key}"),
            sprite: None,
            choices,
        }
    }

    fn go(text: &str, next: Option<&str>) -> Choice {
        Choice {
            text: text.to_string(),
            when: Condition::Always,
            actions: Vec::new(),
            next: next.map(str::to_string),
        }
    }

    /// entries: [a, b]; a -> b, a -> End; b -> a.
    fn small_tree() -> DialogueTree {
        DialogueTree {
            entries: vec![
                Entry {
                    when: Condition::Flag("x".to_string()),
                    node: "b".to_string(),
                },
                Entry {
                    when: Condition::Always,
                    node: "a".to_string(),
                },
            ],
            nodes: vec![
                node("a", vec![go("to b", Some("b")), go("bye", None)]),
                node("b", vec![go("to a", Some("a")), go("again", Some("b"))]),
            ],
        }
    }

    #[test]
    fn a_new_tree_has_start_and_an_always_entry() {
        let tree = new_tree();
        assert_eq!(tree.nodes.len(), 1);
        assert_eq!(tree.nodes[0].key, "start");
        assert_eq!(tree.entries.len(), 1);
        assert_eq!(tree.entries[0].when, Condition::Always);
        assert_eq!(tree.entries[0].node, "start");
        assert!(unreachable_nodes(&tree).is_empty());
    }

    #[test]
    fn added_nodes_get_fresh_keys() {
        let mut tree = new_tree();
        assert_eq!(add_node(&mut tree), "node");
        assert_eq!(add_node(&mut tree), "node-2");
        assert_eq!(tree.nodes.len(), 3);
    }

    #[test]
    fn rename_rewrites_entries_and_choices() {
        let mut tree = small_tree();
        assert_eq!(
            rename_node(&mut tree, "b", " beta "),
            Ok("beta".to_string())
        );
        assert!(tree.node("b").is_none());
        assert_eq!(tree.entries[0].node, "beta");
        assert_eq!(
            tree.node("a").unwrap().choices[0].next.as_deref(),
            Some("beta")
        );
        // A node's link to itself follows it.
        assert_eq!(
            tree.node("beta").unwrap().choices[1].next.as_deref(),
            Some("beta")
        );
        // Links to other nodes and to End are left alone.
        assert_eq!(
            tree.node("beta").unwrap().choices[0].next.as_deref(),
            Some("a")
        );
        assert_eq!(tree.node("a").unwrap().choices[1].next, None);
    }

    #[test]
    fn rename_refuses_empty_taken_and_missing_and_changes_nothing() {
        let mut tree = small_tree();
        let before = tree.clone();
        assert_eq!(rename_node(&mut tree, "a", "  "), Err(RenameError::Empty));
        assert_eq!(rename_node(&mut tree, "a", "b"), Err(RenameError::Taken));
        assert_eq!(
            rename_node(&mut tree, "zzz", "c"),
            Err(RenameError::Missing)
        );
        assert_eq!(tree, before);
        assert_eq!(rename_node(&mut tree, "a", "a"), Ok("a".to_string()));
        assert_eq!(tree, before);
    }

    #[test]
    fn delete_counts_and_ends_links_and_removes_entries() {
        let mut tree = small_tree();
        assert_eq!(
            fallout_of_delete(&tree, "b"),
            Fallout {
                links_ended: 1,
                entries_removed: 1
            },
            "b's own link to itself goes with it and is not counted"
        );
        let fallout = delete_node(&mut tree, "b");
        assert_eq!(
            fallout,
            Fallout {
                links_ended: 1,
                entries_removed: 1
            }
        );
        assert!(tree.node("b").is_none());
        assert_eq!(tree.entries.len(), 1);
        assert_eq!(tree.entries[0].node, "a");
        assert_eq!(tree.node("a").unwrap().choices[0].next, None);
    }

    #[test]
    fn entries_move_and_stop_at_the_ends() {
        let mut tree = small_tree();
        assert!(!move_entry(&mut tree, 0, -1));
        assert!(!move_entry(&mut tree, 1, 1));
        assert!(!move_entry(&mut tree, 5, 1));
        assert!(move_entry(&mut tree, 0, 1));
        assert_eq!(tree.entries[0].node, "a");
        assert_eq!(tree.entries[1].node, "b");
        assert!(move_entry(&mut tree, 1, -1));
        assert_eq!(tree.entries[0].node, "b");
    }

    #[test]
    fn entries_add_and_remove() {
        let mut tree = small_tree();
        assert!(add_entry(&mut tree));
        assert_eq!(tree.entries.len(), 3);
        assert_eq!(tree.entries[2].node, "a");
        remove_entry(&mut tree, 0);
        remove_entry(&mut tree, 9);
        assert_eq!(tree.entries.len(), 2);
        let mut empty = DialogueTree {
            entries: Vec::new(),
            nodes: Vec::new(),
        };
        assert!(!add_entry(&mut empty));
    }

    #[test]
    fn choices_add_and_remove() {
        let mut node = node("n", Vec::new());
        add_choice(&mut node);
        add_choice(&mut node);
        assert_eq!(node.choices.len(), 2);
        assert_eq!(node.choices[0].next, None);
        remove_choice(&mut node, 0);
        remove_choice(&mut node, 4);
        assert_eq!(node.choices.len(), 1);
    }

    /// The three-node tree the brief asks to be built on Hest, made with
    /// the editor's own operations.
    fn hest_tree() -> DialogueTree {
        let mut tree = new_tree();
        // greet: rename start, add the other two.
        rename_node(&mut tree, "start", "greet").unwrap();
        let hostile = add_node(&mut tree);
        let hostile = rename_node(&mut tree, &hostile, "hostile").unwrap();
        let thanks = add_node(&mut tree);
        let thanks = rename_node(&mut tree, &thanks, "thanks").unwrap();

        // The hostile greeting is tried first.
        assert!(add_entry(&mut tree));
        tree.entries[1].node = hostile.clone();
        tree.entries[1].when = Condition::Hostility(Compare::AtMost(30));
        assert!(move_entry(&mut tree, 1, -1));

        for n in &mut tree.nodes {
            n.line = match n.key.as_str() {
                "greet" => "Halt. State your business.".to_string(),
                "hostile" => "Not you again.".to_string(),
                _ => "Pass, friend.".to_string(),
            };
        }
        let greet = tree.nodes.iter_mut().find(|n| n.key == "greet").unwrap();
        add_choice(greet);
        greet.choices[0] = Choice {
            text: "I vouch for myself.".to_string(),
            when: Condition::Attitude(Compare::AtLeast(70)),
            actions: vec![
                Action::ChangeAttitude(5),
                Action::SetFlag("met_hest".to_string()),
            ],
            next: Some(thanks),
        };
        add_choice(greet);
        greet.choices[1].text = "Never mind.".to_string();
        let hostile_node = tree.nodes.iter_mut().find(|n| n.key == hostile).unwrap();
        add_choice(hostile_node);
        hostile_node.choices[0].text = "Back off.".to_string();
        let thanks_node = tree.nodes.iter_mut().find(|n| n.key == "thanks").unwrap();
        add_choice(thanks_node);
        thanks_node.choices[0].text = "Farewell.".to_string();
        tree
    }

    #[test]
    fn a_tree_built_with_the_ops_checks_and_plays_on_hest() {
        let mut hest = people_from(TEST_MAIN, &[])
            .unwrap()
            .into_iter()
            .find(|p| p.key == "hest")
            .unwrap();
        let tree = hest_tree();
        assert!(unreachable_nodes(&tree).is_empty());
        hest.tree = Some(tree);
        check_person_file(TEST_MAIN, &[], &hest).expect("the tree loads cleanly");

        let setup = |attitude: u32, hostility: u32| SandboxSetup {
            attitude,
            hostility,
            follower: false,
            flags: Default::default(),
            hour: 12,
        };

        // Friendly, trusting: the greeting, and the gated choice is offered.
        let (mut world, id) = sandbox(TEST_MAIN, &[], &hest, &setup(80, 100)).unwrap();
        let events = world.apply(Command::Interact { person: id }).unwrap();
        assert!(events.iter().any(|e| matches!(
            e,
            Event::ConversationOpened { opening, .. } if opening == "Halt. State your business."
        )));
        let options: Vec<String> = world
            .dialogue_options(id)
            .into_iter()
            .map(|o| o.text)
            .collect();
        assert_eq!(options, ["I vouch for myself.", "Never mind."]);
        world.apply(Command::Say { line: 0 }).unwrap();
        assert!(world.flags.contains("met_hest"));
        assert_eq!(world.npc_line(), Some("Pass, friend."));
        assert_eq!(world.standing(id).unwrap().attitude, 85);
        world.apply(Command::Say { line: 0 }).unwrap();
        assert_eq!(world.conversation, None, "Farewell ends it");

        // Lukewarm: the gated choice is not offered.
        let (mut world, id) = sandbox(TEST_MAIN, &[], &hest, &setup(50, 100)).unwrap();
        world.apply(Command::Interact { person: id }).unwrap();
        let options: Vec<String> = world
            .dialogue_options(id)
            .into_iter()
            .map(|o| o.text)
            .collect();
        assert_eq!(options, ["Never mind."]);

        // Hostile but still willing to talk: the hostile greeting wins.
        let (mut world, id) = sandbox(TEST_MAIN, &[], &hest, &setup(100, 25)).unwrap();
        let events = world.apply(Command::Interact { person: id }).unwrap();
        assert!(events.iter().any(|e| matches!(
            e,
            Event::ConversationOpened { opening, .. } if opening == "Not you again."
        )));
    }
}
