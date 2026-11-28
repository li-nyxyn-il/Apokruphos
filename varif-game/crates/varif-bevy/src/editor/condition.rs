//! The condition builder: a recursive editor for a dialogue
//! [`Condition`], used by the Dialogue tab for entries and choices.

use super::widgets::enum_combo;
use bevy_egui::egui;
use std::hash::Hash;
use varif_core::{Band, Bucket, Compare, Condition, FactionId};

/// What the condition editor needs to know beyond the condition itself.
pub struct ConditionContext {
    /// The sites of the person's place, as (key, name).
    pub sites: Vec<(String, String)>,
}

const NAMES: [&str; 17] = [
    "Always",
    "All of",
    "Any of",
    "Not",
    "Attitude",
    "Hostility index",
    "Friendliness",
    "Bucket is open",
    "Faction hostility",
    "Is a follower",
    "Is in the party",
    "Is recruitable",
    "Player carries",
    "NPC carries",
    "Flag is set",
    "Hour between",
    "Party is at site",
];

/// Which variant this is, as an index into [`NAMES`].
fn variant_index(condition: &Condition) -> usize {
    match condition {
        Condition::Always => 0,
        Condition::All(_) => 1,
        Condition::Any(_) => 2,
        Condition::Not(_) => 3,
        Condition::Attitude(_) => 4,
        Condition::Hostility(_) => 5,
        Condition::Friendliness(_) => 6,
        Condition::Bucket(_) => 7,
        Condition::FactionHostility { .. } => 8,
        Condition::IsFollower => 9,
        Condition::InParty => 10,
        Condition::Recruitable => 11,
        Condition::PlayerHas { .. } => 12,
        Condition::NpcHas { .. } => 13,
        Condition::Flag(_) => 14,
        Condition::HourBetween { .. } => 15,
        Condition::PartyAtSite(_) => 16,
    }
}

/// A fresh condition of variant `index`.
fn default_of(index: usize) -> Condition {
    match index {
        0 => Condition::Always,
        1 => Condition::All(Vec::new()),
        2 => Condition::Any(Vec::new()),
        3 => Condition::Not(Box::new(Condition::Always)),
        4 => Condition::Attitude(Compare::AtLeast(50)),
        5 => Condition::Hostility(Compare::AtLeast(50)),
        6 => Condition::Friendliness(Compare::AtLeast(50)),
        7 => Condition::Bucket(Bucket::Follower),
        8 => Condition::FactionHostility {
            from: FactionId::Ecumenci,
            toward: FactionId::Player,
            cmp: Compare::AtLeast(50),
        },
        9 => Condition::IsFollower,
        10 => Condition::InParty,
        11 => Condition::Recruitable,
        12 => Condition::PlayerHas {
            item: String::new(),
            count: 1,
        },
        13 => Condition::NpcHas {
            item: String::new(),
            count: 1,
        },
        14 => Condition::Flag(String::new()),
        15 => Condition::HourBetween { from: 8, to: 18 },
        _ => Condition::PartyAtSite(String::new()),
    }
}

/// The comparison of a Compare-carrying leaf.
fn compare_of(condition: &Condition) -> Option<Compare> {
    match condition {
        Condition::Attitude(cmp) | Condition::Hostility(cmp) | Condition::Friendliness(cmp) => {
            Some(*cmp)
        }
        _ => None,
    }
}

/// The item and count of a pack-holding leaf.
fn pack_of(condition: &Condition) -> Option<(String, u32)> {
    match condition {
        Condition::PlayerHas { item, count } | Condition::NpcHas { item, count } => {
            Some((item.clone(), *count))
        }
        _ => None,
    }
}

/// Turns `old` into variant `to`, keeping what carries over: the children
/// of a compound, the comparison between Attitude, Hostility and
/// Friendliness, and the item between the two pack tests. Choosing the
/// variant it already is changes nothing.
pub fn switch_variant(old: Condition, to: usize) -> Condition {
    if variant_index(&old) == to {
        return old;
    }
    let children = match &old {
        Condition::All(children) | Condition::Any(children) => children.clone(),
        Condition::Not(inner) => vec![(**inner).clone()],
        _ => Vec::new(),
    };
    let cmp = compare_of(&old);
    let pack = pack_of(&old);
    match to {
        1 => Condition::All(children),
        2 => Condition::Any(children),
        3 => Condition::Not(Box::new(children.into_iter().next().unwrap_or_default())),
        4 | 5 | 6 => {
            let cmp = cmp.unwrap_or(Compare::AtLeast(50));
            match to {
                4 => Condition::Attitude(cmp),
                5 => Condition::Hostility(cmp),
                _ => Condition::Friendliness(cmp),
            }
        }
        12 | 13 if pack.is_some() => {
            let (item, count) = pack.unwrap_or_default();
            if to == 12 {
                Condition::PlayerHas { item, count }
            } else {
                Condition::NpcHas { item, count }
            }
        }
        _ => default_of(to),
    }
}

const COMPARE_NAMES: [&str; 3] = ["At least", "At most", "Between"];

fn compare_index(cmp: Compare) -> usize {
    match cmp {
        Compare::AtLeast(_) => 0,
        Compare::AtMost(_) => 1,
        Compare::Between(..) => 2,
    }
}

/// Turns `cmp` into kind `to`, keeping the number or numbers that fit.
pub fn switch_compare(cmp: Compare, to: usize) -> Compare {
    if compare_index(cmp) == to {
        return cmp;
    }
    match (cmp, to) {
        (Compare::AtLeast(x), 1) | (Compare::AtMost(x), 0) => {
            if to == 0 {
                Compare::AtLeast(x)
            } else {
                Compare::AtMost(x)
            }
        }
        (Compare::AtLeast(x), _) => Compare::Between(x, 100.max(x)),
        (Compare::AtMost(x), _) => Compare::Between(0, x),
        (Compare::Between(low, _), 0) => Compare::AtLeast(low),
        (Compare::Between(_, high), _) => Compare::AtMost(high),
    }
}

fn compare_editor(ui: &mut egui::Ui, id: &str, cmp: &mut Compare) {
    let mut kind = compare_index(*cmp);
    egui::ComboBox::from_id_salt(id)
        .selected_text(COMPARE_NAMES[kind])
        .show_ui(ui, |ui| {
            for (index, name) in COMPARE_NAMES.iter().enumerate() {
                ui.selectable_value(&mut kind, index, *name);
            }
        });
    *cmp = switch_compare(*cmp, kind);
    match cmp {
        Compare::AtLeast(x) | Compare::AtMost(x) => {
            ui.add(egui::DragValue::new(x).range(0..=100));
        }
        Compare::Between(low, high) => {
            ui.add(egui::DragValue::new(low).range(0..=100));
            ui.label("and");
            ui.add(egui::DragValue::new(high).range(0..=100));
        }
    }
}

const BUCKET_KINDS: [&str; 4] = [
    "Friendliness band",
    "Hostility band",
    "Attitude band",
    "Follower",
];
const BANDS: [Band; 3] = [Band::Low, Band::Average, Band::High];

fn bucket_editor(ui: &mut egui::Ui, bucket: &mut Bucket) {
    let (mut kind, mut band) = match *bucket {
        Bucket::Friendliness(band) => (0, band),
        Bucket::Hostility(band) => (1, band),
        Bucket::Attitude(band) => (2, band),
        Bucket::Follower => (3, Band::Average),
    };
    egui::ComboBox::from_id_salt("bucket_kind")
        .selected_text(BUCKET_KINDS[kind])
        .show_ui(ui, |ui| {
            for (index, name) in BUCKET_KINDS.iter().enumerate() {
                ui.selectable_value(&mut kind, index, *name);
            }
        });
    if kind != 3 {
        enum_combo(ui, "bucket_band", &mut band, &BANDS, Band::name);
    }
    *bucket = match kind {
        0 => Bucket::Friendliness(band),
        1 => Bucket::Hostility(band),
        2 => Bucket::Attitude(band),
        _ => Bucket::Follower,
    };
}

/// A text box for a name, with a hint when it is empty.
fn name_field(ui: &mut egui::Ui, text: &mut String, hint: &str) {
    ui.add(egui::TextEdit::singleline(text).hint_text(hint));
}

/// Edits `condition`. `id_salt` keeps two editors on screen apart.
pub fn ui(
    ui: &mut egui::Ui,
    id_salt: impl Hash + std::fmt::Debug,
    condition: &mut Condition,
    context: &ConditionContext,
) {
    ui.push_id(id_salt, |ui| show(ui, condition, context));
}

fn show(ui: &mut egui::Ui, condition: &mut Condition, context: &ConditionContext) {
    ui.horizontal_wrapped(|ui| {
        let mut chosen = variant_index(condition);
        egui::ComboBox::from_id_salt("variant")
            .selected_text(NAMES[chosen])
            .show_ui(ui, |ui| {
                for (index, name) in NAMES.iter().enumerate() {
                    ui.selectable_value(&mut chosen, index, *name);
                }
            });
        if chosen != variant_index(condition) {
            *condition = switch_variant(std::mem::take(condition), chosen);
        }
        leaf_fields(ui, condition, context);
    });
    match condition {
        Condition::All(children) | Condition::Any(children) => {
            children_editor(ui, children, context)
        }
        Condition::Not(inner) => {
            ui.indent("not_child", |ui| ui_child(ui, 0, inner, context));
        }
        _ => {}
    }
}

fn ui_child(ui: &mut egui::Ui, index: usize, child: &mut Condition, context: &ConditionContext) {
    ui.push_id(index, |ui| show(ui, child, context));
}

fn children_editor(ui: &mut egui::Ui, children: &mut Vec<Condition>, context: &ConditionContext) {
    ui.indent("children", |ui| {
        let mut remove = None;
        for (index, child) in children.iter_mut().enumerate() {
            ui.horizontal_top(|ui| {
                if ui.small_button("Remove").clicked() {
                    remove = Some(index);
                }
                ui.vertical(|ui| ui_child(ui, index, child, context));
            });
        }
        if let Some(index) = remove {
            children.remove(index);
        }
        if ui.small_button("Add condition").clicked() {
            children.push(Condition::Always);
        }
    });
}

/// The fields of a variant with no children.
fn leaf_fields(ui: &mut egui::Ui, condition: &mut Condition, context: &ConditionContext) {
    match condition {
        Condition::Attitude(cmp) | Condition::Hostility(cmp) | Condition::Friendliness(cmp) => {
            compare_editor(ui, "compare", cmp)
        }
        Condition::Bucket(bucket) => bucket_editor(ui, bucket),
        Condition::FactionHostility { from, toward, cmp } => {
            enum_combo(ui, "from", from, &FactionId::ALL, FactionId::name);
            ui.label("toward");
            enum_combo(ui, "toward", toward, &FactionId::ALL, FactionId::name);
            compare_editor(ui, "compare", cmp);
        }
        Condition::PlayerHas { item, count } | Condition::NpcHas { item, count } => {
            ui.label("at least");
            ui.add(egui::DragValue::new(count).range(0..=u32::MAX));
            ui.label("of");
            name_field(ui, item, "item name");
        }
        Condition::Flag(flag) => name_field(ui, flag, "flag"),
        Condition::HourBetween { from, to } => {
            ui.add(egui::DragValue::new(from).range(0..=23));
            ui.label("to");
            ui.add(egui::DragValue::new(to).range(0..=23));
            ui.label("(wraps past midnight)");
        }
        Condition::PartyAtSite(key) => {
            let shown = context
                .sites
                .iter()
                .find(|(site, _)| site == key)
                .map(|(_, name)| name.clone())
                .unwrap_or_else(|| {
                    if key.is_empty() {
                        "pick a site".to_string()
                    } else {
                        key.clone()
                    }
                });
            egui::ComboBox::from_id_salt("site")
                .selected_text(shown)
                .show_ui(ui, |ui| {
                    for (site, name) in &context.sites {
                        ui.selectable_value(key, site.clone(), name);
                    }
                });
            if context.sites.is_empty() {
                ui.weak("This place has no sites.");
            }
        }
        Condition::Always
        | Condition::All(_)
        | Condition::Any(_)
        | Condition::Not(_)
        | Condition::IsFollower
        | Condition::InParty
        | Condition::Recruitable => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flag(name: &str) -> Condition {
        Condition::Flag(name.to_string())
    }

    #[test]
    fn every_variant_index_round_trips_through_its_default() {
        for index in 0..NAMES.len() {
            assert_eq!(variant_index(&default_of(index)), index);
        }
    }

    #[test]
    fn all_and_any_swap_keeping_their_children() {
        let all = Condition::All(vec![flag("a"), flag("b")]);
        let any = switch_variant(all, 2);
        assert_eq!(any, Condition::Any(vec![flag("a"), flag("b")]));
        assert_eq!(
            switch_variant(any, 1),
            Condition::All(vec![flag("a"), flag("b")])
        );
    }

    #[test]
    fn not_to_a_compound_wraps_its_child_and_back_takes_the_first() {
        let not = Condition::Not(Box::new(flag("a")));
        let all = switch_variant(not, 1);
        assert_eq!(all, Condition::All(vec![flag("a")]));

        let many = Condition::Any(vec![flag("x"), flag("y")]);
        assert_eq!(switch_variant(many, 3), Condition::Not(Box::new(flag("x"))));
        assert_eq!(
            switch_variant(Condition::All(Vec::new()), 3),
            Condition::Not(Box::new(Condition::Always))
        );
    }

    #[test]
    fn a_leaf_becomes_a_compound_with_no_children() {
        assert_eq!(
            switch_variant(Condition::InParty, 1),
            Condition::All(Vec::new())
        );
    }

    #[test]
    fn leaves_take_defaults_but_compares_and_items_carry_over() {
        let hostility = switch_variant(Condition::Attitude(Compare::Between(10, 20)), 5);
        assert_eq!(hostility, Condition::Hostility(Compare::Between(10, 20)));

        let npc = switch_variant(
            Condition::PlayerHas {
                item: "Salt".to_string(),
                count: 3,
            },
            13,
        );
        assert_eq!(
            npc,
            Condition::NpcHas {
                item: "Salt".to_string(),
                count: 3
            }
        );
        assert_eq!(
            switch_variant(flag("x"), 14),
            flag("x"),
            "the same variant is left alone"
        );
        assert_eq!(switch_variant(flag("x"), 9), Condition::IsFollower);
        assert_eq!(
            switch_variant(Condition::All(vec![flag("a")]), 4),
            Condition::Attitude(Compare::AtLeast(50))
        );
    }

    #[test]
    fn compare_kinds_keep_the_numbers_that_fit() {
        assert_eq!(
            switch_compare(Compare::AtLeast(70), 2),
            Compare::Between(70, 100)
        );
        assert_eq!(
            switch_compare(Compare::AtMost(30), 2),
            Compare::Between(0, 30)
        );
        assert_eq!(
            switch_compare(Compare::Between(10, 20), 0),
            Compare::AtLeast(10)
        );
        assert_eq!(
            switch_compare(Compare::Between(10, 20), 1),
            Compare::AtMost(20)
        );
        assert_eq!(switch_compare(Compare::AtLeast(5), 1), Compare::AtMost(5));
        assert_eq!(switch_compare(Compare::AtLeast(5), 0), Compare::AtLeast(5));
    }
}
