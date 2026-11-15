//! Tests for finding the ways between places that no single road joins.

use varif_core::{load_world, Command, World};

/// A small map with a loop and a dead end:
///
/// ```text
///          ford --- hill
///         /            \
///   gate                keep
///         \            /
///          mill -------
///   gate --- well (dead end)
/// ```
const LOOP_MAP: &str = r#"(
    start: (year: 192, month: 1, day: 1, seed: 7, party_at: "gate"),
    places: [
        (key: "gate", name: "Gate", kind: Town),
        (key: "ford", name: "Ford", kind: Landmark),
        (key: "hill", name: "Hill", kind: Landmark),
        (key: "mill", name: "Mill", kind: Landmark),
        (key: "keep", name: "Keep", kind: Town),
        (key: "well", name: "Well", kind: Landmark),
        (key: "isle", name: "Isle", kind: Landmark),
    ],
    roads: [
        (from: "gate", to: "ford", leagues: 4),
        (from: "ford", to: "hill", leagues: 4),
        (from: "hill", to: "keep", leagues: 4),
        (from: "gate", to: "mill", leagues: 10),
        (from: "mill", to: "keep", leagues: 10),
        (from: "gate", to: "well", leagues: 2),
    ],
)"#;

fn loop_map() -> World {
    load_world(LOOP_MAP).expect("the loop map should load")
}

fn names(world: &World, stops: &[varif_core::PlaceId]) -> Vec<String> {
    stops.iter().map(|&id| world.place_name(id).to_string()).collect()
}

#[test]
fn every_way_round_the_loop_is_found_quickest_first() {
    let world = loop_map();
    let gate = world.find_place("gate").unwrap();
    let keep = world.find_place("keep").unwrap();

    let routes = world.routes_between(gate, keep);
    let found: Vec<Vec<String>> = routes.iter().map(|r| names(&world, &r.stops)).collect();
    assert_eq!(found, [vec!["Ford", "Hill", "Keep"], vec!["Mill", "Keep"]]);

    // 4 + 4 + 4 leagues is one day a road; 10 + 10 leagues is two a road.
    assert_eq!((routes[0].leagues, routes[0].days), (12, 3));
    assert_eq!((routes[1].leagues, routes[1].days), (20, 4));
}

#[test]
fn a_route_never_passes_the_same_place_twice() {
    let world = loop_map();
    let well = world.find_place("well").unwrap();
    let hill = world.find_place("hill").unwrap();
    for route in world.routes_between(well, hill) {
        let mut seen = route.stops.clone();
        seen.sort();
        seen.dedup();
        assert_eq!(seen.len(), route.stops.len(), "a stop repeats in {:?}", route.stops);
        assert!(!route.stops.contains(&well), "the route returns to its start");
    }
    assert_eq!(world.routes_between(well, hill).len(), 2);
}

#[test]
fn a_cut_off_place_has_no_routes() {
    let world = loop_map();
    let gate = world.find_place("gate").unwrap();
    let isle = world.find_place("isle").unwrap();
    assert!(world.routes_between(gate, isle).is_empty());
    assert!(world.routes_between(gate, gate).is_empty());
}

#[test]
fn atzhaim_can_be_reached_by_the_road_or_through_maphlia() {
    let world = load_world(include_str!("../../../data/first_leg.ron")).unwrap();
    let yhlgrym = world.find_place("yhlgrym").unwrap();
    let atzhaim = world.find_place("atzhaim").unwrap();
    let routes = world.routes_between(yhlgrym, atzhaim);

    // The Pylgrymage road up the canyon is the quickest way.
    assert_eq!(names(&world, &routes[0].stops), ["Hylvynhani Falls", "Atzhaim"]);
    assert_eq!(routes[0].days, 14, "the same 14 days as walking it");

    // The map has many loops, so there are many slower ways. The quickest
    // that avoids the canyon goes round by the south road, Maphlamar and the
    // track over the hills to Heedhaim.
    assert!(routes.len() > 2);
    let round = routes
        .iter()
        .find(|r| !names(&world, &r.stops).contains(&"Hylvynhani Falls".to_string()))
        .expect("a way that avoids the canyon");
    assert_eq!(
        names(&world, &round.stops),
        [
            "Sthepenhall",
            "Pheo",
            "Edun",
            "Laad",
            "Hases",
            "Klate",
            "Corvus",
            "Maphlamar",
            "Heedhaim",
            "Atzhaim"
        ]
    );
}

#[test]
fn a_new_place_can_be_routed_to() {
    let mut world = loop_map();
    let keep = world.find_place("keep").unwrap();
    let well = world.find_place("well").unwrap();
    world.apply(Command::AddPlace { name: "Tor".to_string(), leagues: 1 }).unwrap();
    let tor = world.find_place("tor").unwrap();

    // Two ways reach Gate from Keep, and the road to Tor branches off Gate.
    assert_eq!(world.routes_between(keep, tor).len(), 2);
    assert_eq!(names(&world, &world.routes_between(well, tor)[0].stops), ["Gate", "Tor"]);
}
