//! Dialogue conditions and trees: what they read from the world, and how
//! conversations run through the commands.

use varif_core::{
    load_world, Action, Band, Bucket, Choice, Command, CommandError, Compare, Condition,
    DialogueTree, Entry, Event, FactionId, Item, ItemType, Node, PersonId, Rarity, World,
};

const FIRST_LEG: &str = include_str!("../../../data/first_leg.ron");

fn first_leg() -> World {
    load_world(FIRST_LEG).expect("the first leg data should load")
}

fn person(world: &World, key: &str) -> PersonId {
    let index = world
        .people
        .iter()
        .position(|p| p.key == key)
        .expect("the person should exist");
    PersonId(index as u32)
}

fn walk(world: &mut World, stop: &str) {
    let to = world.find_place(stop).unwrap();
    while world.party_at != to || world.journey.is_some() {
        let left = world
            .journey
            .map(|journey| journey.leagues_left())
            .unwrap_or_else(|| world.road_between(world.party_at, to).unwrap().leagues);
        world
            .apply(Command::Travel {
                to,
                hours: left.min(8),
            })
            .unwrap();
    }
}

/// The party at Yhlgrym's gate, where Hest stands.
fn at_yhlgrym(world: &mut World) {
    walk(world, "falls");
    walk(world, "yhlgrym");
}

fn set_attitude(world: &mut World, who: PersonId, index: u32) {
    world
        .apply(Command::SetAttitude { person: who, index })
        .unwrap();
}

#[test]
fn compare_between_is_inclusive() {
    let between = Compare::Between(30, 70);
    assert!(between.holds(30));
    assert!(between.holds(70));
    assert!(!between.holds(29));
    assert!(!between.holds(71));
    assert!(Compare::AtLeast(5).holds(5));
    assert!(Compare::AtMost(5).holds(5));
    assert!(!Compare::AtMost(5).holds(6));
}

#[test]
fn attitude_condition_reads_the_npc() {
    let mut world = first_leg();
    let hest = person(&world, "hest");
    set_attitude(&mut world, hest, 60);
    assert!(Condition::Attitude(Compare::AtLeast(60)).holds(&world, hest));
    assert!(!Condition::Attitude(Compare::AtLeast(61)).holds(&world, hest));
}

#[test]
fn hostility_condition_reads_primary_faction() {
    let world = first_leg();
    let hest = person(&world, "hest");
    assert!(Condition::Hostility(Compare::AtMost(50)).holds(&world, hest));
    assert!(!Condition::Hostility(Compare::AtMost(49)).holds(&world, hest));
}

#[test]
fn friendliness_and_bucket() {
    let mut world = first_leg();
    let hest = person(&world, "hest");
    set_attitude(&mut world, hest, 50);
    assert!(Condition::Friendliness(Compare::Between(50, 50)).holds(&world, hest));
    assert!(Condition::Bucket(Bucket::Friendliness(Band::Average)).holds(&world, hest));
    assert!(!Condition::Bucket(Bucket::Friendliness(Band::High)).holds(&world, hest));
}

#[test]
fn all_any_not_nest() {
    let mut world = first_leg();
    let hest = person(&world, "hest");
    let cond = Condition::All(vec![
        Condition::Always,
        Condition::Not(Box::new(Condition::Flag("x".into()))),
    ]);
    assert!(cond.holds(&world, hest));
    world.flags.insert("x".into());
    assert!(!cond.holds(&world, hest));
    assert!(!Condition::Any(vec![]).holds(&world, hest));
    assert!(Condition::All(vec![]).holds(&world, hest));
}

#[test]
fn items_and_flags() {
    let mut world = first_leg();
    let hest = person(&world, "hest");
    let has = |count| Condition::PlayerHas {
        item: "Hearth Stone".into(),
        count,
    };
    assert!(has(1).holds(&world, hest));
    assert!(!has(2).holds(&world, hest));
    let npc_has = Condition::NpcHas {
        item: "Hearth Stone".into(),
        count: 1,
    };
    assert!(!npc_has.holds(&world, hest));
    assert!(!Condition::Flag("seen".into()).holds(&world, hest));
    world.flags.insert("seen".into());
    assert!(Condition::Flag("seen".into()).holds(&world, hest));
}

#[test]
fn hour_between_wraps() {
    let mut world = first_leg();
    let hest = person(&world, "hest");
    let night = Condition::HourBetween { from: 22, to: 3 };
    let day = Condition::HourBetween { from: 9, to: 17 };
    for (hour, is_night, is_day) in [
        (23, true, false),
        (1, true, false),
        (12, false, true),
        (9, false, true),
        (17, false, true),
        (18, false, false),
    ] {
        world.hour = hour;
        assert_eq!(night.holds(&world, hest), is_night, "night at {hour}");
        assert_eq!(day.holds(&world, hest), is_day, "day at {hour}");
    }
}

#[test]
fn follower_party_recruitable() {
    let world = first_leg();
    let donold = person(&world, "donold");
    let jonnoj = person(&world, "jonnoj");
    assert!(Condition::IsFollower.holds(&world, donold));
    assert!(Condition::InParty.holds(&world, donold));
    assert!(Condition::Recruitable.holds(&world, jonnoj));
    assert!(!Condition::IsFollower.holds(&world, jonnoj));
    assert!(!Condition::InParty.holds(&world, jonnoj));
}

#[test]
fn party_at_site() {
    let mut world = first_leg();
    at_yhlgrym(&mut world);
    let hest = person(&world, "hest");
    assert!(Condition::PartyAtSite("gate".into()).holds(&world, hest));
    assert!(!Condition::PartyAtSite("market".into()).holds(&world, hest));
}

#[test]
fn faction_hostility_condition_reads_the_table() {
    let mut world = first_leg();
    let hest = person(&world, "hest");
    let cond = |cmp| Condition::FactionHostility {
        from: FactionId::Ecumenci,
        toward: FactionId::Player,
        cmp,
    };
    assert!(cond(Compare::Between(50, 50)).holds(&world, hest));
    world
        .apply(Command::SetHostility {
            from: FactionId::Ecumenci,
            toward: FactionId::Player,
            index: 20,
        })
        .unwrap();
    assert!(cond(Compare::AtMost(20)).holds(&world, hest));
    assert!(!cond(Compare::AtLeast(21)).holds(&world, hest));
}

#[test]
fn npc_has_holds_once_the_npc_carries_the_item() {
    let mut world = first_leg();
    let hest = person(&world, "hest");
    give_hest(&mut world, goods("Seal", 2));
    let has = |count| Condition::NpcHas {
        item: "Seal".into(),
        count,
    };
    assert!(has(2).holds(&world, hest));
    assert!(!has(3).holds(&world, hest));
}

fn goods(name: &str, count: u32) -> Item {
    Item {
        name: name.into(),
        item_type: ItemType::Goods,
        weight: 1,
        value: 1,
        count,
        durability: 1,
        rarity: Rarity::Basic,
        slot: None,
        effects: vec![],
    }
}

fn give_hest(world: &mut World, item: Item) {
    let hest = person(world, "hest");
    world.people[hest.index()]
        .sheet
        .as_mut()
        .unwrap()
        .carry(item);
}

fn count_of(world: &World, who: PersonId, name: &str) -> u32 {
    world.people[who.index()]
        .sheet
        .as_ref()
        .unwrap()
        .inventory
        .iter()
        .filter(|item| item.name == name)
        .map(|item| item.count)
        .sum()
}

fn player_id(world: &World) -> PersonId {
    PersonId(world.people.iter().position(|p| p.is_player()).unwrap() as u32)
}

fn node(key: &str, line: &str, choices: Vec<Choice>) -> Node {
    Node {
        key: key.into(),
        line: line.into(),
        sprite: None,
        choices,
    }
}

fn choice(text: &str, actions: Vec<Action>, next: Option<&str>) -> Choice {
    Choice {
        text: text.into(),
        when: Condition::Always,
        actions,
        next: next.map(String::from),
    }
}

/// A world at Yhlgrym's gate with `tree` on Hest.
fn with_tree(tree: DialogueTree) -> (World, PersonId) {
    let mut world = first_leg();
    at_yhlgrym(&mut world);
    let hest = person(&world, "hest");
    world.people[hest.index()].tree = Some(tree);
    (world, hest)
}

fn entry(when: Condition, node: &str) -> Entry {
    Entry {
        when,
        node: node.into(),
    }
}

/// A tree with one node, "start", whose one choice runs `actions` and leads to `next`.
fn one_choice(actions: Vec<Action>, next: Option<&str>) -> DialogueTree {
    let mut nodes = vec![node(
        "start",
        "start line",
        vec![choice("go", actions, next)],
    )];
    if next == Some("after") {
        nodes.push(node("after", "after line", vec![]));
    }
    DialogueTree {
        entries: vec![entry(Condition::Always, "start")],
        nodes,
    }
}

fn opening(events: &[Event]) -> &str {
    match &events[0] {
        Event::ConversationOpened { opening, .. } => opening,
        other => panic!("expected ConversationOpened, got {other:?}"),
    }
}

#[test]
fn entry_chosen_by_hostility() {
    let tree = DialogueTree {
        entries: vec![
            entry(Condition::Hostility(Compare::AtMost(30)), "cold"),
            entry(Condition::Always, "warm"),
        ],
        nodes: vec![
            node("cold", "cold line", vec![]),
            node("warm", "warm line", vec![]),
        ],
    };
    let (mut world, hest) = with_tree(tree);
    let events = world.apply(Command::Interact { person: hest }).unwrap();
    assert_eq!(opening(&events), "warm line");
    world.apply(Command::EndConversation).unwrap();
    world
        .apply(Command::SetHostility {
            from: FactionId::Ecumenci,
            toward: FactionId::Player,
            index: 20,
        })
        .unwrap();
    let events = world.apply(Command::Interact { person: hest }).unwrap();
    assert_eq!(opening(&events), "cold line");
    assert_eq!(world.npc_line(), Some("cold line"));
}

#[test]
fn no_entry_holds_means_nothing_to_say() {
    let tree = DialogueTree {
        entries: vec![entry(Condition::Flag("never".into()), "a")],
        nodes: vec![node("a", "line", vec![])],
    };
    let (mut world, hest) = with_tree(tree);
    assert_eq!(
        world.apply(Command::Interact { person: hest }),
        Err(CommandError::NothingToSay)
    );
    assert_eq!(world.conversation, None);
    assert_eq!(world.conversation_node, None);
}

#[test]
fn choice_conditions_filter_options() {
    let mut gated = choice("gated", vec![], None);
    gated.when = Condition::Attitude(Compare::AtLeast(90));
    let tree = DialogueTree {
        entries: vec![entry(Condition::Always, "a")],
        nodes: vec![node("a", "hi", vec![choice("open", vec![], None), gated])],
    };
    let (mut world, hest) = with_tree(tree);
    world.apply(Command::Interact { person: hest }).unwrap();
    let options = world.dialogue_options(hest);
    assert_eq!(options.len(), 1);
    assert_eq!(options[0].text, "open");
    assert_eq!(options[0].label, "");
    world.apply(Command::EndConversation).unwrap();
    set_attitude(&mut world, hest, 95);
    world.apply(Command::Interact { person: hest }).unwrap();
    assert_eq!(world.dialogue_options(hest).len(), 2);
}

#[test]
fn say_follows_next_and_replies_with_its_line() {
    let (mut world, hest) = with_tree(one_choice(vec![], Some("after")));
    world.apply(Command::Interact { person: hest }).unwrap();
    assert_eq!(world.npc_line(), Some("start line"));
    let events = world.apply(Command::Say { line: 0 }).unwrap();
    assert!(matches!(
        &events[..],
        [Event::Said { text, reply, .. }] if text == "go" && reply == "after line"
    ));
    assert_eq!(world.npc_line(), Some("after line"));
    assert_eq!(world.conversation, Some(hest));
    assert_eq!(world.conversation_node.as_deref(), Some("after"));
}

#[test]
fn next_none_ends_the_conversation() {
    let (mut world, hest) = with_tree(one_choice(vec![], None));
    world.apply(Command::Interact { person: hest }).unwrap();
    let events = world.apply(Command::Say { line: 0 }).unwrap();
    assert!(matches!(
        &events[..],
        [Event::Said { reply, .. }, Event::ConversationEnded { .. }] if reply.is_empty()
    ));
    assert_eq!(world.conversation, None);
    assert_eq!(world.conversation_node, None);
}

#[test]
fn change_attitude_clamps() {
    let (mut world, hest) = with_tree(one_choice(vec![Action::ChangeAttitude(200)], None));
    world.apply(Command::Interact { person: hest }).unwrap();
    world.apply(Command::Say { line: 0 }).unwrap();
    assert_eq!(world.standing(hest).unwrap().attitude, 100);

    world.people[hest.index()].tree = Some(one_choice(vec![Action::ChangeAttitude(-200)], None));
    world.apply(Command::Interact { person: hest }).unwrap();
    world.apply(Command::Say { line: 0 }).unwrap();
    assert_eq!(world.standing(hest).unwrap().attitude, 0);
}

#[test]
fn change_hostility_moves_the_faction_index() {
    let actions = vec![Action::ChangeHostility {
        from: FactionId::Ecumenci,
        toward: FactionId::Player,
        by: -30,
    }];
    let (mut world, hest) = with_tree(one_choice(actions, None));
    world.apply(Command::Interact { person: hest }).unwrap();
    let events = world.apply(Command::Say { line: 0 }).unwrap();
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::HostilityChanged { index: 20, .. })));
    assert_eq!(
        world.hostility_toward(FactionId::Ecumenci, FactionId::Player),
        20
    );
}

#[test]
fn give_and_take_items_move_stacks() {
    let actions = vec![
        Action::GiveItem {
            item: "Seal".into(),
            count: 1,
        },
        Action::TakeItem {
            item: "Hearth Stone".into(),
            count: 1,
        },
    ];
    let (mut world, hest) = with_tree(one_choice(actions, None));
    give_hest(&mut world, goods("Seal", 2));
    let player = player_id(&world);
    world.apply(Command::Interact { person: hest }).unwrap();
    let events = world.apply(Command::Say { line: 0 }).unwrap();
    assert_eq!(count_of(&world, hest, "Seal"), 1);
    assert_eq!(count_of(&world, player, "Seal"), 1);
    assert_eq!(count_of(&world, hest, "Hearth Stone"), 1);
    assert_eq!(count_of(&world, player, "Hearth Stone"), 0);
    assert!(events.iter().any(|e| matches!(
        e,
        Event::ItemGiven { item, count: 1, .. } if item == "Seal"
    )));
    assert!(events.iter().any(|e| matches!(
        e,
        Event::ItemTaken { item, count: 1, .. } if item == "Hearth Stone"
    )));
}

#[test]
fn supplies_move_by_name_too() {
    let actions = vec![Action::TakeItem {
        item: "Food".into(),
        count: 2,
    }];
    let (mut world, hest) = with_tree(one_choice(actions, None));
    let player = player_id(&world);
    let before = count_of(&world, player, "Food");
    assert!(before >= 2);
    world.apply(Command::Interact { person: hest }).unwrap();
    world.apply(Command::Say { line: 0 }).unwrap();
    assert_eq!(count_of(&world, player, "Food"), before - 2);
    assert_eq!(count_of(&world, hest, "Food"), 2);
}

#[test]
fn unpayable_give_hides_the_choice() {
    let actions = vec![Action::GiveItem {
        item: "Seal".into(),
        count: 5,
    }];
    let (mut world, hest) = with_tree(one_choice(actions, None));
    give_hest(&mut world, goods("Seal", 2));
    world.apply(Command::Interact { person: hest }).unwrap();
    assert!(world.dialogue_options(hest).is_empty());
    assert_eq!(
        world.apply(Command::Say { line: 0 }),
        Err(CommandError::UnknownLine(0))
    );
}

#[test]
fn actions_are_all_or_nothing() {
    let actions = vec![
        Action::SetFlag("a".into()),
        Action::ChangeHostility {
            from: FactionId::Player,
            toward: FactionId::Player,
            by: -10,
        },
    ];
    let (mut world, hest) = with_tree(one_choice(actions, Some("after")));
    world.apply(Command::Interact { person: hest }).unwrap();
    let result = world.apply(Command::Say { line: 0 });
    assert!(
        matches!(result, Err(CommandError::ActionFailed(_))),
        "{result:?}"
    );
    assert!(!world.flags.contains("a"));
    assert_eq!(world.conversation_node.as_deref(), Some("start"));
}

#[test]
fn recruit_and_start_combat() {
    let (mut world, hest) = with_tree(one_choice(vec![Action::Recruit], None));
    world.apply(Command::Interact { person: hest }).unwrap();
    let events = world.apply(Command::Say { line: 0 }).unwrap();
    assert!(events.iter().any(|e| matches!(e, Event::Recruited { .. })));
    let person = &world.people[hest.index()];
    assert!(person.in_party);
    assert!(person.is_follower());
    assert!(person.sheet.as_ref().unwrap().belongs_to(FactionId::Player));

    let (mut world, hest) = with_tree(one_choice(vec![Action::StartCombat], Some("after")));
    world.apply(Command::Interact { person: hest }).unwrap();
    let events = world.apply(Command::Say { line: 0 }).unwrap();
    let kinds: Vec<_> = events
        .iter()
        .filter(|e| {
            matches!(
                e,
                Event::CombatStarted { .. } | Event::ConversationEnded { .. }
            )
        })
        .collect();
    assert!(matches!(
        kinds[..],
        [Event::CombatStarted { .. }, Event::ConversationEnded { .. }]
    ));
    assert_eq!(world.conversation, None);
    assert_eq!(world.conversation_node, None);
}

#[test]
fn grant_and_journal_and_flags() {
    let actions = vec![
        Action::GrantItem(goods("Token", 3)),
        Action::WriteJournal("Met Hest.".into()),
        Action::SetFlag("met".into()),
        Action::SetFlag("temp".into()),
        Action::ClearFlag("temp".into()),
        Action::SetRecruitable(true),
    ];
    let (mut world, hest) = with_tree(one_choice(actions, None));
    let player = player_id(&world);
    world.apply(Command::Interact { person: hest }).unwrap();
    let events = world.apply(Command::Say { line: 0 }).unwrap();
    assert_eq!(count_of(&world, player, "Token"), 3);
    assert!(world.journal.iter().any(|entry| entry.text == "Met Hest."));
    assert!(world.flags.contains("met"));
    assert!(!world.flags.contains("temp"));
    assert!(world.people[hest.index()].recruitable);
    assert!(events.iter().any(|e| matches!(
        e,
        Event::ItemGranted { item, count: 3 } if item == "Token"
    )));
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::FlagSet { flag } if flag == "met")));
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::FlagCleared { flag } if flag == "temp")));
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::JournalWritten { .. })));
    assert!(events.iter().any(|e| matches!(
        e,
        Event::RecruitableChanged {
            recruitable: true,
            ..
        }
    )));
}

#[test]
fn flat_line_npc_still_talks() {
    let mut world = first_leg();
    at_yhlgrym(&mut world);
    let syla = person(&world, "syla");
    let site = world.people[syla.index()].site.expect("at a site");
    world.apply(Command::Walk { to: site }).unwrap();
    let events = world.apply(Command::Interact { person: syla }).unwrap();
    assert_eq!(opening(&events), "");
    assert_eq!(world.conversation_node, None);
    assert_eq!(world.npc_line(), None);
    let options = world.dialogue_options(syla);
    assert!(!options.is_empty());
    assert!(options.iter().any(|o| o.label == "average friendliness"));
}
