//! Commands go in, events come out. The rules of the world live here.

use crate::calendar::Date;
use crate::ids::PlaceId;
use crate::world::{JournalEntry, World};
use serde::{Deserialize, Serialize};
use std::fmt;

/// ASSUMPTION: how far a party on foot covers in a day. Replace with the real
/// figure, or make it depend on the party and the road.
pub const LEAGUES_PER_DAY: u32 = 6;

/// Something the player asks the world to do.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
    /// Walk the road to a neighbouring place.
    Travel { to: PlaceId },
    /// Make camp where the party stands and pass the night.
    MakeCamp,
    /// Add an entry to the journal.
    WriteJournal { text: String },
}

/// Something that happened in the world as the result of a command.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    Departed { from: PlaceId, to: PlaceId, on: Date },
    DayPassed { date: Date },
    RoadEncounter { from: PlaceId, to: PlaceId, on: Date },
    Arrived { at: PlaceId, on: Date },
    Camped { at: PlaceId, on: Date },
    JournalWritten { at: PlaceId, on: Date },
}

/// Why a command was refused. A refused command changes nothing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommandError {
    UnknownPlace(PlaceId),
    AlreadyThere,
    NoRoad { from: PlaceId, to: PlaceId },
    EmptyJournalEntry,
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandError::UnknownPlace(id) => write!(f, "there is no place with id {}", id.0),
            CommandError::AlreadyThere => write!(f, "the party is already there"),
            CommandError::NoRoad { .. } => write!(f, "no road leads there from here"),
            CommandError::EmptyJournalEntry => write!(f, "a journal entry needs some words"),
        }
    }
}

impl std::error::Error for CommandError {}

impl World {
    /// The single entry point for changing the world.
    ///
    /// Every command is checked in full before anything changes, so an `Err`
    /// always leaves the world exactly as it was.
    pub fn apply(&mut self, command: Command) -> Result<Vec<Event>, CommandError> {
        match command {
            Command::Travel { to } => self.travel(to),
            Command::MakeCamp => Ok(self.make_camp()),
            Command::WriteJournal { text } => self.write_journal(text),
        }
    }

    fn travel(&mut self, to: PlaceId) -> Result<Vec<Event>, CommandError> {
        let from = self.party_at;
        if self.place(to).is_none() {
            return Err(CommandError::UnknownPlace(to));
        }
        if from == to {
            return Err(CommandError::AlreadyThere);
        }
        // `Road` is `Copy`, so this takes a copy and stops borrowing `self`.
        let road = *self
            .road_between(from, to)
            .ok_or(CommandError::NoRoad { from, to })?;

        let days = road.leagues.div_ceil(LEAGUES_PER_DAY).max(1);
        let mut events = vec![Event::Departed {
            from,
            to,
            on: self.date,
        }];
        self.camped = false;

        for _ in 0..days {
            self.date = self.date.next_day();
            events.push(Event::DayPassed { date: self.date });
            if self.rng.chance(road.encounter_chance) {
                events.push(Event::RoadEncounter {
                    from,
                    to,
                    on: self.date,
                });
            }
        }

        self.party_at = to;
        for person in self.people.iter_mut().filter(|p| p.in_party) {
            person.at = to;
        }
        events.push(Event::Arrived {
            at: to,
            on: self.date,
        });
        Ok(events)
    }

    fn make_camp(&mut self) -> Vec<Event> {
        let camped = Event::Camped {
            at: self.party_at,
            on: self.date,
        };
        self.camped = true;
        self.date = self.date.next_day();
        vec![camped, Event::DayPassed { date: self.date }]
    }

    fn write_journal(&mut self, text: String) -> Result<Vec<Event>, CommandError> {
        let text = text.trim().to_string();
        if text.is_empty() {
            return Err(CommandError::EmptyJournalEntry);
        }
        self.journal.push(JournalEntry {
            date: self.date,
            at: self.party_at,
            text,
        });
        Ok(vec![Event::JournalWritten {
            at: self.party_at,
            on: self.date,
        }])
    }
}
