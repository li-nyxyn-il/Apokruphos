//! The action builder: an editor for one dialogue [`Action`], used by the
//! Dialogue tab for the actions of a choice.

use super::widgets::{enum_combo, item_editor, new_item, ItemEditorOptions};
use bevy_egui::egui;
use std::hash::Hash;
use varif_core::{Action, FactionId};

const NAMES: [&str; 12] = [
    "Change attitude by",
    "Set attitude to",
    "Change faction hostility",
    "Give an item to the player",
    "Take an item from the player",
    "Grant a new item",
    "Recruit into the party",
    "Set recruitable",
    "Set a flag",
    "Clear a flag",
    "Write a journal entry",
    "Start combat",
];

/// Which variant this is, as an index into [`NAMES`].
fn variant_index(action: &Action) -> usize {
    match action {
        Action::ChangeAttitude(_) => 0,
        Action::SetAttitude(_) => 1,
        Action::ChangeHostility { .. } => 2,
        Action::GiveItem { .. } => 3,
        Action::TakeItem { .. } => 4,
        Action::GrantItem(_) => 5,
        Action::Recruit => 6,
        Action::SetRecruitable(_) => 7,
        Action::SetFlag(_) => 8,
        Action::ClearFlag(_) => 9,
        Action::WriteJournal(_) => 10,
        Action::StartCombat => 11,
    }
}

/// A fresh action of variant `index`.
pub fn default_of(index: usize) -> Action {
    match index {
        0 => Action::ChangeAttitude(5),
        1 => Action::SetAttitude(50),
        2 => Action::ChangeHostility {
            from: FactionId::Ecumenci,
            toward: FactionId::Player,
            by: 5,
        },
        3 => Action::GiveItem {
            item: String::new(),
            count: 1,
        },
        4 => Action::TakeItem {
            item: String::new(),
            count: 1,
        },
        5 => Action::GrantItem(new_item()),
        6 => Action::Recruit,
        7 => Action::SetRecruitable(true),
        8 => Action::SetFlag(String::new()),
        9 => Action::ClearFlag(String::new()),
        10 => Action::WriteJournal("ASSUMPTION: journal entry to be written".to_string()),
        _ => Action::StartCombat,
    }
}

/// Turns `old` into variant `to`, keeping the item and count between
/// Give and Take, and the text between the flag actions.
pub fn switch_variant(old: Action, to: usize) -> Action {
    if variant_index(&old) == to {
        return old;
    }
    match (old, to) {
        (Action::GiveItem { item, count }, 4) => Action::TakeItem { item, count },
        (Action::TakeItem { item, count }, 3) => Action::GiveItem { item, count },
        (Action::SetFlag(flag), 9) => Action::ClearFlag(flag),
        (Action::ClearFlag(flag), 8) => Action::SetFlag(flag),
        _ => default_of(to),
    }
}

/// Edits `action`. `id_salt` keeps two editors on screen apart.
pub fn ui(ui: &mut egui::Ui, id_salt: impl Hash + std::fmt::Debug, action: &mut Action) {
    ui.push_id(id_salt, |ui| {
        let mut chosen = variant_index(action);
        ui.horizontal_wrapped(|ui| {
            egui::ComboBox::from_id_salt("variant")
                .selected_text(NAMES[chosen])
                .show_ui(ui, |ui| {
                    for (index, name) in NAMES.iter().enumerate() {
                        ui.selectable_value(&mut chosen, index, *name);
                    }
                });
            if chosen != variant_index(action) {
                *action = switch_variant(action.clone(), chosen);
            }
            fields(ui, action);
        });
        // The item editor is tall, so it gets the rows below.
        if let Action::GrantItem(item) = action {
            ui.indent("grant", |ui| {
                item_editor(ui, item, ItemEditorOptions::default())
            });
        }
    });
}

fn name_field(ui: &mut egui::Ui, text: &mut String, hint: &str) {
    ui.add(egui::TextEdit::singleline(text).hint_text(hint));
}

fn fields(ui: &mut egui::Ui, action: &mut Action) {
    match action {
        Action::ChangeAttitude(by) => {
            ui.add(egui::DragValue::new(by).range(-100..=100));
        }
        Action::SetAttitude(to) => {
            ui.add(egui::DragValue::new(to).range(0..=100));
        }
        Action::ChangeHostility { from, toward, by } => {
            enum_combo(ui, "from", from, &FactionId::ALL, FactionId::name);
            ui.label("toward");
            enum_combo(ui, "toward", toward, &FactionId::ALL, FactionId::name);
            ui.label("by");
            ui.add(egui::DragValue::new(by).range(-100..=100));
        }
        Action::GiveItem { item, count } | Action::TakeItem { item, count } => {
            ui.add(egui::DragValue::new(count).range(0..=u32::MAX));
            ui.label("x");
            name_field(ui, item, "item name");
        }
        Action::SetRecruitable(value) => {
            ui.checkbox(value, "recruitable");
        }
        Action::SetFlag(text) | Action::ClearFlag(text) => name_field(ui, text, "flag"),
        Action::WriteJournal(text) => name_field(ui, text, "journal text"),
        Action::GrantItem(_) | Action::Recruit | Action::StartCombat => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_variant_index_round_trips_through_its_default() {
        for index in 0..NAMES.len() {
            assert_eq!(variant_index(&default_of(index)), index);
        }
    }

    #[test]
    fn give_and_take_swap_keeping_the_item() {
        let give = Action::GiveItem {
            item: "Bread".to_string(),
            count: 2,
        };
        assert_eq!(
            switch_variant(give, 4),
            Action::TakeItem {
                item: "Bread".to_string(),
                count: 2
            }
        );
    }

    #[test]
    fn flag_actions_swap_keeping_the_flag_and_others_take_defaults() {
        assert_eq!(
            switch_variant(Action::SetFlag("met".to_string()), 9),
            Action::ClearFlag("met".to_string())
        );
        assert_eq!(
            switch_variant(Action::SetFlag("met".to_string()), 6),
            Action::Recruit
        );
        assert_eq!(
            switch_variant(Action::Recruit, 0),
            Action::ChangeAttitude(5)
        );
        assert_eq!(
            switch_variant(Action::Recruit, 6),
            Action::Recruit,
            "the same variant is left alone"
        );
    }
}
