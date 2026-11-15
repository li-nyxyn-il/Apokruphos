# varif-game

The world model for the Apokruphos game: the state and rules of Varif as a plain Rust library, with a command-line program to drive it. Nothing here depends on a game engine.

## Run it

Install Rust from <https://rustup.rs>, then from this folder:

```
cargo test     # run the tests
cargo gui      # open the window
cargo run      # walk the road at the command line
```

In the window, click a place on the map or a road button to travel, a site to walk to it, and answer the question under the map when one appears. The first `cargo gui` takes a few minutes while the window library compiles.

At the command line, type `help` for the list of commands.

## Layout

| Path | What it holds |
|---|---|
| `crates/varif-core` | The library: types, rules, commands and events |
| `crates/varif-cli` | The command-line front end (the `varif` program) |
| `crates/varif-gui` | The window, built on egui (the `varif-gui` program) |
| `data/first_leg.ron` | The first leg of the Pylgrymage, Yhlgrym to Atzhaim, as data |
| `data/map_layout.ron` | Where each place is drawn on the window's map |

Inside `varif-core/src`:

| File | What it holds |
|---|---|
| `world.rs` | `World` and its tables: places, roads, people, journal |
| `command.rs` | `Command`, `Event` and the rules in `World::apply` |
| `data.rs` | Loading a world from a data file |
| `calendar.rs` | `Date`, with eight months and five weekdays |
| `rng.rs` | The seeded random number generator |
| `ids.rs` | `PlaceId` and `PersonId` |

## How it works

- **One struct holds everything.** `World` is the whole state, and saving is writing it out.
- **Things refer to each other by ID.** A person stores a `PlaceId`, not a reference to a place.
- **Change goes through commands.** A front end calls `world.apply(command)` and gets back a list of events, or an error that changed nothing.
- **It is deterministic.** The random number generator is part of the world, so the same seed and commands always give the same history.

## Placeholders to replace

These are marked `ASSUMPTION` in the code and data:

- **Month length:** 45 days, in `calendar.rs`.
- **Travel speed:** 6 leagues a day, in `command.rs`.
- **Distances and encounter chances:** every road in `data/first_leg.ron`.
- **Start date:** 1 Jhuze 192 zdy.
- **The cast:** one stand-in person, "The Pylgrym".

## Adding to it

A new rule usually means four small changes: a `Command` variant, the `Event`s it produces, a function on `World` that `apply` calls, and a test in `crates/varif-core/tests`.
