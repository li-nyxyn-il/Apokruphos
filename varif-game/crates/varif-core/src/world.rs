//! The state of the world: tables of places, roads and people, plus the party.

use crate::calendar::{Date, DAYS_PER_YEAR, HOURS_PER_DAY, MINUTES_PER_DAY, MINUTES_PER_HOUR};
use crate::character::Character;
use crate::dialogue::{Action, Choice, DialogueTree, Sprite};
use crate::faction::{FactionId, Hostility};
use crate::ids::{PersonId, PlaceId, SiteId};
use crate::npc::{DialogueLine, Standing};
use crate::rng::Rng;
use crate::schedule::{Schedule, Whereabouts};
use crate::sky::Sky;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

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
    /// The chance out of 100 of an encounter, once for each calendar day a
    /// travel order spends on this road.
    pub encounter_chance: u32,
}

impl Road {
    pub fn joins(&self, x: PlaceId, y: PlaceId) -> bool {
        (self.a == x && self.b == y) || (self.a == y && self.b == x)
    }

    /// How many hours the road takes. One league takes one hour.
    pub fn hours(self) -> u32 {
        self.leagues
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
    /// How many hours the walk takes, at one league an hour.
    pub hours: u32,
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
    /// How many hours the walk takes, at one league an hour.
    pub hours: u32,
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
    /// Where inside [`Person::at`] they stand, when that place has sites.
    /// `None` for a follower travelling with the party, and for someone in a
    /// place with nowhere to walk to.
    #[serde(default)]
    pub site: Option<SiteId>,
    /// The shared character sheet. A person named only as cast leaves it empty.
    #[serde(default)]
    pub sheet: Option<Character>,
    /// Lines only this person has, on top of the world's general dialogue.
    #[serde(default)]
    pub dialogue: Vec<DialogueLine>,
    /// Where they are across the hours. Empty for someone who stays at
    /// [`Person::at`]. Replaced at runtime with [`crate::Command::SetSchedule`].
    /// A lookup reads this schedule and the world clock; it does not tick
    /// anyone forward.
    #[serde(default)]
    pub schedule: Schedule,
    /// A scripted conversation. Without one they say the flat bucket lines.
    #[serde(default)]
    pub tree: Option<DialogueTree>,
    /// Named pictures the person can be shown with.
    #[serde(default)]
    pub sprites: Vec<Sprite>,
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

/// A walk that is under way. The player orders a number of hours. That order
/// covers that many leagues, one an hour, then stops. The party may camp,
/// sleep, and pack up before the next order.
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
    /// Leagues still ahead. At one league an hour, this is also the hours left.
    pub fn leagues_left(self) -> u32 {
        self.leagues.saturating_sub(self.leagues_walked)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalEntry {
    pub date: Date,
    pub at: PlaceId,
    pub text: String,
}

/// How the world clock moves. Stored on the world. The core never reads the
/// operating system clock.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TimeMode {
    /// The host passes elapsed minutes into [`crate::World::advance`].
    /// Passive time moves the clock only while it is not paused.
    RealTime,
    /// Passive elapsed time does nothing. Time moves when a command spends
    /// it, or on an explicit step.
    #[default]
    TurnBased,
}

impl TimeMode {
    /// The other mode, for a control that switches.
    pub fn other(self) -> TimeMode {
        match self {
            TimeMode::RealTime => TimeMode::TurnBased,
            TimeMode::TurnBased => TimeMode::RealTime,
        }
    }
}

impl fmt::Display for TimeMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimeMode::RealTime => write!(f, "real time"),
            TimeMode::TurnBased => write!(f, "turn-based"),
        }
    }
}

fn default_time_scale() -> u32 {
    1
}

/// One answer the player can give right now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogueOption {
    /// What the player says.
    pub text: String,
    /// The name of the bucket a flat line belongs to. Empty for a tree choice.
    pub label: String,
}

/// Everything that is true of the world at one moment.
///
/// The fields are public so that a front end can read them freely. Change the
/// world only through [`World::apply`], except the host may pass elapsed
/// minutes to [`World::advance`] while [`TimeMode::RealTime`] is set and the
/// clock is not paused.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct World {
    pub date: Date,
    /// The hour of the day, from 0 to 23. This is the world's clock, together
    /// with [`World::date`] and [`World::minute`]. There is no second clock.
    /// Serialized with the world. Only [`World::advance`] moves it.
    #[serde(default)]
    pub hour: u32,
    /// The minute of the hour, from 0 to 59. Serialized with the date and the
    /// hour. Schedules ignore it and keep using the absolute hour.
    #[serde(default)]
    pub minute: u32,
    /// Real time or turn-based.
    #[serde(default)]
    pub time_mode: TimeMode,
    /// Game minutes per real minute while the clock is in real time. One
    /// matches the host clock. The host multiplies its own elapsed time by
    /// this before calling [`World::advance`]. Camp, sleep, travel, and the
    /// other commands that spend time are not scaled. The core does not
    /// read the operating system clock. Change it with
    /// [`crate::Command::SetTimeScale`].
    #[serde(default = "default_time_scale")]
    pub time_scale: u32,
    /// Set when an event needs the player. Real-time elapsed time does
    /// nothing while this is set. A command that spends time clears it, and
    /// sets it again if that command stops on a new event.
    #[serde(default)]
    pub clock_paused: bool,
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
    /// `Some`, the next travel order continues toward `to`.
    #[serde(default)]
    pub journey: Option<Journey>,
    /// How hostile each faction is toward each other faction right now.
    /// A new world starts from [`FactionId::starting_hostility`], and the
    /// numbers change with the world.
    #[serde(default)]
    pub hostility: Hostility,
    /// Lines any NPC can be offered, when the bucket the line needs is open.
    #[serde(default)]
    pub dialogue: Vec<DialogueLine>,
    /// The NPC the player is talking to. While it is `Some`, the world waits
    /// for something to be said or for the conversation to end.
    #[serde(default)]
    pub conversation: Option<PersonId>,
    /// The key of the node of the person's dialogue tree the conversation is
    /// at. `None` when the conversation uses flat lines, or when there is no
    /// conversation. Cleared whenever `conversation` is.
    #[serde(default)]
    pub conversation_node: Option<String>,
    /// Tonight's constellations. Drawn from the game's seed when the world is
    /// made, and turned to the next night each noon by [`World::advance`].
    #[serde(default)]
    pub sky: Sky,
    /// Named marks the story has set. Dialogue conditions read them and
    /// dialogue actions set and clear them.
    #[serde(default)]
    pub flags: BTreeSet<String>,
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
                let (mut leagues, mut hours) = (0, 0);
                let mut at = from;
                for &next in &stops {
                    let road = self.road_between(at, next).expect("the search walks roads");
                    leagues += road.leagues;
                    hours += road.hours();
                    at = next;
                }
                Itinerary {
                    stops,
                    leagues,
                    hours,
                }
            })
            .collect();
        routes.sort_by_key(|r| (r.hours, r.leagues, r.stops.len()));
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
            hours: 0,
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
                    hours: here.hours + road.hours(),
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

    /// The NPCs where the player stands, followers included, in the order of
    /// the people table. Someone without a sheet is cast only and is left out.
    ///
    /// In a place with sites, only the people at the site the party stands at
    /// are nearby: a warden at the gate is not near someone in the palace or
    /// the market. Followers travel with the party, so they are nearby at
    /// every site. A place with no sites has everyone there nearby.
    pub fn nearby_npcs(&self) -> Vec<PersonId> {
        let Some(player) = self.player() else {
            return Vec::new();
        };
        let here = player.at;
        let sited = self
            .place(here)
            .is_some_and(|place| !place.sites.is_empty());
        let standing = self.party_site;
        self.people
            .iter()
            .enumerate()
            .filter(|(_, person)| {
                person.at == here
                    && person
                        .sheet
                        .as_ref()
                        .is_some_and(|sheet| !sheet.role.is_player())
                    && (!sited || person.in_party || person.site == standing)
            })
            .map(|(i, _)| PersonId(i as u32))
            .collect()
    }

    /// Where an NPC stands toward the player: their attitude, their primary
    /// faction's hostility index toward the player, and the friendliness of
    /// the two. `None` for the player and for someone without a sheet.
    pub fn standing(&self, id: PersonId) -> Option<Standing> {
        let sheet = self.person(id)?.sheet.as_ref()?;
        let attitude = sheet.role.attitude()?;
        let hostility = self.hostility_toward(sheet.primary, FactionId::Player);
        Some(Standing::new(attitude, hostility, sheet.role.is_follower()))
    }

    /// The choices of the person's dialogue tree on offer now: those at the
    /// current node whose condition holds and whose item gifts and takings
    /// can be paid. Empty unless the person is the one being talked to.
    pub(crate) fn offered_choices(&self, id: PersonId) -> Vec<&Choice> {
        if self.conversation != Some(id) {
            return Vec::new();
        }
        let node = self
            .person(id)
            .and_then(|person| person.tree.as_ref())
            .zip(self.conversation_node.as_deref())
            .and_then(|(tree, key)| tree.node(key));
        let Some(node) = node else {
            return Vec::new();
        };
        node.choices
            .iter()
            .filter(|choice| choice.when.holds(self, id) && self.can_pay(id, &choice.actions))
            .collect()
    }

    /// Whether the `GiveItem` and `TakeItem` actions together find enough of
    /// each item in the pack it comes from.
    fn can_pay(&self, npc: PersonId, actions: &[Action]) -> bool {
        let player = self.player().and_then(|p| p.sheet.as_ref());
        let npc = self.person(npc).and_then(|p| p.sheet.as_ref());
        let held = |sheet: Option<&Character>, name: &str| -> u32 {
            sheet.map_or(0, |sheet| {
                sheet
                    .inventory
                    .iter()
                    .filter(|item| item.name == name)
                    .fold(0u32, |sum, item| sum.saturating_add(item.count))
            })
        };
        // Total what the whole choice asks of each pack, so two small gifts
        // of one item are not each judged on their own.
        let mut from_npc: Vec<(&str, u32)> = Vec::new();
        let mut from_player: Vec<(&str, u32)> = Vec::new();
        for action in actions {
            let (needs, item, count) = match action {
                Action::GiveItem { item, count } => (&mut from_npc, item, *count),
                Action::TakeItem { item, count } => (&mut from_player, item, *count),
                _ => continue,
            };
            match needs.iter_mut().find(|(name, _)| name == item) {
                Some((_, total)) => *total = total.saturating_add(count),
                None => needs.push((item, count)),
            }
        }
        from_npc
            .iter()
            .all(|(name, total)| held(npc, name) >= *total)
            && from_player
                .iter()
                .all(|(name, total)| held(player, name) >= *total)
    }

    /// What the NPC says now: the line of the current node of their tree.
    /// `None` for a flat-line conversation or when no one is talking.
    pub fn npc_line(&self) -> Option<&str> {
        let id = self.conversation?;
        let tree = self.person(id)?.tree.as_ref()?;
        let node = tree.node(self.conversation_node.as_deref()?)?;
        Some(node.line.as_str())
    }

    /// What the player can say to an NPC right now. For an NPC with a
    /// dialogue tree these are the choices on offer at the current node, in
    /// order, with no label. Otherwise they are the general lines, then the
    /// NPC's own, keeping only those whose bucket is open to them, each
    /// labelled with its bucket.
    pub fn dialogue_options(&self, id: PersonId) -> Vec<DialogueOption> {
        let Some(person) = self.person(id).filter(|_| self.standing(id).is_some()) else {
            return Vec::new();
        };
        if person.tree.is_some() {
            return self
                .offered_choices(id)
                .into_iter()
                .map(|choice| DialogueOption {
                    text: choice.text.clone(),
                    label: String::new(),
                })
                .collect();
        }
        self.open_lines(id)
            .into_iter()
            .map(|line| DialogueOption {
                text: line.text.clone(),
                label: line.bucket.name(),
            })
            .collect()
    }

    /// The flat lines open to the NPC: the general lines, then their own,
    /// keeping only those whose bucket is open to them.
    pub(crate) fn open_lines(&self, id: PersonId) -> Vec<&DialogueLine> {
        let (Some(person), Some(standing)) = (self.person(id), self.standing(id)) else {
            return Vec::new();
        };
        let open = standing.buckets();
        self.dialogue
            .iter()
            .chain(&person.dialogue)
            .filter(|line| open.contains(&line.bucket))
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
        world.settle_clock();
        if world.sky.is_empty() {
            // A save from before the sky existed has no seed left to read, so
            // the sky is seeded from where the world's stream stands.
            world.sky = Sky::new(world.rng.clone().next_u64());
        }
        world.grant_follower_factions();
        Ok(world)
    }

    /// Folds extra hours and minutes into the date, so the clock stays at
    /// hour 0–23 and minute 0–59.
    fn settle_clock(&mut self) {
        let total = u64::from(self.hour) * u64::from(MINUTES_PER_HOUR) + u64::from(self.minute);
        let day = u64::from(MINUTES_PER_DAY);
        let days = (total / day) as u32;
        let rest = total % day;
        self.hour = (rest / u64::from(MINUTES_PER_HOUR)) as u32;
        self.minute = (rest % u64::from(MINUTES_PER_HOUR)) as u32;
        if days > 0 {
            self.date = self.date.plus_days(days);
        }
    }

    /// Minutes from now until the next midnight. At midnight itself this is
    /// a full day, so a check never lands on a zero-length step.
    pub(crate) fn minutes_until_midnight(&self) -> u32 {
        let now = self
            .hour
            .saturating_mul(MINUTES_PER_HOUR)
            .saturating_add(self.minute);
        MINUTES_PER_DAY.saturating_sub(now).max(1)
    }

    /// Hours from hour 0 of year 0 to the current date and hour.
    ///
    /// The minute within the hour is ignored. Schedule lookup uses this and
    /// does not tick any NPC.
    pub fn absolute_hour(&self) -> u64 {
        let days =
            u64::from(self.date.year) * u64::from(DAYS_PER_YEAR) + u64::from(self.date.day_of_year);
        days * u64::from(HOURS_PER_DAY) + u64::from(self.hour)
    }

    /// Where `id`'s schedule puts them at the current hour.
    ///
    /// `None` for an unknown person and for a person with an empty schedule.
    /// The result is [`Schedule::at`] on that one schedule, so other people
    /// are not visited and nobody is moved.
    pub fn whereabouts(&self, id: PersonId) -> Option<Whereabouts> {
        self.person(id)?.schedule.at(self.absolute_hour())
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
