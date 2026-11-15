//! The state of the world: tables of places, roads and people, plus the party.

use crate::calendar::Date;
use crate::ids::{PersonId, PlaceId};
use crate::rng::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaceKind {
    /// One of the cities of the Great Atzch Road, with a Krypt and a nedia.
    TempleCity,
    Town,
    Landmark,
}

/// The four Pynumbra.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Region {
    Aphlora,
    Maphlia,
    Shentzlia,
    Ptormia,
}

/// What makes a temple city a story hub.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Temple {
    /// The Beyond whose Krypt stands here, for example "Phionyx".
    pub krypt_of: String,
    /// The rite of the Pylgrymage performed here, if the notes name one.
    #[serde(default)]
    pub rite: Option<String>,
    /// What the nedia teaches.
    #[serde(default)]
    pub nedia: Vec<String>,
    /// True while the temple lies in ruins.
    #[serde(default)]
    pub ruined: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Place {
    /// A short lowercase name used in data files and at the command line.
    pub key: String,
    pub name: String,
    pub kind: PlaceKind,
    /// `None` where the region is not decided.
    #[serde(default)]
    pub region: Option<Region>,
    #[serde(default)]
    pub temple: Option<Temple>,
}

/// A road joins two places and can be walked in either direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Road {
    pub a: PlaceId,
    pub b: PlaceId,
    pub leagues: u32,
    /// The chance out of 100, on each day of travel, of an encounter.
    pub encounter_chance: u32,
}

impl Road {
    pub fn joins(&self, x: PlaceId, y: PlaceId) -> bool {
        (self.a == x && self.b == y) || (self.a == y && self.b == x)
    }

    /// The far end of the road as seen from `from`, if the road touches it.
    pub fn other_end(&self, from: PlaceId) -> Option<PlaceId> {
        if self.a == from {
            Some(self.b)
        } else if self.b == from {
            Some(self.a)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Person {
    pub key: String,
    pub name: String,
    pub role: String,
    pub at: PlaceId,
    /// True for the player and the camp followers, who move with the party.
    #[serde(default)]
    pub in_party: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalEntry {
    pub date: Date,
    pub at: PlaceId,
    pub text: String,
}

/// Everything that is true of the world at one moment.
///
/// The fields are public so that a front end can read them freely. Change the
/// world only through [`World::apply`], so that every change produces events.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct World {
    pub date: Date,
    pub rng: Rng,
    pub places: Vec<Place>,
    pub roads: Vec<Road>,
    pub people: Vec<Person>,
    pub party_at: PlaceId,
    pub camped: bool,
    pub journal: Vec<JournalEntry>,
}

impl World {
    pub fn place(&self, id: PlaceId) -> Option<&Place> {
        self.places.get(id.index())
    }

    pub fn person(&self, id: PersonId) -> Option<&Person> {
        self.people.get(id.index())
    }

    /// The name of a place, or a placeholder if the ID is not in the table.
    pub fn place_name(&self, id: PlaceId) -> &str {
        self.place(id).map_or("an unknown place", |p| p.name.as_str())
    }

    /// Finds a place by its key or its name, ignoring case.
    pub fn find_place(&self, text: &str) -> Option<PlaceId> {
        let wanted = text.trim().to_lowercase();
        self.places
            .iter()
            .position(|p| p.key.to_lowercase() == wanted || p.name.to_lowercase() == wanted)
            .map(|i| PlaceId(i as u32))
    }

    pub fn road_between(&self, x: PlaceId, y: PlaceId) -> Option<&Road> {
        self.roads.iter().find(|r| r.joins(x, y))
    }

    /// The places one road away from `from`, each with the road that leads there.
    pub fn neighbours(&self, from: PlaceId) -> Vec<(PlaceId, &Road)> {
        self.roads
            .iter()
            .filter_map(|r| r.other_end(from).map(|to| (to, r)))
            .collect()
    }

    /// The people travelling with the party.
    pub fn party(&self) -> impl Iterator<Item = &Person> {
        self.people.iter().filter(|p| p.in_party)
    }

    /// The people at a place who are not in the party.
    pub fn residents(&self, at: PlaceId) -> impl Iterator<Item = &Person> {
        self.people.iter().filter(move |p| !p.in_party && p.at == at)
    }

    /// Writes the whole world as text, for saving.
    pub fn to_ron(&self) -> Result<String, String> {
        ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .map_err(|e| e.to_string())
    }

    /// Reads a world written by [`World::to_ron`].
    pub fn from_ron(text: &str) -> Result<World, String> {
        ron::from_str(text).map_err(|e| e.to_string())
    }
}
