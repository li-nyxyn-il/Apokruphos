# varif-game

The world model for the Apokruphos game: the state and rules of Varif as a plain Rust library, with a command-line program to drive it. Nothing here depends on a game engine.

## Run it

Install Rust from <https://rustup.rs>, then from this folder:

```
cargo test     # run the tests
cargo gui      # open the window
cargo run      # walk the road at the command line
```

The window opens by asking for a new game or a saved one. A new game is a character: name, age, blood, heritage, akstada, tongues, background and faction. Health, fatigue, resilience and level stay at their base, and the inventory starts empty. After that, click a place on the map or a road button to travel, a site to walk to it, and answer the question under the map when one appears. The first `cargo gui` takes a few minutes while the window library compiles.

At the command line, type `help` for the list of commands.

## Play it in a browser

The window also runs in a web browser, phones included. Every push to `main` that touches this folder builds it and publishes it to <https://li-nyxyn-il.github.io/Apokruphos/> (the `Web build` workflow in `.github/workflows/web.yml`).

In the browser the data file and map layout are built in, and **Save** and **Load** use the browser's own storage under the name in the save box. A save made on the phone stays on the phone.

To try the web build on this computer first, install the web target and Trunk once:

```
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
```

then from `crates/varif-gui`:

```
trunk serve --address 0.0.0.0    # http://localhost:8080, or this computer's address from a phone on the same Wi-Fi
```

## Layout

| Path | What it holds |
|---|---|
| `crates/varif-core` | The library: types, rules, commands and events |
| `crates/varif-cli` | The command-line front end (the `varif` program) |
| `crates/varif-gui` | The window, built on egui (the `varif-gui` program) |
| `crates/varif-bevy` | A Bevy front end (the `varif-bevy` program); run with `cargo bevy` |
| `crates/varif-gui/index.html` | The page the web build runs in |
| `crates/varif-bevy/src/editor/` | The NPC editor, run with `cargo bevy -- --edit` |
| `data/first_leg.ron` | The first leg of the Pylgrymage, Yhlgrym to Atzhaim, as data |
| `data/people/` | One file per person made or edited in the NPC editor, `<key>.ron` |
| `data/map_layout.ron` | Where each place is drawn on the window's map, in pixels of the Umbara map |
| `data/umbara_roads_only.png` | The Umbara roads-only map, built into the window as the map's background |
| `assets/sprites/` | The images an NPC can be shown with, picked in the editor's Sprites tab |

Inside `varif-core/src`:

| File | What it holds |
|---|---|
| `world.rs` | `World` and its tables: places, roads, people, journal |
| `command.rs` | `Command`, `Event` and the rules in `World::apply` |
| `data.rs` | Loading a world from a data file |
| `calendar.rs` | `Date`, with eight months and five weekdays |
| `character.rs` | The shared sheet: role, factions, attitude, blood, heritage, akstada, tongues, vitals, inventory |
| `faction.rs` | Ecumenci, Bandit, the Audsen Line and Player, with hostility and who the player can join |
| `rng.rs` | The seeded random number generator |
| `sky.rs` | The night sky: four Akstada and ten minor constellations a night, turned at noon |
| `ids.rs` | `PlaceId` and `PersonId` |
| `dialogue.rs` | Dialogue trees: entries, nodes and choices, with their `Condition`s and `Action`s |
| `sandbox.rs` | A world to try one person's conversation in, for the editor's test pane |

## Editing NPCs

The Bevy front end has an editor for people and their conversations:

```
cargo bevy -- --edit                    # edit the people in data/first_leg.ron
cargo bevy -- --edit path/to/data.ron   # or another data file
```

Plain `cargo bevy` is still the game. The editor lists everyone on the left, with New, Duplicate and Delete, and the selected person's problems under the list. Its tabs are Sheet, Inventory, Location (with the schedule), Factions, Sprites and Dialogue. Save is refused while the person has an error.

**Where files go.** Save writes one file per person, `data/people/<key>.ron`, beside the data file. The editor never writes `data/first_leg.ron`, so its comments stay. A people file replaces the person with the same key in `first_leg.ron`: to change someone who is already there, such as Jonnoj, open them, edit, and save. The editor then says the copy in `first_leg.ron` is shadowed, and you can delete it by hand. Two files with the same key, or a file whose name is not its key, is a load error. The command-line program, the window and the Bevy game all read `data/people/`, and the web build embeds it.

**Sprites.** Put images in `assets/sprites/`. In the Sprites tab, give each sprite a name ("portrait", "token", "hostile"), pick its file, and set a frame grid if the image is a sheet. A dialogue node can name one of these to be shown while it is on screen. The game does not draw them yet.

**Dialogue.** A tree has entries, which pick the node a conversation opens at (the first whose condition holds), and nodes, each with the NPC's line and the player's choices. A choice has a condition, a list of actions, and a next node, or none to end the conversation. A node nothing leads to is a warning.

Conditions: Always, All, Any, Not, Attitude, Hostility, Friendliness, Bucket, FactionHostility, IsFollower, InParty, Recruitable, PlayerHas, NpcHas, Flag, HourBetween and PartyAtSite.

Actions: ChangeAttitude, SetAttitude, ChangeHostility, GiveItem, TakeItem, GrantItem, Recruit, SetRecruitable, SetFlag, ClearFlag, WriteJournal and StartCombat (a placeholder until combat exists).

**Test pane.** On the Dialogue tab, the panel on the right runs the person's conversation in a scratch world. Set their attitude, their faction's hostility, the hour, whether they follow the party and which flags are set (separated by commas or new lines), then press Start. Their line shows with the player's choices as buttons, and a log below lists what each choice did; an error, such as them having nothing to say, shows in red. Restart begins again. Edits to the tree do not carry over by themselves: the pane says "Restart to pick up edits". Testing never changes the people in the editor or any file.

## How it works

- **One struct holds everything.** `World` is the whole state, and saving is writing it out.
- **Things refer to each other by ID.** A person stores a `PlaceId`, not a reference to a place.
- **Change goes through commands.** A front end calls `world.apply(command)` and gets back a list of events, or an error that changed nothing.
- **It is deterministic.** The random number generator is part of the world, so the same seed and commands always give the same history.
- **The calendar.** Eight months of 50 days, so a year is 400 days. A day is 24 hours of 60 minutes. The date, hour, and minute are one clock on the world. Real time moves when the host passes elapsed minutes to `World::advance`; turn-based time ignores that and moves only when a command spends it, or on an explicit step. A time scale from 1 to 60 is how many game minutes pass in one real minute. It does not shorten camp, sleep, travel, wait, or step. The week is five days, Arden through Seldyn.

## Placeholders to replace

These are marked `ASSUMPTION` in the code and data:

- **Travel speed:** one league an hour. An order of eight hours or fewer adds no fatigue. Each hour past that costs the next number, summed once for the order: 1 extra costs 1, 2 extra cost 3, 3 extra cost 6, so `n` extra hours cost `n * (n + 1) / 2`. Pitching camp takes an hour, sleeping in camp takes eight, a night at an inn or a residence takes eight, packing up takes an hour, and a wait is from 1 to 24 hours. Camp, rest, sleep, pack, travel, and wait all call `World::advance` and do not bump the date or the hour themselves. A wait or a travel that would pass a road encounter stops there. An NPC's whereabouts are a repeating cycle of spans on that person, looked up from the absolute hour, and a lookup does not tick every NPC.
- **Distances and encounter chances:** every road in `data/first_leg.ron`.
- **Start date:** 1 Jhuze 192 zdy.
- **The cast:** one stand-in person, "The Pylgrym".
- **The player's sheet:** the player, primary faction Player, age 20, red blood, Shentzi, born under the Pylgrym, speaking Shentz and Maphlian. Health, fatigue and resilience are 100, level is 1, and the inventory is empty. All of that is in `data/first_leg.ron`.
- **Starting hostility:** 100 is least hostile and 0 is most hostile. Each faction starts at 100 toward itself. The other numbers a new world copies into its own table are in `faction.rs`. Ecumenci starts at 50 toward the player and the Audsen Line and 0 toward bandits. The Audsen Line starts at 70 toward the player, 50 toward the Ecumenci and 0 toward bandits. Bandits start at 0 toward the other three. The player starts at 50 toward the other three. An NPC's attitude starts at 50. Both can change, and a save keeps the change. A faction's index toward itself stays at 100.

## Adding to it

A new rule usually means four small changes: a `Command` variant, the `Event`s it produces, a function on `World` that `apply` calls, and a test in `crates/varif-core/tests`.
