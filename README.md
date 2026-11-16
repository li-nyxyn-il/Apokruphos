# Apokruphos

A single-player, story-driven game set in the world of Varif. It blends grand strategy and RPG in a pixel-art rendering of the Viper's Neck and the lands beyond.

The game takes place around the time of Donold Audsen the Unifier and is planned in three acts:

1. **The Pylgrymage.** An RPG journey along the road. The camp is the hub for planning, talking with camp followers and keeping the journal. Towns are for trading, upgrades and quests. The road is where you explore and run into encounters, and the city temples are where the story happens.
2. **Intrigue.** Competing for power and influence across the Neck: alliances, courts and politics.
3. **Unification.** A campaign of war to conquer and unite the regions.

Act 1 is being built first, since it brings in most of the world, characters and systems the later acts use.

## What's in this repository

| Folder | What it holds |
|---|---|
| [`varif-game/`](varif-game/) | The game's world model: a Rust workspace with the core library, a command-line front end and an egui window. See its [README](varif-game/README.md). |
| [`varif-master/`](varif-master/) | The lore of Varif as an Obsidian vault: the Ecumency, the Phionyx, peoples, regions, cities, eras, and every character grouped by Line. Start at [`Variff Home.md`](varif-master/Variff%20Home.md). |
| [`varif-art/`](varif-art/) | Visual reference: the style guide and palette (`style/`), plus the whiteboard script alphabet (`language/`). |

### Key lore documents

- [`Worldbuilding Bible.md`](varif-master/Worldbuilding%20Bible.md): the main reference for the world.
- [`A Pylgrym's Plight — Collated Whiteboard Notes.md`](varif-master/A%20Pylgrym's%20Plight%20%E2%80%94%20Collated%20Whiteboard%20Notes.md): story notes for the Pylgrymage.
- [`Dictionary.md`](varif-master/Dictionary.md): Varifian words and their meanings.
- [`eras/`](varif-master/eras/): ancient times, the Yhlgori conquest, the Audsen dynasty, the Pylgrym's Plight and the reunification wars.

## Running the game

You need Rust, which you can install from <https://rustup.rs>. Then:

```sh
cd varif-game
cargo test                # run the tests
cargo run                 # walk the road at the command line (type `help`)
cargo run -p varif-gui    # open the window
```

The window starts by asking for a new game or a saved one. The first build of the window takes a few minutes while egui compiles.

## Design principles of the world model

- **Engine-independent.** `varif-core` is a plain Rust library with no engine in it, so it can sit behind any front end.
- **One struct holds the whole state.** `World` is everything, and saving a game means writing it out (RON format).
- **Change goes through commands.** A front end calls `world.apply(command)` and gets back events, or an error that leaves the world unchanged.
- **Deterministic.** The seeded random number generator lives inside the world, so the same seed and the same commands always produce the same history.

Placeholder values such as month length, travel speed, distances and the starting cast are marked `ASSUMPTION` in the code and data. The [game README](varif-game/README.md#placeholders-to-replace) lists them.

## Browsing the lore

Open the repository root as a vault in [Obsidian](https://obsidian.md). The notes use `[[wikilinks]]`, so links and the graph view work there. GitHub shows the notes as plain Markdown.

## Status

This is early work in progress. The engine for the full game hasn't been chosen yet (Unity, Bevy, or a Rust/Unity hybrid are the options), and for now the Rust world model is where the systems are being built and tested.
