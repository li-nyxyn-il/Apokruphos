//! A world to try one person's conversation in.
//!
//! The editor tests a person without touching the real data: it builds the
//! world the main file and the people files would make, with the person's
//! file in place, then stands the party in front of them.

use crate::calendar::HOURS_PER_DAY;
use crate::character::{Role, MAX_ATTITUDE, MIN_ATTITUDE};
use crate::data::{load_world_with_people, with_person_file, DataError, PersonFile};
use crate::faction::{FactionId, MAX_HOSTILITY, MIN_HOSTILITY};
use crate::ids::{PersonId, PlaceId};
use crate::world::{TimeMode, World};
use std::collections::BTreeSet;

/// How the editor sets the scene before a test conversation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SandboxSetup {
    /// The person's attitude toward the player, from 0 to 100.
    pub attitude: u32,
    /// Their primary faction's hostility toward the player, from 0 to 100.
    /// Ignored for someone whose primary faction is the player's.
    pub hostility: u32,
    /// Whether they travel with the party.
    pub follower: bool,
    /// The world flags that are set.
    pub flags: BTreeSet<String>,
    /// The hour of the day, from 0 to 23.
    pub hour: u32,
}

/// Builds the world with `file` in place and the party standing with them,
/// ready for [`crate::Command::Interact`]. Returns the world and the person's
/// ID. The text passed in is never changed.
///
/// The party stands at the person's place and site. They are in the party
/// only when `setup.follower` says so, and then carry the Player faction.
/// The clock runs by turns, so nothing moves while the editor looks.
///
/// The party is moved to the person's own `at` and site, whatever the file
/// says about `in_party`: the point is to stand the party beside them. A
/// non-follower with no site of their own is given the place's gate, and the
/// party stands there too, so they are always nearby. A sheet whose role is
/// the player's is left as it is, and `Interact` then answers `NotAnNpc`.
pub fn sandbox(
    main: &str,
    people: &[(String, String)],
    file: &PersonFile,
    setup: &SandboxSetup,
) -> Result<(World, PersonId), DataError> {
    if file.sheet.is_none() {
        return Err(DataError::NoSheet(file.key.clone()));
    }
    let mut world = load_world_with_people(main, &with_person_file(people, file))?;
    let index = world
        .people
        .iter()
        .position(|p| p.key == file.key)
        .expect("the file was loaded, so the person is there");
    let id = PersonId(index as u32);

    // Where they stand. A follower with no place of their own is with the
    // party wherever it started.
    let place = world
        .places
        .iter()
        .position(|p| p.key == file.at)
        .map(|i| PlaceId(i as u32))
        .unwrap_or(world.party_at);
    let own_site = file
        .site
        .as_deref()
        .and_then(|key| world.places[place.index()].find_site(key));
    let gate = world.places[place.index()].gate_for(None);
    world.party_at = place;
    world.party_site = own_site.or(gate);
    for person in world.people.iter_mut().filter(|p| p.in_party) {
        person.at = place;
    }
    let person = &mut world.people[index];
    person.at = place;
    person.in_party = setup.follower;
    person.site = if setup.follower {
        None
    } else {
        own_site.or(gate)
    };
    if let Some(sheet) = &mut person.sheet {
        if matches!(sheet.role, Role::Npc { .. }) {
            sheet.role = Role::Npc {
                follower: setup.follower,
                attitude: setup.attitude.clamp(MIN_ATTITUDE, MAX_ATTITUDE),
            };
        }
        sheet.adopt_player_faction();
        let primary = sheet.primary;
        if primary != FactionId::Player {
            let index = setup.hostility.clamp(MIN_HOSTILITY, MAX_HOSTILITY);
            world.hostility.set(primary, FactionId::Player, index);
        }
    }

    world.flags = setup.flags.clone();
    world.hour = setup.hour.min(HOURS_PER_DAY - 1);
    world.minute = 0;
    world.time_mode = TimeMode::TurnBased;
    world.clock_paused = false;
    world.journey = None;
    world.camped = false;
    world.conversation = None;
    world.conversation_node = None;
    Ok((world, id))
}
