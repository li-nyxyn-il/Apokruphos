# NPC Editor and Scripted Dialogue Trees Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add scripted dialogue trees, flags, named sprites and per-NPC files to `varif-core`, and an egui-based NPC editor mode to `varif-bevy`.

**Architecture:** Trees, conditions and actions are plain data in a new `varif-core/src/dialogue.rs`. They run through the existing `Interact` / `Say` / `EndConversation` commands, so every change still goes through `World::apply`. The editor works on a public, key-based `PersonFile` (today's private `PersonData`, made public and serializable). It checks the file with the loader's own validation and writes it to `data/people/<key>.ron`.

**Tech Stack:** Rust 2021, serde + ron 0.12, Bevy 0.20.1, bevy_egui 0.43.1 (which targets Bevy 0.20 and egui 0.36).

**Spec:** `docs/superpowers/specs/2026-11-27-npc-editor-design.md`

## Global Constraints

- `varif-core` gains no new dependencies and stays engine-free.
- Every new serialized field is `#[serde(default)]`, so `data/first_leg.ron` and old saves load unchanged.
- A choice's actions apply all or nothing: if any one fails, the world is unchanged and `Say` returns an error.
- Determinism: no wall-clock time, no `HashMap` iteration order in rules. Use `BTreeSet` / `BTreeMap`.
- The editor never writes `data/first_leg.ron`.
- Sprite paths are relative to `varif-game/assets/`. Images live in `varif-game/assets/sprites/`.
- The editor starts with `cargo bevy -- --edit [data file]`. Plain `cargo bevy` is unchanged.
- Keep doc comments in the house style: plain sentences, `ASSUMPTION:` and `COMBAT:` markers where a placeholder stands in.

## Review Focus

1. **A choice whose actions partly fail** (for example `GiveItem` followed by a `ChangeHostility` on the Player faction toward itself). Nothing should change. The test is in Task 2.
2. **An NPC with a tree but no entry whose condition holds.** `Interact` should open no conversation and report that the NPC has nothing to say, not panic or open an empty node. The test is in Task 2.
3. **Attitude clamping** with `ChangeAttitude(-200)` or `ChangeAttitude(+200)`. The result should be 0 or 100, never a wrapped `u32`. The test is in Task 2.
4. **`HourBetween { from: 22, to: 3 }`.** This wraps past midnight: hour 23 and hour 1 hold, hour 12 does not. The test is in Task 1.
5. **Loading a save made before this change** (no `flags`, no `conversation_node`, no `tree` or `sprites`). It should load, and a flat-line NPC should still talk. The test is in Task 3.

---

### Task 1: Dialogue types, conditions and world flags

**Files:**
- Create: `crates/varif-core/src/dialogue.rs`
- Modify: `crates/varif-core/src/lib.rs` (module + re-exports)
- Modify: `crates/varif-core/src/world.rs` (`World.flags`, `Person.tree`, `Person.sprites`)
- Test: `crates/varif-core/tests/dialogue.rs`

**Interfaces:**
- Produces (all `Clone, Debug, PartialEq, Serialize, Deserialize`, plus `Eq` where possible):
  - `DialogueTree { entries: Vec<Entry>, nodes: Vec<Node> }`, with `fn node(&self, key: &str) -> Option<&Node>`
  - `Entry { when: Condition, node: String }`
  - `Node { key: String, line: String, #[serde(default)] sprite: Option<String>, #[serde(default)] choices: Vec<Choice> }`
  - `Choice { text: String, #[serde(default)] when: Condition, #[serde(default)] actions: Vec<Action>, #[serde(default)] next: Option<String> }`
  - `Compare { AtLeast(u32), AtMost(u32), Between(u32, u32) }`, with `fn holds(self, value: u32) -> bool` (`Between` is inclusive)
  - `Condition`: variants exactly as in the spec. `#[derive(Default)]` gives `Always`.
  - `Action`: variants exactly as in the spec (`GrantItem(Item)` reuses `character::Item`).
  - `Sprite { name: String, path: String, #[serde(default)] frame: Option<Frame> }`, and `Frame { columns: u32, rows: u32, index: u32 }`
  - `impl Condition { pub fn holds(&self, world: &World, npc: PersonId) -> bool }`
  - `World.flags: BTreeSet<String>` (`#[serde(default)]`)
  - `Person.tree: Option<DialogueTree>` and `Person.sprites: Vec<Sprite>` (both `#[serde(default)]`)

- [ ] **Step 1: Write failing tests** in `tests/dialogue.rs`, using the `first_leg()` / `person()` helpers copied from `tests/npc.rs`. Set Hest's attitude with `Command::SetAttitude`.
  - `compare_between_is_inclusive`: `Between(30,70)` holds for 30 and 70, not for 29 or 71.
  - `attitude_condition_reads_the_npc`: Hest at 60. `Attitude(AtLeast(60))` holds; `Attitude(AtLeast(61))` does not.
  - `hostility_condition_reads_primary_faction`: Hest is Ecumenci, and Ecumenci toward Player starts at 50. `Hostility(AtMost(50))` holds.
  - `friendliness_and_bucket`: attitude 50 and hostility 50 give friendliness 50. `Friendliness(Between(50,50))` and `Bucket(Friendliness(Average))` hold.
  - `all_any_not_nest`: `All([Always, Not(Box(Flag("x")))])` holds; after inserting `"x"` into `world.flags`, it does not. `Any([])` is false and `All([])` is true.
  - `items_and_flags`: `PlayerHas{item:"Hearth Stone",count:1}` holds and `count:2` does not. `NpcHas` is false for Hest.
  - `hour_between_wraps`: set `world.hour`. `HourBetween{from:22,to:3}` holds at 23 and 1, not at 12. `{from:9,to:17}` holds at 9 and 17.
  - `follower_party_recruitable`: Donold satisfies `IsFollower` and `InParty`. Jonnoj satisfies `Recruitable` and not `IsFollower`.
  - `party_at_site`: with `world.party_site` at Hest's gate site, `PartyAtSite("gate")` holds and `PartyAtSite("market")` does not.
- [ ] **Step 2:** Run `cargo test -p varif-core --test dialogue`. Expected: it fails to compile (no `dialogue` module).
- [ ] **Step 3: Implement the types and `Condition::holds`.**
  - `Attitude`, `Hostility`, `Friendliness` and `Bucket` read `world.standing(npc)`. With no standing they are false.
  - `HourBetween` holds when `from <= to ? (from..=to) : (hour >= from || hour <= to)`.
  - `PartyAtSite` compares the site key of `world.party_site`.
  - Re-export every new type from `lib.rs`.
- [ ] **Step 4:** Run `cargo test -p varif-core`. Expected: all tests pass, including the existing ones.
- [ ] **Step 5:** Commit: `feat(core): dialogue tree types, conditions and world flags`.

### Task 2: Running trees through the conversation commands

**Files:**
- Modify: `crates/varif-core/src/command.rs` (`interact`, `say`, new `apply_actions`, new events and errors)
- Modify: `crates/varif-core/src/world.rs` (`conversation_node`, `dialogue_options`, `npc_line`)
- Modify: `crates/varif-cli/src/main.rs`, `crates/varif-gui/src/main.rs` (adapt to the new option type and events)
- Test: `crates/varif-core/tests/dialogue.rs`

**Interfaces:**
- Consumes: everything from Task 1.
- Produces:
  - `World.conversation_node: Option<String>` (`#[serde(default)]`). `World.conversation: Option<PersonId>` keeps its type, so the hosts' `world.conversation == Some(p)` checks still work. This is the spec's `Conversation { person, node }` split into two fields.
  - `pub struct DialogueOption { pub text: String, pub label: String }`. `label` is the bucket name for a flat line and empty for a tree choice.
  - `World::dialogue_options(&self, id: PersonId) -> Vec<DialogueOption>`. For a tree, these are the current node's choices whose `when` holds and whose `GiveItem`/`TakeItem` can be paid, in order. Otherwise, today's bucket lines.
  - `World::npc_line(&self) -> Option<&str>`: the current node's line.
  - `Event::ConversationOpened { person, standing, opening: String }`. `opening` is the entry node's line, or empty for flat lines.
  - New events: `FlagSet { flag: String }`, `FlagCleared { flag: String }`, `ItemGiven { person: PersonId, item: String, count: u32 }` (NPC to player), `ItemTaken { person: PersonId, item: String, count: u32 }` (player to NPC), `ItemGranted { item: String, count: u32 }`, `CombatStarted { person: PersonId }`, `RecruitableChanged { person: PersonId, recruitable: bool }`.
  - New errors: `CommandError::NothingToSay`, `CommandError::ActionFailed(String)`.

- [ ] **Step 1: Write failing tests.** Build each tree in Rust and set it on `world.people[hest].tree` directly. Use `at_yhlgrym` + walk to the gate (copy them from `tests/npc.rs`).
  - `entry_chosen_by_hostility`: entries `[(Hostility(AtMost(30)) → "cold"), (Always → "warm")]`. Opening at hostility 50 gives `opening == "warm line"`. After `SetHostility` to 20, it gives `"cold line"`.
  - `no_entry_holds_means_nothing_to_say`: the only entry is `Flag("never")`. `Interact` returns `Err(NothingToSay)` and `world.conversation` is `None`.
  - `choice_conditions_filter_options`: two choices, the second gated on `Attitude(AtLeast(90))`. One option is offered at attitude 50, two at 95.
  - `say_follows_next_and_replies_with_its_line`: `Say{line:0}` gives an `Event::Said` whose `reply` is the next node's line, and `npc_line()` is now that node's line.
  - `next_none_ends_the_conversation`: the events end with `ConversationEnded`, and `conversation` and `conversation_node` are both `None`.
  - `change_attitude_clamps`: `ChangeAttitude(200)` gives 100, and `ChangeAttitude(-200)` from there gives 0.
  - `give_and_take_items_move_stacks`: Hest carries 2 "Seal". `GiveItem{"Seal",1}` leaves Hest 1 and the player 1. `TakeItem{"Hearth Stone",1}` moves it to Hest.
  - `unpayable_give_hides_the_choice`: `GiveItem{"Seal",5}` while Hest has 2 is not offered.
  - `actions_are_all_or_nothing`: `[SetFlag("a"), ChangeHostility{from:Player,toward:Player,by:-10}]`. `Say` returns `Err(ActionFailed(_))`, `"a"` is not set, and the conversation stays on the same node.
  - `recruit_and_start_combat`: `Recruit` on Hest puts him in the party as a follower with the Player faction. `StartCombat` emits `CombatStarted` and then `ConversationEnded`.
  - `grant_and_journal_and_flags`: `GrantItem`, `WriteJournal`, `SetFlag` and `ClearFlag` emit their events and change the world.
  - `flat_line_npc_still_talks`: Syla (no tree) still gets the bucket lines, with labels such as `"average friendliness"`.
- [ ] **Step 2:** Run `cargo test -p varif-core --test dialogue`. Expected: FAIL.
- [ ] **Step 3: Implement.**
  - `apply_actions(&mut self, npc, &[Action]) -> Result<Vec<Event>, CommandError>` works on a `self.clone()`. It applies each action in order, reusing the existing `set_attitude`, `recruit` (with `recruitable` forced to true first), `set_hostility` and journal code paths. Only when every action succeeds does it assign the clone back to `*self`. Clone-and-swap is what makes "all or nothing" hold; the world is small enough that cloning is fine. Recruiting from a tree ignores the "same place" check, because the NPC is by definition nearby.
  - `interact` picks the entry, sets `conversation_node`, and fills `opening`.
  - `say` resolves the option index through `dialogue_options`, runs the actions, then moves to `next` or ends the conversation. `StartCombat` always ends it.
- [ ] **Step 4: Adapt the hosts.**
  - CLI `show_options`: print `line.label` in brackets only when it is non-empty, and print `npc_line()` above the options.
  - GUI conversation panel: show `npc_line()` above the buttons, and use `label` for the hover text.
  - Both: add a `..` to their `ConversationOpened` match arms and print `opening` when it is non-empty. Add display arms for the new events.
- [ ] **Step 5:** Run `cargo test` and `cargo build -p varif-gui`. Expected: everything passes and builds.
- [ ] **Step 6:** Commit: `feat(core): scripted dialogue trees run through Interact and Say`.

### Task 3: Loader support and validation for trees and sprites

**Files:**
- Modify: `crates/varif-core/src/data.rs`
- Test: `crates/varif-core/tests/dialogue_data.rs`

**Interfaces:**
- Produces:
  - `pub struct PersonFile`: today's `PersonData`, made `pub`, `Serialize + Deserialize + Clone + Default`, with new fields `#[serde(default)] tree: Option<DialogueTree>` and `#[serde(default)] sprites: Vec<Sprite>`. `ScheduleData` and `SpanData` become `pub` and serializable under the same names. These are the editor's working types.
  - New `DataError` variants:
    - `BadTree { person: String, problem: TreeProblem }`, where `TreeProblem` is `NoEntries`, `DuplicateNode(String)`, `MissingNode(String)`, `BadHour(u32)`, `UnknownSite(String)`, `BlankFlag` or `UnknownSprite(String)`
    - `DuplicateSprite { person: String, sprite: String }`
  - `pub fn check_tree(tree: &DialogueTree, sprites: &[Sprite], place_sites: &[String]) -> Result<(), TreeProblem>`. It walks every `Condition` and `Action` recursively.
  - `pub fn unreachable_nodes(tree: &DialogueTree) -> Vec<String>`. Nodes not reachable from any entry, for the editor's warning.

- [ ] **Step 1: Write failing tests.** Build data strings by appending a person to a copy of `first_leg.ron`, or use a tiny inline world.
  - One test per `TreeProblem` variant, asserting the exact `DataError::BadTree { person: "hest", problem: … }`.
  - `duplicate_sprite_name_is_refused`.
  - `tree_and_sprites_round_trip_through_save`: load, `World::save` (or the existing write-out function), reload, and compare `people[hest].tree`, `.sprites` and `world.flags` (with one flag set before saving).
  - `unreachable_nodes_are_reported`: a node no entry or `next` reaches is in the result.
  - `old_save_without_new_fields_loads`: deserialize a saved world string with no `flags`, `conversation_node`, `tree` or `sprites`. It loads, and Syla still talks.
- [ ] **Step 2:** Run `cargo test -p varif-core --test dialogue_data`. Expected: FAIL.
- [ ] **Step 3:** Implement: convert `PersonFile` into `Person`, copying `tree` and `sprites`, and call `check_tree` with the person's place's site keys.
- [ ] **Step 4:** Run `cargo test -p varif-core`. Expected: PASS.
- [ ] **Step 5:** Commit: `feat(core): load and validate dialogue trees and sprites`.

### Task 4: One file per person, and the editor's core helpers

**Files:**
- Modify: `crates/varif-core/src/data.rs`
- Create: `crates/varif-core/src/sandbox.rs`
- Modify: `crates/varif-cli/src/main.rs`, `crates/varif-gui/src/main.rs`, `crates/varif-bevy/src/main.rs` (read `data/people/`)
- Create: `crates/varif-gui/build.rs` (embed `data/people/*.ron` for the web build)
- Create: `data/people/.gitkeep`
- Test: `crates/varif-core/tests/people_files.rs`

**Interfaces:**
- Produces:
  - `pub fn load_world_with_people(text: &str, people: &[(String, String)]) -> Result<World, DataError>`. Each pair is (file name without `.ron`, file text). `load_world(text)` becomes `load_world_with_people(text, &[])`.
  - New `DataError::PeopleFile { file: String, error: Box<DataError> }` and `DataError::FileKeyMismatch { file: String, key: String }`.
  - A people file replaces a same-key entry from the main file, in place, keeping its index. A new key is appended. Two files with one key is `DuplicatePerson`.
  - `pub fn read_people_dir(dir: &Path) -> std::io::Result<Vec<(String, String)>>`, sorted by file name. A missing directory gives an empty list. `std` only.
  - `pub fn person_file_text(file: &PersonFile) -> String`. Pretty RON (`ron::ser::PrettyConfig::default()`) of one person.
  - `pub fn person_file_of(world: &World, id: PersonId) -> PersonFile`. Turns IDs back into keys, so the editor can open people from `first_leg.ron`.
  - `pub fn check_person_file(main: &str, people: &[(String, String)], file: &PersonFile) -> Result<(), DataError>`. It loads with this file substituted. This is the editor's save gate.
  - `sandbox.rs`: `pub struct SandboxSetup { pub attitude: u32, pub hostility: u32, pub follower: bool, pub flags: BTreeSet<String>, pub hour: u32 }`, and `pub fn sandbox(main: &str, people: &[(String, String)], file: &PersonFile, setup: &SandboxSetup) -> Result<(World, PersonId), DataError>`. The returned world has the party standing at the person's place and site, the NPC's attitude and follower status set, their primary faction's hostility toward Player set, the flags and hour set, and the clock in turn-based mode.

- [ ] **Step 1: Write failing tests.**
  - `people_file_overrides_main_entry_in_place`: a "hest" file with name "Hest the Elder" keeps Hest's index and the new name.
  - `people_file_adds_new_person`.
  - `file_name_must_match_key`: gives `FileKeyMismatch`.
  - `bad_people_file_names_the_file`: gives `PeopleFile { file: "hest", .. }`.
  - `person_file_round_trips`: `person_file_of` → `person_file_text` → load as a people file gives an equal `Person`, for every person in `first_leg.ron`.
  - `sandbox_lets_the_tree_be_walked`: with attitude 95 a gated choice is offered, `Interact` then `Say` both succeed, and the real main text is untouched.
  - `read_people_dir_missing_is_empty`.
- [ ] **Step 2:** Run `cargo test -p varif-core --test people_files`. Expected: FAIL.
- [ ] **Step 3:** Implement the core functions above.
- [ ] **Step 4: Wire up the hosts.**
  - CLI, GUI (native) and Bevy call `read_people_dir(<data file's dir>/people)` and `load_world_with_people`.
  - `varif-gui/build.rs` writes `$OUT_DIR/people.rs` containing `pub const PEOPLE: &[(&str, &str)] = &[("key", include_str!("…/data/people/key.ron")), …];`, with `cargo:rerun-if-changed=../../data/people`. The web path passes it in.
- [ ] **Step 5:** Run `cargo test`, `cargo build -p varif-gui`, `cargo build -p varif-bevy`. Expected: PASS. Then `cargo run -- ` still starts the CLI with no `data/people` files.
- [ ] **Step 6:** Commit: `feat(core): per-person data files and editor helpers`.

### Task 5: Editor shell in varif-bevy

**Files:**
- Modify: `Cargo.toml` (workspace dependency `bevy_egui = "0.43.1"`), `crates/varif-bevy/Cargo.toml`
- Modify: `crates/varif-bevy/src/main.rs` (argument parsing, `AssetPlugin { file_path: <workspace>/assets }`, mode switch)
- Create: `crates/varif-bevy/src/editor/mod.rs` (plugin, `EditorState`, layout, people list, save/delete, problems panel)
- Create: `crates/varif-bevy/src/editor/sheet.rs`, `crates/varif-bevy/src/editor/factions.rs`
- Create: `assets/sprites/.gitkeep`

**Interfaces:**
- Consumes: `PersonFile`, `person_file_of`, `person_file_text`, `check_person_file`, `read_people_dir`, `unreachable_nodes` from Tasks 3 and 4.
- Produces:
  - `EditorState` resource: `{ main_text: String, data_dir: PathBuf, people: Vec<PersonFile>, from_file: Vec<bool>, selected: Option<usize>, tab: Tab, dirty: bool, problems: Vec<String>, status: String }`.
  - `enum Tab { Sheet, Inventory, Location, Factions, Sprites, Dialogue }`.
  - Each tab is a `fn ui(ui: &mut egui::Ui, file: &mut PersonFile, ctx: &EditorContext)`, where `EditorContext { places: Vec<(String, String, Vec<(String, String)>)> /* place key, name, sites */, hostility: Hostility }` is built once from the loaded world.

- [ ] **Step 1:** Add the dependency. Parse `--edit` and an optional data-file path. Add `EguiPlugin` and the editor plugin only in edit mode, and set the asset root to the workspace `assets/` folder.
- [ ] **Step 2: Build the layout.**
  - Left side panel: the people list, with an asterisk after people coming from `first_leg.ron` that have not been saved to `data/people/`. Then New (a blank NPC sheet at the first place), Duplicate (key + `"-copy"`), Delete (removes `data/people/<key>.ron` only, after a confirm dialog), and Save.
  - Problems below the list: the result of `check_person_file`, plus "unreachable node" warnings.
  - Central panel: the tab bar.
- [ ] **Step 3:** Save writes `person_file_text` to `data/people/<key>.ron`. It is disabled while `check_person_file` errs. After saving a person that is also in `first_leg.ron`, show: "Saved. The copy of <key> in first_leg.ron is now shadowed and can be deleted by hand."
- [ ] **Step 4: Fill in the Sheet and Factions tabs.**
  - Sheet: role NPC or follower, attitude slider 0–100, age, blood, heritage and akstada combos from their `ALL` arrays, tongues as checkboxes, background, health, fatigue, resilience, resilience max, and level. Plus a "no sheet (cast only)" toggle.
  - Factions: primary combo, secondary checkboxes excluding the primary, and a read-only "hostility toward the player" label for each.
- [ ] **Step 5: Verify.** Run `cargo bevy -- --edit`: the window opens, you can select Jonnoj, change his age, and Save writes `data/people/jonnoj.ron`. Then `cargo test` passes, which proves the file loads. Delete the test file afterwards.
- [ ] **Step 6:** Commit: `feat(bevy): NPC editor shell with sheet and faction tabs`.

### Task 6: Inventory, Location & schedule tabs

**Files:**
- Create: `crates/varif-bevy/src/editor/inventory.rs`, `crates/varif-bevy/src/editor/location.rs`
- Create: `crates/varif-bevy/src/editor/widgets.rs` (shared `enum_combo<T: Copy + PartialEq>(ui, id, &mut T, all: &[T], name: fn(T) -> &str)` and an `item_editor(ui, &mut Item)` used here and in Task 8)

**Interfaces:**
- Consumes: `EditorContext`, `Tab` from Task 5.
- Produces: `widgets::enum_combo` and `widgets::item_editor`.

- [ ] **Step 1: Inventory tab.**
  - Pack: one collapsible row per stack, edited with `item_editor` (name, type, weight, value, count, durability, rarity, slot, and an effects list with kind and payload combos plus an amount). Add and remove buttons.
  - Worn: one row per `EquipSlot::ALL`, either empty or an item whose `slot` is forced to that slot and `count` to 1.
- [ ] **Step 2: Location tab.**
  - Place combo and site combo, the site combo filtered to the chosen place's sites, with "none" when the place has no sites. `in_party` and `recruitable` checkboxes.
  - Schedule: a span list with hours (≥1), place, site, and an optional road from/to. Move up/down and delete buttons. Show the cycle as "N hours (D days H hours)".
- [ ] **Step 3: Verify.** Run `cargo bevy -- --edit`. Give Bren a "Seal" with a Consumed `RestoreHealth(5)` effect and add a 4-hour span, then save. `cargo test` passes and `data/people/bren.ron` holds both. Delete the test file.
- [ ] **Step 4:** Commit: `feat(bevy): inventory and location/schedule editor tabs`.

### Task 7: Sprites tab

**Files:**
- Create: `crates/varif-bevy/src/editor/sprites.rs`

**Interfaces:**
- Consumes: `Sprite`, `Frame` from Task 1.

- [ ] **Step 1: Build the sprite list.** Each sprite has a unique name field (a red label on a duplicate), a path combo listing `*.png` files under `assets/sprites/` (re-scanned with a Refresh button), and an optional frame grid (columns, rows, index < columns × rows).
- [ ] **Step 2: Add the preview.** Load the selected path with `AssetServer::load`, register it with `EguiContexts::add_image`, and draw it with `egui::Image`. When a frame is set, use a UV rect of that cell. Cache handles by path in a `SpritePreviews` resource.
- [ ] **Step 3: Verify.** Copy any PNG from `../varif-art` into `assets/sprites/`, give Jonnoj "portrait" and "token" sprites, and check that the preview shows and the frame cell crops correctly. Save, then confirm `cargo test` passes.
- [ ] **Step 4:** Commit: `feat(bevy): named sprite set editor with preview`.

### Task 8: Dialogue tab (entries, nodes, conditions, actions)

**Files:**
- Create: `crates/varif-bevy/src/editor/dialogue.rs`, `crates/varif-bevy/src/editor/condition.rs`, `crates/varif-bevy/src/editor/action.rs`

**Interfaces:**
- Consumes: `DialogueTree`, `Condition`, `Action`, `widgets::enum_combo`, `widgets::item_editor`, `unreachable_nodes`.
- Produces: `condition::ui(ui, id_salt, &mut Condition, ctx)` (recursive) and `action::ui(ui, id_salt, &mut Action, ctx)`.

- [ ] **Step 1: Basic layout.** With no tree, show a "Create tree" button. It makes one node "start" and an `Always` entry to it.
- [ ] **Step 2: Entries list.** Each entry has a condition editor and a node combo. Add, delete, move up and move down. The order matters, so label it "first that holds wins".
- [ ] **Step 3: Node list and node editor.**
  - Node list on the left, with add, rename (which also rewrites every `next` and entry that points to it) and delete (which warns how many links will become "End").
  - Selected node: a multi-line NPC line, a sprite combo from the person's sprite names, and its choices.
  - Each choice has text, a condition, an actions list, and a `next` combo of node keys plus "End".
- [ ] **Step 4: `condition::ui`.** A variant combo. Switching variant keeps the children where it can. `All`/`Any` show an indented child list with add and remove, and `Not` shows one child. Leaves show their fields: `Compare` as a kind combo plus one or two numbers, factions as combos, the site as a combo of the person's place's sites, and flags and items as text fields.
- [ ] **Step 5: `action::ui`.** A variant combo plus fields. `GrantItem` uses `item_editor`.
- [ ] **Step 6: Verify.** Run `cargo bevy -- --edit` and build a three-node tree on Hest. It should have a hostile-greeting entry, a choice gated on `Attitude(AtLeast(70))` with `ChangeAttitude(5)` and `SetFlag("met_hest")`, and a choice that ends the conversation. Save. `cargo test` passes, and in `cargo gui` talking to Hest at the gate walks the tree. Delete the test file.
- [ ] **Step 7:** Commit: `feat(bevy): dialogue tree editor with condition and action builders`.

### Task 9: Test-conversation pane, README

**Files:**
- Create: `crates/varif-bevy/src/editor/test_pane.rs`
- Modify: `README.md` (an "Editing NPCs" section; add `editor/` and `data/people/` to the layout table)

**Interfaces:**
- Consumes: `sandbox`, `SandboxSetup` from Task 4, and `World::apply`, `dialogue_options`, `npc_line`.

- [ ] **Step 1: Build the pane.** It is a right-side panel on the Dialogue tab. It has sliders for attitude, hostility and hour, a follower checkbox, a flags text list, and a Start button. Start calls `sandbox`, then `Interact`.
- [ ] **Step 2: Show the conversation.**
  - Show `npc_line()`, the offered options as buttons (each calling `Say`), and a running log of the returned events, formatted with `{:?}`.
  - Restart resets the sandbox. Errors such as `NothingToSay` or `ActionFailed` appear in the log in red.
  - Nothing here touches `EditorState.people` or the files on disk.
- [ ] **Step 3: Update the README.** Cover how to start the editor, where files go, how files override `first_leg.ron`, where sprites go, and the list of conditions and actions.
- [ ] **Step 4: Verify.** Walk the Task 8 tree in the pane at attitude 40 and at 80, and check that the gated choice appears only at 80. Then `cargo test`, `cargo clippy --workspace`, `cargo build -p varif-gui` and `cargo build -p varif-bevy` should all be clean.
- [ ] **Step 5:** Commit: `feat(bevy): test-conversation pane and editor docs`.
