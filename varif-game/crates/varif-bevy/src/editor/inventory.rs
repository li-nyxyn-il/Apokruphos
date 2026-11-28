//! The Inventory tab: the pack's stacks and the worn slots.

use super::{
    widgets::{item_editor, new_item, ItemEditorOptions},
    EditorContext,
};
use bevy_egui::egui;
use varif_core::{Character, EquipSlot, Equipped, PersonFile};

pub fn ui(ui: &mut egui::Ui, file: &mut PersonFile, _context: &EditorContext) {
    let Some(sheet) = file.sheet.as_mut() else {
        ui.label("This person has no sheet, so no inventory. Give them one on the Sheet tab.");
        return;
    };

    ui.heading("Pack");
    let mut remove = None;
    for (index, item) in sheet.inventory.iter_mut().enumerate() {
        let title = format!("{} x{}", item.name, item.count);
        egui::CollapsingHeader::new(title)
            .id_salt(("pack", index))
            .show(ui, |ui| {
                ui.push_id(index, |ui| {
                    item_editor(ui, item, ItemEditorOptions::default())
                });
                if ui.button("Remove stack").clicked() {
                    remove = Some(index);
                }
            });
    }
    if let Some(index) = remove {
        sheet.inventory.remove(index);
    }
    if ui.button("Add stack").clicked() {
        sheet.inventory.push(new_item());
    }

    ui.separator();
    ui.heading("Worn");
    for slot in EquipSlot::ALL {
        worn_row(ui, sheet, slot);
    }
}

/// One slot: empty with an add button, or the worn item to edit.
fn worn_row(ui: &mut egui::Ui, sheet: &mut Character, slot: EquipSlot) {
    let Some(at) = sheet.equipped.iter().position(|worn| worn.slot == slot) else {
        ui.horizontal(|ui| {
            ui.label(format!("{}: empty", slot.name()));
            if ui.button("Wear an item").clicked() {
                let mut item = new_item();
                item.slot = Some(slot);
                sheet.equipped.push(Equipped { slot, item });
            }
        });
        return;
    };
    let title = format!("{}: {}", slot.name(), sheet.equipped[at].item.name);
    let mut action = None;
    egui::CollapsingHeader::new(title)
        .id_salt(("worn", slot.name()))
        .show(ui, |ui| {
            ui.push_id(slot.name(), |ui| {
                ui.label(format!("Slot: {} (fixed), count 1", slot.name()));
                item_editor(ui, &mut sheet.equipped[at].item, ItemEditorOptions::WORN)
            });
            ui.horizontal(|ui| {
                if ui.button("Empty").clicked() {
                    action = Some(false);
                }
                if ui.button("Move to pack").clicked() {
                    action = Some(true);
                }
            });
        });
    force_worn(&mut sheet.equipped[at]);
    match action {
        Some(true) => move_to_pack(sheet, slot),
        Some(false) => empty_slot(sheet, slot),
        None => {}
    }
}

/// A worn item fits its slot and is one of a kind.
pub fn force_worn(worn: &mut Equipped) {
    worn.item.slot = Some(worn.slot);
    worn.item.count = 1;
}

/// Takes whatever is worn in `slot` off, and drops it.
pub fn empty_slot(sheet: &mut Character, slot: EquipSlot) {
    sheet.equipped.retain(|worn| worn.slot != slot);
}

/// Takes whatever is worn in `slot` off and carries it in the pack.
pub fn move_to_pack(sheet: &mut Character, slot: EquipSlot) {
    if let Some(at) = sheet.equipped.iter().position(|worn| worn.slot == slot) {
        let worn = sheet.equipped.remove(at);
        sheet.inventory.push(worn.item);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::logic::blank_sheet;

    fn worn_in(slot: EquipSlot) -> Equipped {
        let mut item = new_item();
        item.count = 7;
        item.slot = None;
        Equipped { slot, item }
    }

    #[test]
    fn forcing_a_worn_item_sets_its_slot_and_count() {
        let mut worn = worn_in(EquipSlot::ALL[2]);
        force_worn(&mut worn);
        assert_eq!(worn.item.slot, Some(EquipSlot::ALL[2]));
        assert_eq!(worn.item.count, 1);
    }

    #[test]
    fn emptying_a_slot_removes_only_that_entry() {
        let mut sheet = blank_sheet();
        sheet.equipped.push(worn_in(EquipSlot::ALL[0]));
        sheet.equipped.push(worn_in(EquipSlot::ALL[1]));
        let packed = sheet.inventory.len();
        empty_slot(&mut sheet, EquipSlot::ALL[0]);
        assert_eq!(sheet.equipped.len(), 1);
        assert_eq!(sheet.equipped[0].slot, EquipSlot::ALL[1]);
        assert_eq!(sheet.inventory.len(), packed);
    }

    #[test]
    fn moving_to_pack_carries_the_item() {
        let mut sheet = blank_sheet();
        let before = sheet.inventory.len();
        sheet.equipped.push(worn_in(EquipSlot::ALL[0]));
        move_to_pack(&mut sheet, EquipSlot::ALL[0]);
        assert!(sheet.equipped.is_empty());
        assert_eq!(sheet.inventory.len(), before + 1);
        // Nothing worn there: nothing happens.
        move_to_pack(&mut sheet, EquipSlot::ALL[0]);
        assert_eq!(sheet.inventory.len(), before + 1);
    }
}
