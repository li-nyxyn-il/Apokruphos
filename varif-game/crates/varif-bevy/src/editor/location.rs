//! The Location & schedule tab: where the person starts, whether they
//! join the party, and the round they walk.

use super::{widgets::key_combo, EditorContext};
use bevy_egui::egui;
use varif_core::{PersonFile, SpanData, HOURS_PER_DAY};

pub fn ui(ui: &mut egui::Ui, file: &mut PersonFile, context: &EditorContext) {
    ui.heading("Starting place");
    egui::Grid::new("location_grid")
        .num_columns(2)
        .spacing([12.0, 6.0])
        .show(ui, |ui| {
            ui.label("Place");
            if place_combo(ui, "start_place", &mut file.at, context) {
                file.site = first_site(context, &file.at);
            }
            ui.end_row();
            ui.label("Site");
            key_combo(
                ui,
                "start_site",
                &mut file.site,
                &sites_of(context, &file.at),
            );
            ui.end_row();
        });
    if sites_of(context, &file.at).is_empty() {
        ui.weak("This place has no sites.");
    }
    ui.checkbox(&mut file.in_party, "In the party");
    ui.checkbox(&mut file.recruitable, "Recruitable");

    ui.separator();
    ui.heading("Schedule");
    let total: u64 = file
        .schedule
        .spans
        .iter()
        .map(|span| u64::from(span.hours))
        .sum();
    ui.label(format!("Round: {}", cycle_label(total)));

    let count = file.schedule.spans.len();
    let mut remove = None;
    let mut shift: Option<(usize, isize)> = None;
    for (index, span) in file.schedule.spans.iter_mut().enumerate() {
        ui.push_id(index, |ui| {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("Span {}", index + 1));
                    if ui.add_enabled(index > 0, egui::Button::new("Up")).clicked() {
                        shift = Some((index, -1));
                    }
                    if ui
                        .add_enabled(index + 1 < count, egui::Button::new("Down"))
                        .clicked()
                    {
                        shift = Some((index, 1));
                    }
                    if ui.button("Delete").clicked() {
                        remove = Some(index);
                    }
                });
                span_fields(ui, span, context);
            });
        });
    }
    if let Some(index) = remove {
        file.schedule.spans.remove(index);
    }
    if let Some((index, delta)) = shift {
        move_span(&mut file.schedule.spans, index, delta);
    }
    if ui.button("Add span").clicked() {
        file.schedule
            .spans
            .push(new_span(context, &file.at, file.site.clone()));
    }
}

fn span_fields(ui: &mut egui::Ui, span: &mut SpanData, context: &EditorContext) {
    egui::Grid::new("span_grid")
        .num_columns(2)
        .spacing([12.0, 6.0])
        .show(ui, |ui| {
            ui.label("Hours");
            ui.add(egui::DragValue::new(&mut span.hours).range(1..=9999));
            ui.end_row();
            ui.label("Place");
            if place_combo(ui, "span_place", &mut span.place, context) {
                span.site = first_site(context, &span.place);
            }
            ui.end_row();
            ui.label("Site");
            key_combo(
                ui,
                "span_site",
                &mut span.site,
                &sites_of(context, &span.place),
            );
            ui.end_row();
            let places = place_options(context);
            ui.label("Road from");
            key_combo(ui, "span_road_from", &mut span.road_from, &places);
            ui.end_row();
            ui.label("Road to");
            key_combo(ui, "span_road_to", &mut span.road_to, &places);
            ui.end_row();
        });
}

/// A dropdown over the world's places. True when the choice changed.
fn place_combo(ui: &mut egui::Ui, id: &str, place: &mut String, context: &EditorContext) -> bool {
    let shown = context
        .places
        .iter()
        .find(|(key, _, _)| key == place)
        .map_or_else(|| place.clone(), |(_, name, _)| name.clone());
    let before = place.clone();
    egui::ComboBox::from_id_salt(id)
        .selected_text(shown)
        .show_ui(ui, |ui| {
            for (key, name, _) in &context.places {
                ui.selectable_value(place, key.clone(), name);
            }
        });
    *place != before
}

fn place_options(context: &EditorContext) -> Vec<(String, String)> {
    context
        .places
        .iter()
        .map(|(key, name, _)| (key.clone(), name.clone()))
        .collect()
}

/// The sites of one place, as (key, name). Empty for an unknown place.
fn sites_of(context: &EditorContext, place: &str) -> Vec<(String, String)> {
    context
        .places
        .iter()
        .find(|(key, _, _)| key == place)
        .map(|(_, _, sites)| sites.clone())
        .unwrap_or_default()
}

/// The site to stand at on arriving in `place`: its first, or none if it
/// has no sites. The loader refuses a sited place with no site named.
pub fn first_site(context: &EditorContext, place: &str) -> Option<String> {
    sites_of(context, place)
        .into_iter()
        .next()
        .map(|(key, _)| key)
}

/// A new span of one hour, standing where the person starts.
fn new_span(context: &EditorContext, place: &str, site: Option<String>) -> SpanData {
    SpanData {
        hours: 1,
        place: place.to_string(),
        site: site.or_else(|| first_site(context, place)),
        road_from: None,
        road_to: None,
    }
}

/// Swaps the span at `index` with its neighbour `delta` away. Returns
/// false and changes nothing when there is no such neighbour.
pub fn move_span(spans: &mut [SpanData], index: usize, delta: isize) -> bool {
    let Some(other) = index.checked_add_signed(delta) else {
        return false;
    };
    if index >= spans.len() || other >= spans.len() {
        return false;
    }
    spans.swap(index, other);
    true
}

/// The round's length as "30 hours (1 day 6 hours)".
pub fn cycle_label(hours: u64) -> String {
    fn plural(count: u64, unit: &str) -> String {
        format!("{count} {unit}{}", if count == 1 { "" } else { "s" })
    }
    if hours == 0 {
        return "No timed round".to_string();
    }
    let per_day = u64::from(HOURS_PER_DAY);
    let (days, rest) = (hours / per_day, hours % per_day);
    if days == 0 {
        return plural(hours, "hour");
    }
    let mut parts = vec![plural(days, "day")];
    if rest > 0 {
        parts.push(plural(rest, "hour"));
    }
    format!("{} ({})", plural(hours, "hour"), parts.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::logic::{people_from, TEST_MAIN};
    use varif_core::{
        check_person_file, load_world_with_people, Effect, EffectKind, EffectPayload, Item,
        ItemType,
    };

    fn span(hours: u32) -> SpanData {
        SpanData {
            hours,
            ..SpanData::default()
        }
    }

    fn hours_of(spans: &[SpanData]) -> Vec<u32> {
        spans.iter().map(|span| span.hours).collect()
    }

    #[test]
    fn cycle_label_reads_in_hours_and_days() {
        assert_eq!(cycle_label(0), "No timed round");
        assert_eq!(cycle_label(1), "1 hour");
        assert_eq!(cycle_label(5), "5 hours");
        assert_eq!(cycle_label(24), "24 hours (1 day)");
        assert_eq!(cycle_label(30), "30 hours (1 day 6 hours)");
        assert_eq!(cycle_label(49), "49 hours (2 days 1 hour)");
        assert_eq!(cycle_label(48), "48 hours (2 days)");
    }

    #[test]
    fn spans_move_and_stop_at_the_ends() {
        let mut spans = vec![span(1), span(2), span(3)];
        assert!(move_span(&mut spans, 1, -1));
        assert_eq!(hours_of(&spans), [2, 1, 3]);
        assert!(move_span(&mut spans, 1, 1));
        assert_eq!(hours_of(&spans), [2, 3, 1]);
        assert!(!move_span(&mut spans, 0, -1));
        assert!(!move_span(&mut spans, 2, 1));
        assert!(!move_span(&mut spans, 9, 1));
        assert_eq!(hours_of(&spans), [2, 3, 1]);
    }

    fn context() -> EditorContext {
        let world = load_world_with_people(TEST_MAIN, &[]).unwrap();
        EditorContext {
            places: world
                .places
                .iter()
                .map(|place| {
                    let sites = place
                        .sites
                        .iter()
                        .map(|site| (site.key.clone(), site.name.clone()));
                    (place.key.clone(), place.name.clone(), sites.collect())
                })
                .collect(),
            hostility: world.hostility.clone(),
        }
    }

    #[test]
    fn first_site_follows_the_place() {
        let context = context();
        let (sited, _, sites) = context
            .places
            .iter()
            .find(|(_, _, sites)| !sites.is_empty())
            .unwrap();
        assert_eq!(first_site(&context, sited), Some(sites[0].0.clone()));
        assert_eq!(first_site(&context, "nowhere"), None);
        if let Some((siteless, _, _)) = context.places.iter().find(|(_, _, s)| s.is_empty()) {
            assert_eq!(first_site(&context, siteless), None);
        }
    }

    #[test]
    fn bren_with_a_seal_and_an_extra_span_passes_the_check() {
        let context = context();
        let mut bren = people_from(TEST_MAIN, &[])
            .unwrap()
            .into_iter()
            .find(|person| person.key == "bren")
            .unwrap();
        let mut seal = Item::supply(ItemType::Goods, 1);
        seal.name = "Seal".to_string();
        seal.effects.push(Effect {
            kind: EffectKind::Consumed,
            payload: EffectPayload::RestoreHealth { amount: 5 },
        });
        bren.sheet.as_mut().unwrap().inventory.push(seal);
        let mut extra = new_span(&context, &bren.at, bren.site.clone());
        extra.hours = 4;
        bren.schedule.spans.push(extra);

        check_person_file(TEST_MAIN, &[], &bren).unwrap();
    }
}
