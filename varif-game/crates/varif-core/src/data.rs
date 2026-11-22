//! Loading a world from a data file.
//!
//! Data files name places by key ("yhlgrym"), which is pleasant to write by
//! hand. Loading checks the file and turns every key into an ID.

use crate::calendar::{Date, HOURS_PER_DAY};
use crate::character::{Character, SheetError};
use crate::faction::Hostility;
use crate::ids::PlaceId;
use crate::npc::DialogueLine;
use crate::rng::Rng;
use crate::schedule::{RoadLeg, Schedule, Span, Whereabouts};
use crate::sky::Sky;
use crate::world::{Person, Place, Road, SiteKind, TimeMode, World};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Deserialize)]
struct WorldData {
    start: StartData,
    places: Vec<Place>,
    #[serde(default)]
    roads: Vec<RoadData>,
    #[serde(default)]
    people: Vec<PersonData>,
    #[serde(default)]
    dialogue: Vec<DialogueLine>,
}

#[derive(Debug, Deserialize)]
struct StartData {
    year: u32,
    month: u32,
    day: u32,
    /// Hour of the day, 0 to 23. Omitted files start at hour 0.
    #[serde(default)]
    hour: u32,
    seed: u64,
    party_at: String,
}

#[derive(Debug, Deserialize)]
struct RoadData {
    from: String,
    to: String,
    leagues: u32,
    #[serde(default)]
    encounter_chance: u32,
}

#[derive(Debug, Deserialize)]
struct PersonData {
    key: String,
    name: String,
    title: String,
    at: String,
    /// The key of a site inside `at`, when that place has sites.
    #[serde(default)]
    site: Option<String>,
    #[serde(default)]
    in_party: bool,
    #[serde(default)]
    recruitable: bool,
    #[serde(default)]
    sheet: Option<Character>,
    #[serde(default)]
    dialogue: Vec<DialogueLine>,
    #[serde(default)]
    schedule: ScheduleData,
}

#[derive(Debug, Default, Deserialize)]
struct ScheduleData {
    #[serde(default)]
    spans: Vec<SpanData>,
}

#[derive(Debug, Deserialize)]
struct SpanData {
    hours: u32,
    place: String,
    #[serde(default)]
    site: Option<String>,
    /// The place key the NPC is walking from, when the span is travel.
    #[serde(default)]
    road_from: Option<String>,
    /// The place key the NPC is walking toward, when the span is travel.
    #[serde(default)]
    road_to: Option<String>,
}

/// What is wrong with a data file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DataError {
    Parse(String),
    BadStartDate {
        year: u32,
        month: u32,
        day: u32,
    },
    /// The start hour was not from 0 to 23.
    BadStartHour {
        hour: u32,
    },
    DuplicatePlace(String),
    DuplicatePerson(String),
    UnknownPlace {
        key: String,
        used_by: String,
    },
    RoadToItself(String),
    DuplicateRoad {
        from: String,
        to: String,
    },
    DuplicateSite {
        place: String,
        site: String,
    },
    NoGate(String),
    BadSheet {
        person: String,
        error: SheetError,
    },
    TwoPlayers,
    /// A dialogue line has no words for the player to say. `owner` is the
    /// person it belongs to, or "the general dialogue".
    BlankDialogue {
        owner: String,
    },
    /// A person names a site that their place does not have.
    UnknownSite {
        person: String,
        place: String,
        site: String,
    },
    /// A person in a place with sites was not given one. Followers are
    /// excused: they travel with the party.
    MissingSite {
        person: String,
        place: String,
    },
    /// A schedule span lasts no hours.
    EmptySpan {
        person: String,
    },
    /// A travel span named one end of the road and not the other.
    RoadNeedsTwoEnds {
        person: String,
    },
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::Parse(message) => write!(f, "the file could not be read: {message}"),
            DataError::BadStartDate { year, month, day } => {
                write!(f, "the start date {day}/{month}/{year} is not a real date")
            }
            DataError::BadStartHour { hour } => {
                write!(f, "the start hour {hour} is not from 0 to 23")
            }
            DataError::DuplicatePlace(key) => write!(f, "two places share the key \"{key}\""),
            DataError::DuplicatePerson(key) => write!(f, "two people share the key \"{key}\""),
            DataError::UnknownPlace { key, used_by } => {
                write!(
                    f,
                    "{used_by} names a place \"{key}\" that is not in the file"
                )
            }
            DataError::RoadToItself(key) => write!(f, "a road joins \"{key}\" to itself"),
            DataError::DuplicateRoad { from, to } => {
                write!(f, "there are two roads between \"{from}\" and \"{to}\"")
            }
            DataError::DuplicateSite { place, site } => {
                write!(f, "two sites in \"{place}\" share the key \"{site}\"")
            }
            DataError::NoGate(place) => {
                write!(f, "\"{place}\" has sites but no gate to come in by")
            }
            DataError::BadSheet { person, error } => {
                write!(f, "{person}'s sheet is not valid: {error}")
            }
            DataError::TwoPlayers => write!(f, "two people are the player"),
            DataError::BlankDialogue { owner } => {
                write!(f, "a line in {owner} has nothing to say")
            }
            DataError::UnknownSite {
                person,
                place,
                site,
            } => write!(f, "{person} names a site \"{site}\" that is not in {place}"),
            DataError::MissingSite { person, place } => {
                write!(f, "{person} is in {place} but names no site")
            }
            DataError::EmptySpan { person } => {
                write!(f, "a span in {person}'s schedule lasts no hours")
            }
            DataError::RoadNeedsTwoEnds { person } => {
                write!(
                    f,
                    "a travel span in {person}'s schedule names one end of the road and not the other"
                )
            }
        }
    }
}

impl std::error::Error for DataError {}

/// Builds a [`World`] from the text of a data file.
pub fn load_world(text: &str) -> Result<World, DataError> {
    let data: WorldData = ron::from_str(text).map_err(|e| DataError::Parse(e.to_string()))?;

    // A BTreeMap keeps its keys in order, so nothing here depends on the
    // arbitrary ordering a HashMap would have.
    let mut place_ids: BTreeMap<String, PlaceId> = BTreeMap::new();
    for (index, place) in data.places.iter().enumerate() {
        let id = PlaceId(index as u32);
        if place_ids.insert(place.key.clone(), id).is_some() {
            return Err(DataError::DuplicatePlace(place.key.clone()));
        }
    }
    let lookup = |key: &str, used_by: &str| -> Result<PlaceId, DataError> {
        place_ids
            .get(key)
            .copied()
            .ok_or_else(|| DataError::UnknownPlace {
                key: key.to_string(),
                used_by: used_by.to_string(),
            })
    };

    for place in &data.places {
        if place.sites.is_empty() {
            continue;
        }
        if !place.sites.iter().any(|s| s.kind == SiteKind::Gate) {
            return Err(DataError::NoGate(place.key.clone()));
        }
        for (i, site) in place.sites.iter().enumerate() {
            if place.sites[..i].iter().any(|s| s.key == site.key) {
                return Err(DataError::DuplicateSite {
                    place: place.key.clone(),
                    site: site.key.clone(),
                });
            }
            for faced in &site.faces {
                lookup(faced, &format!("the {} of {}", site.name, place.name))?;
            }
        }
    }

    let start = &data.start;
    let date = Date::new(start.year, start.month, start.day).ok_or(DataError::BadStartDate {
        year: start.year,
        month: start.month,
        day: start.day,
    })?;
    if start.hour >= HOURS_PER_DAY {
        return Err(DataError::BadStartHour { hour: start.hour });
    }
    let party_at = lookup(&start.party_at, "the start")?;
    let party_site = data.places[party_at.index()].gate_for(None);

    let mut roads: Vec<Road> = Vec::new();
    for road in &data.roads {
        let a = lookup(&road.from, "a road")?;
        let b = lookup(&road.to, "a road")?;
        if a == b {
            return Err(DataError::RoadToItself(road.from.clone()));
        }
        if roads.iter().any(|r| r.joins(a, b)) {
            return Err(DataError::DuplicateRoad {
                from: road.from.clone(),
                to: road.to.clone(),
            });
        }
        roads.push(Road {
            a,
            b,
            leagues: road.leagues,
            encounter_chance: road.encounter_chance,
        });
    }

    let blank = |lines: &[DialogueLine]| lines.iter().any(|l| l.text.trim().is_empty());
    if blank(&data.dialogue) {
        return Err(DataError::BlankDialogue {
            owner: "the general dialogue".to_string(),
        });
    }

    let mut people: Vec<Person> = Vec::new();
    let mut seen_player = false;
    for mut person in data.people {
        if people.iter().any(|p| p.key == person.key) {
            return Err(DataError::DuplicatePerson(person.key));
        }
        // People in the party start wherever the party starts.
        let at = if person.in_party {
            party_at
        } else {
            lookup(&person.at, &person.name)?
        };
        if let Some(sheet) = &mut person.sheet {
            sheet.adopt_player_faction();
            sheet.check().map_err(|error| DataError::BadSheet {
                person: person.name.clone(),
                error,
            })?;
            if sheet.role.is_player() {
                if seen_player {
                    return Err(DataError::TwoPlayers);
                }
                seen_player = true;
            }
        }
        if blank(&person.dialogue) {
            return Err(DataError::BlankDialogue {
                owner: format!("{}'s dialogue", person.name),
            });
        }
        let schedule = load_schedule(
            &person.name,
            &person.schedule,
            &data.places,
            &lookup,
        )?;
        let place = &data.places[at.index()];
        let site = match person.site.as_deref() {
            Some(key) => Some(place.find_site(key).ok_or_else(|| DataError::UnknownSite {
                person: person.name.clone(),
                place: place.name.clone(),
                site: key.to_string(),
            })?),
            None if !person.in_party && !place.sites.is_empty() => {
                return Err(DataError::MissingSite {
                    person: person.name.clone(),
                    place: place.name.clone(),
                });
            }
            None => None,
        };
        people.push(Person {
            key: person.key,
            name: person.name,
            title: person.title,
            at,
            in_party: person.in_party,
            recruitable: person.recruitable,
            sheet: person.sheet,
            dialogue: person.dialogue,
            site,
            schedule,
        });
    }

    Ok(World {
        date,
        hour: start.hour,
        minute: 0,
        time_mode: TimeMode::TurnBased,
        clock_paused: false,
        rng: Rng::new(start.seed),
        places: data.places,
        roads,
        people,
        party_at,
        party_site,
        camped: false,
        journal: Vec::new(),
        journey: None,
        hostility: Hostility::starting(),
        dialogue: data.dialogue,
        conversation: None,
        sky: Sky::new(start.seed),
    })
}

/// Turns schedule keys into ids. A span of zero hours is refused. A travel
/// span names both ends of the road or neither.
fn load_schedule(
    person: &str,
    data: &ScheduleData,
    places: &[Place],
    lookup: &impl Fn(&str, &str) -> Result<PlaceId, DataError>,
) -> Result<Schedule, DataError> {
    let mut spans = Vec::with_capacity(data.spans.len());
    for span in &data.spans {
        if span.hours == 0 {
            return Err(DataError::EmptySpan {
                person: person.to_string(),
            });
        }
        let place = lookup(&span.place, &format!("{person}'s schedule"))?;
        let site = match span.site.as_deref() {
            Some(key) => {
                let place_data = &places[place.index()];
                Some(place_data.find_site(key).ok_or_else(|| DataError::UnknownSite {
                    person: person.to_string(),
                    place: place_data.name.clone(),
                    site: key.to_string(),
                })?)
            }
            None => None,
        };
        let on_the_road = match (&span.road_from, &span.road_to) {
            (None, None) => None,
            (Some(from), Some(to)) => Some(RoadLeg {
                from: lookup(from, &format!("{person}'s schedule"))?,
                to: lookup(to, &format!("{person}'s schedule"))?,
            }),
            _ => {
                return Err(DataError::RoadNeedsTwoEnds {
                    person: person.to_string(),
                })
            }
        };
        spans.push(Span {
            hours: span.hours,
            whereabouts: Whereabouts {
                place,
                site,
                on_the_road,
            },
        });
    }
    Ok(Schedule { spans })
}
