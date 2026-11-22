//! The night sky: seeded, turned at noon, and following the draw rules.

use std::collections::BTreeSet;
use varif_core::{
    load_world, AdvanceKind, Akstada, Command, Minor, Sky, World, MAJORS_PER_NIGHT,
    MINORS_PER_NIGHT, MINOR_CONSTELLATIONS,
};

const FIRST_LEG: &str = include_str!("../../../data/first_leg.ron");
/// The seed in the first-leg data file.
const FIRST_LEG_SEED: u64 = 192;

fn first_leg() -> World {
    load_world(FIRST_LEG).expect("the first leg data should load")
}

#[test]
fn every_night_has_four_different_akstada_and_ten_different_minors() {
    let mut sky = Sky::new(42);
    for _ in 0..500 {
        let majors: BTreeSet<_> = sky.majors().iter().map(|a| a.name()).collect();
        let minors: BTreeSet<_> = sky.minors().iter().copied().collect();
        assert_eq!(majors.len(), MAJORS_PER_NIGHT);
        assert_eq!(minors.len(), MINORS_PER_NIGHT);
        sky.turn();
    }
}

#[test]
fn the_same_seed_gives_the_same_skies() {
    let mut a = Sky::new(192);
    let mut b = Sky::new(192);
    for _ in 0..200 {
        assert_eq!(a, b);
        a.turn();
        b.turn();
    }
    assert_eq!(Sky::on_night(192, 37), {
        let mut s = Sky::new(192);
        for _ in 0..37 {
            s.turn();
        }
        s
    });
    assert_ne!(Sky::new(1).majors(), Sky::new(2).majors());
}

#[test]
fn a_minor_never_shows_two_nights_running() {
    let mut sky = Sky::new(7);
    for _ in 0..500 {
        let last: Vec<Minor> = sky.minors().to_vec();
        sky.turn();
        assert!(sky.minors().iter().all(|m| !last.contains(m)));
    }
}

#[test]
fn streaks_count_nights_running_and_reset() {
    let mut sky = Sky::new(11);
    for _ in 0..500 {
        let before: Vec<u32> = Akstada::ALL.iter().map(|&a| sky.streak(a)).collect();
        sky.turn();
        for (i, &a) in Akstada::ALL.iter().enumerate() {
            let up = sky.majors().contains(&a);
            assert_eq!(sky.streak(a), if up { before[i] + 1 } else { 0 });
        }
    }
}

#[test]
fn the_first_night_is_drawn_evenly() {
    // Over many seeds, every Akstada and every minor turns up on night 0
    // about as often as the others: 4/21 and 10/40 of the time.
    let seeds = 20_000;
    let mut majors = vec![0u32; Akstada::ALL.len()];
    let mut minors = vec![0u32; MINOR_CONSTELLATIONS.len()];
    for seed in 0..seeds {
        let sky = Sky::new(seed);
        for a in sky.majors() {
            majors[Akstada::ALL.iter().position(|x| x == a).unwrap()] += 1;
        }
        for m in sky.minors() {
            minors[usize::from(m.0)] += 1;
        }
    }
    let expect_major = seeds as f64 * 4.0 / 21.0;
    let expect_minor = seeds as f64 * 10.0 / 40.0;
    for n in majors {
        assert!(
            (f64::from(n) - expect_major).abs() < expect_major * 0.08,
            "{n}"
        );
    }
    for n in minors {
        assert!(
            (f64::from(n) - expect_minor).abs() < expect_minor * 0.08,
            "{n}"
        );
    }
}

#[test]
fn akstada_return_more_and_then_fade() {
    // How often an Akstada is up the next night, by how long it has been up.
    let mut sky = Sky::new(5);
    let mut seen = [0u32; 7];
    let mut back = [0u32; 7];
    for _ in 0..200_000 {
        let before: Vec<u32> = Akstada::ALL.iter().map(|&a| sky.streak(a)).collect();
        sky.turn();
        for (i, &a) in Akstada::ALL.iter().enumerate() {
            let s = before[i].min(6) as usize;
            seen[s] += 1;
            if sky.majors().contains(&a) {
                back[s] += 1;
            }
        }
    }
    let rate = |s: usize| f64::from(back[s]) / f64::from(seen[s]);
    assert!(rate(1) > rate(0) * 1.3, "a returning one is more likely");
    assert!(
        (rate(1) - rate(3)).abs() < 0.02,
        "the boost holds to night 3"
    );
    // The fade is 70% off the return weight: 450 against 1500 and 1000.
    assert!(rate(4) < rate(3) * 0.4, "four nights running fades it");
    assert!(rate(4) < rate(0) * 0.6, "below an unseen one too");
    assert!(rate(5) < rate(4), "and it keeps fading");
}

#[test]
fn the_world_starts_with_a_sky_from_its_seed() {
    let a = first_leg();
    let b = first_leg();
    assert!(!a.sky.is_empty());
    assert_eq!(a.sky, b.sky);
    assert_eq!(a.sky.night(), 0);
}

#[test]
fn the_sky_turns_at_noon_and_only_at_noon() {
    let mut world = first_leg();
    world.hour = 10;
    world.minute = 0;
    let start = world.sky.clone();

    world.advance(119, AdvanceKind::Spend); // 11:59
    assert_eq!(world.sky, start);
    world.advance(1, AdvanceKind::Spend); // 12:00
    assert_eq!(world.sky.night(), 1);
    world.advance(23 * 60 + 59, AdvanceKind::Spend); // 11:59 next day
    assert_eq!(world.sky.night(), 1);
    world.advance(1, AdvanceKind::Spend); // noon again
    assert_eq!(world.sky.night(), 2);
    world.advance(3 * 24 * 60, AdvanceKind::Spend); // three more noons
    assert_eq!(world.sky.night(), 5);
    assert_eq!(world.sky, Sky::on_night(FIRST_LEG_SEED, 5));
}

#[test]
fn other_draws_on_the_world_do_not_change_the_sky() {
    // The sky has its own stream: the night number alone decides it.
    let mut a = first_leg();
    let mut b = first_leg();
    for _ in 0..50 {
        b.rng.next_u64();
    }
    a.hour = 0;
    b.hour = 0;
    a.advance(10 * 24 * 60, AdvanceKind::Spend);
    for _ in 0..10 {
        b.apply(Command::Wait { hours: 24 }).ok();
    }
    assert_eq!(a.sky.night(), b.sky.night());
    assert_eq!(a.sky, b.sky);
}

#[test]
fn the_sky_survives_a_save() {
    let mut world = first_leg();
    world.advance(3 * 24 * 60, AdvanceKind::Spend);
    let loaded = World::from_ron(&world.to_ron().unwrap()).unwrap();
    assert_eq!(loaded.sky, world.sky);
}
