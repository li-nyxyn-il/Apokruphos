//! A character: who they are, what they can endure, and what they carry.
//!
//! The same sheet is used for the player, for NPCs and for followers. [`Role`]
//! says which. A follower is an NPC with that status set. A character's name
//! is the [`crate::Person`] they belong to. A person named only as cast can
//! still leave the sheet empty.

use crate::faction::FactionId;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Youngest age the player can be. An NPC may be younger.
pub const MIN_AGE: u32 = 16;
/// Oldest age the player can be. An NPC may be older.
pub const MAX_AGE: u32 = 40;

/// Health a new character starts with.
pub const BASE_HEALTH: u32 = 100;
/// Fatigue a new character starts with.
pub const BASE_FATIGUE: u32 = 100;
/// Resilience a new character starts with.
pub const BASE_RESILIENCE: u32 = 100;
/// Level a new character starts at.
pub const BASE_LEVEL: u32 = 1;

/// Lowest attitude an NPC can hold toward the player. 0 is set against them.
pub const MIN_ATTITUDE: u32 = 0;
/// Highest attitude. 100 is devoted to the player.
pub const MAX_ATTITUDE: u32 = 100;
/// Attitude a new NPC starts at: neither set against the player nor devoted.
pub const BASE_ATTITUDE: u32 = 50;

/// The colour of a person's blood. The Neck sorts people by it, and it is
/// the world's notion of race.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Blood {
    /// The human baseline of the Neck.
    Red,
    /// Sky-coloured skin and inverted eyes. The aristocratic "blue blood"
    /// is the metaphor; this is the fact.
    Blue,
    /// The people of Kamlor and the far west.
    Black,
}

impl Blood {
    pub const ALL: [Blood; 3] = [Blood::Red, Blood::Blue, Blood::Black];

    pub fn name(self) -> &'static str {
        match self {
            Blood::Red => "red",
            Blood::Blue => "blue",
            Blood::Black => "black",
        }
    }
}

/// The people a character comes from.
///
/// These are the nine peoples in the ethnicity notes. Aphlian and Maphlian
/// are spelled as the bible and the dictionary spell them; those two notes
/// shorten the names to Aphian and Maphian, and the Yhlmahni note to Ylhmahi.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Heritage {
    Aphlian,
    Maphlian,
    Shentzi,
    Ptormic,
    Vhinhili,
    Yhlgori,
    Yhlmahni,
    Theylix,
    Zechi,
}

impl Heritage {
    pub const ALL: [Heritage; 9] = [
        Heritage::Aphlian,
        Heritage::Maphlian,
        Heritage::Shentzi,
        Heritage::Ptormic,
        Heritage::Vhinhili,
        Heritage::Yhlgori,
        Heritage::Yhlmahni,
        Heritage::Theylix,
        Heritage::Zechi,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Heritage::Aphlian => "Aphlian",
            Heritage::Maphlian => "Maphlian",
            Heritage::Shentzi => "Shentzi",
            Heritage::Ptormic => "Ptormic",
            Heritage::Vhinhili => "Vhinhili",
            Heritage::Yhlgori => "Yhlgori",
            Heritage::Yhlmahni => "Yhlmahni",
            Heritage::Theylix => "Theylix",
            Heritage::Zechi => "Zechi",
        }
    }
}

/// A birth sign. The Akstada is a deck of twenty-one constellation cards,
/// and a person is born under one of them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Akstada {
    Thief,
    Forgotten,
    Harbinger,
    Raven,
    Crow,
    BloodSage,
    /// The notes also spell this Chizel.
    Chisel,
    Dragon,
    Lamb,
    Wretched,
    Vagybonde,
    Pylgrym,
    Wraith,
    WillowMan,
    Wall,
    Flame,
    Sprait,
    Bastion,
    Acolyte,
    Fox,
    Hare,
}

impl Akstada {
    pub const ALL: [Akstada; 21] = [
        Akstada::Thief,
        Akstada::Forgotten,
        Akstada::Harbinger,
        Akstada::Raven,
        Akstada::Crow,
        Akstada::BloodSage,
        Akstada::Chisel,
        Akstada::Dragon,
        Akstada::Lamb,
        Akstada::Wretched,
        Akstada::Vagybonde,
        Akstada::Pylgrym,
        Akstada::Wraith,
        Akstada::WillowMan,
        Akstada::Wall,
        Akstada::Flame,
        Akstada::Sprait,
        Akstada::Bastion,
        Akstada::Acolyte,
        Akstada::Fox,
        Akstada::Hare,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Akstada::Thief => "the Thief",
            Akstada::Forgotten => "the Forgotten",
            Akstada::Harbinger => "the Harbinger",
            Akstada::Raven => "the Raven",
            Akstada::Crow => "the Crow",
            Akstada::BloodSage => "the Blood Sage",
            Akstada::Chisel => "the Chisel",
            Akstada::Dragon => "the Dragon",
            Akstada::Lamb => "the Lamb",
            Akstada::Wretched => "the Wretched",
            Akstada::Vagybonde => "the Vagybonde",
            Akstada::Pylgrym => "the Pylgrym",
            Akstada::Wraith => "the Wraith",
            Akstada::WillowMan => "the Willow Man",
            Akstada::Wall => "the Wall",
            Akstada::Flame => "the Flame",
            Akstada::Sprait => "the Sprait",
            Akstada::Bastion => "the Bastion",
            Akstada::Acolyte => "the Acolyte",
            Akstada::Fox => "the Fox",
            Akstada::Hare => "the Hare",
        }
    }
}

/// A tongue the character can speak.
///
/// Vinhili is the mountain dialects; the people who speak them are the
/// Vhinhili. Old Shentz and Shentz are the two ages of the valley's tongue.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tongue {
    Aphlian,
    Maphlian,
    Shentz,
    OldShentz,
    Ptormyk,
    Vinhili,
    Yhlgori,
    Ecumene,
    Koloudian,
    Kamloran,
}

impl Tongue {
    pub const ALL: [Tongue; 10] = [
        Tongue::Aphlian,
        Tongue::Maphlian,
        Tongue::Shentz,
        Tongue::OldShentz,
        Tongue::Ptormyk,
        Tongue::Vinhili,
        Tongue::Yhlgori,
        Tongue::Ecumene,
        Tongue::Koloudian,
        Tongue::Kamloran,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Tongue::Aphlian => "Aphlian",
            Tongue::Maphlian => "Maphlian",
            Tongue::Shentz => "Shentz",
            Tongue::OldShentz => "Old Shentz",
            Tongue::Ptormyk => "Ptormyk",
            Tongue::Vinhili => "Vinhili",
            Tongue::Yhlgori => "Yhlgori",
            Tongue::Ecumene => "Ecumene",
            Tongue::Koloudian => "Koloudian",
            Tongue::Kamloran => "Kamloran",
        }
    }
}

/// Whether the character is the player or an NPC. A follower is an NPC.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Player,
    /// `follower` is true once this NPC travels in the player's service.
    /// `attitude` is their personal stance toward the player, from 0 to 100.
    /// Together with their faction's hostility index it gives their
    /// friendliness; see [`crate::npc`].
    Npc {
        follower: bool,
        #[serde(default = "base_attitude")]
        attitude: u32,
    },
}

fn base_attitude() -> u32 {
    BASE_ATTITUDE
}

impl Role {
    pub fn name(self) -> &'static str {
        match self {
            Role::Player => "player",
            Role::Npc { .. } => "npc",
        }
    }

    pub fn is_player(self) -> bool {
        matches!(self, Role::Player)
    }

    pub fn is_follower(self) -> bool {
        matches!(self, Role::Npc { follower: true, .. })
    }

    /// The NPC's attitude toward the player. The player has none.
    pub fn attitude(self) -> Option<u32> {
        match self {
            Role::Npc { attitude, .. } => Some(attitude),
            Role::Player => None,
        }
    }
}

/// Weight of one food, water, wood or copper. Nothing reads this yet.
pub const SUPPLY_WEIGHT: u32 = 1;
/// What one food, water, wood or copper is worth, in copper. Nothing reads this yet.
pub const SUPPLY_VALUE: u32 = 1;
/// How much use one food, water, wood or copper has left. Nothing reads this yet.
pub const SUPPLY_DURABILITY: u32 = 1;

fn supply_stat() -> u32 {
    SUPPLY_WEIGHT
}

/// How rare an item is. The levels run from Basic to Apocryphal.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Rarity {
    #[default]
    Basic,
    Common,
    Notable,
    Great,
    Legendary,
    Mythic,
    Apocryphal,
}

impl Rarity {
    pub const ALL: [Rarity; 7] = [
        Rarity::Basic,
        Rarity::Common,
        Rarity::Notable,
        Rarity::Great,
        Rarity::Legendary,
        Rarity::Mythic,
        Rarity::Apocryphal,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Rarity::Basic => "Basic",
            Rarity::Common => "Common",
            Rarity::Notable => "Notable",
            Rarity::Great => "Great",
            Rarity::Legendary => "Legendary",
            Rarity::Mythic => "Mythic",
            Rarity::Apocryphal => "Apocryphal",
        }
    }
}

/// What a stack is. Food, water, wood and copper are the supplies. Anything
/// else is goods. Goods may name one equipment slot they fit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemType {
    Food,
    Water,
    Wood,
    Copper,
    Goods,
}

impl ItemType {
    pub fn label(self) -> &'static str {
        match self {
            ItemType::Food => "Food",
            ItemType::Water => "Water",
            ItemType::Wood => "Wood",
            ItemType::Copper => "Copper",
            ItemType::Goods => "Goods",
        }
    }

    /// True for the four supplies. Goods are not spent by camp or an inn.
    pub fn is_supply(self) -> bool {
        match self {
            ItemType::Food | ItemType::Water | ItemType::Wood | ItemType::Copper => true,
            ItemType::Goods => false,
        }
    }
}

/// Where one item can be worn. A character has one of each, and each holds
/// one item.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EquipSlot {
    Shoes,
    Legs,
    Body,
    Arms,
    Gloves,
    Head,
    Back,
    MainHand,
    OffHand,
    Robes,
    Neck,
}

impl EquipSlot {
    pub const ALL: [EquipSlot; 11] = [
        EquipSlot::Shoes,
        EquipSlot::Legs,
        EquipSlot::Body,
        EquipSlot::Arms,
        EquipSlot::Gloves,
        EquipSlot::Head,
        EquipSlot::Back,
        EquipSlot::MainHand,
        EquipSlot::OffHand,
        EquipSlot::Robes,
        EquipSlot::Neck,
    ];

    pub fn name(self) -> &'static str {
        match self {
            EquipSlot::Shoes => "shoes",
            EquipSlot::Legs => "legs",
            EquipSlot::Body => "body",
            EquipSlot::Arms => "arms",
            EquipSlot::Gloves => "gloves",
            EquipSlot::Head => "head",
            EquipSlot::Back => "back",
            EquipSlot::MainHand => "main hand",
            EquipSlot::OffHand => "off hand",
            EquipSlot::Robes => "robes",
            EquipSlot::Neck => "neck",
        }
    }

    /// Reads a slot name. "main-hand" and "main hand" are the main hand.
    pub fn from_name(text: &str) -> Option<EquipSlot> {
        let text = text.trim().to_lowercase().replace('-', " ");
        match text.as_str() {
            "shoes" => Some(EquipSlot::Shoes),
            "legs" => Some(EquipSlot::Legs),
            "body" => Some(EquipSlot::Body),
            "arms" => Some(EquipSlot::Arms),
            "gloves" => Some(EquipSlot::Gloves),
            "head" => Some(EquipSlot::Head),
            "back" => Some(EquipSlot::Back),
            "main" | "main hand" | "mainhand" => Some(EquipSlot::MainHand),
            "off" | "off hand" | "offhand" => Some(EquipSlot::OffHand),
            "robes" => Some(EquipSlot::Robes),
            "neck" => Some(EquipSlot::Neck),
            _ => None,
        }
    }
}

impl fmt::Display for EquipSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// When an [`Effect`] applies.
///
/// Equipment and inventory effects stay in force while their condition
/// holds. Use and consumed effects happen once, when the item is used or
/// consumed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectKind {
    /// In force while the item is worn in its slot.
    Equipment,
    /// In force while the item is carried in the pack, and not while it is worn.
    Inventory,
    /// Happens when the item is used. The item stays in the pack.
    Use,
    /// Happens when the item is consumed.
    Consumed,
}

impl EffectKind {
    pub fn name(self) -> &'static str {
        match self {
            EffectKind::Equipment => "equipped",
            EffectKind::Inventory => "carried",
            EffectKind::Use => "used",
            EffectKind::Consumed => "consumed",
        }
    }
}

/// What an [`Effect`] does. This is a closed list: there is no script.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectPayload {
    /// Adds health. A used or consumed effect adds it to the stored health.
    /// An equipment or inventory effect adds it only while that effect is in
    /// force, and leaves the stored health alone.
    RestoreHealth { amount: u32 },
    /// Raises the maximum resilience. A used or consumed effect raises the
    /// stored maximum. An equipment or inventory effect adds to the maximum
    /// only while that effect is in force.
    RaiseMaxResilience { amount: u32 },
    /// Adds vitality after `floor(sqrt(health * fatigue))`. It does not
    /// change health or fatigue. A used or consumed effect keeps the bonus.
    /// An equipment or inventory effect adds it only while that effect is in
    /// force.
    AddVitality { amount: u32 },
}

impl EffectPayload {
    /// A short description of the payload, for a host to show.
    pub fn note(self) -> String {
        match self {
            EffectPayload::RestoreHealth { amount } => format!("restores {amount} health"),
            EffectPayload::RaiseMaxResilience { amount } => {
                format!("raises max resilience by {amount}")
            }
            EffectPayload::AddVitality { amount } => format!("adds {amount} vitality"),
        }
    }
}

/// One effect on an item. An item may carry several.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Effect {
    pub kind: EffectKind,
    pub payload: EffectPayload,
}

/// One item worn in a slot. It is not in the pack.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Equipped {
    pub slot: EquipSlot,
    pub item: Item,
}

/// One stack in an inventory.
///
/// `count` is how many are carried, and it is the only field a rule spends.
/// Weight, value, durability and rarity are stored for later. Two supplies of the
/// same [`ItemType`] are one stack. Two goods are one stack when the name matches.
/// Effects are stored on the stack and saved with it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    pub name: String,
    #[serde(rename = "type")]
    pub item_type: ItemType,
    /// How heavy one of the stack is.
    #[serde(default = "supply_stat")]
    pub weight: u32,
    /// What one of the stack is worth, in copper.
    #[serde(default = "supply_stat")]
    pub value: u32,
    /// How many are carried. Spending takes from this.
    pub count: u32,
    /// How much use one of the stack has left.
    #[serde(default = "supply_stat")]
    pub durability: u32,
    /// How rare one of the stack is. Food, water, wood and copper start at
    /// [`Rarity::Basic`]. Nothing reads this yet.
    #[serde(default)]
    pub rarity: Rarity,
    /// The only slot this item fits. `None` means it cannot be equipped.
    #[serde(default)]
    pub slot: Option<EquipSlot>,
    /// Effects stored on the item. Empty when it has none.
    #[serde(default)]
    pub effects: Vec<Effect>,
}

impl Item {
    /// A supply stack with the placeholder weight, value and durability,
    /// at [`Rarity::Basic`].
    pub fn supply(item_type: ItemType, count: u32) -> Item {
        Item {
            name: item_type.label().to_string(),
            item_type,
            weight: SUPPLY_WEIGHT,
            value: SUPPLY_VALUE,
            count,
            durability: SUPPLY_DURABILITY,
            rarity: Rarity::Basic,
            slot: None,
            effects: Vec::new(),
        }
    }

    /// Something that is not a supply. The name is its identity.
    /// Rarity starts at [`Rarity::Basic`].
    pub fn goods(name: impl Into<String>, count: u32) -> Item {
        Item {
            name: name.into(),
            item_type: ItemType::Goods,
            weight: SUPPLY_WEIGHT,
            value: SUPPLY_VALUE,
            count,
            durability: SUPPLY_DURABILITY,
            rarity: Rarity::Basic,
            slot: None,
            effects: Vec::new(),
        }
    }

    /// Supplies stack by type. Goods stack by name.
    fn stacks_with(&self, other: &Item) -> bool {
        match self.item_type {
            ItemType::Goods => other.item_type == ItemType::Goods && other.name == self.name,
            kind => other.item_type == kind,
        }
    }
}

/// What is wrong with a character sheet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SheetError {
    AgeOutOfRange(u32),
    NoTongue,
    DuplicateTongue(Tongue),
    BlankBackground,
    BlankItem,
    EmptyStack(String),
    DuplicateItem(String),
    AttitudeOutOfRange(u32),
    FactionRepeated(FactionId),
    FollowerWithoutPlayerFaction,
    /// Two worn items name the same slot.
    DuplicateSlot(EquipSlot),
    /// A worn item does not fit the slot it sits in.
    ItemDoesNotFit {
        item: String,
        slot: EquipSlot,
    },
    /// A worn slot holds a stack rather than one item.
    EquippedCount(String),
}

impl fmt::Display for SheetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SheetError::AgeOutOfRange(age) => {
                write!(f, "age {age} is outside {MIN_AGE} to {MAX_AGE}")
            }
            SheetError::NoTongue => write!(f, "no tongue is known"),
            SheetError::DuplicateTongue(tongue) => {
                write!(f, "the tongue {} is listed twice", tongue.name())
            }
            SheetError::BlankBackground => write!(f, "the background is blank"),
            SheetError::BlankItem => write!(f, "an item has no name"),
            SheetError::EmptyStack(name) => write!(f, "none of \"{name}\" are carried"),
            SheetError::DuplicateItem(key) => write!(f, "\"{key}\" is carried twice"),
            SheetError::AttitudeOutOfRange(attitude) => {
                write!(
                    f,
                    "attitude {attitude} is outside {MIN_ATTITUDE} to {MAX_ATTITUDE}"
                )
            }
            SheetError::FactionRepeated(faction) => {
                write!(f, "the faction {} is listed twice", faction.name())
            }
            SheetError::FollowerWithoutPlayerFaction => {
                write!(f, "a follower does not carry the Player faction")
            }
            SheetError::DuplicateSlot(slot) => {
                write!(f, "two items are worn on the {slot}")
            }
            SheetError::ItemDoesNotFit { item, slot } => {
                write!(f, "\"{item}\" does not fit the {slot}")
            }
            SheetError::EquippedCount(name) => {
                write!(f, "\"{name}\" must be one item when it is worn")
            }
        }
    }
}

/// The stats of one character.
///
/// Health, fatigue, resilience and level start at the `BASE_` values. A data
/// file may set them to something else; omitting them keeps the base.
/// Vitality, willpower and fortitude are not stored. Each is the integer
/// floor of the square root of the product of two of those body stats.
/// An effect may add vitality after that floor, or raise the maximum
/// resilience, without changing the two stats the floor was taken from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Character {
    /// Player, or an NPC who may also be a follower.
    pub role: Role,
    pub age: u32,
    pub blood: Blood,
    pub heritage: Heritage,
    pub akstada: Akstada,
    /// How much harm the body can still take. A new character starts at
    /// [`BASE_HEALTH`].
    #[serde(default = "base_health")]
    pub health: u32,
    /// How much road the body has left in it. A new character starts at
    /// [`BASE_FATIGUE`], and rest never raises it above that. A travel order
    /// past eight hours spends it. Camping on the road and a night at an inn
    /// or residence restore some.
    #[serde(default = "base_fatigue")]
    pub fatigue: u32,
    /// Set when a travel order has left fatigue below 50. While it is set,
    /// camping on the road will not raise fatigue above 50. A night at an
    /// inn or a residence clears it.
    #[serde(default)]
    pub road_rest_debt: bool,
    /// How well the character weathers what the road does to them. A new
    /// character starts at [`BASE_RESILIENCE`]. This is the current value.
    /// [`Character::max_resilience`] is how high it can sit.
    #[serde(default = "base_resilience")]
    pub resilience: u32,
    /// Stored maximum resilience, before equipment and inventory effects.
    /// A new character starts at [`BASE_RESILIENCE`]. A used or consumed
    /// effect can raise it. Omitting it in a data file keeps the base.
    #[serde(default = "base_resilience")]
    pub resilience_max: u32,
    /// Vitality added by a used or consumed effect. Equipment and inventory
    /// vitality is computed, not stored here.
    #[serde(default)]
    pub vitality_bonus: u32,
    /// Tongues the character can speak, with no repeats.
    pub tongues: Vec<Tongue>,
    /// The life before the road. There is no closed list of backgrounds yet,
    /// so this is plain text.
    pub background: String,
    /// The faction this character belongs to first.
    #[serde(default = "default_primary")]
    pub primary: FactionId,
    /// Further factions, not repeating [`Character::primary`]. A follower
    /// carries the Player faction here unless it is already their primary.
    #[serde(default)]
    pub secondary: Vec<FactionId>,
    #[serde(default = "base_level")]
    pub level: u32,
    /// The pack. Equipped items are not in this list.
    #[serde(default)]
    pub inventory: Vec<Item>,
    /// Items worn in slots. Each slot holds one item. Nothing here is counted
    /// as carried.
    #[serde(default)]
    pub equipped: Vec<Equipped>,
}

fn base_health() -> u32 {
    BASE_HEALTH
}
fn base_fatigue() -> u32 {
    BASE_FATIGUE
}
fn base_resilience() -> u32 {
    BASE_RESILIENCE
}
fn base_level() -> u32 {
    BASE_LEVEL
}
fn default_primary() -> FactionId {
    FactionId::Player
}

/// Integer floor of `sqrt(x * y)`. The product is widened so two large
/// stats cannot overflow a `u32`.
fn floor_sqrt_product(x: u32, y: u32) -> u32 {
    let product = u64::from(x) * u64::from(y);
    // `sqrt(u32::MAX * u32::MAX)` is `u32::MAX`, so the root fits.
    product.isqrt() as u32
}

impl Character {
    /// A new character at full health, fatigue and resilience, level 1,
    /// carrying nothing.
    pub fn new(
        role: Role,
        age: u32,
        blood: Blood,
        heritage: Heritage,
        akstada: Akstada,
        tongues: Vec<Tongue>,
        background: impl Into<String>,
        primary: FactionId,
        secondary: Vec<FactionId>,
    ) -> Result<Character, SheetError> {
        let mut sheet = Character {
            role,
            age,
            blood,
            heritage,
            akstada,
            health: base_health(),
            fatigue: base_fatigue(),
            road_rest_debt: false,
            resilience: base_resilience(),
            resilience_max: base_resilience(),
            vitality_bonus: 0,
            tongues,
            background: background.into(),
            primary,
            secondary,
            level: base_level(),
            inventory: Vec::new(),
            equipped: Vec::new(),
        };
        sheet.adopt_player_faction();
        sheet.check()?;
        Ok(sheet)
    }

    /// Reports the first thing wrong with the sheet.
    pub fn check(&self) -> Result<(), SheetError> {
        if self.role.is_player() && !(MIN_AGE..=MAX_AGE).contains(&self.age) {
            return Err(SheetError::AgeOutOfRange(self.age));
        }
        if self.tongues.is_empty() {
            return Err(SheetError::NoTongue);
        }
        let mut seen_tongues: Vec<Tongue> = Vec::new();
        for tongue in &self.tongues {
            if seen_tongues.contains(tongue) {
                return Err(SheetError::DuplicateTongue(*tongue));
            }
            seen_tongues.push(*tongue);
        }
        if self.background.trim().is_empty() {
            return Err(SheetError::BlankBackground);
        }
        let mut seen_items: Vec<&Item> = Vec::new();
        for item in &self.inventory {
            if item.name.trim().is_empty() {
                return Err(SheetError::BlankItem);
            }
            if item.count == 0 {
                return Err(SheetError::EmptyStack(item.name.clone()));
            }
            if seen_items.iter().any(|held| held.stacks_with(item)) {
                return Err(SheetError::DuplicateItem(item.name.clone()));
            }
            seen_items.push(item);
        }
        let mut seen_slots: Vec<EquipSlot> = Vec::new();
        for worn in &self.equipped {
            if worn.item.name.trim().is_empty() {
                return Err(SheetError::BlankItem);
            }
            if worn.item.count != 1 {
                return Err(SheetError::EquippedCount(worn.item.name.clone()));
            }
            match worn.item.slot {
                Some(fits) if fits == worn.slot => {}
                _ => {
                    return Err(SheetError::ItemDoesNotFit {
                        item: worn.item.name.clone(),
                        slot: worn.slot,
                    })
                }
            }
            if seen_slots.contains(&worn.slot) {
                return Err(SheetError::DuplicateSlot(worn.slot));
            }
            seen_slots.push(worn.slot);
        }
        if let Some(attitude) = self.role.attitude() {
            if !(MIN_ATTITUDE..=MAX_ATTITUDE).contains(&attitude) {
                return Err(SheetError::AttitudeOutOfRange(attitude));
            }
        }
        let mut seen = vec![self.primary];
        for faction in &self.secondary {
            if seen.contains(faction) {
                return Err(SheetError::FactionRepeated(*faction));
            }
            seen.push(*faction);
        }
        if self.role.is_follower() && !self.belongs_to(FactionId::Player) {
            return Err(SheetError::FollowerWithoutPlayerFaction);
        }
        Ok(())
    }

    /// Vitality, from health and fatigue: `floor(sqrt(health * fatigue))`,
    /// plus vitality from effects.
    ///
    /// Health 100 and fatigue 100 is 100 before any effect. An equipment or
    /// inventory effect adds its vitality after that floor, and only while
    /// the effect is in force. It does not change health or fatigue. A used
    /// or consumed effect adds to [`Character::vitality_bonus`] and stays.
    pub fn vitality(&self) -> u32 {
        let passive = self.passive();
        floor_sqrt_product(self.health, self.fatigue)
            .saturating_add(self.vitality_bonus)
            .saturating_add(passive.vitality)
    }

    /// The maximum resilience: the stored maximum, plus equipment and
    /// inventory effects that are in force now.
    ///
    /// Raising the maximum does not change [`Character::resilience`].
    pub fn max_resilience(&self) -> u32 {
        self.resilience_max
            .saturating_add(self.passive().max_resilience)
    }

    /// Stored health plus any equipment or inventory effect that restores
    /// health while it is in force. The stored health is left as it is.
    pub fn effective_health(&self) -> u32 {
        self.health.saturating_add(self.passive().health)
    }

    /// The item worn in `slot`, if any.
    pub fn in_slot(&self, slot: EquipSlot) -> Option<&Item> {
        self.equipped
            .iter()
            .find(|worn| worn.slot == slot)
            .map(|worn| &worn.item)
    }

    /// Equipment and inventory effects that are in force, counted once per
    /// item in a stack.
    fn passive(&self) -> Passive {
        let mut passive = Passive::default();
        for item in &self.inventory {
            passive.add(item, EffectKind::Inventory);
        }
        for worn in &self.equipped {
            passive.add(&worn.item, EffectKind::Equipment);
        }
        passive
    }

    /// Applies a used or consumed payload to the stored stats.
    pub(crate) fn apply_instant(&mut self, payload: EffectPayload) {
        match payload {
            EffectPayload::RestoreHealth { amount } => {
                self.health = self.health.saturating_add(amount);
            }
            EffectPayload::RaiseMaxResilience { amount } => {
                self.resilience_max = self.resilience_max.saturating_add(amount);
            }
            EffectPayload::AddVitality { amount } => {
                self.vitality_bonus = self.vitality_bonus.saturating_add(amount);
            }
        }
    }

    /// Willpower, from fatigue and resilience: `floor(sqrt(fatigue * resilience))`.
    ///
    /// Fatigue 50 and resilience 110 is 74. Computed from the sheet, not stored.
    pub fn willpower(&self) -> u32 {
        floor_sqrt_product(self.fatigue, self.resilience)
    }

    /// Fortitude, from resilience and health: `floor(sqrt(resilience * health))`.
    ///
    /// Computed from the sheet, not stored.
    pub fn fortitude(&self) -> u32 {
        floor_sqrt_product(self.resilience, self.health)
    }

    /// True when the faction is this character's primary or one of their
    /// secondaries.
    pub fn belongs_to(&self, faction: FactionId) -> bool {
        self.primary == faction || self.secondary.contains(&faction)
    }

    /// Gives a follower the Player faction as a secondary, unless they
    /// already belong to it.
    pub fn adopt_player_faction(&mut self) {
        if self.role.is_follower() && !self.belongs_to(FactionId::Player) {
            self.secondary.push(FactionId::Player);
        }
    }

    /// The primary faction's name.
    pub fn faction_label(&self) -> &str {
        self.primary.name()
    }

    /// The secondary factions, in the order they were taken.
    pub fn secondary_list(&self) -> String {
        self.secondary
            .iter()
            .map(|faction| faction.name())
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// The tongues as a single line, in the order they are known.
    pub fn tongue_list(&self) -> String {
        self.tongues
            .iter()
            .map(|tongue| tongue.name())
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Adds `item` to the inventory. A stack of the same supply, or goods of
    /// the same name, grows by `count`. A count of zero adds nothing.
    /// Weight, value, durability and rarity stay as the stack already stored them.
    pub fn carry(&mut self, item: Item) {
        if item.count == 0 {
            return;
        }
        if let Some(stack) = self
            .inventory
            .iter_mut()
            .find(|held| held.stacks_with(&item))
        {
            stack.count = stack.count.saturating_add(item.count);
        } else {
            self.inventory.push(item);
        }
    }

    /// Removes up to `count` of `item_type`. Returns how many were removed.
    /// A stack that reaches zero is dropped. Worn items are not taken.
    pub fn take(&mut self, item_type: ItemType, mut count: u32) -> u32 {
        let mut taken = 0;
        while count > 0 && self.take_one(item_type).is_some() {
            taken += 1;
            count -= 1;
        }
        taken
    }

    /// Removes one of `item_type` from the pack. The returned item has a
    /// count of 1 and the stack's effects. Worn items are not taken.
    pub(crate) fn take_one(&mut self, item_type: ItemType) -> Option<Item> {
        let pos = self
            .inventory
            .iter()
            .position(|item| item.item_type == item_type && item.count > 0)?;
        Some(self.split_off(pos))
    }

    /// Removes one item of this name from the pack. Worn items are not taken.
    pub(crate) fn take_named(&mut self, name: &str) -> Option<Item> {
        let pos = self
            .inventory
            .iter()
            .position(|item| item.name == name && item.count > 0)?;
        Some(self.split_off(pos))
    }

    /// Takes one from the stack at `pos`. The caller has found `pos`.
    fn split_off(&mut self, pos: usize) -> Item {
        let mut taken = self.inventory[pos].clone();
        taken.count = 1;
        if self.inventory[pos].count == 1 {
            self.inventory.remove(pos);
        } else {
            self.inventory[pos].count -= 1;
        }
        taken
    }

    /// How many of `item_type` are carried in the pack. Worn items are not
    /// counted.
    pub fn holding(&self, item_type: ItemType) -> u32 {
        self.inventory
            .iter()
            .filter(|item| item.item_type == item_type)
            .fold(0, |sum, item| sum.saturating_add(item.count))
    }

    /// How many goods of this name are carried in the pack. Worn items are
    /// not counted. Supplies are not goods.
    pub fn holding_goods(&self, name: &str) -> u32 {
        self.inventory
            .iter()
            .find(|item| item.item_type == ItemType::Goods && item.name == name)
            .map_or(0, |item| item.count)
    }
}

/// Equipment and inventory bonuses that are in force.
#[derive(Clone, Copy, Debug, Default)]
struct Passive {
    health: u32,
    max_resilience: u32,
    vitality: u32,
}

impl Passive {
    fn add(&mut self, item: &Item, kind: EffectKind) {
        for effect in item.effects.iter().filter(|effect| effect.kind == kind) {
            let amount = match effect.payload {
                EffectPayload::RestoreHealth { amount }
                | EffectPayload::RaiseMaxResilience { amount }
                | EffectPayload::AddVitality { amount } => amount,
            };
            let amount = amount.saturating_mul(item.count);
            match effect.payload {
                EffectPayload::RestoreHealth { .. } => {
                    self.health = self.health.saturating_add(amount);
                }
                EffectPayload::RaiseMaxResilience { .. } => {
                    self.max_resilience = self.max_resilience.saturating_add(amount);
                }
                EffectPayload::AddVitality { .. } => {
                    self.vitality = self.vitality.saturating_add(amount);
                }
            }
        }
    }
}
