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
pub mod npc;
pub mod rng;
pub mod schedule;
pub mod world;

pub use calendar::{Date, DAYS_PER_WEEK, HOURS_PER_DAY};
pub use character::{
    Akstada, Blood, Character, Effect, EffectKind, EffectPayload, EquipSlot, Equipped, Heritage,
    Item, ItemType, Rarity, Role, SheetError, Tongue, BASE_ATTITUDE, BASE_FATIGUE, BASE_HEALTH,
    BASE_LEVEL, BASE_RESILIENCE, MAX_AGE, MAX_ATTITUDE, MIN_AGE, MIN_ATTITUDE, SUPPLY_DURABILITY,
    SUPPLY_VALUE, SUPPLY_WEIGHT,
};
pub use command::{
    travel_fatigue, Command, CommandError, Event, MemberFatigue, MAKE_CAMP_HOURS, PACK_UP_HOURS,
    REST_HOURS, UNTIRED_HOURS,
};
pub use data::{load_world, DataError};
pub use faction::{FactionId, Hostility, MAX_HOSTILITY, MIN_HOSTILITY};
pub use ids::{PersonId, PlaceId, SiteId};
pub use npc::{friendliness, Band, Bucket, DialogueLine, Standing, MIN_FRIENDLINESS_TO_TALK};
pub use rng::Rng;
pub use schedule::{RoadLeg, Schedule, Span, Whereabouts};
pub use world::{
    Itinerary, JournalEntry, Journey, Person, Place, PlaceKind, Region, Road, Route, Site,
    SiteKind, Temple, World,
};
