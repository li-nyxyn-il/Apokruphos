//! The Factions tab: the primary faction, the secondaries, and how each
//! faction currently stands toward the player.

use super::{combo, EditorContext};
use bevy_egui::egui;
use varif_core::{FactionId, PersonFile};

pub fn ui(ui: &mut egui::Ui, file: &mut PersonFile, context: &EditorContext) {
    let Some(sheet) = file.sheet.as_mut() else {
        ui.label("A cast-only person has no sheet, so no factions.");
        return;
    };

    ui.horizontal(|ui| {
        ui.label("Primary");
        combo(
            ui,
            "primary_faction",
            &mut sheet.primary,
            &FactionId::ALL,
            FactionId::name,
        );
    });
    let primary = sheet.primary;
    sheet.secondary.retain(|faction| *faction != primary);

    ui.separator();
    ui.label("Secondary factions");
    for faction in FactionId::ALL {
        if faction == primary {
            continue;
        }
        let mut member = sheet.secondary.contains(&faction);
        if ui.checkbox(&mut member, faction.name()).changed() {
            if member {
                sheet.secondary.push(faction);
            } else {
                sheet.secondary.retain(|other| *other != faction);
            }
        }
    }

    ui.separator();
    ui.label("Hostility toward the player (100 is least hostile, 0 most)");
    egui::Grid::new("faction_hostility")
        .num_columns(2)
        .show(ui, |ui| {
            for faction in FactionId::ALL {
                let role = if faction == primary {
                    " (primary)"
                } else if sheet.secondary.contains(&faction) {
                    " (secondary)"
                } else {
                    ""
                };
                ui.label(format!("{}{role}", faction.name()));
                ui.label(
                    context
                        .hostility
                        .get(faction, FactionId::Player)
                        .to_string(),
                );
                ui.end_row();
            }
        });
}
