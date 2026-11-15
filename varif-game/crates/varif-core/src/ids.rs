//! Typed identifiers.
//!
//! Things in the world refer to each other by ID, never by reference. A person
//! stores the `PlaceId` of where they are, not a pointer to the `Place`. This
//! keeps the borrow checker out of the way, makes saving trivial, and gives a
//! front end plain numbers to pass around.
//!
//! An ID is the index of the thing in its table on [`crate::World`].

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PlaceId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PersonId(pub u32);

/// A location inside a place: the index into that place's `sites`. It only
/// means something together with the place it belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SiteId(pub u32);

impl PlaceId {
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

impl PersonId {
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

impl SiteId {
    pub fn index(self) -> usize {
        self.0 as usize
    }
}
