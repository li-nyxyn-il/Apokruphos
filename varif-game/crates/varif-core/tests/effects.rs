//! Item effects and equipment slots.

use varif_core::{
    load_world, Command, CommandError, Effect, EffectKind, EffectPayload, EquipSlot, Event, Item,
    ItemType, World, BASE_HEALTH, BASE_RESILIENCE,
};

const FIRST_LEG: &str = include_str!("../../../data/first_leg.ron");

/// One sample item for each equipment slot, in the starting pack.
const SLOT_SAMPLES: [(&str, EquipSlot); 11] = [
    ("Trail Shoes", EquipSlot::Shoes),
    ("Trail Breeches", EquipSlot::Legs),
    ("Trail Coat", EquipSlot::Body),
    ("Trail Bracers", EquipSlot::Arms),
    ("Trail Gloves", EquipSlot::Gloves),
    ("Trail Hood", EquipSlot::Head),
    ("Trail Roll", EquipSlot::Back),
    ("Trail Staff", EquipSlot::MainHand),
    ("Trail Knife", EquipSlot::OffHand),
    ("Trail Robes", EquipSlot::Robes),
    ("Trail Torc", EquipSlot::Neck),
];

fn first_leg() -> World {
    load_world(FIRST_LEG).expect("the first leg data should load")
}

/// Leaves Sthepenhall onto the road, where a camp may be pitched.
fn leave_town(world: &mut World) {
    let falls = world.find_place("falls").unwrap();
    world
        .apply(Command::Travel {
            to: falls,
            hours: 1,
        })
        .unwrap();
}

fn player_sheet(world: &World) -> &varif_core::Character {
    world
        .player()
        .and_then(|person| person.sheet.as_ref())
        .expect("the player should have a sheet")
}

fn player_sheet_mut(world: &mut World) -> &mut varif_core::Character {
    world
        .people
        .iter_mut()
        .find(|person| person.is_player())
        .and_then(|person| person.sheet.as_mut())
        .expect("the player should have a sheet")
}

fn holding(world: &World, name: &str) -> u32 {
    let sheet = player_sheet(world);
    sheet
        .inventory
        .iter()
        .find(|item| item.name == name)
        .map_or(0, |item| item.count)
}

#[test]
fn the_consumed_sample_restores_ten_health() {
    let mut world = first_leg();
    let food = player_sheet(&world)
        .inventory
        .iter()
        .find(|item| item.name == "Food")
        .expect("the starting pack carries food");
    assert_eq!(
        food.effects,
        vec![Effect {
            kind: EffectKind::Consumed,
            payload: EffectPayload::RestoreHealth { amount: 10 },
        }]
    );
    player_sheet_mut(&mut world).health = 40;

    leave_town(&mut world);
    world.apply(Command::MakeCamp).unwrap();
    let events = world.apply(Command::Rest).unwrap();

    assert!(events.iter().any(|event| matches!(
        event,
        Event::EffectFired {
            item,
            kind: EffectKind::Consumed,
            payload: EffectPayload::RestoreHealth { amount: 10 },
            ..
        } if item == "Food"
    )));
    assert_eq!(player_sheet(&world).health, 50);
    assert_eq!(holding(&world, "Food"), 4);
    let donold = world
        .people
        .iter()
        .find(|person| person.key == "donold")
        .and_then(|person| person.sheet.as_ref())
        .unwrap();
    assert_eq!(donold.health, BASE_HEALTH);
}

#[test]
fn a_consumed_restore_stops_at_base_health() {
    let mut world = first_leg();
    player_sheet_mut(&mut world).health = 95;

    leave_town(&mut world);
    world.apply(Command::MakeCamp).unwrap();
    world.apply(Command::Rest).unwrap();

    assert_eq!(player_sheet(&world).health, BASE_HEALTH);
}

#[test]
fn the_inventory_sample_raises_max_resilience_by_ten() {
    let mut world = first_leg();
    let sheet = player_sheet(&world);
    let stone = sheet
        .inventory
        .iter()
        .find(|item| item.name == "Hearth Stone")
        .expect("the starting pack carries the hearth stone");
    assert_eq!(
        stone.effects,
        vec![Effect {
            kind: EffectKind::Inventory,
            payload: EffectPayload::RaiseMaxResilience { amount: 10 },
        }]
    );
    assert!(stone.slot.is_none());
    assert_eq!(sheet.resilience, BASE_RESILIENCE);
    assert_eq!(sheet.resilience_max, BASE_RESILIENCE);
    assert_eq!(sheet.max_resilience(), BASE_RESILIENCE + 10);
    // Willpower still reads the current resilience, not the raised maximum.
    assert_eq!(sheet.willpower(), 100);

    player_sheet_mut(&mut world)
        .inventory
        .retain(|item| item.name != "Hearth Stone");
    let sheet = player_sheet(&world);
    assert_eq!(sheet.resilience, BASE_RESILIENCE);
    assert_eq!(sheet.max_resilience(), BASE_RESILIENCE);
}

#[test]
fn the_equipment_sample_adds_five_vitality_after_the_derived_floor() {
    let mut world = first_leg();
    let sheet = player_sheet_mut(&mut world);
    // 15 * 100 = 1500. 38² = 1444 and 39² = 1521, so the floor is 38.
    sheet.health = 15;
    sheet.fatigue = 100;
    assert_eq!(sheet.vitality(), 38);
    assert_eq!(sheet.vitality_bonus, 0);

    let events = world
        .apply(Command::Equip {
            item: "Pulse Cord".to_string(),
            slot: EquipSlot::Neck,
        })
        .unwrap();
    assert!(matches!(
        events.as_slice(),
        [Event::Equipped {
            item,
            slot: EquipSlot::Neck,
            ..
        }] if item == "Pulse Cord"
    ));

    let sheet = player_sheet(&world);
    assert_eq!(sheet.health, 15);
    assert_eq!(sheet.fatigue, 100);
    assert_eq!(sheet.vitality(), 38 + 5);
    assert_eq!(sheet.vitality_bonus, 0);
    assert_eq!(holding(&world, "Pulse Cord"), 0);
    assert!(sheet.inventory.iter().all(|item| item.name != "Pulse Cord"));
    assert_eq!(sheet.in_slot(EquipSlot::Neck).unwrap().name, "Pulse Cord");

    world
        .apply(Command::Unequip {
            slot: EquipSlot::Neck,
        })
        .unwrap();
    let sheet = player_sheet(&world);
    assert_eq!(sheet.health, 15);
    assert_eq!(sheet.fatigue, 100);
    assert_eq!(sheet.vitality(), 38);
    assert_eq!(holding(&world, "Pulse Cord"), 1);
    assert!(sheet.in_slot(EquipSlot::Neck).is_none());
}

#[test]
fn an_item_can_carry_two_effects_and_use_does_not_consume() {
    let text = r#"(
        start: (year: 192, month: 1, day: 1, seed: 1, party_at: "a"),
        places: [(key: "a", name: "A", kind: Landmark)],
        people: [(
            key: "pylgrym",
            name: "The Pylgrym",
            title: "Pylgrym",
            at: "a",
            in_party: true,
            sheet: Some((
                role: Player,
                age: 20,
                blood: Red,
                heritage: Shentzi,
                akstada: Pylgrym,
                tongues: [Shentz],
                background: "Pylgrym",
                health: 20,
                inventory: [(
                    name: "Food",
                    type: Food,
                    count: 1,
                    effects: [
                        (kind: Use, payload: RestoreHealth(amount: 4)),
                        (kind: Consumed, payload: RaiseMaxResilience(amount: 6)),
                    ],
                ), (name: "Water", type: Water, count: 1), (name: "Wood", type: Wood, count: 1)],
            )),
        )],
    )"#;
    let mut world = load_world(text).expect("the sheet should load");
    let before = world.clone();

    let events = world
        .apply(Command::Use {
            item: "Food".to_string(),
        })
        .unwrap();
    assert!(matches!(
        events.first(),
        Some(Event::Used { item, .. }) if item == "Food"
    ));
    assert!(events.iter().any(|event| matches!(
        event,
        Event::EffectFired {
            kind: EffectKind::Use,
            payload: EffectPayload::RestoreHealth { amount: 4 },
            ..
        }
    )));
    assert!(events.iter().all(|event| !matches!(
        event,
        Event::EffectFired {
            kind: EffectKind::Consumed,
            ..
        }
    )));
    assert_eq!(player_sheet(&world).health, 24);
    assert_eq!(player_sheet(&world).max_resilience(), BASE_RESILIENCE);
    assert_eq!(holding(&world, "Food"), 1);
    assert_ne!(world, before);

    world.apply(Command::MakeCamp).unwrap();
    let events = world.apply(Command::Rest).unwrap();
    assert!(events.iter().any(|event| matches!(
        event,
        Event::Rested {
            spent_food: true,
            spent_water: true,
            spent_wood: true,
            ..
        }
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        Event::EffectFired {
            item,
            kind: EffectKind::Consumed,
            payload: EffectPayload::RaiseMaxResilience { amount: 6 },
            ..
        } if item == "Food"
    )));
    assert!(events.iter().all(|event| !matches!(
        event,
        Event::EffectFired {
            kind: EffectKind::Use,
            ..
        }
    )));
    assert_eq!(holding(&world, "Food"), 0);
    assert_eq!(player_sheet(&world).health, 24);
    assert_eq!(player_sheet(&world).resilience, BASE_RESILIENCE);
    assert_eq!(player_sheet(&world).max_resilience(), BASE_RESILIENCE + 6);
}

#[test]
fn camp_sleep_consumes_food_and_fires_its_consumed_effect() {
    let mut world = first_leg();
    player_sheet_mut(&mut world).health = 70;
    assert_eq!(holding(&world, "Food"), 5);
    assert_eq!(holding(&world, "Water"), 5);
    assert_eq!(holding(&world, "Wood"), 5);

    leave_town(&mut world);
    world.apply(Command::MakeCamp).unwrap();
    let events = world.apply(Command::Rest).unwrap();

    assert!(matches!(
        events.first(),
        Some(Event::Rested {
            spent_food: true,
            spent_water: true,
            spent_wood: true,
            ..
        })
    ));
    let fired: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            Event::EffectFired {
                item,
                kind,
                payload,
                ..
            } => Some((item.as_str(), *kind, *payload)),
            _ => None,
        })
        .collect();
    assert_eq!(
        fired,
        vec![(
            "Food",
            EffectKind::Consumed,
            EffectPayload::RestoreHealth { amount: 10 }
        )]
    );
    assert_eq!(holding(&world, "Food"), 4);
    assert_eq!(holding(&world, "Water"), 4);
    assert_eq!(holding(&world, "Wood"), 4);
    assert_eq!(player_sheet(&world).health, 80);
}

#[test]
fn equip_and_unequip_move_the_item_between_pack_and_slot() {
    let mut world = first_leg();
    assert_eq!(holding(&world, "Trail Shoes"), 1);
    assert!(player_sheet(&world).equipped.is_empty());

    world
        .apply(Command::Equip {
            item: "Trail Shoes".to_string(),
            slot: EquipSlot::Shoes,
        })
        .unwrap();
    assert_eq!(holding(&world, "Trail Shoes"), 0);
    assert!(player_sheet(&world)
        .inventory
        .iter()
        .all(|item| item.name != "Trail Shoes"));
    assert_eq!(
        player_sheet(&world).in_slot(EquipSlot::Shoes).unwrap().name,
        "Trail Shoes"
    );

    let events = world
        .apply(Command::Unequip {
            slot: EquipSlot::Shoes,
        })
        .unwrap();
    assert!(matches!(
        events.as_slice(),
        [Event::Unequipped {
            item,
            slot: EquipSlot::Shoes,
            ..
        }] if item == "Trail Shoes"
    ));
    assert_eq!(holding(&world, "Trail Shoes"), 1);
    assert!(player_sheet(&world).in_slot(EquipSlot::Shoes).is_none());
    assert!(player_sheet(&world).equipped.is_empty());
}

#[test]
fn a_slot_rejects_an_item_that_does_not_fit() {
    let mut world = first_leg();
    let before = world.clone();

    assert_eq!(
        world.apply(Command::Equip {
            item: "Trail Shoes".to_string(),
            slot: EquipSlot::Head,
        }),
        Err(CommandError::DoesNotFit {
            item: "Trail Shoes".to_string(),
            slot: EquipSlot::Head,
        })
    );
    assert_eq!(world, before);

    assert_eq!(
        world.apply(Command::Equip {
            item: "Hearth Stone".to_string(),
            slot: EquipSlot::Neck,
        }),
        Err(CommandError::DoesNotFit {
            item: "Hearth Stone".to_string(),
            slot: EquipSlot::Neck,
        })
    );
    assert_eq!(world, before);
}

#[test]
fn the_eleven_slot_samples_sit_in_the_starting_pack() {
    let world = first_leg();
    let sheet = player_sheet(&world);
    assert!(sheet.equipped.is_empty());
    assert_eq!(EquipSlot::ALL.len(), 11);
    for (name, slot) in SLOT_SAMPLES {
        let item = sheet
            .inventory
            .iter()
            .find(|item| item.name == name)
            .unwrap_or_else(|| panic!("{name} should be in the starting pack"));
        assert_eq!(item.slot, Some(slot));
        assert_eq!(item.count, 1);
        assert_eq!(item.item_type, ItemType::Goods);
    }
    let saved = World::from_ron(&world.to_ron().unwrap()).unwrap();
    assert_eq!(saved.player().unwrap().sheet, world.player().unwrap().sheet);
}

#[test]
fn a_carried_restore_does_not_change_stored_health() {
    let mut item = Item::goods("Balm", 1);
    item.effects.push(Effect {
        kind: EffectKind::Inventory,
        payload: EffectPayload::RestoreHealth { amount: 7 },
    });
    let mut world = first_leg();
    player_sheet_mut(&mut world).carry(item);
    let sheet = player_sheet(&world);
    assert_eq!(sheet.health, BASE_HEALTH);
    assert_eq!(sheet.effective_health(), BASE_HEALTH + 7);
}
