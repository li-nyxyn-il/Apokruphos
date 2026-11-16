//! The state of the world: tables of places, roads and people, plus the party.

use crate::calendar::Date;
use crate::character::Character;
use crate::faction::{FactionId, Hostility};
use crate::ids::{PersonId, PlaceId, SiteId};
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

/// What kind of location a site inside a place is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SiteKind {
    /// Where a party comes in from the road. A place may have several.
    Gate,
    Inn,
    /// A Krypt or other house of a Beyond.
    Temple,
    /// A school of a nedia.
    Nedia,
    /// The city court, where the governor rules civil affairs.
    Court,
    Palace,
    /// The house or estate of a Line.
    Residence,
    Market,
    /// Anything else worth walking to.
    Landmark,
}

/// A location inside a place that the party can walk to.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Site {
    /// A short lowercase name used in data files and at the command line.
    /// Only needs to be unique within its place.
    pub key: String,
    pub name: String,
    pub kind: SiteKind,
    /// For a gate: the keys of the places whose roads come in by this gate.
    /// A party arriving from a place no gate names comes in by the first gate.
    #[serde(default)]
    pub faces: Vec<String>,
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
    /// The locations inside the place. Empty for a place with nowhere to walk
    /// to; otherwise there is at least one gate.
    #[serde(default)]
    pub sites: Vec<Site>,
}

impl Place {
    pub fn site(&self, id: SiteId) -> Option<&Site> {
        self.sites.get(id.index())
    }

    /// The gate a party coming from the place with key `from_key` enters by:
    /// the gate that faces it, or else the first gate. `None` if the place
    /// has no gates.
    pub fn gate_for(&self, from_key: Option<&str>) -> Option<SiteId> {
        let gates = || {
            self.sites
                .iter()
                .enumerate()
                .filter(|(_, s)| s.kind == SiteKind::Gate)
        };
        let facing =
            from_key.and_then(|key| gates().find(|(_, s)| s.faces.iter().any(|f| f == key)));
        facing
            .or_else(|| gates().next())
            .map(|(i, _)| SiteId(i as u32))
    }

    /// Finds a site by its key or its name, ignoring case and a leading "the".
    pub fn find_site(&self, text: &str) -> Option<SiteId> {
        let wanted = text.trim().to_lowercase();
        let wanted = wanted.strip_prefix("the ").unwrap_or(&wanted);
        self.sites
            .iter()
            .position(|s| s.key.to_lowercase() == wanted || s.name.to_lowercase() == wanted)
            .map(|i| SiteId(i as u32))
    }
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

    /// How many days a camped march takes. Pressing on can finish the road
    /// in fewer calendar days; that is settled while the party walks.
    pub fn days(&self) -> u32 {
        self.leagues
            .div_ceil(crate::command::LEAGUES_PER_DAY)
            .max(1)
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

/// The shortest way by road from one place to another, from [`World::reachable`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Route {
    pub to: PlaceId,
    /// The total length of the route.
    pub leagues: u32,
    /// How many days the walk takes, road by road.
    pub days: u32,
    /// How many roads the route walks.
    pub roads: u32,
    /// The neighbouring place to travel to first.
    pub first_step: PlaceId,
}

/// One way from a place to another along a chain of roads, from
/// [`World::routes_between`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Itinerary {
    /// Every place the party walks to in turn. The last is the destination;
    /// the place it sets out from is left out.
    pub stops: Vec<PlaceId>,
    pub leagues: u32,
    /// How many days the walk takes, road by road.
    pub days: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Person {
    pub key: String,
    pub name: String,
    /// How they are known in a place: "Pylgrym", "Innkeeper". The player,
    /// NPC and follower distinction lives on the sheet's [`crate::character::Role`].
    pub title: String,
    pub at: PlaceId,
    /// True for the player and for followers travelling with the party.
    #[serde(default)]
    pub in_party: bool,
    /// True for someone who can be asked to join the party. When the party
    /// arrives where they are, an [`crate::Event::RecruitOffered`] asks.
    #[serde(default)]
    pub recruitable: bool,
    /// The shared character sheet. A person named only as cast leaves it empty.
    #[serde(default)]
    pub sheet: Option<Character>,
}

impl Person {
    pub fn is_player(&self) -> bool {
        self.sheet
            .as_ref()
            .is_some_and(|sheet| sheet.role.is_player())
    }

    pub fn is_follower(&self) -> bool {
        self.sheet
            .as_ref()
            .is_some_and(|sheet| sheet.role.is_follower())
    }
}

/// A walk that is under way. Between days the party waits on the road for
/// the player to say whether it camps for the night.
///
/// A camped day covers [`crate::command::LEAGUES_PER_DAY`] leagues. Pressing
/// on covers more, so the march is kept in leagues and the party arrives on
/// however many calendar days it actually spends.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Journey {
    pub from: PlaceId,
    pub to: PlaceId,
    /// The length of the road.
    pub leagues: u32,
    /// How many of those leagues are already behind the party.
    pub leagues_walked: u32,
    pub encounter_chance: u32,
}

impl Journey {
    /// Leagues still ahead.
    pub fn leagues_left(self) -> u32 {
        self.leagues.saturating_sub(self.leagues_walked)
    }

    /// Calendar days the rest of the road would take at a camped march.
    pub fn days_left(self) -> u32 {
        let left = self.leagues_left();
        if left == 0 {
            0
        } else {
            left.div_ceil(crate::command::LEAGUES_PER_DAY)
        }
    }
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
    /// Where inside `party_at` the party stands, if the place has sites.
    /// `None` while on the road or in a place with nowhere to walk to.
    #[serde(default)]
    pub party_site: Option<SiteId>,
    /// True while a camp is pitched. Making camp does not pass the night, and
    /// the party must pack the camp up before it can travel.
    pub camped: bool,
    pub journal: Vec<JournalEntry>,
    /// The walk in progress, if the party is between places. While it is
    /// `Some`, the world waits for camp, sleep, pack up, or pressing on.
    #[serde(default)]
    pub journey: Option<Journey>,
    /// Nights in a row the party has pressed on without sleeping. The next
    /// press is night `nights_without_camp + 1`: it costs more fatigue and
    /// adds fewer extra leagues as the streak grows. Sleeping in camp, and a
    /// night at an inn or a residence, sets this back to zero. Pitching camp
    /// does not.
    #[serde(default)]
    pub nights_without_camp: u32,
    /// How hostile each faction is toward each other faction right now.
    /// A new world starts from [`FactionId::starting_hostility`], and the
    /// numbers change with the world.
    #[serde(default)]
    pub hostility: Hostility,
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
        self.place(id)
            .map_or("an unknown place", |p| p.name.as_str())
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

    /// Every way from `from` to `to` that never passes the same place twice,
    /// found by a depth-first search. Quickest first, then shortest, then
    /// fewest stops; ties keep the order the search found them in.
    ///
    /// The number of ways grows quickly on a map with many loops, so this is
    /// meant for maps the size of a region, not a continent.
    pub fn routes_between(&self, from: PlaceId, to: PlaceId) -> Vec<Itinerary> {
        if from == to || self.place(from).is_none() || self.place(to).is_none() {
            return Vec::new();
        }
        let mut visited = vec![false; self.places.len()];
        let mut path = Vec::new();
        let mut found = Vec::new();
        self.search(from, to, &mut visited, &mut path, &mut found);

        let mut routes: Vec<Itinerary> = found
            .into_iter()
            .map(|stops| {
                let (mut leagues, mut days) = (0, 0);
                let mut at = from;
                for &next in &stops {
                    let road = self.road_between(at, next).expect("the search walks roads");
                    leagues += road.leagues;
                    days += road.days();
                    at = next;
                }
                Itinerary {
                    stops,
                    leagues,
                    days,
                }
            })
            .collect();
        routes.sort_by_key(|r| (r.days, r.leagues, r.stops.len()));
        routes
    }

    /// One step of the depth-first search behind [`World::routes_between`].
    fn search(
        &self,
        at: PlaceId,
        to: PlaceId,
        visited: &mut [bool],
        path: &mut Vec<PlaceId>,
        found: &mut Vec<Vec<PlaceId>>,
    ) {
        if at == to {
            found.push(path.clone());
            return;
        }
        visited[at.index()] = true;
        for (next, _) in self.neighbours(at) {
            if !visited[next.index()] {
                path.push(next);
                self.search(next, to, visited, path, found);
                path.pop();
            }
        }
        visited[at.index()] = false;
    }

    /// Every place the party could walk to from `from` by any chain of roads,
    /// with the shortest route to each. Nearest first; `from` is left out.
    pub fn reachable(&self, from: PlaceId) -> Vec<Route> {
        // Dijkstra's shortest paths. The map is small, so a plain scan for the
        // nearest unsettled place is enough and keeps the order deterministic.
        let count = self.places.len();
        let mut best: Vec<Option<Route>> = vec![None; count];
        let mut settled = vec![false; count];
        if from.index() >= count {
            return Vec::new();
        }
        best[from.index()] = Some(Route {
            to: from,
            leagues: 0,
            days: 0,
            roads: 0,
            first_step: from,
        });

        loop {
            let next = (0..count)
                .filter(|&i| !settled[i])
                .filter_map(|i| best[i].map(|r| (r.leagues, r.roads, i)))
                .min();
            let Some((_, _, i)) = next else { break };
            settled[i] = true;
            let here = best[i].expect("a settled place has a route");

            for (to, road) in self.neighbours(PlaceId(i as u32)) {
                let candidate = Route {
                    to,
                    leagues: here.leagues + road.leagues,
                    days: here.days + road.days(),
                    roads: here.roads + 1,
                    first_step: if here.roads == 0 { to } else { here.first_step },
                };
                let better = best[to.index()].map_or(true, |old| {
                    (candidate.leagues, candidate.roads) < (old.leagues, old.roads)
                });
                if !settled[to.index()] && better {
                    best[to.index()] = Some(candidate);
                }
            }
        }

        let mut routes: Vec<Route> = best
            .into_iter()
            .flatten()
            .filter(|r| r.to != from)
            .collect();
        routes.sort_by(|x, y| {
            (x.leagues, x.roads, self.place_name(x.to)).cmp(&(
                y.leagues,
                y.roads,
                self.place_name(y.to),
            ))
        });
        routes
    }

    /// The people travelling with the party.
    pub fn party(&self) -> impl Iterator<Item = &Person> {
        self.people.iter().filter(|p| p.in_party)
    }

    /// The player, the person whose sheet says they are the player.
    pub fn player(&self) -> Option<&Person> {
        self.people.iter().find(|person| person.is_player())
    }

    /// NPCs whose sheet marks them as followers.
    pub fn followers(&self) -> impl Iterator<Item = &Person> {
        self.people.iter().filter(|person| person.is_follower())
    }

    /// The people where the party stands who could be recruited into it.
    pub fn recruit_offers(&self) -> Vec<PersonId> {
        self.people
            .iter()
            .enumerate()
            .filter(|(_, p)| p.recruitable && !p.in_party && p.at == self.party_at)
            .map(|(i, _)| PersonId(i as u32))
            .collect()
    }

    /// The people at a place who are not in the party.
    pub fn residents(&self, at: PlaceId) -> impl Iterator<Item = &Person> {
        self.people
            .iter()
            .filter(move |p| !p.in_party && p.at == at)
    }

    /// Writes the whole world as text, for saving.
    pub fn to_ron(&self) -> Result<String, String> {
        ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .map_err(|e| e.to_string())
    }

    /// Reads a world written by [`World::to_ron`].
    pub fn from_ron(text: &str) -> Result<World, String> {
        let mut world: World = ron::from_str(text).map_err(|e| e.to_string())?;
        world.grant_follower_factions();
        Ok(world)
    }

    /// How hostile `from` is toward `toward` at this moment.
    pub fn hostility_toward(&self, from: FactionId, toward: FactionId) -> u32 {
        self.hostility.get(from, toward)
    }

    /// Gives every follower the Player faction, if they do not already have it.
    pub fn grant_follower_factions(&mut self) {
        for person in &mut self.people {
            if let Some(sheet) = &mut person.sheet {
                sheet.adopt_player_faction();
            }
        }
    }
}
