//! Commands go in, events come out. The rules of the world live here.

use crate::calendar::{Date, HOURS_PER_DAY, MINUTES_PER_HOUR};
use crate::character::{
    Character, EffectKind, EffectPayload, EquipSlot, Equipped, Item, ItemType, Role, BASE_FATIGUE,
    MAX_ATTITUDE, MIN_ATTITUDE,
};
use crate::dialogue::{Action, DialogueTree};
use crate::faction::{FactionId, MAX_HOSTILITY, MIN_HOSTILITY};
use crate::ids::{PersonId, PlaceId, SiteId};
use crate::npc::Standing;
use crate::schedule::Schedule;
use crate::sky::SKY_TURNS_AT_HOUR;
use crate::world::{JournalEntry, Journey, Place, PlaceKind, Road, SiteKind, TimeMode, World};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Pitching camp takes an hour. It does not sleep and does not spend supplies.
pub const MAKE_CAMP_HOURS: u32 = 1;
/// Sleeping in camp takes eight hours. That is the night.
pub const REST_HOURS: u32 = 8;
/// Packing the camp up takes an hour.
pub const PACK_UP_HOURS: u32 = 1;
/// A night at an inn or a residence takes eight hours, the same length as
/// sleeping in camp. The date rolls only when those hours cross midnight.
const INN_HOURS: u32 = REST_HOURS;
/// Hours of one travel order that add no fatigue. Each hour past this costs
/// the next whole number, summed once for the order.
pub const UNTIRED_HOURS: u32 = 8;
/// The shortest wait the player can order.
pub const MIN_WAIT_HOURS: u32 = 1;
/// The longest wait the player can order. Zero and anything above this are
/// refused, and the world is left unchanged.
pub const MAX_WAIT_HOURS: u32 = 24;
/// One explicit step of the clock, in turn-based time.
pub const STEP_HOURS: u32 = 1;
/// One game minute per real minute. This matches the host clock.
pub const MIN_TIME_SCALE: u32 = 1;
/// The fastest real-time clock: one game minute per real second. Zero and
/// anything above this are refused, and the world is left unchanged.
pub const MAX_TIME_SCALE: u32 = 60;
/// Real milliseconds in one game minute, before the time scale.
const REAL_MS_PER_GAME_MINUTE: u128 = 60_000;

/// Why the clock was asked to move. Every change of date, hour, or minute
/// goes through [`World::advance`] with one of these.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdvanceKind {
    /// Minutes the host measured on its own clock, already multiplied by
    /// [`World::time_scale`]. Applied only in [`TimeMode::RealTime`], and
    /// only while the world clock is not paused. [`World::advance`] does
    /// not scale these minutes again.
    Elapsed,
    /// A command spends this time: camp, rest, pack, sleep, or an explicit
    /// step. Applied in either mode.
    Spend,
    /// Travel or wait. A road encounter can stop this before the order ends.
    Pass,
}

/// How far [`World::advance`] actually moved the clock.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClockAdvance {
    /// Minutes added to the clock. Less than the request when a pause hook
    /// stopped the advance early.
    pub minutes: u32,
    pub events: Vec<Event>,
    /// True when a pause hook stopped the advance before the request was done.
    pub stopped: bool,
}

/// ASSUMPTION: the encounter chance on a road the player adds by hand.
pub const NEW_ROAD_ENCOUNTER_CHANCE: u32 = 15;

/// Whole game minutes in `real_ms` at `scale`, and the real milliseconds
/// still short of the next one.
///
/// `scale` is game minutes per real minute. One matches the host clock.
/// Zero is read as one, so a save that lost its scale still runs. Camp,
/// sleep, travel, and the other commands do not use this. Only a host
/// turning its own elapsed time into an [`AdvanceKind::Elapsed`] call does.
pub fn game_minutes(real_ms: u128, scale: u32) -> (u32, u128) {
    let scale = u128::from(scale.max(MIN_TIME_SCALE));
    let game_ms = real_ms.saturating_mul(scale);
    let minutes_u128 = game_ms / REAL_MS_PER_GAME_MINUTE;
    if minutes_u128 > u128::from(u32::MAX) {
        return (u32::MAX, 0);
    }
    let minutes = minutes_u128 as u32;
    let used_real = u128::from(minutes) * REAL_MS_PER_GAME_MINUTE / scale;
    (minutes, real_ms.saturating_sub(used_real))
}

/// Real milliseconds until the next game minute, at `scale`.
///
/// A host uses this to wake in time for the next minute on screen. At 1×
/// a fresh carry waits one real minute. At 60× it waits one real second.
pub fn real_ms_until_next_game_minute(carry_real_ms: u128, scale: u32) -> u128 {
    let (_, rest) = game_minutes(carry_real_ms, scale);
    let scale = u128::from(scale.max(MIN_TIME_SCALE));
    let into = rest.saturating_mul(scale);
    let left_game = REAL_MS_PER_GAME_MINUTE.saturating_sub(into);
    left_game.div_ceil(scale).max(1)
}

/// Fatigue one travel order of `hours` costs. Eight hours or fewer cost
/// nothing. `n` hours past that cost `n * (n + 1) / 2`, once, not per league.
pub fn travel_fatigue(hours: u32) -> u32 {
    let extra = hours.saturating_sub(UNTIRED_HOURS);
    extra.saturating_mul(extra.saturating_add(1)) / 2
}

/// Fatigue a night camped beside the road restores, and never past [`BASE_FATIGUE`].
const ROAD_CAMP_RESTORE: u32 = 5;
/// Fatigue a night at an inn or a residence restores, and never past [`BASE_FATIGUE`].
const INN_RESTORE: u32 = 15;
/// After fatigue has fallen below this, road camping stops here until a night
/// at an inn or a residence.
const ROAD_REST_CEILING: u32 = 50;
/// Fatigue a missing ration costs each party member who has a sheet.
const MISSING_FOOD_FATIGUE: u32 = 5;
/// Health a dry camp costs each party member who has a sheet.
const MISSING_WATER_HEALTH: u32 = 5;
/// Resilience a camp without a fire costs each party member who has a sheet.
const MISSING_WOOD_RESILIENCE: u32 = 5;
/// Copper a night at an inn costs. A residence costs nothing.
const INN_COPPER: u32 = 5;

/// Something the player asks the world to do.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
    /// Walk `hours` along the road to `to`, one league an hour, then stop.
    /// `to` is a neighbour, or the place the current journey is already bound
    /// for. Hours past [`UNTIRED_HOURS`] cost [`travel_fatigue`] once for the
    /// order. Refused when that cost would drop any member who has a sheet
    /// below zero; the world is left unchanged.
    Travel { to: PlaceId, hours: u32 },
    /// Walk to a site inside the place where the party is.
    Walk { to: SiteId },
    /// Pitch camp where the party stands, or beside the road when travelling.
    /// Takes [`MAKE_CAMP_HOURS`]. Refused inside a town. Does not sleep, spend
    /// supplies, or restore fatigue.
    MakeCamp,
    /// Sleep in the pitched camp for [`REST_HOURS`]. Spends one food, one
    /// water and one wood when the party carries them. That spend consumes
    /// the item, so its consumed effects fire. A missing supply is a
    /// penalty, not a refusal. The camp stays pitched.
    Rest,
    /// Sleep eight hours where the party stands, when that is an inn or a
    /// residence. An inn costs [`INN_COPPER`] copper. A residence does not,
    /// unless the site is marked as an inn. The date rolls only when those
    /// hours cross midnight.
    Sleep,
    /// Take down the pitched camp. Takes [`PACK_UP_HOURS`]. The party cannot
    /// travel until it does.
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
    /// Interact with a nearby NPC. In a city that is someone at the site the
    /// party stands at, or a follower. One friendly enough opens a
    /// conversation; one too hostile to talk to starts combat instead.
    Interact { person: PersonId },
    /// Say one of the lines on offer, by its place in
    /// [`World::dialogue_options`].
    Say { line: usize },
    /// Leave the conversation.
    EndConversation,
    /// Replace an NPC's schedule. The next whereabouts lookup follows the new cycle.
    SetSchedule {
        person: PersonId,
        schedule: Schedule,
    },
    /// Jump the clock forward by `hours`, from [`MIN_WAIT_HOURS`] to
    /// [`MAX_WAIT_HOURS`]. Zero and anything above the maximum are refused
    /// and leave the world unchanged. On the road, a road encounter stops
    /// the wait at that point and the rest of the order is not spent.
    Wait { hours: u32 },
    /// Pass [`STEP_HOURS`] on the clock. In turn-based mode this is how time
    /// moves when no other command spends it.
    Step,
    /// Switch the clock between real time and turn-based.
    SetTimeMode { mode: TimeMode },
    /// How many game minutes pass in one real minute, from
    /// [`MIN_TIME_SCALE`] to [`MAX_TIME_SCALE`]. One matches the host clock.
    /// This does not change camp, sleep, travel, wait, or step. Zero and
    /// anything above the maximum are refused and leave the world unchanged.
    SetTimeScale { scale: u32 },
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
    /// The party walked `leagues` leagues in this order, one per hour, and
    /// stopped. `leagues_left` is what remains. Zero means the next event is
    /// the arrival.
    Traveled {
        from: PlaceId,
        to: PlaceId,
        leagues: u32,
        leagues_left: u32,
        hours: u32,
        on: Date,
        hour: u32,
        /// Fatigue of each party member who has a sheet, after this order.
        fatigue: Vec<MemberFatigue>,
    },
    /// The party pitched camp beside the road. The night has not passed.
    CampedOnRoad {
        from: PlaceId,
        to: PlaceId,
        on: Date,
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
    /// The party slept eight hours at an inn or a residence.
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
        /// What the NPC says first: the line of the entry node of their
        /// dialogue tree. Blank for flat lines.
        opening: String,
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
    /// A dialogue action set a world flag.
    FlagSet {
        flag: String,
    },
    /// A dialogue action cleared a world flag.
    FlagCleared {
        flag: String,
    },
    /// An NPC handed items from their pack to the player.
    ItemGiven {
        person: PersonId,
        item: String,
        count: u32,
    },
    /// The player handed items from the pack to an NPC.
    ItemTaken {
        person: PersonId,
        item: String,
        count: u32,
    },
    /// Items were made from nothing and given to the player.
    ItemGranted {
        item: String,
        count: u32,
    },
    /// A conversation ended in a fight. COMBAT: combat is its own system and
    /// is not built yet, so for now this event is all that happens.
    CombatStarted {
        person: PersonId,
    },
    /// An NPC became willing, or unwilling, to join the party.
    RecruitableChanged {
        person: PersonId,
        recruitable: bool,
    },
    /// The NPC is too hostile to talk to, so interacting with them starts
    /// combat. COMBAT: combat is its own system and is not built yet, so for
    /// now this event is all that happens and the world does not change.
    TooHostileToTalk {
        person: PersonId,
        friendliness: u32,
    },
    /// An NPC's schedule was replaced.
    ScheduleChanged {
        person: PersonId,
    },
    /// The party waited `hours` hours. This is how far the clock moved, which
    /// is shorter than the order when a road encounter stopped it.
    Waited {
        hours: u32,
        on: Date,
        hour: u32,
        minute: u32,
    },
    /// An explicit step moved the clock.
    Stepped {
        on: Date,
        hour: u32,
        minute: u32,
    },
    /// The clock changed mode.
    TimeModeChanged {
        mode: TimeMode,
    },
    /// How many game minutes now pass in one real minute.
    TimeScaleChanged {
        scale: u32,
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
    /// The party is between places, and this command does not belong there.
    OnTheRoad,
    /// A travel order named no hours.
    NoHours,
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
    /// The travel order would drop a party member's fatigue below zero.
    TooTired,
    /// A schedule span lasts no hours.
    EmptySpan,
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
    /// The NPC has a dialogue tree, but none of its entries holds, so they
    /// have nothing to say.
    NothingToSay,
    /// An action of a dialogue choice could not be carried out, so none of
    /// the choice took effect. Holds the reason.
    ActionFailed(String),
    /// A dialogue tree names a node it does not have.
    MissingNode(String),
    /// A wait was not from 1 to 24 hours.
    BadWait,
    /// A time scale was not from 1 to 60.
    BadTimeScale,
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
            CommandError::OnTheRoad => write!(f, "the party is on the road"),
            CommandError::NoHours => write!(f, "say how many hours to travel"),
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
            CommandError::TooTired => write!(f, "the party is too tired to travel that long"),
            CommandError::EmptySpan => write!(f, "a schedule span needs at least one hour"),
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
            CommandError::NothingToSay => write!(f, "they have nothing to say to you"),
            CommandError::MissingNode(key) => {
                write!(f, "the dialogue has no node called \"{key}\"")
            }
            CommandError::ActionFailed(why) => {
                write!(f, "the choice could not be carried out: {why}")
            }
            CommandError::BadWait => write!(f, "wait from 1 to 24 hours"),
            CommandError::BadTimeScale => write!(f, "the time scale must be from 1 to 60"),
        }
    }
}

impl std::error::Error for CommandError {}

/// The error for a dialogue tree that points at a node it does not have.
fn missing_node(key: &str) -> CommandError {
    CommandError::MissingNode(key.to_string())
}

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
                Command::Travel { .. }
                    | Command::MakeCamp
                    | Command::Rest
                    | Command::PackUp
                    | Command::Wait { .. }
                    | Command::Step
                    | Command::SetTimeMode { .. }
                    | Command::SetTimeScale { .. }
                    | Command::SetHostility { .. }
                    | Command::SetAttitude { .. }
                    | Command::Equip { .. }
                    | Command::Unequip { .. }
                    | Command::Use { .. }
                    | Command::SetSchedule { .. }
            )
        {
            return Err(CommandError::OnTheRoad);
        }
        match command {
            Command::Travel { to, hours } => self.travel(to, hours),
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
            Command::SetSchedule { person, schedule } => self.set_schedule(person, schedule),
            Command::Wait { hours } => self.wait(hours),
            Command::Step => self.step(),
            Command::SetTimeMode { mode } => self.set_time_mode(mode),
            Command::SetTimeScale { scale } => self.set_time_scale(scale),
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
        let (node, opening) = match &person.tree {
            None => (None, String::new()),
            Some(tree) => {
                let entry = tree
                    .entries
                    .iter()
                    .find(|entry| entry.when.holds(self, id))
                    .ok_or(CommandError::NothingToSay)?;
                let node = tree
                    .node(&entry.node)
                    .ok_or_else(|| missing_node(&entry.node))?;
                (Some(node.key.clone()), node.line.clone())
            }
        };
        self.conversation = Some(id);
        self.conversation_node = node;
        Ok(vec![Event::ConversationOpened {
            person: id,
            standing,
            opening,
        }])
    }

    fn say(&mut self, line: usize) -> Result<Vec<Event>, CommandError> {
        let person = self.conversation.ok_or(CommandError::NotInConversation)?;
        if let Some(tree) = self.person(person).and_then(|p| p.tree.clone()) {
            return self.say_from_tree(person, &tree, line);
        }
        let said = self
            .open_lines(person)
            .get(line)
            .map(|said| (said.text.clone(), said.reply.clone()))
            .ok_or(CommandError::UnknownLine(line))?;
        Ok(vec![Event::Said {
            person,
            text: said.0,
            reply: said.1,
        }])
    }

    /// Runs the `line`-th offered choice of `tree`: its actions, then on to
    /// the next node or the end of the conversation.
    fn say_from_tree(
        &mut self,
        person: PersonId,
        tree: &DialogueTree,
        line: usize,
    ) -> Result<Vec<Event>, CommandError> {
        let choice = self
            .offered_choices(person)
            .get(line)
            .map(|choice| (*choice).clone())
            .ok_or(CommandError::UnknownLine(line))?;
        let next = match &choice.next {
            Some(key) => Some(tree.node(key).ok_or_else(|| missing_node(key))?),
            None => None,
        };
        let action_events = self.apply_actions(person, &choice.actions)?;
        let fight = action_events
            .iter()
            .any(|event| matches!(event, Event::CombatStarted { .. }));
        let next = if fight { None } else { next };
        let mut events = vec![Event::Said {
            person,
            text: choice.text,
            reply: next.map(|node| node.line.clone()).unwrap_or_default(),
        }];
        events.extend(action_events);
        match next {
            Some(node) => self.conversation_node = Some(node.key.clone()),
            None => {
                self.conversation = None;
                self.conversation_node = None;
                events.push(Event::ConversationEnded { person });
            }
        }
        Ok(events)
    }

    /// Runs the actions of a dialogue choice in order for the NPC `npc`. All
    /// of them take effect, or none: they run on a copy of the world, which
    /// replaces this one only when every action succeeded.
    fn apply_actions(
        &mut self,
        npc: PersonId,
        actions: &[Action],
    ) -> Result<Vec<Event>, CommandError> {
        let mut trial = self.clone();
        let mut events = Vec::new();
        for action in actions {
            events.extend(
                trial
                    .apply_action(npc, action)
                    .map_err(|error| CommandError::ActionFailed(error.to_string()))?,
            );
        }
        *self = trial;
        Ok(events)
    }

    fn apply_action(&mut self, npc: PersonId, action: &Action) -> Result<Vec<Event>, CommandError> {
        match action {
            Action::ChangeAttitude(by) => {
                let current = self.standing(npc).ok_or(CommandError::NoAttitude)?.attitude;
                let index = (i64::from(current) + i64::from(*by))
                    .clamp(i64::from(MIN_ATTITUDE), i64::from(MAX_ATTITUDE));
                self.set_attitude(npc, index as u32)
            }
            Action::SetAttitude(index) => self.set_attitude(npc, *index),
            Action::ChangeHostility { from, toward, by } => {
                let current = self.hostility_toward(*from, *toward);
                let index = (i64::from(current) + i64::from(*by))
                    .clamp(i64::from(MIN_HOSTILITY), i64::from(MAX_HOSTILITY));
                self.set_hostility(*from, *toward, index as u32)
            }
            Action::GiveItem { count: 0, .. } | Action::TakeItem { count: 0, .. } => Ok(vec![]),
            Action::GiveItem { item, count } => {
                let player = self.player_index()?;
                self.move_item(npc.index(), player, item, *count)?;
                Ok(vec![Event::ItemGiven {
                    person: npc,
                    item: item.clone(),
                    count: *count,
                }])
            }
            Action::TakeItem { item, count } => {
                let player = self.player_index()?;
                self.move_item(player, npc.index(), item, *count)?;
                Ok(vec![Event::ItemTaken {
                    person: npc,
                    item: item.clone(),
                    count: *count,
                }])
            }
            Action::GrantItem(item) => {
                let player = self.player_index()?;
                let event = Event::ItemGranted {
                    item: item.name.clone(),
                    count: item.count,
                };
                self.sheet_mut(player)?.carry(item.clone());
                Ok(vec![event])
            }
            Action::Recruit => {
                // The NPC stands where the party does, or they could not be
                // talked to, so only `recruitable` needs setting aside.
                self.people
                    .get_mut(npc.index())
                    .ok_or(CommandError::UnknownPerson(npc))?
                    .recruitable = true;
                self.recruit(npc)
            }
            Action::SetRecruitable(recruitable) => {
                self.people
                    .get_mut(npc.index())
                    .ok_or(CommandError::UnknownPerson(npc))?
                    .recruitable = *recruitable;
                Ok(vec![Event::RecruitableChanged {
                    person: npc,
                    recruitable: *recruitable,
                }])
            }
            Action::SetFlag(flag) => {
                self.flags.insert(flag.clone());
                Ok(vec![Event::FlagSet { flag: flag.clone() }])
            }
            Action::ClearFlag(flag) => {
                self.flags.remove(flag);
                Ok(vec![Event::FlagCleared { flag: flag.clone() }])
            }
            Action::WriteJournal(text) => self.write_journal(text.clone()),
            // COMBAT: combat is not built yet, so this only says it began.
            Action::StartCombat => Ok(vec![Event::CombatStarted { person: npc }]),
        }
    }

    /// The sheet of the person at `index`.
    fn sheet_mut(&mut self, index: usize) -> Result<&mut Character, CommandError> {
        self.people
            .get_mut(index)
            .and_then(|person| person.sheet.as_mut())
            .ok_or(CommandError::NoAttitude)
    }

    /// Moves `count` of the item called `name` from the pack of the person
    /// at `from` to the pack of the person at `to`. The moved items keep
    /// the stack's weight, value and effects.
    fn move_item(
        &mut self,
        from: usize,
        to: usize,
        name: &str,
        count: u32,
    ) -> Result<(), CommandError> {
        let giver = self.sheet_mut(from)?;
        let held = giver
            .inventory
            .iter()
            .filter(|item| item.name == name)
            .fold(0u32, |sum, item| sum.saturating_add(item.count));
        if held < count {
            return Err(CommandError::NotCarrying(name.to_string()));
        }
        let Some(mut moved) = giver
            .inventory
            .iter()
            .find(|item| item.name == name)
            .cloned()
        else {
            // A count of zero of something the pack does not hold moves nothing.
            return Ok(());
        };
        moved.count = count;
        let mut left = count;
        for stack in giver.inventory.iter_mut().filter(|item| item.name == name) {
            let taken = stack.count.min(left);
            stack.count -= taken;
            left -= taken;
        }
        giver.inventory.retain(|item| item.count > 0);
        self.sheet_mut(to)?.carry(moved);
        Ok(())
    }

    fn end_conversation(&mut self) -> Result<Vec<Event>, CommandError> {
        let person = self
            .conversation
            .take()
            .ok_or(CommandError::NotInConversation)?;
        self.conversation_node = None;
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

    fn travel(&mut self, to: PlaceId, ordered: u32) -> Result<Vec<Event>, CommandError> {
        if ordered == 0 {
            return Err(CommandError::NoHours);
        }
        if self.camped {
            return Err(CommandError::StillCamped);
        }
        if self.place(to).is_none() {
            return Err(CommandError::UnknownPlace(to));
        }

        let (from, leagues, walked, encounter_chance, starting) =
            if let Some(journey) = self.journey {
                if journey.to != to {
                    return Err(CommandError::OnTheRoad);
                }
                (
                    journey.from,
                    journey.leagues,
                    journey.leagues_walked,
                    journey.encounter_chance,
                    false,
                )
            } else {
                let here = self.party_at;
                if here == to {
                    return Err(CommandError::AlreadyThere);
                }
                // `Road` is `Copy`, so this takes a copy and stops borrowing `self`.
                let road = *self
                    .road_between(here, to)
                    .ok_or(CommandError::NoRoad { from: here, to })?;
                (here, road.leagues, 0, road.encounter_chance, true)
            };

        let left = leagues.saturating_sub(walked);
        // The order stops at the destination. Fatigue is for the hours
        // actually walked, once, not for each league and not for hours the
        // road does not have.
        let hours = ordered.min(left);
        if hours == 0 {
            return Err(CommandError::AlreadyThere);
        }
        let cost = travel_fatigue(hours);
        if !self.can_afford_fatigue(cost) {
            return Err(CommandError::TooTired);
        }

        let mut events = Vec::new();
        if starting {
            events.push(Event::Departed {
                from,
                to,
                on: self.date,
            });
            self.party_site = None;
        }
        // The journey is stored before the clock moves, so the pause hook
        // can see the road this order is walking.
        self.journey = Some(Journey {
            from,
            to,
            leagues,
            leagues_walked: walked,
            encounter_chance,
        });

        let moved = self.advance(hours_as_minutes(hours), AdvanceKind::Pass);
        let walked_hours = moved.minutes / MINUTES_PER_HOUR;
        // The check above used the full order. A pause may have stopped the
        // walk early, so only the hours actually walked are paid.
        let spent = travel_fatigue(walked_hours);
        if spent > 0 {
            self.spend_fatigue(spent);
        }
        let fatigue = self.member_fatigue();
        events.extend(moved.events);

        let leagues_walked = walked + walked_hours;
        let leagues_left = leagues.saturating_sub(leagues_walked);
        events.push(Event::Traveled {
            from,
            to,
            leagues: walked_hours,
            leagues_left,
            hours: walked_hours,
            on: self.date,
            hour: self.hour,
            fatigue,
        });

        if leagues_left > 0 {
            if let Some(journey) = self.journey.as_mut() {
                journey.leagues_walked = leagues_walked;
            }
            return Ok(events);
        }

        self.journey = None;
        self.party_at = to;
        for person in self.people.iter_mut().filter(|person| person.in_party) {
            person.at = to;
        }
        events.push(Event::Arrived {
            at: to,
            on: self.date,
        });
        let from_key = self.place(from).map(|place| place.key.clone());
        self.party_site = self
            .place(to)
            .and_then(|place| place.gate_for(from_key.as_deref()));
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
        Ok(events)
    }

    fn set_schedule(
        &mut self,
        id: PersonId,
        schedule: Schedule,
    ) -> Result<Vec<Event>, CommandError> {
        let person = self.person(id).ok_or(CommandError::UnknownPerson(id))?;
        if person.is_player() {
            return Err(CommandError::NotAnNpc);
        }
        for span in &schedule.spans {
            if span.hours == 0 {
                return Err(CommandError::EmptySpan);
            }
            let place = self
                .place(span.whereabouts.place)
                .ok_or(CommandError::UnknownPlace(span.whereabouts.place))?;
            if let Some(site) = span.whereabouts.site {
                if place.site(site).is_none() {
                    return Err(CommandError::UnknownSite(site));
                }
            }
            if let Some(road) = span.whereabouts.on_the_road {
                if self.place(road.from).is_none() {
                    return Err(CommandError::UnknownPlace(road.from));
                }
                if self.place(road.to).is_none() {
                    return Err(CommandError::UnknownPlace(road.to));
                }
            }
        }
        self.people[id.index()].schedule = schedule;
        Ok(vec![Event::ScheduleChanged { person: id }])
    }

    /// The one place the date, hour, or minute changes.
    ///
    /// `minutes` is how far the caller asked the clock to move.
    /// [`AdvanceKind::Elapsed`] does nothing in turn-based mode, and nothing
    /// while the clock is paused. [`AdvanceKind::Spend`] and
    /// [`AdvanceKind::Pass`] always spend time. A pass on the road stops at a
    /// road encounter and does not spend the rest. [`World::pause_hook`] is
    /// where a later event stops this same function.
    pub fn advance(&mut self, minutes: u32, kind: AdvanceKind) -> ClockAdvance {
        if minutes == 0 || self.elapsed_is_idle(kind) {
            return ClockAdvance {
                minutes: 0,
                events: Vec::new(),
                stopped: false,
            };
        }
        if matches!(kind, AdvanceKind::Spend | AdvanceKind::Pass) {
            self.clock_paused = false;
        }

        let mut left = minutes;
        let mut moved = 0;
        let mut events = Vec::new();
        let mut stopped = false;
        while left > 0 {
            let step = self.minutes_before_pause_check(kind, left);
            self.add_minutes(step, &mut events);
            moved += step;
            left -= step;
            if let Some(event) = self.pause_hook(kind) {
                events.push(event);
                self.clock_paused = true;
                stopped = true;
                break;
            }
        }
        ClockAdvance {
            minutes: moved,
            events,
            stopped,
        }
    }

    /// True when this call is passive time that the mode or the pause ignores.
    fn elapsed_is_idle(&self, kind: AdvanceKind) -> bool {
        match kind {
            AdvanceKind::Elapsed => self.time_mode != TimeMode::RealTime || self.clock_paused,
            AdvanceKind::Spend | AdvanceKind::Pass => false,
        }
    }

    /// How many of the next `within` minutes to move before [`World::pause_hook`].
    ///
    /// On a road, travel and wait check at midnight, which is once for each
    /// calendar day the order spends there. A later event can stop sooner by
    /// returning a smaller step from here.
    fn minutes_before_pause_check(&self, kind: AdvanceKind, within: u32) -> u32 {
        let on_the_road = match kind {
            AdvanceKind::Pass => self.journey.is_some(),
            AdvanceKind::Elapsed | AdvanceKind::Spend => false,
        };
        if on_the_road {
            within.min(self.minutes_until_midnight())
        } else {
            within
        }
    }

    /// An event that needs the player at the current moment, if one is due.
    ///
    /// A road encounter during travel or wait is the one event that stops the
    /// clock today. A later event stops this same function by returning here.
    fn pause_hook(&mut self, kind: AdvanceKind) -> Option<Event> {
        if let Some(event) = self.road_encounter_pause(kind) {
            return Some(event);
        }
        None
    }

    /// Once for each calendar day a travel or wait spends on the road.
    fn road_encounter_pause(&mut self, kind: AdvanceKind) -> Option<Event> {
        match kind {
            AdvanceKind::Pass => {}
            AdvanceKind::Elapsed | AdvanceKind::Spend => return None,
        }
        let journey = self.journey?;
        if self.rng.chance(journey.encounter_chance) {
            Some(Event::RoadEncounter {
                from: journey.from,
                to: journey.to,
                on: self.date,
            })
        } else {
            None
        }
    }

    /// Adds minutes to the clock. Crossing 60 rolls the hour. Crossing 24
    /// hours rolls the date and records [`Event::DayPassed`]. Reaching noon
    /// turns the sky to the coming night. Callers do not bump the date or the
    /// hour themselves.
    fn add_minutes(&mut self, minutes: u32, events: &mut Vec<Event>) {
        if minutes == 0 {
            return;
        }
        let day = u64::from(HOURS_PER_DAY) * u64::from(MINUTES_PER_HOUR);
        let now = u64::from(self.hour) * u64::from(MINUTES_PER_HOUR) + u64::from(self.minute);
        let total = now + u64::from(minutes);
        for _ in 0..noons_crossed(now, total) {
            self.sky.turn();
        }
        let days = (total / day) as u32;
        let rest = total % day;
        self.hour = (rest / u64::from(MINUTES_PER_HOUR)) as u32;
        self.minute = (rest % u64::from(MINUTES_PER_HOUR)) as u32;
        for _ in 0..days {
            self.date = self.date.next_day();
            events.push(Event::DayPassed { date: self.date });
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
        let mut events = vec![Event::Slept {
            at,
            site,
            on,
            fatigue,
            copper,
        }];
        events.extend(spent);
        let moved = self.advance(hours_as_minutes(INN_HOURS), AdvanceKind::Spend);
        events.extend(moved.events);
        Ok(events)
    }

    /// True when every party member who has a sheet can pay `cost`.
    fn can_afford_fatigue(&self, cost: u32) -> bool {
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
        let at = self.party_at;
        self.camped = true;
        let mut events = if let Some(journey) = self.journey {
            vec![Event::CampedOnRoad {
                from: journey.from,
                to: journey.to,
                on,
            }]
        } else {
            vec![Event::Camped { at, on }]
        };
        let moved = self.advance(hours_as_minutes(MAKE_CAMP_HOURS), AdvanceKind::Spend);
        events.extend(moved.events);
        Ok(events)
    }

    /// Sleep in the pitched camp for [`REST_HOURS`]. This is the night:
    /// supplies are spent, consumed effects fire, and fatigue is restored as
    /// on a camped road. The camp stays pitched. The clock moves eight hours,
    /// and the date rolls only when those hours cross midnight.
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
        let moved = self.advance(hours_as_minutes(REST_HOURS), AdvanceKind::Spend);
        events.extend(moved.events);
        Ok(events)
    }

    fn pack_up(&mut self) -> Result<Vec<Event>, CommandError> {
        if !self.camped {
            return Err(CommandError::NotCamped);
        }
        let on = self.date;
        let at = self.party_at;
        self.camped = false;
        let mut events = vec![Event::PackedUp { at, on }];
        let moved = self.advance(hours_as_minutes(PACK_UP_HOURS), AdvanceKind::Spend);
        events.extend(moved.events);
        Ok(events)
    }

    /// Jump the clock forward. Refuses 0 and anything above 24 without
    /// changing the world. On the road, [`World::advance`] stops at a road
    /// encounter and the remaining hours are not spent.
    fn wait(&mut self, hours: u32) -> Result<Vec<Event>, CommandError> {
        if !(MIN_WAIT_HOURS..=MAX_WAIT_HOURS).contains(&hours) {
            return Err(CommandError::BadWait);
        }
        let moved = self.advance(hours_as_minutes(hours), AdvanceKind::Pass);
        let spent = moved.minutes / MINUTES_PER_HOUR;
        let mut events = moved.events;
        events.push(Event::Waited {
            hours: spent,
            on: self.date,
            hour: self.hour,
            minute: self.minute,
        });
        Ok(events)
    }

    /// Pass one hour. Turn-based time uses this when no other command spends it.
    fn step(&mut self) -> Result<Vec<Event>, CommandError> {
        let moved = self.advance(hours_as_minutes(STEP_HOURS), AdvanceKind::Spend);
        let mut events = moved.events;
        events.push(Event::Stepped {
            on: self.date,
            hour: self.hour,
            minute: self.minute,
        });
        Ok(events)
    }

    fn set_time_mode(&mut self, mode: TimeMode) -> Result<Vec<Event>, CommandError> {
        self.time_mode = mode;
        Ok(vec![Event::TimeModeChanged { mode }])
    }

    fn set_time_scale(&mut self, scale: u32) -> Result<Vec<Event>, CommandError> {
        if !(MIN_TIME_SCALE..=MAX_TIME_SCALE).contains(&scale) {
            return Err(CommandError::BadTimeScale);
        }
        if self.time_scale == scale {
            return Ok(Vec::new());
        }
        self.time_scale = scale;
        Ok(vec![Event::TimeScaleChanged { scale }])
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

/// How many times the clock reaches noon going from minute `from` to minute
/// `to` of the same day, counting `to` itself but not `from`. `to` may run
/// past the end of the day.
fn noons_crossed(from: u64, to: u64) -> u64 {
    let day = u64::from(HOURS_PER_DAY) * u64::from(MINUTES_PER_HOUR);
    let noon = u64::from(SKY_TURNS_AT_HOUR) * u64::from(MINUTES_PER_HOUR);
    // Count noons up to a minute, shifted so the count never goes negative.
    let up_to = |m: u64| (m + day - noon) / day;
    up_to(to) - up_to(from)
}

fn hours_as_minutes(hours: u32) -> u32 {
    hours.saturating_mul(MINUTES_PER_HOUR)
}

/// Turns a place name into a key: "Old Mill Ford" becomes "old-mill-ford".
fn key_for(name: &str) -> String {
    name.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
