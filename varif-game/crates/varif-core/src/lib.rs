//! The world model of Varif.
//!
//! This crate holds the whole state of the Neck in one [`World`] value. Nothing
//! here knows about rendering, input or any game engine. A front end (the
//! command-line program today, Unity or Bevy later) does three things:
//!
//! 1. loads a world from a data file with [`load_world`],
//! 2. sends it [`Command`]s through [`World::apply`],
//! 3. shows the player the [`Event`]s that come back.

pub mod calendar;
pub mod character;
pub mod command;
pub mod data;
pub mod faction;
pub mod ids;
pub mod rng;
pub mod world;

pub use calendar::Date;
pub use character::{
    Akstada, Blood, Character, Heritage, Item, ItemType, Role, SheetError, Tongue, BASE_ATTITUDE,
    BASE_FATIGUE, BASE_HEALTH, BASE_LEVEL, BASE_RESILIENCE, MAX_AGE, MAX_ATTITUDE, MIN_AGE,
    MIN_ATTITUDE, SUPPLY_DURABILITY, SUPPLY_VALUE, SUPPLY_WEIGHT,
};
pub use command::{Command, CommandError, Event, MemberFatigue};
pub use data::{load_world, DataError};
pub use faction::{FactionId, Hostility, MAX_HOSTILITY, MIN_HOSTILITY};
pub use ids::{PersonId, PlaceId, SiteId};
pub use rng::Rng;
pub use world::{
    Itinerary, JournalEntry, Journey, Person, Place, PlaceKind, Region, Road, Route, Site,
    SiteKind, Temple, World,
};
