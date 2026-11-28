//! The Sheet tab: role, standing and the body.

use super::{combo, logic::blank_sheet, EditorContext};
use bevy_egui::egui;
use varif_core::{Akstada, Blood, Heritage, PersonFile, Role, Tongue, MAX_ATTITUDE, MIN_ATTITUDE};

pub fn ui(ui: &mut egui::Ui, file: &mut PersonFile, _context: &EditorContext) {
    let mut cast_only = file.sheet.is_none();
    if ui
        .checkbox(&mut cast_only, "No sheet (cast only)")
        .changed()
    {
        file.sheet = if cast_only { None } else { Some(blank_sheet()) };
    }
    let Some(sheet) = file.sheet.as_mut() else {
        ui.label("A cast-only person has no sheet. Untick the box to give them one.");
        return;
    };
    ui.separator();

    egui::Grid::new("sheet_grid")
        .num_columns(2)
        .spacing([12.0, 6.0])
        .show(ui, |ui| {
            ui.label("Role");
            match &mut sheet.role {
                Role::Player => {
                    ui.label("Player (not edited here)");
                    ui.end_row();
                }
                Role::Npc { follower, attitude } => {
                    ui.checkbox(follower, "Follower");
                    ui.end_row();
                    ui.label("Attitude");
                    ui.add(egui::Slider::new(attitude, MIN_ATTITUDE..=MAX_ATTITUDE));
                    ui.end_row();
                }
            }

            ui.label("Age");
            ui.add(egui::DragValue::new(&mut sheet.age).range(0..=999));
            ui.end_row();

            ui.label("Blood");
            combo(ui, "blood", &mut sheet.blood, &Blood::ALL, Blood::name);
            ui.end_row();

            ui.label("Heritage");
            combo(
                ui,
                "heritage",
                &mut sheet.heritage,
                &Heritage::ALL,
                Heritage::name,
            );
            ui.end_row();

            ui.label("Akstada");
            combo(
                ui,
                "akstada",
                &mut sheet.akstada,
                &Akstada::ALL,
                Akstada::name,
            );
            ui.end_row();

            ui.label("Tongues");
            ui.horizontal_wrapped(|ui| {
                for tongue in Tongue::ALL {
                    let mut spoken = sheet.tongues.contains(&tongue);
                    if ui.checkbox(&mut spoken, tongue.name()).changed() {
                        if spoken {
                            sheet.tongues.push(tongue);
                        } else {
                            sheet.tongues.retain(|known| *known != tongue);
                        }
                    }
                }
            });
            ui.end_row();

            ui.label("Background");
            ui.add(
                egui::TextEdit::multiline(&mut sheet.background)
                    .desired_rows(3)
                    .desired_width(f32::INFINITY),
            );
            ui.end_row();

            ui.label("Health");
            ui.add(egui::DragValue::new(&mut sheet.health));
            ui.end_row();

            ui.label("Fatigue");
            ui.add(egui::DragValue::new(&mut sheet.fatigue));
            ui.end_row();

            ui.label("Resilience");
            ui.add(egui::DragValue::new(&mut sheet.resilience));
            ui.end_row();

            ui.label("Resilience max");
            ui.add(egui::DragValue::new(&mut sheet.resilience_max));
            ui.end_row();

            ui.label("Level");
            ui.add(egui::DragValue::new(&mut sheet.level).range(1..=u32::MAX));
            ui.end_row();
        });
}
