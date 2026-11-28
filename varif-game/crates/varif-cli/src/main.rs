//! A command-line front end: load a world, type commands, read the events.
//!
//! Run it from the workspace folder with `cargo run`, or pass a data file:
//! `cargo run -- data/first_leg.ron`.

use std::io::{self, BufRead, Write};
use std::path::Path;
use std::time::Instant;
use std::{env, fs, process};
use varif_core::{
    game_minutes, load_world_with_people, AdvanceKind, Command, EquipSlot, Event, Itinerary,
    MemberFatigue, PersonId, Place, PlaceId, TimeMode, World, MAX_TIME_SCALE, MIN_TIME_SCALE,
};

const DEFAULT_DATA: &str = "data/first_leg.ron";
/// A map with many loops has hundreds of ways between two places. Only this
/// many, the quickest, are offered.
const MAX_WAYS: usize = 5;

const HELP: &str = "\
Commands:
  look            describe where the party is and where the roads lead
  sheet           show the player character
  go <hours> <place>
                  travel that many hours toward a place, one league an hour,
                  by key or by name; if no road runs straight there you are
                  offered the ways and asked before each leg (y/n); on a
                  longer road the party stops after those hours, and you
                  enter another number of hours or camp;
                  eight hours or fewer add no fatigue, and each extra hour
                  adds the next number, once (1, then 3, then 6);
                  when someone where you arrive could join the party, you
                  are asked whether to recruit them (y/n)
  nearby          list the NPCs at the spot the player stands, followers too
  interact <npc>  talk to a nearby NPC, by key, by name, or by its number
                  from nearby; pick a line by number, or bye to leave.
                  One too hostile to talk to would start combat instead,
                  which is not built yet
  map             list every place the roads can take the party to
  walk            list the places inside the city the party can walk to
  walk <site>     walk to one of them, by key or by name
  addplace <leagues> <name>
                  put a new place on the map, a road of that many leagues
                  from where the party stands
  camp            pitch camp; this takes an hour and does not sleep
  rest            sleep eight hours in camp; spends one food, one water
                  and one wood
  pack            pack up the camp; this takes an hour
  wait <hours>    wait from 1 to 24 hours; on the road an encounter
                  stops the wait and the rest of the order is not spent
  step            pass one hour
  mode            show whether the clock is real time or turn-based
  mode real       follow elapsed minutes from this program, while the
                  clock is not paused; the game never reads the clock itself
  mode turn       ignore elapsed minutes; time moves when a command spends
                  it, or on step
  scale           show how many game minutes pass per real minute
  scale <n>       set that, from 1 to 60; 1 matches the wall clock.
                  Camp, sleep, travel, wait, and step are not sped up
  sleep           sleep eight hours at an inn for 5 copper, or at a
                  residence for free
  write <text>    add a journal entry
  journal         read the journal
  sky             name tonight's constellations; the sky turns at noon
  equip <item> in <slot>
                  wear an item from the pack. Slots: shoes, legs, body,
                  arms, gloves, head, back, main hand, off hand, robes, neck
  unequip <slot>  put a worn item back in the pack
  use <item>      use an item in the pack; it stays there
  save <file>     write the whole world to a file
  help            show this list
  quit            leave";

fn main() {
    let path = env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_DATA.to_string());
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("Could not open {path}: {error}");
            process::exit(1);
        }
    };
    let people = match varif_core::read_people_dir(&Path::new(&path).with_file_name("people")) {
        Ok(people) => people,
        Err(error) => {
            eprintln!("Could not read the people files beside {path}: {error}");
            process::exit(1);
        }
    };
    let mut world = match load_world_with_people(&text, &people) {
        Ok(world) => world,
        Err(error) => {
            eprintln!("Could not load {path}: {error}");
            process::exit(1);
        }
    };

    println!("{HELP}\n");
    look(&world);

    let stdin = io::stdin();
    let mut asking = Asking::Nothing;
    // The stops still to come after the leg the party is walking now.
    let mut heading: Vec<PlaceId> = Vec::new();
    // Where the party was when the loop last looked, to notice an arrival.
    let mut seen_at = world.party_at;
    let mut host_clock = HostClock {
        last: Instant::now(),
        carry_ms: 0,
    };
    loop {
        pass_elapsed(&mut world, &mut host_clock);
        // Just arrived somewhere: first ask about anyone who could join.
        if world.journey.is_none() && world.party_at != seen_at {
            seen_at = world.party_at;
            let offers = world.recruit_offers();
            if !offers.is_empty() && matches!(asking, Asking::Nothing) {
                asking = Asking::Recruit(offers);
            }
        }

        // Arrived at a stop on the way somewhere further: ask about the next leg.
        if world.journey.is_none() && !heading.is_empty() && matches!(asking, Asking::Nothing) {
            asking = Asking::NextStop {
                stops: std::mem::take(&mut heading),
                first_leg: false,
                hours: 0,
            };
        }

        match &asking {
            _ if world.conversation.is_some() => {
                let options = world.dialogue_options(world.conversation.unwrap()).len();
                if options == 0 {
                    print!("Nothing to say. (bye to leave) ");
                } else {
                    print!("Say which? (1-{options}, or bye) ");
                }
            }
            // On the road the order is a number of hours, or camp. Once camped,
            // the question is whether to sleep.
            _ if world.journey.is_some() && world.camped => print!("Sleep in camp? (y/n) "),
            _ if world.journey.is_some() => {
                let left = world.journey.unwrap().leagues_left();
                print!("On the road, {left} leagues left. Hours to travel, camp, or wait? ")
            }
            Asking::WhichWay { routes, .. } => {
                print!("Which way? (1-{}, or n to stay) ", routes.len())
            }
            Asking::Recruit(offers) => {
                let name = world.person(offers[0]).map_or("them", |p| p.name.as_str());
                print!("Recruit {name} into the party? (y/n) ");
            }
            Asking::NextStop {
                stops,
                first_leg,
                hours,
            } => {
                let name = world.place_name(stops[0]);
                if *first_leg {
                    print!("Travel to {name} first, for {hours} hours? (y/n) ");
                } else {
                    print!("Continue on to {name}? Enter hours, or n to stay. ");
                }
            }
            Asking::Nothing => print!("\n> "),
        }
        io::stdout().flush().ok();

        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) | Err(_) => break, // end of input
            Ok(_) => {}
        }
        let line = line.trim();
        let answer = line.to_lowercase();
        if answer == "quit" || answer == "exit" {
            break;
        }

        if let Some(person) = world.conversation {
            match answer.as_str() {
                "bye" | "leave" | "n" | "no" => {
                    run(&mut world, Command::EndConversation);
                }
                _ => match answer.parse::<usize>() {
                    Ok(n) if n >= 1 && n <= world.dialogue_options(person).len() => {
                        if run(&mut world, Command::Say { line: n - 1 }) {
                            show_options(&world, person);
                        }
                    }
                    _ => println!("Answer with the number of a line, or bye to leave."),
                },
            }
            continue;
        }

        if world.journey.is_some() && !is_clock_command(&answer) {
            if world.camped {
                match answer.as_str() {
                    "y" | "yes" => {
                        run(&mut world, Command::Rest);
                    }
                    "n" | "no" => {
                        run(&mut world, Command::PackUp);
                    }
                    _ => println!("Answer y to sleep or n to pack up."),
                }
            } else {
                match answer.as_str() {
                    "camp" | "c" => {
                        run(&mut world, Command::MakeCamp);
                    }
                    _ => match answer.parse::<u32>() {
                        Ok(hours) if hours >= 1 => {
                            let to = world.journey.unwrap().to;
                            run(&mut world, Command::Travel { to, hours });
                        }
                        _ => println!("Enter the hours to travel, or camp."),
                    },
                }
            }
            continue;
        }

        match std::mem::replace(&mut asking, Asking::Nothing) {
            Asking::Nothing => {}
            Asking::Recruit(mut offers) => {
                match answer.as_str() {
                    "y" | "yes" => {
                        run(
                            &mut world,
                            Command::Recruit {
                                person: offers.remove(0),
                            },
                        );
                    }
                    "n" | "no" => {
                        let person = offers.remove(0);
                        let name = world.person(person).map_or("They", |p| p.name.as_str());
                        println!("{name} stays behind.");
                    }
                    _ => println!("Answer y or n."),
                }
                if !offers.is_empty() {
                    asking = Asking::Recruit(offers);
                }
                continue;
            }
            Asking::WhichWay { routes, hours } => {
                match answer.parse::<usize>() {
                    Ok(n) if (1..=routes.len()).contains(&n) => {
                        let stops = routes[n - 1].stops.clone();
                        println!(
                            "The party takes the way through {}.",
                            through(&world, &stops)
                        );
                        heading = set_out(&mut world, stops, hours);
                    }
                    _ if answer == "n" || answer == "no" => stay(&world),
                    _ => {
                        println!("Answer with the number of a way, or n to stay.");
                        asking = Asking::WhichWay { routes, hours };
                    }
                }
                continue;
            }
            Asking::NextStop {
                stops,
                first_leg,
                hours,
            } => {
                if first_leg {
                    match answer.as_str() {
                        "y" | "yes" => heading = set_out(&mut world, stops, hours),
                        "n" | "no" => stay(&world),
                        _ => {
                            println!("Answer y or n.");
                            asking = Asking::NextStop {
                                stops,
                                first_leg,
                                hours,
                            };
                        }
                    }
                } else {
                    match answer.as_str() {
                        "n" | "no" => stay(&world),
                        _ => match answer.parse::<u32>() {
                            Ok(hours) if hours >= 1 => {
                                heading = set_out(&mut world, stops, hours);
                            }
                            _ => {
                                println!("Enter the hours to travel, or n to stay.");
                                asking = Asking::NextStop {
                                    stops,
                                    first_leg,
                                    hours: 0,
                                };
                            }
                        },
                    }
                }
                continue;
            }
        }

        // Split "go Atzhaim" into the word "go" and the rest, "Atzhaim".
        let (word, rest) = match line.split_once(' ') {
            Some((word, rest)) => (word, rest.trim()),
            None => (line, ""),
        };

        match word {
            "" => {}
            "quit" | "exit" => break,
            "help" => println!("{HELP}"),
            "look" => look(&world),
            "nearby" => nearby(&world),
            "interact" | "talk" => interact(&mut world, rest),
            "sheet" => sheet(&world),
            "journal" => journal(&world),
            "sky" => sky(&world),
            "map" => map(&world),
            "walk" => walk(&mut world, rest),
            "addplace" => add_place(&mut world, rest),
            "camp" => {
                run(&mut world, Command::MakeCamp);
            }
            "rest" => {
                run(&mut world, Command::Rest);
            }
            "pack" => {
                run(&mut world, Command::PackUp);
            }
            "wait" => wait(&mut world, rest),
            "step" => {
                run(&mut world, Command::Step);
            }
            "mode" | "time" => set_mode(&mut world, rest),
            "scale" => set_scale(&mut world, &mut host_clock, rest),
            "sleep" => {
                run(&mut world, Command::Sleep);
            }
            "write" => {
                run(
                    &mut world,
                    Command::WriteJournal {
                        text: rest.to_string(),
                    },
                );
            }
            "equip" => equip(&mut world, rest),
            "unequip" => unequip(&mut world, rest),
            "use" => {
                if rest.is_empty() {
                    println!("Usage: use <item>");
                } else {
                    run(
                        &mut world,
                        Command::Use {
                            item: rest.to_string(),
                        },
                    );
                }
            }
            "go" => asking = go(&mut world, rest),
            "save" if rest.is_empty() => println!("Save to which file?"),
            "save" => match world.to_ron() {
                Ok(saved) => match fs::write(rest, saved) {
                    Ok(()) => println!("Saved to {rest}."),
                    Err(error) => println!("Could not write {rest}: {error}"),
                },
                Err(error) => println!("Could not save: {error}"),
            },
            other => println!("Unknown command \"{other}\". Type help for the list."),
        }
    }
}

/// A question the program is waiting for the player to answer.
enum Asking {
    Nothing,
    /// These people could join the party; the first is asked about now.
    Recruit(Vec<PersonId>),
    /// Several ways lead to the place; the player picks one by number.
    /// `hours` is how long the first order on that way lasts.
    WhichWay {
        routes: Vec<Itinerary>,
        hours: u32,
    },
    /// Walk to the first of these stops? The rest follow, one question each.
    NextStop {
        stops: Vec<PlaceId>,
        first_leg: bool,
        hours: u32,
    },
}

/// Sends one command to the world and prints what happened. True if the
/// world accepted it.
fn run(world: &mut World, command: Command) -> bool {
    match world.apply(command) {
        Ok(events) => {
            for event in &events {
                println!("{}", describe(world, event));
            }
            true
        }
        Err(error) => {
            println!("Refused: {error}.");
            false
        }
    }
}

/// Handles "go <hours> <place>". Walks straight there when a road joins the
/// two places; otherwise searches for the ways that lead there and asks.
fn go(world: &mut World, rest: &str) -> Asking {
    let Some((hours_text, place_text)) = rest.split_once(char::is_whitespace) else {
        println!("Say how many hours: go <hours> <place>.");
        return Asking::Nothing;
    };
    let Ok(hours) = hours_text.parse::<u32>() else {
        println!("Say how many hours: go <hours> <place>.");
        return Asking::Nothing;
    };
    if hours == 0 {
        println!("Travel at least one hour.");
        return Asking::Nothing;
    }
    let Some(to) = world.find_place(place_text) else {
        println!("There is no place called \"{place_text}\".");
        return Asking::Nothing;
    };
    let here = world.party_at;
    if to == here || world.road_between(here, to).is_some() {
        run(world, Command::Travel { to, hours });
        return Asking::Nothing;
    }

    let (from_name, to_name) = (world.place_name(here), world.place_name(to));
    let mut routes = world.routes_between(here, to);
    let hidden = routes.len().saturating_sub(MAX_WAYS);
    routes.truncate(MAX_WAYS);
    match routes.len() {
        0 => {
            println!("No road leads from {from_name} to {to_name}, by any way.");
            Asking::Nothing
        }
        1 => {
            let route = routes.remove(0);
            println!(
                "No road runs straight from {from_name} to {to_name}. The way goes through {} ({}).",
                through(world, &route.stops),
                length(&route)
            );
            Asking::NextStop {
                stops: route.stops,
                first_leg: true,
                hours,
            }
        }
        n => {
            if hidden == 0 {
                println!(
                    "No road runs straight from {from_name} to {to_name}. There are {n} ways:"
                );
            } else {
                println!(
                    "No road runs straight from {from_name} to {to_name}. The {n} quickest ways:"
                );
            }
            for (i, route) in routes.iter().enumerate() {
                println!(
                    "  {}. through {}: {}",
                    i + 1,
                    through(world, &route.stops),
                    length(route)
                );
            }
            if hidden > 0 {
                println!("  ({hidden} slower ways are not shown.)");
            }
            Asking::WhichWay { routes, hours }
        }
    }
}

/// Starts the first leg of a route with an order of `hours`. Returns the
/// stops still to come, or nothing if the world refused the leg.
fn set_out(world: &mut World, stops: Vec<PlaceId>, hours: u32) -> Vec<PlaceId> {
    let Some((&first, rest)) = stops.split_first() else {
        return Vec::new();
    };
    if run(world, Command::Travel { to: first, hours }) {
        rest.to_vec()
    } else {
        Vec::new()
    }
}

fn stay(world: &World) {
    println!("The party stays at {}.", world.place_name(world.party_at));
}

/// The places a route passes on the way, without its destination.
fn through(world: &World, stops: &[PlaceId]) -> String {
    let passed = &stops[..stops.len().saturating_sub(1)];
    passed
        .iter()
        .map(|&id| world.place_name(id))
        .collect::<Vec<_>>()
        .join(", ")
}

fn length(route: &Itinerary) -> String {
    format!("{} leagues, {} hours", route.leagues, route.hours)
}

fn describe(world: &World, event: &Event) -> String {
    match event {
        Event::Departed { from, to, .. } => format!(
            "The party leaves {} for {}.",
            world.place_name(*from),
            world.place_name(*to)
        ),
        Event::DayPassed { date } => format!("  {date}"),
        Event::RoadEncounter { .. } => "  Something meets the party on the road.".to_string(),
        Event::Traveled {
            to,
            leagues,
            leagues_left,
            hours,
            fatigue,
            ..
        } => {
            let stop = if *leagues_left == 0 {
                String::new()
            } else {
                format!(
                    " It stops with {leagues_left} leagues left to {}.",
                    world.place_name(*to)
                )
            };
            format!(
                "The party travels {leagues} leagues in {hours} hours.{stop}{}",
                fatigue_note(world, fatigue)
            )
        }
        Event::CampedOnRoad { .. } => {
            "The party makes camp beside the road. An hour passes.".to_string()
        }
        Event::Arrived { at, .. } => format!("The party arrives at {}.", world.place_name(*at)),
        Event::EnteredBy { at, gate, .. } => {
            format!("It comes in by the {}.", site_name(world.place(*at), *gate))
        }
        Event::Walked { at, from, to, .. } => {
            let place = world.place(*at);
            match from {
                Some(from) => format!(
                    "The party walks from the {} to the {}.",
                    site_name(place, *from),
                    site_name(place, *to)
                ),
                None => format!("The party walks to the {}.", site_name(place, *to)),
            }
        }
        Event::Camped { at, .. } => {
            format!(
                "The party makes camp at {}. An hour passes.",
                world.place_name(*at)
            )
        }
        Event::Rested {
            at,
            fatigue,
            spent_food,
            spent_water,
            spent_wood,
            ..
        } => format!(
            "The party sleeps eight hours in camp at {}.{}{}",
            world.place_name(*at),
            camp_note(*spent_food, *spent_water, *spent_wood),
            fatigue_note(world, fatigue)
        ),
        Event::PackedUp { at, .. } => {
            format!(
                "The party packs up the camp at {}. An hour passes.",
                world.place_name(*at)
            )
        }
        Event::Slept {
            at,
            site,
            fatigue,
            copper,
            ..
        } => {
            let paid = if *copper > 0 {
                format!(" Pays {copper} copper.")
            } else {
                String::new()
            };
            format!(
                "The party sleeps eight hours at the {} in {}.{}{}",
                site_name(world.place(*at), *site),
                world.place_name(*at),
                paid,
                fatigue_note(world, fatigue)
            )
        }
        Event::JournalWritten { .. } => "The entry is written.".to_string(),
        Event::RecruitOffered { person, .. } => format!(
            "{} is here, and could join the party.",
            person_name(world, *person)
        ),
        Event::Recruited { person, .. } => {
            format!("{} joins the party.", person_name(world, *person))
        }
        Event::ConversationOpened {
            person, standing, ..
        } => format!(
            "You speak with {} (friendliness {}).",
            person_name(world, *person),
            standing.friendliness
        ),
        Event::FlagSet { flag } => format!("Flag set: {flag}."),
        Event::FlagCleared { flag } => format!("Flag cleared: {flag}."),
        Event::ItemGiven {
            person,
            item,
            count,
        } => format!("{} gives you {count} {item}.", person_name(world, *person)),
        Event::ItemTaken {
            person,
            item,
            count,
        } => format!("You give {} {count} {item}.", person_name(world, *person)),
        Event::ItemGranted { item, count } => format!("You receive {count} {item}."),
        Event::CombatStarted { person } => format!(
            "{} attacks. Combat would start here, but it is not built yet.",
            person_name(world, *person)
        ),
        Event::RecruitableChanged {
            person,
            recruitable,
        } => {
            if *recruitable {
                format!("{} could now join the party.", person_name(world, *person))
            } else {
                format!("{} will not join the party.", person_name(world, *person))
            }
        }
        Event::Said {
            person,
            text,
            reply,
        } => {
            if world.npc_line() == Some(reply.as_str()) {
                // A tree node: the options listing shows what they said.
                format!("You: \"{text}\"")
            } else if reply.trim().is_empty() {
                format!("You: \"{text}\"
{} says nothing.", person_name(world, *person))
            } else {
                format!("You: \"{text}\"
{}: \"{reply}\"", person_name(world, *person))
            }
        }
        Event::ConversationEnded { person } => {
            format!("You leave {}.", person_name(world, *person))
        }
        Event::TooHostileToTalk {
            person,
            friendliness,
        } => format!(
            "{} is too hostile to talk to (friendliness {friendliness}). Combat would start here, but it is not built yet.",
            person_name(world, *person)
        ),
        Event::HostilityChanged {
            from,
            toward,
            index,
        } => format!(
            "The {}'s hostility toward {} is now {index}.",
            from.name(),
            toward.name()
        ),
        Event::AttitudeChanged { person, index } => format!(
            "{}'s attitude toward the player is now {index}.",
            person_name(world, *person)
        ),
        Event::Equipped { person, item, slot } => format!(
            "{} equips the {item} on the {slot}.",
            person_name(world, *person)
        ),
        Event::Unequipped { person, item, slot } => format!(
            "{} takes the {item} off the {slot} and puts it back in the pack.",
            person_name(world, *person)
        ),
        Event::Used { person, item } => {
            format!("{} uses the {item}.", person_name(world, *person))
        }
        Event::EffectFired {
            person,
            item,
            kind,
            payload,
        } => format!(
            "{}'s {item}, {}, {}.",
            person_name(world, *person),
            kind.name(),
            payload.note()
        ),
        Event::PlaceAdded {
            place,
            from,
            leagues,
            ..
        } => {
            let key = world.place(*place).map_or("?", |p| p.key.as_str());
            format!(
                "{} [{}] is added to the map, {} leagues from {}.",
                world.place_name(*place),
                key,
                leagues,
                world.place_name(*from)
            )
        }
        Event::ScheduleChanged { person } => {
            format!("{} keeps a new schedule.", person_name(world, *person))
        }
        Event::Waited { hours, .. } => format!("The party waits {hours} hours."),
        Event::Stepped { .. } => "An hour passes.".to_string(),
        Event::TimeModeChanged { mode } => format!("The clock is now {mode}."),
        Event::TimeScaleChanged { scale } => format!("The clock runs at {scale}× real time."),
    }
}

/// Minutes the host has measured and not yet handed to the world clock.
struct HostClock {
    last: Instant,
    carry_ms: u128,
}

/// Passes minutes this process has seen since the last call. The world's
/// time scale turns those real milliseconds into game minutes first. The
/// core does not read the operating system clock. In turn-based mode, and
/// while the clock is paused, the gap is dropped.
fn pass_elapsed(world: &mut World, clock: &mut HostClock) {
    let elapsed = clock.last.elapsed();
    clock.last = Instant::now();
    if world.time_mode != TimeMode::RealTime || world.clock_paused {
        clock.carry_ms = 0;
        return;
    }
    clock.carry_ms = clock.carry_ms.saturating_add(elapsed.as_millis());
    let (minutes, rest) = game_minutes(clock.carry_ms, world.time_scale);
    clock.carry_ms = rest;
    if minutes == 0 {
        return;
    }
    let moved = world.advance(minutes, AdvanceKind::Elapsed);
    for event in &moved.events {
        println!("{}", describe(world, event));
    }
}

fn is_clock_command(answer: &str) -> bool {
    matches!(
        answer.split_whitespace().next(),
        Some("wait" | "step" | "mode" | "time" | "scale")
    )
}

fn wait(world: &mut World, rest: &str) {
    match rest.parse::<u32>() {
        Ok(hours) => {
            run(world, Command::Wait { hours });
        }
        Err(_) => println!("Wait from 1 to 24 hours: wait <hours>."),
    }
}

fn set_mode(world: &mut World, rest: &str) {
    let mode = match rest {
        "real" | "realtime" | "real-time" => TimeMode::RealTime,
        "turn" | "turns" | "turn-based" | "turnbased" => TimeMode::TurnBased,
        "" => {
            println!("{}", clock_line(world));
            return;
        }
        _ => {
            println!("Use: mode real, or mode turn.");
            return;
        }
    };
    run(world, Command::SetTimeMode { mode });
}

fn set_scale(world: &mut World, clock: &mut HostClock, rest: &str) {
    if rest.is_empty() {
        println!("{}", clock_line(world));
        return;
    }
    let Ok(scale) = rest.parse::<u32>() else {
        println!("The time scale is a number from {MIN_TIME_SCALE} to {MAX_TIME_SCALE}.");
        return;
    };
    if run(world, Command::SetTimeScale { scale }) {
        clock.carry_ms = 0;
    }
}

fn clock_line(world: &World) -> String {
    let paused = if world.clock_paused { ", paused" } else { "" };
    format!(
        "{}, {:02}:{:02} ({}, {}×){paused}",
        world.date, world.hour, world.minute, world.time_mode, world.time_scale
    )
}

fn sheet(world: &World) {
    let Some(person) = world.player() else {
        println!("There is no player.");
        return;
    };
    let Some(sheet) = &person.sheet else {
        println!("{} has no sheet.", person.name);
        return;
    };

    println!("{}", person.name);
    println!("  Role: {}", sheet.role.name());
    println!("  Faction: {}", sheet.faction_label());
    if !sheet.secondary.is_empty() {
        println!("  Also: {}", sheet.secondary_list());
    }
    if let Some(attitude) = sheet.role.attitude() {
        println!("  Attitude: {attitude}");
    }
    println!(
        "  {} years, {} blood, {}",
        sheet.age,
        sheet.blood.name(),
        sheet.heritage.name()
    );
    println!("  Akstada: {}", sheet.akstada.name());
    println!("  Tongues: {}", sheet.tongue_list());
    println!("  Background: {}", sheet.background);
    println!("  Level {}", sheet.level);
    println!(
        "  Health {}    Fatigue {}    Resilience {} (max {})",
        sheet.health,
        sheet.fatigue,
        sheet.resilience,
        sheet.max_resilience()
    );
    if sheet.effective_health() != sheet.health {
        println!(
            "  Health while items are carried or worn: {}",
            sheet.effective_health()
        );
    }
    println!(
        "  Vitality {}    Willpower {}    Fortitude {}",
        sheet.vitality(),
        sheet.willpower(),
        sheet.fortitude()
    );
    if sheet.inventory.is_empty() {
        println!("  Carrying nothing.");
    } else {
        println!("  Carrying:");
        for item in &sheet.inventory {
            if item.count == 1 {
                println!("    {} {}", item.rarity.name(), item.name);
            } else {
                println!("    {} {} ({})", item.rarity.name(), item.name, item.count);
            }
        }
    }
    if sheet.equipped.is_empty() {
        println!("  Wearing nothing.");
    } else {
        println!("  Wearing:");
        for worn in &sheet.equipped {
            println!("    {} on the {}", worn.item.name, worn.slot.name());
        }
    }
}

fn equip(world: &mut World, rest: &str) {
    const USAGE: &str = "Usage: equip <item> in <slot>, for example: equip Trail Shoes in shoes";
    let Some((item, slot_text)) = rest.rsplit_once(" in ") else {
        println!("{USAGE}");
        return;
    };
    let item = item.trim();
    if item.is_empty() {
        println!("{USAGE}");
        return;
    }
    let Some(slot) = EquipSlot::from_name(slot_text) else {
        println!("There is no slot called \"{slot_text}\".");
        return;
    };
    run(
        world,
        Command::Equip {
            item: item.to_string(),
            slot,
        },
    );
}

fn unequip(world: &mut World, rest: &str) {
    if rest.is_empty() {
        println!("Usage: unequip <slot>");
        return;
    }
    let Some(slot) = EquipSlot::from_name(rest) else {
        println!("There is no slot called \"{rest}\".");
        return;
    };
    run(world, Command::Unequip { slot });
}

fn look(world: &World) {
    let here = world.party_at;
    let Some(place) = world.place(here) else {
        println!("The party is nowhere.");
        return;
    };

    println!("{}", clock_line(world));
    println!("{} ({:?})", place.name, place.kind);
    if let Some(temple) = &place.temple {
        let state = if temple.ruined { ", in ruins" } else { "" };
        println!("  Krypt of {}{}", temple.krypt_of, state);
        if let Some(rite) = &temple.rite {
            println!("  Rite: {rite}");
        }
        if !temple.nedia.is_empty() {
            println!("  Nedia of {}", temple.nedia.join(", "));
        }
    }
    if let Some(site) = world.party_site {
        println!(
            "  The party stands at the {}.",
            site_name(Some(place), site)
        );
        println!("  Type walk to see where it can go in {}.", place.name);
    }
    // In a city, only the people at the site the party stands at are here.
    for person in world
        .residents(here)
        .filter(|person| place.sites.is_empty() || person.site == world.party_site)
    {
        println!("  {} ({}) is here.", person.name, person.title);
    }
    if !world.nearby_npcs().is_empty() {
        println!("  Type nearby to see who you can interact with.");
    }
    if world.camped {
        println!("  The party is camped.");
    }

    println!("Roads:");
    for (to, road) in world.neighbours(here) {
        let key = world.place(to).map_or("?", |p| p.key.as_str());
        println!(
            "  {} [{}], {} leagues",
            world.place_name(to),
            key,
            road.leagues
        );
    }
}

fn sky(world: &World) {
    let majors: Vec<&str> = world.sky.majors().iter().map(|a| a.name()).collect();
    let minors: Vec<&str> = world.sky.minors().iter().map(|m| m.name()).collect();
    println!("Tonight the Akstada are {}.", majors.join(", "));
    println!("Among the lesser stars: {}.", minors.join(", "));
}

fn journal(world: &World) {
    if world.journal.is_empty() {
        println!("The journal is empty.");
    }
    for entry in &world.journal {
        println!("{}, at {}", entry.date, world.place_name(entry.at));
        println!("  {}", entry.text);
    }
}

fn map(world: &World) {
    let here = world.party_at;
    println!("From {}:", world.place_name(here));

    let routes = world.reachable(here);
    if routes.is_empty() {
        println!("  No road leads anywhere from here.");
    }
    for route in &routes {
        let key = world.place(route.to).map_or("?", |p| p.key.as_str());
        let way = if route.roads == 1 {
            "road from here".to_string()
        } else {
            format!(
                "{} roads, first to {}",
                route.roads,
                world.place_name(route.first_step)
            )
        };
        println!(
            "  {} [{}], {} leagues, {} hours ({})",
            world.place_name(route.to),
            key,
            route.leagues,
            route.hours,
            way
        );
    }

    let cut_off: Vec<&str> = world
        .places
        .iter()
        .enumerate()
        .map(|(i, p)| (PlaceId(i as u32), p))
        .filter(|(id, _)| *id != here && !routes.iter().any(|r| r.to == *id))
        .map(|(_, p)| p.name.as_str())
        .collect();
    if !cut_off.is_empty() {
        println!("No road reaches: {}", cut_off.join(", "));
    }
}

/// Reads "addplace <leagues> <name>" and sends the command.
fn add_place(world: &mut World, rest: &str) {
    const USAGE: &str = "Usage: addplace <leagues> <name>, for example: addplace 3 Old Mill Ford";
    let (leagues, name) = match rest.split_once(' ') {
        Some((leagues, name)) => (leagues, name.trim()),
        None => (rest, ""),
    };
    let Ok(leagues) = leagues.parse::<u32>() else {
        println!("{USAGE}");
        return;
    };
    if name.is_empty() {
        println!("{USAGE}");
        return;
    }
    run(
        world,
        Command::AddPlace {
            name: name.to_string(),
            leagues,
        },
    );
}

/// Handles "walk" and "walk <site>".
fn walk(world: &mut World, rest: &str) {
    let Some(place) = world.place(world.party_at) else {
        println!("The party is nowhere.");
        return;
    };
    if place.sites.is_empty() {
        println!("There is nowhere to walk to in {} yet.", place.name);
        return;
    }

    if rest.is_empty() {
        match world.party_site {
            Some(here) => println!(
                "In {} the party stands at the {}. It can walk to:",
                place.name,
                site_name(Some(place), here)
            ),
            None => println!("In {} the party can walk to:", place.name),
        }
        for (i, site) in place.sites.iter().enumerate() {
            if world.party_site.map(|s| s.index()) != Some(i) {
                println!("  {} [{}] ({:?})", site.name, site.key, site.kind);
            }
        }
        return;
    }

    match place.find_site(rest) {
        Some(to) => {
            run(world, Command::Walk { to });
        }
        None => println!(
            "There is no \"{rest}\" in {}. Type walk to see where the party can go.",
            place.name
        ),
    }
}

fn site_name(place: Option<&Place>, id: varif_core::SiteId) -> &str {
    place
        .and_then(|p| p.site(id))
        .map_or("an unknown spot", |s| s.name.as_str())
}

fn person_name(world: &World, id: PersonId) -> &str {
    world.person(id).map_or("someone", |p| p.name.as_str())
}

fn camp_note(spent_food: bool, spent_water: bool, spent_wood: bool) -> String {
    let mut note = String::new();
    let mut used = Vec::new();
    if spent_food {
        used.push("food");
    }
    if spent_water {
        used.push("water");
    }
    if spent_wood {
        used.push("wood");
    }
    if !used.is_empty() {
        note.push_str(&format!(" Uses {}.", used.join(", ")));
    }
    if !spent_food {
        note.push_str(" No food: 5 fatigue.");
    }
    if !spent_water {
        note.push_str(" No water: 5 health.");
    }
    if !spent_wood {
        note.push_str(" No wood for a fire: 5 resilience.");
    }
    note
}

fn fatigue_note(world: &World, fatigue: &[MemberFatigue]) -> String {
    if fatigue.is_empty() {
        return String::new();
    }
    let parts = fatigue
        .iter()
        .map(|entry| format!("{} {}", person_name(world, entry.person), entry.fatigue))
        .collect::<Vec<_>>()
        .join(", ");
    format!(" Fatigue: {parts}.")
}

/// Lists the NPCs at the spot the player stands, numbered for `interact`.
fn nearby(world: &World) {
    let people = world.nearby_npcs();
    if people.is_empty() {
        println!("No one is nearby.");
        return;
    }
    match world
        .party_site
        .and_then(|id| world.place(world.party_at).and_then(|place| place.site(id)))
    {
        Some(site) => println!("Nearby, at the {}:", site.name),
        None => println!("Nearby:"),
    }
    for (i, &id) in people.iter().enumerate() {
        let Some(person) = world.person(id) else {
            continue;
        };
        let follower = if person.is_follower() {
            ", follower"
        } else {
            ""
        };
        let mood = match world.standing(id) {
            Some(s) if !s.will_talk() => " - hostile".to_string(),
            Some(s) => format!(" - friendliness {}", s.friendliness),
            None => String::new(),
        };
        println!(
            "  {}. {} ({}{follower}) [{}]{mood}",
            i + 1,
            person.name,
            person.title,
            person.key
        );
    }
}

/// Handles "interact <npc>": finds them among the nearby NPCs by number, key
/// or name, and opens the conversation.
fn interact(world: &mut World, rest: &str) {
    let people = world.nearby_npcs();
    if rest.is_empty() {
        println!("Interact with whom? Type nearby for the list.");
        return;
    }
    let wanted = rest.to_lowercase();
    let found = match wanted.parse::<usize>() {
        Ok(n) if (1..=people.len()).contains(&n) => Some(people[n - 1]),
        _ => people.iter().copied().find(|&id| {
            world
                .person(id)
                .is_some_and(|p| p.key.to_lowercase() == wanted || p.name.to_lowercase() == wanted)
        }),
    };
    let Some(person) = found else {
        println!("No one called \"{rest}\" is nearby.");
        return;
    };
    if run(world, Command::Interact { person }) && world.conversation == Some(person) {
        show_options(world, person);
    }
}

/// Prints the lines the player can say to `person` right now, numbered.
fn show_options(world: &World, person: PersonId) {
    if let Some(line) = world.npc_line() {
        println!("{}: \"{line}\"", person_name(world, person));
    }
    for (i, line) in world.dialogue_options(person).iter().enumerate() {
        if line.label.is_empty() {
            println!("  {}. \"{}\"", i + 1, line.text);
        } else {
            println!("  {}. \"{}\"  [{}]", i + 1, line.text, line.label);
        }
    }
}
