//! Commands go in, events come out. The rules of the world live here.

use crate::calendar::Date;
use crate::character::{
    EffectKind, EffectPayload, EquipSlot, Equipped, Item, ItemType, Role, BASE_FATIGUE,
    MAX_ATTITUDE, MIN_ATTITUDE,
};
use crate::faction::{FactionId, MAX_HOSTILITY, MIN_HOSTILITY};
use crate::ids::{PersonId, PlaceId, SiteId};
use crate::npc::Standing;
use crate::world::{JournalEntry, Journey, Place, PlaceKind, Road, SiteKind, World};
use serde::{Deserialize, Serialize};
use std::fmt;

/// How far a camped march covers in a calendar day. Pressing on covers
/// extra leagues that night, fewer as the streak grows.
pub const LEAGUES_PER_DAY: u32 = 9;

/// ASSUMPTION: the daily encounter chance on a road the player adds by hand.
pub const NEW_ROAD_ENCOUNTER_CHANCE: u32 = 15;

/// Fatigue a night camped beside the road restores, and never past [`BASE_FATIGUE`].
const ROAD_CAMP_RESTORE: u32 = 5;
/// Fatigue a night at an inn or a residence restores, and never past [`BASE_FATIGUE`].
const INN_RESTORE: u32 = 15;
/// After pressing on has left a member below this, road camping stops here
/// until a night at an inn or a residence.
const ROAD_REST_CEILING: u32 = 50;
/// Fatigue the first night of pressing on costs. Each further night of the
/// streak costs [`PRESS_FATIGUE_STEP`] more.
const PRESS_FATIGUE: u32 = 10;
const PRESS_FATIGUE_STEP: u32 = 5;
/// Fatigue a missing ration costs each party member who has a sheet.
const MISSING_FOOD_FATIGUE: u32 = 5;
/// Health a dry camp costs each party member who has a sheet.
const MISSING_WATER_HEALTH: u32 = 5;
/// Resilience a camp without a fire costs each party member who has a sheet.
const MISSING_WOOD_RESILIENCE: u32 = 5;
/// Copper a night at an inn costs. A residence costs nothing.
const INN_COPPER: u32 = 5;

/// Extra leagues the nth night of pressing on adds to that day's march.
/// Night 1 adds 6, night 2 adds 4, night 3 adds 2, and every later night adds 1.
fn extra_leagues(night: u32) -> u32 {
    match night {
        1 => 6,
        2 => 4,
        3 => 2,
        n if n >= 4 => 1,
        _ => 0,
    }
}

/// Fatigue pressing on costs on night `night` of the current streak.
/// Night 1 costs 10, then 15, 20, 25, and so on.
fn press_cost(night: u32) -> u32 {
    PRESS_FATIGUE.saturating_add(PRESS_FATIGUE_STEP.saturating_mul(night.saturating_sub(1)))
}

/// Something the player asks the world to do.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
    /// Walk the road to a neighbouring place. A walk of more than one day
    /// stops at the end of each day but the last, waiting for [`Command::EndDay`].
    Travel { to: PlaceId },
    /// Answer a night on the road. `camp: false` presses on. `camp: true`
    /// pitches camp and does not pass the night; the road prompt uses
    /// [`Command::MakeCamp`], and [`Command::Rest`] spends the night.
    EndDay { camp: bool },
    /// Walk to a site inside the place where the party is.
    Walk { to: SiteId },
    /// Pitch camp where the party stands, or beside the road when travelling.
    /// Refused inside a town. Does not pass the night, spend supplies, restore
    /// fatigue, or clear the sleepless streak.
    MakeCamp,
    /// Sleep in the pitched camp and pass the night. Spends one food, one
    /// water and one wood when the party carries them. That spend consumes
    /// the item, so its consumed effects fire. A missing supply is a
    /// penalty, not a refusal. On the road this is also the camped march.
    Rest,
    /// Sleep one night where the party stands, when that is an inn or a
    /// residence. An inn costs [`INN_COPPER`] copper. A residence does not,
    /// unless the site is marked as an inn.
    Sleep,
    /// Take down the pitched camp. The party cannot travel until it does.
    PackUp,
    /// Add an entry to the journal.
    WriteJournal { text: String },
    /// Put a new place on the map, joined by a new road to where the party
    /// stands. It takes the region of the place the party is at.
    AddPlace { name: String, leagues: u32 },
    /// Take someone into the party as a follower. They must be recruitable
    /// and where the party stands. Turning an offer down needs no command.
    Recruit { person: PersonId },
    /// Set one faction's hostility toward another. 100 is least hostile and 0
    /// is most hostile. A faction's index toward itself stays at 100. Any
    /// other index must lie from [`MIN_HOSTILITY`] to [`MAX_HOSTILITY`].
    SetHostility {
        from: FactionId,
        toward: FactionId,
        index: u32,
    },
    /// Set an NPC's attitude toward the player, from [`MIN_ATTITUDE`] to
    /// [`MAX_ATTITUDE`]. The player has no attitude.
    SetAttitude { person: PersonId, index: u32 },
    /// Wear an item from the player's pack in `slot`. The item leaves the
    /// pack. The slot refuses an item that does not fit it, and a slot that
    /// already holds something.
    Equip { item: String, slot: EquipSlot },
    /// Take the item in `slot` off and return it to the player's pack.
    Unequip { slot: EquipSlot },
    /// Fire the use effects of an item in the player's pack. The item stays
    /// in the pack. Consumed effects do not fire.
    Use { item: String },
    /// Interact with an NPC where the player is. One friendly enough opens a
    /// conversation; one too hostile to talk to starts combat instead.
    Interact { person: PersonId },
    /// Say one of the lines on offer, by its place in
    /// [`World::dialogue_options`].
    Say { line: usize },
    /// Leave the conversation.
    EndConversation,
}

/// Something that happened in the world as the result of a command.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    Departed {
        from: PlaceId,
        to: PlaceId,
        on: Date,
    },
    DayPassed {
        date: Date,
    },
    RoadEncounter {
        from: PlaceId,
        to: PlaceId,
        on: Date,
    },
    /// A day on the road is over and the walk is not. The world now waits for
    /// a [`Command::EndDay`].
    NightFalls {
        from: PlaceId,
        to: PlaceId,
        days_left: u32,
        on: Date,
    },
    /// The party pitched camp beside the road. The night has not passed.
    CampedOnRoad {
        from: PlaceId,
        to: PlaceId,
        on: Date,
    },
    /// The party went through the night without making camp. `extra_leagues`
    /// is how far past a camped day this march reaches.
    PressedOn {
        from: PlaceId,
        to: PlaceId,
        nights_without_camp: u32,
        extra_leagues: u32,
        on: Date,
        /// Fatigue of each party member who has a sheet, after the night.
        fatigue: Vec<MemberFatigue>,
    },
    Arrived {
        at: PlaceId,
        on: Date,
    },
    /// The party came into a place with sites by one of its gates.
    EnteredBy {
        at: PlaceId,
        gate: SiteId,
        on: Date,
    },
    /// The party walked from one site to another inside a place. `from` is
    /// `None` if it was not standing at any site.
    Walked {
        at: PlaceId,
        from: Option<SiteId>,
        to: SiteId,
        on: Date,
    },
    /// The party pitched camp. The night has not passed and nothing was spent.
    Camped {
        at: PlaceId,
        on: Date,
    },
    /// The party slept in its pitched camp and the night passed.
    Rested {
        at: PlaceId,
        on: Date,
        /// Fatigue of each party member who has a sheet, after the night.
        fatigue: Vec<MemberFatigue>,
        /// True when the party spent one of that supply. False means it was
        /// missing, and the penalty for it was applied.
        spent_food: bool,
        spent_water: bool,
        spent_wood: bool,
    },
    /// The pitched camp was taken down.
    PackedUp {
        at: PlaceId,
        on: Date,
    },
    /// The party slept one night at an inn or a residence.
    Slept {
        at: PlaceId,
        site: SiteId,
        on: Date,
        /// Fatigue of each party member who has a sheet, after the night.
        fatigue: Vec<MemberFatigue>,
        /// Copper paid. Zero at a residence. An inn pays [`INN_COPPER`].
        copper: u32,
    },
    JournalWritten {
        at: PlaceId,
        on: Date,
    },
    PlaceAdded {
        place: PlaceId,
        from: PlaceId,
        leagues: u32,
        on: Date,
    },
    /// Someone where the party has just arrived could join it. The front end
    /// asks the player, and answers yes with [`Command::Recruit`].
    RecruitOffered {
        person: PersonId,
        at: PlaceId,
        on: Date,
    },
    /// Someone joined the party as a follower.
    Recruited {
        person: PersonId,
        at: PlaceId,
        on: Date,
    },
    /// One faction's hostility toward another changed.
    HostilityChanged {
        from: FactionId,
        toward: FactionId,
        index: u32,
    },
    /// An NPC's attitude toward the player changed.
    AttitudeChanged {
        person: PersonId,
        index: u32,
    },
    /// The player put an item from the pack into a slot.
    Equipped {
        person: PersonId,
        item: String,
        slot: EquipSlot,
    },
    /// The player took an item out of a slot and put it back in the pack.
    Unequipped {
        person: PersonId,
        item: String,
        slot: EquipSlot,
    },
    /// The player used an item in the pack. The item is still carried.
    Used {
        person: PersonId,
        item: String,
    },
    /// A use or consumed effect fired.
    EffectFired {
        person: PersonId,
        item: String,
        kind: EffectKind,
        payload: EffectPayload,
    },
    /// A conversation began. While it lasts, only [`Command::Say`] and
    /// [`Command::EndConversation`] are accepted.
    ConversationOpened {
        person: PersonId,
        standing: Standing,
    },
    /// The player said a line, and the NPC answered with `reply`, which may be
    /// blank.
    Said {
        person: PersonId,
        text: String,
        reply: String,
    },
    ConversationEnded {
        person: PersonId,
    },
    /// The NPC is too hostile to talk to, so interacting with them starts
    /// combat. COMBAT: combat is its own system and is not built yet, so for
    /// now this event is all that happens and the world does not change.
    TooHostileToTalk {
        person: PersonId,
        friendliness: u32,
    },
}

/// Why a command was refused. A refused command changes nothing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommandError {
    UnknownPlace(PlaceId),
    AlreadyThere,
    NoRoad {
        from: PlaceId,
        to: PlaceId,
    },
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
    UnknownPerson(PersonId),
    AlreadyInParty,
    /// The person is somewhere other than where the party stands.
    NotHere,
    /// The person will not join the party.
    NotRecruitable,
    /// A hostility or attitude index was outside 0 to 100.
    IndexOutOfRange,
    /// A faction's hostility toward itself stays at 100, the least hostile.
    HostilityTowardItself,
    /// The person is not an NPC, so they have no attitude.
    NoAttitude,
    /// Pressing on would drop a party member's fatigue below zero.
    TooTired,
    /// The party is not standing at an inn or a residence.
    NotAtAnInn,
    /// A camp is already pitched.
    AlreadyCamped,
    /// No camp is pitched.
    NotCamped,
    /// The party must pack up before it travels.
    StillCamped,
    /// The party is inside a town, and cannot pitch camp there.
    CannotCampInTown,
    /// An inn night costs copper the party does not have.
    CannotAfford,
    /// The party has no player.
    NoPlayer,
    /// The player's pack does not hold this item.
    NotCarrying(String),
    /// The item does not fit the slot.
    DoesNotFit {
        item: String,
        slot: EquipSlot,
    },
    /// The slot already holds an item.
    SlotTaken(EquipSlot),
    /// The slot is empty.
    SlotEmpty(EquipSlot),
    /// Only an NPC can be talked to.
    NotAnNpc,
    /// A conversation is under way and must be ended first.
    InConversation,
    /// There is no conversation to say anything in.
    NotInConversation,
    /// No line on offer has this number.
    UnknownLine(usize),
}

/// One party member's fatigue after a night that changed it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemberFatigue {
    pub person: PersonId,
    pub fatigue: u32,
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
                write!(
                    f,
                    "the party is on the road and must first decide whether to camp"
                )
            }
            CommandError::NotOnTheRoad => write!(f, "the party is not on the road"),
            CommandError::NowhereToWalk => write!(f, "there is nowhere to walk to here"),
            CommandError::UnknownSite(id) => write!(f, "there is no site with id {} here", id.0),
            CommandError::UnknownPerson(id) => write!(f, "there is no person with id {}", id.0),
            CommandError::AlreadyInParty => write!(f, "they are already in the party"),
            CommandError::NotHere => write!(f, "they are not here"),
            CommandError::NotRecruitable => write!(f, "they will not join the party"),
            CommandError::NoDistance => {
                write!(f, "the road to a new place must be at least 1 league")
            }
            CommandError::IndexOutOfRange => {
                write!(
                    f,
                    "the index must lie from {MIN_HOSTILITY} to {MAX_HOSTILITY}"
                )
            }
            CommandError::HostilityTowardItself => {
                write!(
                    f,
                    "a faction's hostility toward itself stays at {MAX_HOSTILITY}"
                )
            }
            CommandError::NoAttitude => write!(f, "they have no attitude toward the player"),
            CommandError::TooTired => write!(f, "the party is too tired to press on"),
            CommandError::NotAtAnInn => {
                write!(f, "the party is not at an inn or a residence")
            }
            CommandError::AlreadyCamped => write!(f, "the party is already camped"),
            CommandError::NotCamped => write!(f, "the party is not camped"),
            CommandError::StillCamped => write!(f, "the party must pack up before it travels"),
            CommandError::CannotCampInTown => write!(f, "the party cannot camp inside a town"),
            CommandError::CannotAfford => write!(f, "the party cannot pay for the inn"),
            CommandError::NoPlayer => write!(f, "there is no player"),
            CommandError::NotCarrying(name) => {
                write!(f, "the pack does not hold \"{name}\"")
            }
            CommandError::DoesNotFit { item, slot } => {
                write!(f, "\"{item}\" does not fit the {slot}")
            }
            CommandError::SlotTaken(slot) => write!(f, "the {slot} already holds an item"),
            CommandError::SlotEmpty(slot) => write!(f, "nothing is worn on the {slot}"),
            CommandError::NotAnNpc => write!(f, "only an NPC can be talked to"),
            CommandError::InConversation => {
                write!(
                    f,
                    "the player is talking and must end the conversation first"
                )
            }
            CommandError::NotInConversation => write!(f, "the player is not talking to anyone"),
            CommandError::UnknownLine(n) => write!(f, "there is no line {n} to say"),
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
        if self.conversation.is_some()
            && !matches!(command, Command::Say { .. } | Command::EndConversation)
        {
            return Err(CommandError::InConversation);
        }
        if self.journey.is_some()
            && !matches!(
                command,
                Command::EndDay { .. }
                    | Command::MakeCamp
                    | Command::Rest
                    | Command::PackUp
                    | Command::SetHostility { .. }
                    | Command::SetAttitude { .. }
                    | Command::Equip { .. }
                    | Command::Unequip { .. }
                    | Command::Use { .. }
            )
        {
            return Err(CommandError::OnTheRoad);
        }
        match command {
            Command::EndDay { camp } => self.end_day(camp),
            Command::Travel { to } => self.travel(to),
            Command::Walk { to } => self.walk(to),
            Command::MakeCamp => self.make_camp(),
            Command::Rest => self.rest(),
            Command::Sleep => self.sleep(),
            Command::PackUp => self.pack_up(),
            Command::WriteJournal { text } => self.write_journal(text),
            Command::AddPlace { name, leagues } => self.add_place(name, leagues),
            Command::Recruit { person } => self.recruit(person),
            Command::SetHostility {
                from,
                toward,
                index,
            } => self.set_hostility(from, toward, index),
            Command::SetAttitude { person, index } => self.set_attitude(person, index),
            Command::Equip { item, slot } => self.equip(item, slot),
            Command::Unequip { slot } => self.unequip(slot),
            Command::Use { item } => self.use_item(item),
            Command::Interact { person } => self.interact(person),
            Command::Say { line } => self.say(line),
            Command::EndConversation => self.end_conversation(),
        }
    }

    fn set_hostility(
        &mut self,
        from: FactionId,
        toward: FactionId,
        index: u32,
    ) -> Result<Vec<Event>, CommandError> {
        if from == toward {
            return Err(CommandError::HostilityTowardItself);
        }
        if !(MIN_HOSTILITY..=MAX_HOSTILITY).contains(&index) {
            return Err(CommandError::IndexOutOfRange);
        }
        self.hostility.set(from, toward, index);
        Ok(vec![Event::HostilityChanged {
            from,
            toward,
            index,
        }])
    }

    fn set_attitude(&mut self, id: PersonId, index: u32) -> Result<Vec<Event>, CommandError> {
        if !(MIN_ATTITUDE..=MAX_ATTITUDE).contains(&index) {
            return Err(CommandError::IndexOutOfRange);
        }
        let person = self.person(id).ok_or(CommandError::UnknownPerson(id))?;
        let Some(follower) = person.sheet.as_ref().and_then(|sheet| match sheet.role {
            Role::Npc { follower, .. } => Some(follower),
            Role::Player => None,
        }) else {
            return Err(CommandError::NoAttitude);
        };
        let sheet = self.people[id.index()]
            .sheet
            .as_mut()
            .expect("checked above");
        sheet.role = Role::Npc {
            follower,
            attitude: index,
        };
        Ok(vec![Event::AttitudeChanged { person: id, index }])
    }

    fn interact(&mut self, id: PersonId) -> Result<Vec<Event>, CommandError> {
        let person = self.person(id).ok_or(CommandError::UnknownPerson(id))?;
        if person.is_player() {
            return Err(CommandError::NotAnNpc);
        }
        let standing = self.standing(id).ok_or(CommandError::NotAnNpc)?;
        if !self.nearby_npcs().contains(&id) {
            return Err(CommandError::NotHere);
        }
        if !standing.will_talk() {
            // COMBAT: start combat with this NPC here.
            return Ok(vec![Event::TooHostileToTalk {
                person: id,
                friendliness: standing.friendliness,
            }]);
        }
        self.conversation = Some(id);
        Ok(vec![Event::ConversationOpened {
            person: id,
            standing,
        }])
    }

    fn say(&mut self, line: usize) -> Result<Vec<Event>, CommandError> {
        let person = self.conversation.ok_or(CommandError::NotInConversation)?;
        let said = self
            .dialogue_options(person)
            .get(line)
            .map(|l| (l.text.clone(), l.reply.clone()))
            .ok_or(CommandError::UnknownLine(line))?;
        Ok(vec![Event::Said {
            person,
            text: said.0,
            reply: said.1,
        }])
    }

    fn end_conversation(&mut self) -> Result<Vec<Event>, CommandError> {
        let person = self
            .conversation
            .take()
            .ok_or(CommandError::NotInConversation)?;
        Ok(vec![Event::ConversationEnded { person }])
    }

    fn recruit(&mut self, id: PersonId) -> Result<Vec<Event>, CommandError> {
        let at = self.party_at;
        let person = self.person(id).ok_or(CommandError::UnknownPerson(id))?;
        if person.in_party {
            return Err(CommandError::AlreadyInParty);
        }
        if person.at != at {
            return Err(CommandError::NotHere);
        }
        if !person.recruitable {
            return Err(CommandError::NotRecruitable);
        }
        let person = &mut self.people[id.index()];
        person.in_party = true;
        person.recruitable = false;
        if let Some(sheet) = &mut person.sheet {
            let attitude = sheet
                .role
                .attitude()
                .unwrap_or(crate::character::BASE_ATTITUDE);
            sheet.role = Role::Npc {
                follower: true,
                attitude,
            };
            sheet.adopt_player_faction();
        }
        Ok(vec![Event::Recruited {
            person: id,
            at,
            on: self.date,
        }])
    }

    fn travel(&mut self, to: PlaceId) -> Result<Vec<Event>, CommandError> {
        if self.camped {
            return Err(CommandError::StillCamped);
        }
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
            leagues: road.leagues,
            leagues_walked: 0,
            encounter_chance: road.encounter_chance,
        };
        // Setting out is a camped march. Pressing on is chosen at day's end.
        self.walk_day(&mut journey, &mut events, LEAGUES_PER_DAY);
        Ok(events)
    }

    fn end_day(&mut self, camp: bool) -> Result<Vec<Event>, CommandError> {
        if self.journey.is_none() {
            return Err(CommandError::NotOnTheRoad);
        }
        // Pitching camp is MakeCamp. It must not skip the day.
        if camp {
            return self.make_camp();
        }
        if self.camped {
            return Err(CommandError::StillCamped);
        }
        let mut journey = self.journey.expect("the journey was checked");
        let (from, to, on) = (journey.from, journey.to, self.date);

        let night = self.nights_without_camp + 1;
        let cost = press_cost(night);
        if !self.can_afford_press(cost) {
            return Err(CommandError::TooTired);
        }
        let extra = extra_leagues(night);
        self.nights_without_camp = night;
        let fatigue = self.spend_fatigue(cost);
        let mut events = vec![Event::PressedOn {
            from,
            to,
            nights_without_camp: night,
            extra_leagues: extra,
            on,
            fatigue,
        }];
        self.walk_day(&mut journey, &mut events, LEAGUES_PER_DAY + extra);
        Ok(events)
    }

    /// Walks one day of a journey, covering `leagues_today`. Afterwards the
    /// party has either arrived or stops for the night, with the journey
    /// saved on the world.
    fn walk_day(&mut self, journey: &mut Journey, events: &mut Vec<Event>, leagues_today: u32) {
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
        journey.leagues_walked = journey
            .leagues_walked
            .saturating_add(leagues_today)
            .min(journey.leagues);

        if journey.leagues_walked < journey.leagues {
            self.journey = Some(*journey);
            events.push(Event::NightFalls {
                from,
                to,
                days_left: journey.days_left(),
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
        for person in self.recruit_offers() {
            events.push(Event::RecruitOffered {
                person,
                at: to,
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

    fn sleep(&mut self) -> Result<Vec<Event>, CommandError> {
        let at = self.party_at;
        let site = self.party_site.ok_or(CommandError::NotAtAnInn)?;
        let kind = self
            .place(at)
            .and_then(|place| place.site(site))
            .map(|site| site.kind)
            .ok_or(CommandError::NotAtAnInn)?;
        let copper = match kind {
            SiteKind::Inn => INN_COPPER,
            SiteKind::Residence => 0,
            SiteKind::Gate
            | SiteKind::Temple
            | SiteKind::Nedia
            | SiteKind::Court
            | SiteKind::Palace
            | SiteKind::Market
            | SiteKind::Landmark => return Err(CommandError::NotAtAnInn),
        };
        if copper > 0 && self.party_holding(ItemType::Copper) < copper {
            return Err(CommandError::CannotAfford);
        }

        let spent = if copper > 0 {
            let (spent, effects) = self.party_spends(ItemType::Copper, copper);
            if !spent {
                return Err(CommandError::CannotAfford);
            }
            effects
        } else {
            Vec::new()
        };
        let on = self.date;
        let fatigue = self.restore_at_inn();
        self.nights_without_camp = 0;
        self.date = self.date.next_day();
        let mut events = vec![Event::Slept {
            at,
            site,
            on,
            fatigue,
            copper,
        }];
        events.extend(spent);
        events.push(Event::DayPassed { date: self.date });
        Ok(events)
    }

    /// True when every party member who has a sheet can pay `cost`.
    fn can_afford_press(&self, cost: u32) -> bool {
        self.people
            .iter()
            .filter(|person| person.in_party)
            .filter_map(|person| person.sheet.as_ref())
            .all(|sheet| sheet.fatigue >= cost)
    }

    fn spend_fatigue(&mut self, cost: u32) -> Vec<MemberFatigue> {
        let mut fatigue = Vec::new();
        for (index, person) in self.people.iter_mut().enumerate() {
            if !person.in_party {
                continue;
            }
            let Some(sheet) = person.sheet.as_mut() else {
                continue;
            };
            sheet.fatigue = sheet.fatigue.saturating_sub(cost);
            if sheet.fatigue < ROAD_REST_CEILING {
                sheet.road_rest_debt = true;
            }
            fatigue.push(MemberFatigue {
                person: PersonId(index as u32),
                fatigue: sheet.fatigue,
            });
        }
        fatigue
    }

    fn restore_on_road(&mut self) -> Vec<MemberFatigue> {
        let mut fatigue = Vec::new();
        for (index, person) in self.people.iter_mut().enumerate() {
            if !person.in_party {
                continue;
            }
            let Some(sheet) = person.sheet.as_mut() else {
                continue;
            };
            let current = sheet.fatigue;
            let restored = current.saturating_add(ROAD_CAMP_RESTORE).min(BASE_FATIGUE);
            sheet.fatigue = if sheet.road_rest_debt {
                restored.min(ROAD_REST_CEILING.max(current))
            } else {
                restored
            };
            fatigue.push(MemberFatigue {
                person: PersonId(index as u32),
                fatigue: sheet.fatigue,
            });
        }
        fatigue
    }

    fn restore_at_inn(&mut self) -> Vec<MemberFatigue> {
        let mut fatigue = Vec::new();
        for (index, person) in self.people.iter_mut().enumerate() {
            if !person.in_party {
                continue;
            }
            let Some(sheet) = person.sheet.as_mut() else {
                continue;
            };
            let restored = if sheet.fatigue < ROAD_REST_CEILING {
                sheet
                    .fatigue
                    .saturating_add(INN_RESTORE)
                    .max(ROAD_REST_CEILING)
            } else {
                sheet.fatigue.saturating_add(INN_RESTORE)
            };
            sheet.fatigue = restored.min(BASE_FATIGUE);
            sheet.road_rest_debt = false;
            fatigue.push(MemberFatigue {
                person: PersonId(index as u32),
                fatigue: sheet.fatigue,
            });
        }
        fatigue
    }

    fn make_camp(&mut self) -> Result<Vec<Event>, CommandError> {
        if self.camped {
            return Err(CommandError::AlreadyCamped);
        }
        // On the road the party has left the place, even while `party_at`
        // still names it. A town only refuses a camp pitched inside it.
        if self.journey.is_none()
            && self
                .place(self.party_at)
                .is_some_and(|place| place.kind == PlaceKind::Town)
        {
            return Err(CommandError::CannotCampInTown);
        }
        let on = self.date;
        self.camped = true;
        if let Some(journey) = self.journey {
            Ok(vec![Event::CampedOnRoad {
                from: journey.from,
                to: journey.to,
                on,
            }])
        } else {
            Ok(vec![Event::Camped {
                at: self.party_at,
                on,
            }])
        }
    }

    /// Sleep in the pitched camp. This is the night: supplies are spent,
    /// fatigue is restored as on a camped march, and the date advances.
    /// Spending a supply consumes it, so its consumed effects fire.
    /// Beside the road the camped march of [`LEAGUES_PER_DAY`] follows, and
    /// the camp is struck so the next night asks again.
    fn rest(&mut self) -> Result<Vec<Event>, CommandError> {
        if !self.camped {
            return Err(CommandError::NotCamped);
        }
        let (spent_food, food_effects) = self.party_spends(ItemType::Food, 1);
        let (spent_water, water_effects) = self.party_spends(ItemType::Water, 1);
        let (spent_wood, wood_effects) = self.party_spends(ItemType::Wood, 1);
        let at = self.party_at;
        let on = self.date;
        self.restore_on_road();
        self.penalize_missing_supplies(spent_food, spent_water, spent_wood);
        let fatigue = self.member_fatigue();
        self.nights_without_camp = 0;

        let mut events = vec![Event::Rested {
            at,
            on,
            fatigue,
            spent_food,
            spent_water,
            spent_wood,
        }];
        events.extend(food_effects);
        events.extend(water_effects);
        events.extend(wood_effects);
        if let Some(mut journey) = self.journey {
            self.camped = false;
            self.walk_day(&mut journey, &mut events, LEAGUES_PER_DAY);
        } else {
            self.date = self.date.next_day();
            events.push(Event::DayPassed { date: self.date });
        }
        Ok(events)
    }

    fn pack_up(&mut self) -> Result<Vec<Event>, CommandError> {
        if !self.camped {
            return Err(CommandError::NotCamped);
        }
        self.camped = false;
        Ok(vec![Event::PackedUp {
            at: self.party_at,
            on: self.date,
        }])
    }

    /// How many of `item_type` the party carries.
    fn party_holding(&self, item_type: ItemType) -> u32 {
        self.people
            .iter()
            .filter(|person| person.in_party)
            .filter_map(|person| person.sheet.as_ref())
            .fold(0, |sum, sheet| sum.saturating_add(sheet.holding(item_type)))
    }

    /// Spends `count` of `item_type` from the party. False, and nothing is
    /// taken, when the party does not carry that many. A count of zero spends
    /// nothing and succeeds. Each item taken is consumed: its consumed
    /// effects fire on the person who carried it. Worn items are not spent.
    fn party_spends(&mut self, item_type: ItemType, count: u32) -> (bool, Vec<Event>) {
        if count == 0 {
            return (true, Vec::new());
        }
        if self.party_holding(item_type) < count {
            return (false, Vec::new());
        }
        let mut left = count;
        let mut events = Vec::new();
        for index in 0..self.people.len() {
            if !self.people[index].in_party || left == 0 {
                continue;
            }
            if self.people[index].sheet.is_none() {
                continue;
            }
            while left > 0 {
                let taken = {
                    let sheet = self.people[index]
                        .sheet
                        .as_mut()
                        .expect("the sheet was checked");
                    match sheet.take_one(item_type) {
                        Some(item) => item,
                        None => break,
                    }
                };
                left -= 1;
                events.extend(self.fire_consumed(index, &taken));
            }
        }
        (left == 0, events)
    }

    /// Fires the consumed effects of one item on the person at `index`.
    fn fire_consumed(&mut self, index: usize, item: &Item) -> Vec<Event> {
        let payloads: Vec<EffectPayload> = item
            .effects
            .iter()
            .filter(|effect| effect.kind == EffectKind::Consumed)
            .map(|effect| effect.payload)
            .collect();
        let mut events = Vec::new();
        let Some(sheet) = self.people[index].sheet.as_mut() else {
            return events;
        };
        for payload in payloads {
            sheet.apply_instant(payload);
            events.push(Event::EffectFired {
                person: PersonId(index as u32),
                item: item.name.clone(),
                kind: EffectKind::Consumed,
                payload,
            });
        }
        events
    }

    fn player_index(&self) -> Result<usize, CommandError> {
        self.people
            .iter()
            .position(|person| person.is_player())
            .ok_or(CommandError::NoPlayer)
    }

    fn equip(&mut self, name: String, slot: EquipSlot) -> Result<Vec<Event>, CommandError> {
        let index = self.player_index()?;
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(CommandError::NotCarrying(name));
        }
        let sheet = self.people[index]
            .sheet
            .as_ref()
            .ok_or(CommandError::NoPlayer)?;
        let Some(item) = sheet.inventory.iter().find(|item| item.name == name) else {
            return Err(CommandError::NotCarrying(name));
        };
        match item.slot {
            Some(fits) if fits == slot => {}
            _ => return Err(CommandError::DoesNotFit { item: name, slot }),
        }
        if sheet.equipped.iter().any(|worn| worn.slot == slot) {
            return Err(CommandError::SlotTaken(slot));
        }
        let sheet = self.people[index]
            .sheet
            .as_mut()
            .expect("the player sheet was checked");
        let item = sheet
            .take_named(&name)
            .expect("the pack was checked for this item");
        sheet.equipped.push(Equipped { slot, item });
        Ok(vec![Event::Equipped {
            person: PersonId(index as u32),
            item: name,
            slot,
        }])
    }

    fn unequip(&mut self, slot: EquipSlot) -> Result<Vec<Event>, CommandError> {
        let index = self.player_index()?;
        let sheet = self.people[index]
            .sheet
            .as_mut()
            .ok_or(CommandError::NoPlayer)?;
        let Some(pos) = sheet.equipped.iter().position(|worn| worn.slot == slot) else {
            return Err(CommandError::SlotEmpty(slot));
        };
        let worn = sheet.equipped.remove(pos);
        let name = worn.item.name.clone();
        sheet.carry(worn.item);
        Ok(vec![Event::Unequipped {
            person: PersonId(index as u32),
            item: name,
            slot,
        }])
    }

    fn use_item(&mut self, name: String) -> Result<Vec<Event>, CommandError> {
        let index = self.player_index()?;
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(CommandError::NotCarrying(name));
        }
        let sheet = self.people[index]
            .sheet
            .as_mut()
            .ok_or(CommandError::NoPlayer)?;
        let Some(item) = sheet.inventory.iter().find(|item| item.name == name) else {
            return Err(CommandError::NotCarrying(name));
        };
        let payloads: Vec<EffectPayload> = item
            .effects
            .iter()
            .filter(|effect| effect.kind == EffectKind::Use)
            .map(|effect| effect.payload)
            .collect();
        let person = PersonId(index as u32);
        let mut events = vec![Event::Used {
            person,
            item: name.clone(),
        }];
        for payload in payloads {
            sheet.apply_instant(payload);
            events.push(Event::EffectFired {
                person,
                item: name.clone(),
                kind: EffectKind::Use,
                payload,
            });
        }
        Ok(events)
    }

    /// Fatigue of each party member who has a sheet, as it stands now.
    fn member_fatigue(&self) -> Vec<MemberFatigue> {
        self.people
            .iter()
            .enumerate()
            .filter_map(|(index, person)| {
                if !person.in_party {
                    return None;
                }
                person.sheet.as_ref().map(|sheet| MemberFatigue {
                    person: PersonId(index as u32),
                    fatigue: sheet.fatigue,
                })
            })
            .collect()
    }

    /// Each missing supply hits every party member who has a sheet. A member
    /// without a sheet is skipped. The date and the sleepless streak stay put.
    fn penalize_missing_supplies(&mut self, spent_food: bool, spent_water: bool, spent_wood: bool) {
        if spent_food && spent_water && spent_wood {
            return;
        }
        for person in self.people.iter_mut().filter(|person| person.in_party) {
            let Some(sheet) = person.sheet.as_mut() else {
                continue;
            };
            if !spent_food {
                sheet.fatigue = sheet.fatigue.saturating_sub(MISSING_FOOD_FATIGUE);
            }
            if !spent_water {
                sheet.health = sheet.health.saturating_sub(MISSING_WATER_HEALTH);
            }
            if !spent_wood {
                sheet.resilience = sheet.resilience.saturating_sub(MISSING_WOOD_RESILIENCE);
            }
        }
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
            return Err(CommandError::PlaceExists(
                self.place_name(existing).to_string(),
            ));
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
