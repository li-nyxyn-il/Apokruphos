# NPC editor and scripted dialogue trees — design

Date: 2026-11-27. Status: awaiting the author's approval.

## Intent

Give the author a tool inside the Bevy front end for building the cast of
Varif without hand-writing RON: every part of an NPC (sheet, pack, worn
items, starting place and site, schedule, factions, sprite) and their
dialogue. Dialogue becomes a **tree that scripts events**, not branching
text: a choice can be gated on the NPC's standing toward the player and on
world state, and picking it can change that state (attitude, hostility,
items, recruitment, flags, combat).

## Goals

- An **Editor mode** in `varif-bevy`, built with `bevy_egui`, that loads a
  data file, lets the author create, duplicate, edit and delete people, and
  saves them in a form `load_world` reads back unchanged.
- **Dialogue trees** in `varif-core`, with a closed list of conditions and
  actions (approach A below), run by the existing `Interact` / `Say` /
  `EndConversation` commands.
- **Conditional dialogue** based on attitude, the faction hostility index,
  friendliness, follower status, items, flags and the other things listed
  under Conditions.
- **Sprites** assigned per person, with a live preview in the editor.
- A **test-conversation pane** in the editor: set attitude, hostility,
  follower and flags by hand and walk the tree as the player would, seeing
  which choices are offered and what each action would do.
- Validation in the editor using the same checks the loader applies, so a
  saved file never fails to load.

## Non-goals

- A visual node graph (boxes and wires). The data is shaped so one can be
  added later over it (approach C).
- Creating new factions. `FactionId` stays the closed list of four.
- Combat. `StartCombat` emits an event and ends the conversation, as the
  existing `TooHostileToTalk` path does; the fight itself is a `COMBAT:`
  placeholder.
- Creating places, sites or roads in the editor. It picks from what the
  data file already has.
- Sprite animation, and drawing NPCs in the world view. The sprite is
  stored and previewed only.
- The editor in the web build.

## Anti-goals (a failure even if it technically works)

- The editor saves something the game then refuses to load.
- Saving silently throws away the hand-written comments and `ASSUMPTION`
  notes in `data/first_leg.ron`.
- Existing saves, the CLI, or the egui window stop working.
- A dialogue action changes the world outside `World::apply`, breaking
  determinism or saves.

## Constraints

- `varif-core` stays engine-free. Trees, conditions, actions and flags live
  there. Bevy and egui stay in `varif-bevy`.
- Every change to the world goes through `World::apply` and comes back as
  events. A choice's actions are applied **all or nothing**: if any one
  would fail, the choice is refused and nothing changes.
- Same seed and commands give the same history.
- New serialized fields use `#[serde(default)]` so older saves and data
  files still load.
- Bevy is 0.20.1. Use the `bevy_egui` release that targets it.

## Approaches

- **A. A closed list of conditions and actions, stored as data.** Chosen.
  It matches the core's existing style (item effects are a closed list, "no
  script"), keeps every change inside `World::apply`, and every option can
  be shown as a dropdown or a number field and checked before saving. The
  cost is that a new kind of event needs a new Rust variant.
- **B. An embedded script language (Rhai or Lua) per choice.** Rejected:
  the editor becomes a text box, errors only appear while playing, and
  scripts could change the world outside the command system.
- **C. A visual node graph over A.** Deferred: a lot of UI, and it can sit
  on top of A's data later.

Editor host: a mode inside `varif-bevy` with `bevy_egui` (chosen by the
author), not a separate program.

## Design

### Core: dialogue trees (`varif-core/src/dialogue.rs`, new)

```rust
pub struct DialogueTree {
    /// Tried in order; the first whose condition holds is where the
    /// conversation opens. This is how an NPC greets a hostile player
    /// differently from a friend.
    pub entries: Vec<Entry>,          // Entry { when: Condition, node: String }
    pub nodes: Vec<Node>,
}
pub struct Node {
    pub key: String,                  // unique within the tree
    pub line: String,                 // what the NPC says on arriving here
    pub choices: Vec<Choice>,
}
pub struct Choice {
    pub text: String,                 // what the player says
    #[serde(default)] pub when: Condition,   // Always by default
    #[serde(default)] pub actions: Vec<Action>,
    /// The node this leads to. None ends the conversation.
    #[serde(default)] pub next: Option<String>,
}
```

`Person` gains `#[serde(default)] tree: Option<DialogueTree>`. A person
with a tree talks through it. A person without one keeps today's flat
bucket lines (theirs plus the world's general `dialogue`), so nothing
already written stops working.

### Conditions

```rust
pub enum Condition {
    Always,
    All(Vec<Condition>), Any(Vec<Condition>), Not(Box<Condition>),
    Attitude(Compare),               // this NPC's attitude
    Hostility(Compare),              // their primary faction toward the player
    Friendliness(Compare),
    Bucket(Bucket),                  // reuse today's bands: Attitude(High) etc.
    FactionHostility { from: FactionId, toward: FactionId, cmp: Compare },
    IsFollower, InParty, Recruitable,
    PlayerHas { item: String, count: u32 },
    NpcHas { item: String, count: u32 },
    Flag(String),                    // set in the world
    HourBetween { from: u32, to: u32 },  // wraps past midnight
    PartyAtSite(String),
}
pub enum Compare { AtLeast(u32), AtMost(u32), Between(u32, u32) }
```

Conditions read the world and never change it.

### Actions

```rust
pub enum Action {
    ChangeAttitude(i32),             // clamped to 0..=100
    SetAttitude(u32),
    ChangeHostility { from: FactionId, toward: FactionId, by: i32 },
    GiveItem { item: String, count: u32 },   // NPC's pack -> player's
    TakeItem { item: String, count: u32 },   // player's pack -> NPC's
    GrantItem(Item),                 // made from nothing, to the player
    Recruit,                         // as Command::Recruit, ignoring `recruitable`
    SetRecruitable(bool),
    SetFlag(String), ClearFlag(String),
    WriteJournal(String),
    StartCombat,                     // event + ends the conversation; COMBAT: placeholder
}
```

A choice whose `GiveItem` / `TakeItem` could not be paid is not offered,
as if its condition were false. Each action reports its effect as an
`Event`, reusing existing events where they fit (`AttitudeChanged`,
`Recruited`, …) and adding `FlagSet`, `ItemGiven`, `ItemTaken`,
`CombatStarted`.

### Conversation state

- `World::conversation` becomes `Option<Conversation { person, node }>`
  (`node` is `None` for a flat-line conversation). Serde-default so old
  saves load.
- `Interact` picks the first entry whose condition holds. If none holds,
  the NPC has nothing to say and the conversation does not open.
- `World::dialogue_options` returns the choices offered at the current
  node, in order, so `Say { line }` keeps meaning "the n-th offered
  option" and the CLI and egui window need only small changes to show the
  NPC's line.
- `Say` applies the choice's actions, then moves to `next` (the event
  carries the new node's line as the reply) or ends the conversation.
- `World` gains `#[serde(default)] flags: BTreeSet<String>`.

### Sprites (a named set, decided by the author)

`Person` gains `#[serde(default)] sprites: Vec<Sprite>`, any number of
named sprites:

```rust
pub struct Sprite {
    pub name: String,                // "portrait", "token", "hostile", "wounded"…; unique per person
    pub path: String,                // relative to varif-game/assets/
    #[serde(default)] pub frame: Option<Frame>,  // for a sheet: columns, rows, index
}
```

`Node` gains `#[serde(default)] sprite: Option<String>`, naming one of the
person's sprites to show while that node is on screen. It is stored and
previewed now. Nothing in the game draws it yet.

Images live in `varif-game/assets/sprites/`. `varif-bevy` sets its
`AssetPlugin` root to that `assets` folder.

### Loader and validation

`data.rs` reads `tree` and `sprites` on each person. On load it refuses:
duplicate node keys, an entry or `next` naming a missing node, a tree with
no entries, `HourBetween` outside 0–23, a `PartyAtSite` that is not a site
of the person's place, an empty flag name, two sprites with the same name,
and a node naming a sprite the person doesn't have.

### People files (decided by the author)

The editor saves each person to their own file, `data/people/<key>.ron`,
holding one person in the same shape as an entry in `people: [...]`. The
editor never writes `data/first_leg.ron`, so its comments stay.

- Core gains `load_world_with_people(text, people: &[(label, text)])`.
  `load_world` stays, with no people files.
- A person file **replaces** the entry with the same key in the main file.
  That is how an existing person such as Jonnoj moves over: open them in
  the editor and save, and the editor says that the copy in
  `first_leg.ron` is now shadowed and can be deleted by hand.
- Two people files with the same key, or a file whose name doesn't match
  its key, is a load error naming the file.
- The native CLI, egui window and Bevy front end all read `data/people/`
  beside the data file. The web build embeds that folder at build time so
  that people made in the editor appear in the browser too. Unreachable nodes are a
warning in the editor, not a load error. The editor calls the same check
functions, exposed from core, before saving.

### Editor mode (`varif-bevy`)

- Started with `cargo bevy -- --edit [data file]`; plain `cargo bevy`
  stays the game.
- **Left panel:** list of people, with New, Duplicate and Delete, and a
  problems list for the selected person.
- **Tabs:**
  - *Sheet:* role (NPC / follower), attitude, age, blood, heritage, akstada,
    tongues, background, health, fatigue, resilience and its max, level.
  - *Inventory:* pack stacks and worn slots, with every `Item` field and
    its effects.
  - *Location & schedule:* starting place and site from the data file's
    lists, `in_party`, `recruitable`, and the list of spans (hours, place,
    site, or road from/to), showing the cycle length in hours and days.
  - *Factions:* primary and secondaries, showing the live hostility of
    each toward the player.
  - *Sprites:* the person's named set. Add or rename a sprite, pick its
    file from `assets/sprites/`, set the frame grid, and preview it. A node's
    sprite dropdown lists these names.
  - *Dialogue:* the entries list, the node list, and the selected node's
    NPC line and choices. Each choice has a nested condition editor (a
    dropdown per variant, with All/Any/Not nesting), an actions list, and
    a `next` dropdown (node keys plus "End"). Plus the **test pane**:
    sliders and toggles for the standing and flags, then walk the tree.
- **Save** writes `data/people/<key>.ron` and is refused while the person
  has errors. Delete removes that file. It never edits `first_leg.ron`.

### Tests (core)

Condition evaluation for each variant and nesting; each action; a choice
refused all or nothing; entries choosing the opening node by hostility; a
flat-line NPC still talks; a tree and a sprite round-trip through save and
load; the loader rejecting each bad-tree case; flags saved with the world.

## Builder's calls (made without the author; check these)

- The flat bucket lines are kept as a fallback rather than converted.
- `Recruit` in a tree ignores `recruitable`: the tree's own conditions
  decide.
- `GiveItem` / `TakeItem` hide a choice that can't be paid rather than
  failing when it is picked.
- `StartCombat` ends the conversation and emits an event until combat
  exists.
- The editor entry point is a `--edit` flag, not a menu.
- The editor refuses to save while there are errors.

- A node can name a sprite now, even though nothing draws it yet.
- A people file overrides a same-key person in the main file instead of
  being a duplicate error.

## Left to the builder

The exact `bevy_egui` version, egui widget layout and styling, event names
and payloads, how the editor's in-memory state maps to `World`, and how the
web build embeds `data/people/` (a `build.rs` or an include macro).
