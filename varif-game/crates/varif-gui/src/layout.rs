//! Where places sit on the map.
//!
//! Positions are how the map is drawn, not part of the world, so they live in
//! their own file beside the data and the rules never see them. A place the
//! file leaves out (one added in play, say) is put beside a neighbour.

use std::collections::BTreeMap;
use std::f32::consts::TAU;
#[cfg(not(target_arch = "wasm32"))]
use std::{fs, io::ErrorKind, path::Path};
use varif_core::{PlaceId, World};

/// Used when no road has both ends placed, so there is nothing to measure.
const DEFAULT_PIXELS_PER_LEAGUE: f32 = 6.0;

/// The positions read from the layout file: place key to map pixels.
#[derive(Default)]
pub struct MapLayout {
    placed: BTreeMap<String, (i32, i32)>,
}

impl MapLayout {
    /// Reads a layout file. A missing file is an empty layout, not an error.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn read(path: &Path) -> Result<MapLayout, String> {
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(MapLayout::default()),
            Err(error) => return Err(error.to_string()),
        };
        MapLayout::from_text(&text)
    }

    /// Reads a layout from the text of a layout file. The web build has no
    /// files, so it carries its layout built in and reads it with this.
    pub fn from_text(text: &str) -> Result<MapLayout, String> {
        let placed = ron::from_str(text).map_err(|e| e.to_string())?;
        Ok(MapLayout { placed })
    }

    /// True if the layout file gives this place a spot on the painted map.
    /// The map already shows the roads between such places.
    pub fn has(&self, key: &str) -> bool {
        self.placed.contains_key(key)
    }

    /// A position for every place in the world, in the order of its table.
    pub fn positions(&self, world: &World) -> Vec<(f32, f32)> {
        let mut spots: Vec<Option<(f32, f32)>> = world
            .places
            .iter()
            .map(|p| self.placed.get(&p.key).map(|&(x, y)| (x as f32, y as f32)))
            .collect();

        // How long a league is on the map, measured on the roads that have
        // both ends placed.
        let mut pixels = 0.0;
        let mut leagues = 0.0;
        for road in &world.roads {
            if let (Some(a), Some(b)) = (spots[road.a.index()], spots[road.b.index()]) {
                pixels += (a.0 - b.0).hypot(a.1 - b.1);
                leagues += road.leagues as f32;
            }
        }
        let per_league = if leagues > 0.0 && pixels > 0.0 {
            pixels / leagues
        } else {
            DEFAULT_PIXELS_PER_LEAGUE
        };

        // Put each unplaced place beside a placed neighbour, as far away as
        // its road is long. Repeat, since placing one place may give another
        // its first placed neighbour. A place no road reaches stands alone.
        loop {
            let mut changed = false;
            for index in 0..spots.len() {
                if spots[index].is_some() {
                    continue;
                }
                let id = PlaceId(index as u32);
                let beside = world
                    .neighbours(id)
                    .into_iter()
                    .find_map(|(other, road)| spots[other.index()].map(|at| (at, road.leagues)));
                if let Some((at, road_leagues)) = beside {
                    let angle = direction(&world.places[index].key);
                    let reach = road_leagues as f32 * per_league;
                    spots[index] = Some((at.0 + angle.cos() * reach, at.1 + angle.sin() * reach));
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }

        // Anything still unplaced is cut off from every placed place. Line
        // those up below the rest of the map.
        let placed: Vec<(f32, f32)> = spots.iter().flatten().copied().collect();
        let (left, floor) = match bounds(&placed) {
            Some((min, max)) => (min.0, max.1 + 60.0),
            None => (0.0, 0.0),
        };
        let mut column = 0.0;
        spots
            .into_iter()
            .map(|spot| {
                spot.unwrap_or_else(|| {
                    column += 1.0;
                    (left + (column - 1.0) * 80.0, floor)
                })
            })
            .collect()
    }
}

/// The top-left and bottom-right corners of the box that holds every position.
pub fn bounds(spots: &[(f32, f32)]) -> Option<((f32, f32), (f32, f32))> {
    let first = *spots.first()?;
    Some(spots.iter().fold((first, first), |(min, max), &(x, y)| {
        ((min.0.min(x), min.1.min(y)), (max.0.max(x), max.1.max(y)))
    }))
}

/// A direction worked out from a place's key, so that an unplaced place is
/// drawn in the same spot on every frame and every run.
fn direction(key: &str) -> f32 {
    // FNV-1a, a small hash that gives the same answer on every machine.
    let mut hash: u32 = 0x811C_9DC5;
    for byte in key.bytes() {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(0x0100_0193);
    }
    (hash % 360) as f32 / 360.0 * TAU
}
