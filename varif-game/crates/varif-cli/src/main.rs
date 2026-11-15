//! A command-line front end: load a world, type commands, read the events.
//!
//! Run it from the workspace folder with `cargo run`, or pass a data file:
//! `cargo run -- data/first_leg.ron`.

use std::io::{self, BufRead, Write};
use std::{env, fs, process};
use varif_core::{load_world, Command, Event, World};

const DEFAULT_DATA: &str = "data/first_leg.ron";

const HELP: &str = "\
Commands:
  look            describe where the party is and where the roads lead
  go <place>      travel to a neighbouring place, by key or by name
  camp            make camp and pass the night
  write <text>    add a journal entry
  journal         read the journal
  save <file>     write the whole world to a file
  help            show this list
  quit            leave";

fn main() {
    let path = env::args().nth(1).unwrap_or_else(|| DEFAULT_DATA.to_string());
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
    loop {
        print!("\n> ");
        io::stdout().flush().ok();

        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) | Err(_) => break, // end of input
            Ok(_) => {}
        }
        let line = line.trim();
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
            "journal" => journal(&world),
            "camp" => run(&mut world, Command::MakeCamp),
            "write" => run(&mut world, Command::WriteJournal { text: rest.to_string() }),
            "go" => match world.find_place(rest) {
                Some(to) => run(&mut world, Command::Travel { to }),
                None => println!("There is no place called \"{rest}\"."),
            },
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

/// Sends one command to the world and prints what happened.
fn run(world: &mut World, command: Command) {
    match world.apply(command) {
        Ok(events) => {
            for event in &events {
                println!("{}", describe(world, event));
            }
        }
        Err(error) => println!("Refused: {error}."),
    }
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
        Event::Arrived { at, .. } => format!("The party arrives at {}.", world.place_name(*at)),
        Event::Camped { at, .. } => format!("The party makes camp at {}.", world.place_name(*at)),
        Event::JournalWritten { .. } => "The entry is written.".to_string(),
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
    for person in world.residents(here) {
        println!("  {} ({}) is here.", person.name, person.role);
    }
    if world.camped {
        println!("  The party is camped.");
    }

    println!("Roads:");
    for (to, road) in world.neighbours(here) {
        let key = world.place(to).map_or("?", |p| p.key.as_str());
        println!("  {} [{}], {} leagues", world.place_name(to), key, road.leagues);
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
