//! Commands go in, events come out. The rules of the world live here.

use crate::calendar::Date;
use crate::ids::{PlaceId, SiteId};
use crate::world::{JournalEntry, Journey, Place, PlaceKind, Road, World};
use serde::{Deserialize, Serialize};
use std::fmt;

/// How far a party on foot covers in a day. It could later depend on the
/// party and the road.
pub const LEAGUES_PER_DAY: u32 = 9;

/// ASSUMPTION: the daily encounter chance on a road the player adds by hand.
pub const NEW_ROAD_ENCOUNTER_CHANCE: u32 = 15;

/// Something the player asks the world to do.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
    /// Walk the road to a neighbouring place. A walk of more than one day
    /// stops at the end of each day but the last, waiting for [`Command::EndDay`].
    Travel { to: PlaceId },
    /// Answer the end of a day on the road: make camp for the night, or press
    /// on without camping.
    EndDay { camp: bool },
    /// Walk to a site inside the place where the party is.
    Walk { to: SiteId },
    /// Make camp where the party stands and pass the night.
    MakeCamp,
    /// Add an entry to the journal.
    WriteJournal { text: String },
    /// Put a new place on the map, joined by a new road to where the party
    /// stands. It takes the region of the place the party is at.
    AddPlace { name: String, leagues: u32 },
}

/// Something that happened in the world as the result of a command.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    Departed { from: PlaceId, to: PlaceId, on: Date },
    DayPassed { date: Date },
    RoadEncounter { from: PlaceId, to: PlaceId, on: Date },
    /// A day on the road is over and the walk is not. The world now waits for
    /// a [`Command::EndDay`].
    NightFalls { from: PlaceId, to: PlaceId, days_left: u32, on: Date },
    CampedOnRoad { from: PlaceId, to: PlaceId, on: Date },
    /// The party went through the night without making camp.
    PressedOn { from: PlaceId, to: PlaceId, nights_without_camp: u32, on: Date },
    Arrived { at: PlaceId, on: Date },
    /// The party came into a place with sites by one of its gates.
    EnteredBy { at: PlaceId, gate: SiteId, on: Date },
    /// The party walked from one site to another inside a place. `from` is
    /// `None` if it was not standing at any site.
    Walked { at: PlaceId, from: Option<SiteId>, to: SiteId, on: Date },
    Camped { at: PlaceId, on: Date },
    JournalWritten { at: PlaceId, on: Date },
    PlaceAdded { place: PlaceId, from: PlaceId, leagues: u32, on: Date },
}

/// Why a command was refused. A refused command changes nothing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommandError {
    UnknownPlace(PlaceId),
    AlreadyThere,
    NoRoad { from: PlaceId, to: PlaceId },
    EmptyJournalEntry,
    EmptyPlaceName,
    PlaceExists(String),
    NoDistance,
    /// The party is between places and must first say whether it camps.
    OnTheRoad,
    /// There is no day on the road to end.
    NotOnTheRoad,
    /// The place has nowhere inside it to walk to.
    NowhereToWalk,
    UnknownSite(SiteId),
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandError::UnknownPlace(id) => write!(f, "there is no place with id {}", id.0),
            CommandError::AlreadyThere => write!(f, "the party is already there"),
            CommandError::NoRoad { .. } => write!(f, "no road leads there from here"),
            CommandError::EmptyJournalEntry => write!(f, "a journal entry needs some words"),
            CommandError::EmptyPlaceName => write!(f, "a new place needs a name"),
            CommandError::PlaceExists(name) => {
                write!(f, "there is already a place called \"{name}\"")
            }
            CommandError::OnTheRoad => {
                write!(f, "the party is on the road and must first decide whether to camp")
            }
            CommandError::NotOnTheRoad => write!(f, "the party is not on the road"),
            CommandError::NowhereToWalk => write!(f, "there is nowhere to walk to here"),
            CommandError::UnknownSite(id) => write!(f, "there is no site with id {} here", id.0),
            CommandError::NoDistance => write!(f, "the road to a new place must be at least 1 league"),
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
        if self.journey.is_some() && !matches!(command, Command::EndDay { .. }) {
            return Err(CommandError::OnTheRoad);
        }
        match command {
            Command::EndDay { camp } => self.end_day(camp),
            Command::Travel { to } => self.travel(to),
            Command::Walk { to } => self.walk(to),
            Command::MakeCamp => Ok(self.make_camp()),
            Command::WriteJournal { text } => self.write_journal(text),
            Command::AddPlace { name, leagues } => self.add_place(name, leagues),
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

        let mut events = vec![Event::Departed {
            from,
            to,
            on: self.date,
        }];
        self.camped = false;
        self.party_site = None;
        let mut journey = Journey {
            from,
            to,
            days: road.days(),
            days_walked: 0,
            encounter_chance: road.encounter_chance,
        };
        self.walk_day(&mut journey, &mut events);
        Ok(events)
    }

    fn end_day(&mut self, camp: bool) -> Result<Vec<Event>, CommandError> {
        let mut journey = self.journey.ok_or(CommandError::NotOnTheRoad)?;
        let (from, to, on) = (journey.from, journey.to, self.date);

        let mut events = Vec::new();
        if camp {
            self.nights_without_camp = 0;
            events.push(Event::CampedOnRoad { from, to, on });
        } else {
            // FATIGUE: a fatigue system would charge the party for this night
            // here, using `nights_without_camp`.
            self.nights_without_camp += 1;
            events.push(Event::PressedOn {
                from,
                to,
                nights_without_camp: self.nights_without_camp,
                on,
            });
        }
        self.walk_day(&mut journey, &mut events);
        Ok(events)
    }

    /// Walks one day of a journey. Afterwards the party has either arrived or
    /// stops for the night, with the journey saved on the world.
    fn walk_day(&mut self, journey: &mut Journey, events: &mut Vec<Event>) {
        let (from, to) = (journey.from, journey.to);
        self.date = self.date.next_day();
        events.push(Event::DayPassed { date: self.date });
        if self.rng.chance(journey.encounter_chance) {
            events.push(Event::RoadEncounter {
                from,
                to,
                on: self.date,
            });
        }
        journey.days_walked += 1;

        if journey.days_walked < journey.days {
            self.journey = Some(*journey);
            events.push(Event::NightFalls {
                from,
                to,
                days_left: journey.days - journey.days_walked,
                on: self.date,
            });
            return;
        }

        self.journey = None;
        self.party_at = to;
        for person in self.people.iter_mut().filter(|p| p.in_party) {
            person.at = to;
        }
        events.push(Event::Arrived {
            at: to,
            on: self.date,
        });

        let from_key = self.place(from).map(|p| p.key.as_str());
        self.party_site = self.place(to).and_then(|p| p.gate_for(from_key));
        if let Some(gate) = self.party_site {
            events.push(Event::EnteredBy {
                at: to,
                gate,
                on: self.date,
            });
        }
    }

    fn walk(&mut self, to: SiteId) -> Result<Vec<Event>, CommandError> {
        let at = self.party_at;
        let place = self.place(at).ok_or(CommandError::UnknownPlace(at))?;
        if place.sites.is_empty() {
            return Err(CommandError::NowhereToWalk);
        }
        if place.site(to).is_none() {
            return Err(CommandError::UnknownSite(to));
        }
        let from = self.party_site;
        if from == Some(to) {
            return Err(CommandError::AlreadyThere);
        }
        // ASSUMPTION: walking inside a place takes no time on the calendar.
        self.party_site = Some(to);
        Ok(vec![Event::Walked {
            at,
            from,
            to,
            on: self.date,
        }])
    }

    fn make_camp(&mut self) -> Vec<Event> {
        let camped = Event::Camped {
            at: self.party_at,
            on: self.date,
        };
        self.camped = true;
        self.nights_without_camp = 0;
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

    fn add_place(&mut self, name: String, leagues: u32) -> Result<Vec<Event>, CommandError> {
        let name = name.split_whitespace().collect::<Vec<_>>().join(" ");
        let key = key_for(&name);
        if key.is_empty() {
            return Err(CommandError::EmptyPlaceName);
        }
        if leagues == 0 {
            return Err(CommandError::NoDistance);
        }
        if let Some(existing) = self.find_place(&name).or_else(|| self.find_place(&key)) {
            return Err(CommandError::PlaceExists(self.place_name(existing).to_string()));
        }

        let from = self.party_at;
        let region = self.place(from).and_then(|p| p.region);
        let place = PlaceId(self.places.len() as u32);
        self.places.push(Place {
            key,
            name,
            kind: PlaceKind::Landmark,
            region,
            temple: None,
            sites: Vec::new(),
        });
        self.roads.push(Road {
            a: from,
            b: place,
            leagues,
            encounter_chance: NEW_ROAD_ENCOUNTER_CHANCE,
        });
        Ok(vec![Event::PlaceAdded {
            place,
            from,
            leagues,
            on: self.date,
        }])
    }
}

/// Turns a place name into a key: "Old Mill Ford" becomes "old-mill-ford".
fn key_for(name: &str) -> String {
    name.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
