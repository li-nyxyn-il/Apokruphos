//! Where an NPC is, as a repeating cycle of spans.
//!
//! Every NPC has one schedule. A span lasts a number of hours and names a
//! whereabouts: a place, an optional site inside it, and, when the span is
//! travel, the road from one place to another. A farmer's day and a courier's
//! month are the same type. The week (Arden through Seldyn, five days) and
//! the month (fifty days) only describe how long the cycle is.
//!
//! [`Schedule::at`] is a pure function of that schedule and an absolute hour
//! counted from hour 0 of year 0. Looking one NPC up does not walk the others
//! forward, so a world of hundreds stays cheap.

use crate::ids::{PlaceId, SiteId};
use serde::{Deserialize, Serialize};

/// The road an NPC is on during a travel span.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoadLeg {
    pub from: PlaceId,
    pub to: PlaceId,
}

/// Where an NPC is during one span of their schedule.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Whereabouts {
    pub place: PlaceId,
    /// A site inside [`Whereabouts::place`], when they are at one.
    #[serde(default)]
    pub site: Option<SiteId>,
    /// Set when the span is travel along the road from one place to another.
    #[serde(default)]
    pub on_the_road: Option<RoadLeg>,
}

/// One stretch of a schedule.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    /// How long this stretch lasts, in hours. Zero hours is refused.
    pub hours: u32,
    pub whereabouts: Whereabouts,
}

/// A repeating cycle of spans. Empty means the NPC has no timed round.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Schedule {
    pub spans: Vec<Span>,
}

impl Schedule {
    /// The length of one cycle, in hours. Zero when there are no spans.
    pub fn cycle_hours(&self) -> u64 {
        self.spans.iter().map(|span| u64::from(span.hours)).sum()
    }

    /// Whereabouts at `absolute_hour`. `None` when the cycle is empty.
    ///
    /// This reads only this schedule. It does not look at any other NPC.
    pub fn at(&self, absolute_hour: u64) -> Option<Whereabouts> {
        let cycle = self.cycle_hours();
        if cycle == 0 {
            return None;
        }
        let mut cursor = absolute_hour % cycle;
        for span in &self.spans {
            let hours = u64::from(span.hours);
            if hours == 0 {
                continue;
            }
            if cursor < hours {
                return Some(span.whereabouts);
            }
            cursor -= hours;
        }
        None
    }
}
