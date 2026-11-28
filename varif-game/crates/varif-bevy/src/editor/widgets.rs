//! Widgets shared by the editor's tabs: a dropdown over a fixed list, a
//! dropdown over keyed choices, and the item editor used by the Inventory
//! tab (and by the Dialogue tab's actions).

use bevy_egui::egui;
use std::hash::Hash;
use varif_core::{Effect, EffectKind, EffectPayload, EquipSlot, Item, ItemType, Rarity};

/// A dropdown over a fixed list of choices.
pub fn enum_combo<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    id: impl Hash + std::fmt::Debug,
    value: &mut T,
    all: &[T],
    name: fn(T) -> &'static str,
) {
    egui::ComboBox::from_id_salt(id)
        .selected_text(name(*value))
        .show_ui(ui, |ui| {
            for &option in all {
                ui.selectable_value(value, option, name(option));
            }
        });
}

/// A dropdown over keyed choices, `(key, name)`, for an optional key.
/// "none" is always offered. A key not in `options` is shown as it is.
pub fn key_combo(
    ui: &mut egui::Ui,
    id: impl Hash + std::fmt::Debug,
    value: &mut Option<String>,
    options: &[(String, String)],
) {
    let shown = match value {
        None => "none".to_string(),
        Some(key) => options
            .iter()
            .find(|(option, _)| option == key)
            .map(|(_, name)| name.clone())
            .unwrap_or_else(|| key.clone()),
    };
    egui::ComboBox::from_id_salt(id)
        .selected_text(shown)
        .show_ui(ui, |ui| {
            ui.selectable_value(value, None, "none");
            for (key, name) in options {
                ui.selectable_value(value, Some(key.clone()), name);
            }
        });
}

pub const ITEM_TYPES: [ItemType; 5] = [
    ItemType::Food,
    ItemType::Water,
    ItemType::Wood,
    ItemType::Copper,
    ItemType::Goods,
];

pub const EFFECT_KINDS: [EffectKind; 4] = [
    EffectKind::Equipment,
    EffectKind::Inventory,
    EffectKind::Use,
    EffectKind::Consumed,
];

const PAYLOAD_NAMES: [&str; 3] = ["Restore health", "Raise max resilience", "Add vitality"];

/// Which of the payload variants this is, as an index into [`PAYLOAD_NAMES`].
fn payload_index(payload: EffectPayload) -> usize {
    match payload {
        EffectPayload::RestoreHealth { .. } => 0,
        EffectPayload::RaiseMaxResilience { .. } => 1,
        EffectPayload::AddVitality { .. } => 2,
    }
}

pub fn payload_amount(payload: EffectPayload) -> u32 {
    match payload {
        EffectPayload::RestoreHealth { amount }
        | EffectPayload::RaiseMaxResilience { amount }
        | EffectPayload::AddVitality { amount } => amount,
    }
}

/// The payload of variant `index` carrying `amount`.
fn payload_of(index: usize, amount: u32) -> EffectPayload {
    match index {
        0 => EffectPayload::RestoreHealth { amount },
        1 => EffectPayload::RaiseMaxResilience { amount },
        _ => EffectPayload::AddVitality { amount },
    }
}

/// A new effect to start editing from.
pub fn new_effect() -> Effect {
    Effect {
        kind: EffectKind::Consumed,
        payload: EffectPayload::RestoreHealth { amount: 1 },
    }
}

/// A new goods stack to start editing from.
pub fn new_item() -> Item {
    let mut item = Item::supply(ItemType::Goods, 1);
    item.name = "New item".to_string();
    item
}

fn slot_name(slot: Option<EquipSlot>) -> &'static str {
    slot.map_or("none", EquipSlot::name)
}

/// Every field of an item, with its effects. The caller wraps this in
/// `ui.push_id` so two editors on screen do not share widget ids.
pub fn item_editor(ui: &mut egui::Ui, item: &mut Item) {
    egui::Grid::new("item_fields")
        .num_columns(2)
        .spacing([12.0, 6.0])
        .show(ui, |ui| {
            ui.label("Name");
            ui.text_edit_singleline(&mut item.name);
            ui.end_row();

            ui.label("Type");
            enum_combo(
                ui,
                "item_type",
                &mut item.item_type,
                &ITEM_TYPES,
                ItemType::label,
            );
            ui.end_row();

            ui.label("Weight");
            ui.add(egui::DragValue::new(&mut item.weight).range(0..=u32::MAX));
            ui.end_row();

            ui.label("Value (copper)");
            ui.add(egui::DragValue::new(&mut item.value).range(0..=u32::MAX));
            ui.end_row();

            ui.label("Count");
            ui.add(egui::DragValue::new(&mut item.count).range(0..=u32::MAX));
            ui.end_row();

            ui.label("Durability");
            ui.add(egui::DragValue::new(&mut item.durability).range(0..=u32::MAX));
            ui.end_row();

            ui.label("Rarity");
            enum_combo(
                ui,
                "item_rarity",
                &mut item.rarity,
                &Rarity::ALL,
                Rarity::name,
            );
            ui.end_row();

            ui.label("Slot");
            egui::ComboBox::from_id_salt("item_slot")
                .selected_text(slot_name(item.slot))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut item.slot, None, "none");
                    for slot in EquipSlot::ALL {
                        ui.selectable_value(&mut item.slot, Some(slot), slot.name());
                    }
                });
            ui.end_row();
        });

    ui.label("Effects");
    let mut remove = None;
    for (index, effect) in item.effects.iter_mut().enumerate() {
        ui.push_id(index, |ui| {
            ui.horizontal(|ui| {
                enum_combo(
                    ui,
                    "effect_kind",
                    &mut effect.kind,
                    &EFFECT_KINDS,
                    EffectKind::name,
                );
                let mut variant = payload_index(effect.payload);
                let mut amount = payload_amount(effect.payload);
                egui::ComboBox::from_id_salt("effect_payload")
                    .selected_text(PAYLOAD_NAMES[variant])
                    .show_ui(ui, |ui| {
                        for (which, name) in PAYLOAD_NAMES.iter().enumerate() {
                            ui.selectable_value(&mut variant, which, *name);
                        }
                    });
                ui.add(egui::DragValue::new(&mut amount).range(0..=u32::MAX));
                effect.payload = payload_of(variant, amount);
                if ui.button("Remove").clicked() {
                    remove = Some(index);
                }
            });
        });
    }
    if let Some(index) = remove {
        item.effects.remove(index);
    }
    if ui.button("Add effect").clicked() {
        item.effects.push(new_effect());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn switching_payload_variant_keeps_the_amount() {
        let before = EffectPayload::RestoreHealth { amount: 5 };
        let after = payload_of(1, payload_amount(before));
        assert_eq!(after, EffectPayload::RaiseMaxResilience { amount: 5 });
        assert_eq!(payload_index(after), 1);
        assert_eq!(payload_of(2, 3), EffectPayload::AddVitality { amount: 3 });
        for index in 0..3 {
            assert_eq!(payload_index(payload_of(index, 9)), index);
        }
    }

    #[test]
    fn a_new_item_is_a_named_goods_stack_of_one() {
        let item = new_item();
        assert_eq!(item.item_type, ItemType::Goods);
        assert_eq!(item.count, 1);
        assert!(!item.name.is_empty());
        assert_eq!(new_effect().kind, EffectKind::Consumed);
    }
}
