//! A command-line front end: load a world, type commands, read the events.
//!
//! Run it from the workspace folder with `cargo run`, or pass a data file:
//! `cargo run -- data/first_leg.ron`.

use std::io::{self, BufRead, Write};
use std::{env, fs, process};
use varif_core::{
    load_world, Command, Event, Itinerary, MemberFatigue, PersonId, Place, PlaceId, World,
};

const DEFAULT_DATA: &str = "data/first_leg.ron";
/// A map with many loops has hundreds of ways between two places. Only this
/// many, the quickest, are offered.
const MAX_WAYS: usize = 5;

const HELP: &str = "\
Commands:
  look            describe where the party is and where the roads lead
  sheet           show the player character
  go <place>      travel to a place, by key or by name; if no road runs
                  straight there you are offered the ways that lead there
                  and asked before each leg (y/n); on a walk of several
                  days you are asked each night whether to make camp (y/n),
                  then whether to sleep (y/n);
                  when someone where you arrive could join the party, you
                  are asked whether to recruit them (y/n)
  map             list every place the roads can take the party to
  walk            list the places inside the city the party can walk to
  walk <site>     walk to one of them, by key or by name
  addplace <leagues> <name>
                  put a new place on the map, a road of that many leagues
                  from where the party stands
  camp            pitch camp; this does not pass the night
  rest            sleep in camp; spends one food, one water and one wood
  pack            pack up the camp
  sleep           sleep at an inn for 5 copper, or at a residence for free
  write <text>    add a journal entry
  journal         read the journal
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
    let mut world = match load_world(&text) {
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
    loop {
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
            };
        }

        match &asking {
            // Between days on the road the question is camp, or sleep once camped.
            _ if world.journey.is_some() && world.camped => print!("Sleep in camp? (y/n) "),
            _ if world.journey.is_some() => print!("Make camp for the night? (y/n) "),
            Asking::WhichWay(routes) => print!("Which way? (1-{}, or n to stay) ", routes.len()),
            Asking::Recruit(offers) => {
                let name = world.person(offers[0]).map_or("them", |p| p.name.as_str());
                print!("Recruit {name} into the party? (y/n) ");
            }
            Asking::NextStop { stops, first_leg } => {
                let name = world.place_name(stops[0]);
                if *first_leg {
                    print!("Travel to {name} first? (y/n) ");
                } else {
                    print!("Continue on to {name}? (y/n) ");
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

        if world.journey.is_some() {
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
                    "y" | "yes" => {
                        run(&mut world, Command::MakeCamp);
                    }
                    "n" | "no" => {
                        run(&mut world, Command::EndDay { camp: false });
                    }
                    _ => println!("Answer y to make camp or n to press on."),
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
            Asking::WhichWay(routes) => {
                match answer.parse::<usize>() {
                    Ok(n) if (1..=routes.len()).contains(&n) => {
                        let stops = routes[n - 1].stops.clone();
                        println!(
                            "The party takes the way through {}.",
                            through(&world, &stops)
                        );
                        heading = set_out(&mut world, stops);
                    }
                    _ if answer == "n" || answer == "no" => stay(&world),
                    _ => {
                        println!("Answer with the number of a way, or n to stay.");
                        asking = Asking::WhichWay(routes);
                    }
                }
                continue;
            }
            Asking::NextStop { stops, first_leg } => {
                match answer.as_str() {
                    "y" | "yes" => heading = set_out(&mut world, stops),
                    "n" | "no" => stay(&world),
                    _ => {
                        println!("Answer y or n.");
                        asking = Asking::NextStop { stops, first_leg };
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
            "sheet" => sheet(&world),
            "journal" => journal(&world),
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
    WhichWay(Vec<Itinerary>),
    /// Walk to the first of these stops? The rest follow, one question each.
    NextStop {
        stops: Vec<PlaceId>,
        first_leg: bool,
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

/// Handles "go <place>". Walks straight there when a road joins the two
/// places; otherwise searches for the ways that lead there and asks.
fn go(world: &mut World, rest: &str) -> Asking {
    let Some(to) = world.find_place(rest) else {
        println!("There is no place called \"{rest}\".");
        return Asking::Nothing;
    };
    let here = world.party_at;
    if to == here || world.road_between(here, to).is_some() {
        run(world, Command::Travel { to });
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
            Asking::WhichWay(routes)
        }
    }
}

/// Starts the first leg of a route. Returns the stops still to come, or
/// nothing if the world refused the leg.
fn set_out(world: &mut World, stops: Vec<PlaceId>) -> Vec<PlaceId> {
    let Some((&first, rest)) = stops.split_first() else {
        return Vec::new();
    };
    if run(world, Command::Travel { to: first }) {
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
    let s = if route.days == 1 { "" } else { "s" };
    format!("{} leagues, {} day{s}", route.leagues, route.days)
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
        Event::NightFalls { to, days_left, .. } => format!(
            "Night falls on the road, {} day{} from {}.",
            days_left,
            if *days_left == 1 { "" } else { "s" },
            world.place_name(*to)
        ),
        Event::CampedOnRoad { .. } => "The party makes camp beside the road.".to_string(),
        Event::PressedOn {
            nights_without_camp,
            extra_leagues,
            fatigue,
            ..
        } => {
            let night = match nights_without_camp {
                1 => "The party presses on through the night".to_string(),
                n => format!("The party presses on through the night, {n} nights without camp"),
            };
            format!(
                "{night}, covering {extra_leagues} extra leagues.{}",
                fatigue_note(world, fatigue)
            )
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
            format!("The party makes camp at {}.", world.place_name(*at))
        }
        Event::Rested {
            at,
            fatigue,
            spent_food,
            spent_water,
            spent_wood,
            ..
        } => format!(
            "The party sleeps in camp at {}.{}{}",
            world.place_name(*at),
            camp_note(*spent_food, *spent_water, *spent_wood),
            fatigue_note(world, fatigue)
        ),
        Event::PackedUp { at, .. } => {
            format!("The party packs up the camp at {}.", world.place_name(*at))
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
                "The party sleeps at the {} in {}.{}{}",
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
    }
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
        "  Health {}    Fatigue {}    Resilience {}",
        sheet.health, sheet.fatigue, sheet.resilience
    );
    if sheet.inventory.is_empty() {
        println!("  Carrying nothing.");
    } else {
        println!("  Carrying:");
        for item in &sheet.inventory {
            if item.count == 1 {
                println!("    {}", item.name);
            } else {
                println!("    {} ({})", item.name, item.count);
            }
        }
    }
}

fn look(world: &World) {
    let here = world.party_at;
    let Some(place) = world.place(here) else {
        println!("The party is nowhere.");
        return;
    };

    println!("{}", world.date);
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
    for person in world.residents(here) {
        println!("  {} ({}) is here.", person.name, person.title);
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
        let days = route.days;
        println!(
            "  {} [{}], {} leagues, {} day{} ({})",
            world.place_name(route.to),
            key,
            route.leagues,
            days,
            if days == 1 { "" } else { "s" },
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
