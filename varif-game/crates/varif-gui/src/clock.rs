//! The seconds a frame hands to the world clock.
//!
//! The core does not read the operating system clock. In real-time mode the
//! host passes elapsed minutes to [`World::advance`]. Turn-based mode, and a
//! paused clock, ignore that call. Camp, sleep, travel, pack-up, and wait
//! keep using the same function with a different [`AdvanceKind`].

use std::time::Duration;

use varif_core::{AdvanceKind, ClockAdvance, TimeMode, World};

/// Whole minutes of `frame_seconds` to pass on, keeping what is left over.
///
/// `running` is false in turn-based mode and while the clock is paused. Those
/// frames pass nothing and drop the remainder, so that time does not arrive
/// later when the clock starts.
pub(crate) fn take_minutes(carry_seconds: &mut f32, frame_seconds: f32, running: bool) -> u32 {
    if !running {
        *carry_seconds = 0.0;
        return 0;
    }
    if !frame_seconds.is_finite() || frame_seconds <= 0.0 {
        return 0;
    }
    *carry_seconds += frame_seconds;
    let minutes = (*carry_seconds / 60.0) as u32;
    if minutes == 0 {
        return 0;
    }
    *carry_seconds -= minutes as f32 * 60.0;
    if !carry_seconds.is_finite() || *carry_seconds < 0.0 {
        *carry_seconds = 0.0;
    }
    minutes
}

/// What one frame does with the seconds since the previous frame.
///
/// The returned advance is that call. Zero minutes means the frame did not
/// move the clock.
pub(crate) fn apply_elapsed_frame(
    world: &mut World,
    carry_seconds: &mut f32,
    frame_seconds: f32,
) -> ClockAdvance {
    let minutes = take_minutes(carry_seconds, frame_seconds, is_running(world));
    if minutes == 0 {
        return ClockAdvance {
            minutes: 0,
            events: Vec::new(),
            stopped: false,
        };
    }
    world.advance(minutes, AdvanceKind::Elapsed)
}

/// Real time is running: passive minutes move the clock.
pub(crate) fn is_running(world: &World) -> bool {
    world.time_mode == TimeMode::RealTime && !world.clock_paused
}

/// How long to wait before drawing again, so the next minute can show.
///
/// The clock on screen changes by the minute, so the frame waits out the
/// seconds it has not yet handed over. A late wake still counts. The next
/// frame passes the seconds that actually passed, not this estimate.
pub(crate) fn until_next_minute(carry_seconds: f32) -> Duration {
    let into = if carry_seconds.is_finite() {
        carry_seconds.clamp(0.0, 59.5)
    } else {
        0.0
    };
    let left = (60.0 - into).clamp(0.5, 60.0);
    Duration::try_from_secs_f32(left).unwrap_or(Duration::from_secs(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use varif_core::{load_world, Command, Event};

    const FIRST_LEG: &str = include_str!("../../../data/first_leg.ron");

    fn loaded() -> World {
        load_world(FIRST_LEG).expect("the first leg data should load")
    }

    fn realtime() -> World {
        let mut world = loaded();
        world
            .apply(Command::SetTimeMode {
                mode: TimeMode::RealTime,
            })
            .unwrap();
        world
    }

    #[test]
    fn sixty_one_second_frames_pass_one_minute() {
        let mut world = realtime();
        let mut carry = 0.0;
        for _ in 0..59 {
            let moved = apply_elapsed_frame(&mut world, &mut carry, 1.0);
            assert_eq!(moved.minutes, 0);
        }
        assert_eq!((world.hour, world.minute), (0, 0));

        let moved = apply_elapsed_frame(&mut world, &mut carry, 1.0);
        assert_eq!(moved.minutes, 1);
        assert_eq!((world.hour, world.minute), (0, 1));
        assert_eq!(carry, 0.0);
    }

    #[test]
    fn a_predicted_frame_slice_is_not_yet_a_minute() {
        let mut world = realtime();
        let mut carry = 0.0;
        // What `stable_dt` is when the previous frame did not ask to repaint.
        let moved = apply_elapsed_frame(&mut world, &mut carry, 1.0 / 60.0);
        assert_eq!(moved.minutes, 0);
        assert_eq!(world.minute, 0);
    }

    #[test]
    fn ninety_minutes_of_frame_time_rolls_the_same_clock() {
        let mut world = realtime();
        let start = world.date;
        let mut carry = 0.0;
        let moved = apply_elapsed_frame(&mut world, &mut carry, 90.0 * 60.0);
        assert_eq!(moved.minutes, 90);
        assert_eq!((world.hour, world.minute), (1, 30));
        assert_eq!(carry, 0.0);
        assert_eq!(world.date, start);
    }

    #[test]
    fn a_day_of_frame_time_rolls_the_date_through_advance() {
        let mut world = realtime();
        let start = world.date;
        let mut carry = 0.0;
        let moved = apply_elapsed_frame(&mut world, &mut carry, 24.0 * 60.0 * 60.0);
        assert_eq!(moved.minutes, 24 * 60);
        assert_eq!(world.date, start.next_day());
        assert_eq!((world.hour, world.minute), (0, 0));
        assert!(moved
            .events
            .iter()
            .any(|event| matches!(event, Event::DayPassed { date } if *date == world.date)));
    }

    #[test]
    fn turn_based_frames_pass_no_minutes() {
        let mut world = loaded();
        let before = world.clone();
        let mut carry = 45.0;
        let moved = apply_elapsed_frame(&mut world, &mut carry, 3600.0);
        assert_eq!(moved.minutes, 0);
        assert!(moved.events.is_empty());
        assert!(!moved.stopped);
        assert_eq!(world, before);
        assert_eq!(carry, 0.0);

        world.apply(Command::Step).unwrap();
        assert_eq!((world.hour, world.minute), (1, 0));
    }

    #[test]
    fn a_paused_clock_drops_the_gap_instead_of_saving_it() {
        let mut world = realtime();
        world.clock_paused = true;
        let mut carry = 0.0;
        let moved = apply_elapsed_frame(&mut world, &mut carry, 120.0);
        assert_eq!(moved.minutes, 0);
        assert_eq!((world.hour, world.minute), (0, 0));
        assert_eq!(carry, 0.0);

        world.clock_paused = false;
        let moved = apply_elapsed_frame(&mut world, &mut carry, 60.0);
        assert_eq!(moved.minutes, 1);
        assert_eq!(world.minute, 1);
    }

    #[test]
    fn the_wake_waits_out_the_seconds_not_yet_a_minute() {
        assert_eq!(until_next_minute(0.0), Duration::from_secs(60));
        assert_eq!(until_next_minute(30.0), Duration::from_secs(30));
        assert_eq!(
            until_next_minute(59.9),
            Duration::try_from_secs_f32(0.5).unwrap()
        );
    }

    /// Draws one play frame at `time` seconds on egui's clock, with no window.
    fn show_play(app: &mut crate::App, ctx: &eframe::egui::Context, time: f64) {
        let mut raw = eframe::egui::RawInput::default();
        raw.screen_rect = Some(eframe::egui::Rect::from_min_size(
            eframe::egui::Pos2::ZERO,
            eframe::egui::vec2(1240.0, 800.0),
        ));
        raw.time = Some(time);
        let mut frame = eframe::Frame::_new_kittest();
        ctx.run_ui(raw, |ui| eframe::App::ui(app, ui, &mut frame))
            .drop_without_applying_deltas();
    }

    fn play_app(mode: TimeMode) -> crate::App {
        let mut app = crate::App::new(loaded(), crate::layout::MapLayout::default(), None);
        app.stage = crate::Stage::Play;
        if mode == TimeMode::RealTime {
            app.world
                .apply(Command::SetTimeMode {
                    mode: TimeMode::RealTime,
                })
                .unwrap();
        }
        app
    }

    #[test]
    fn the_play_frame_passes_the_real_gap_and_asks_to_draw_again() {
        use std::sync::{Arc, Mutex};

        let mut app = play_app(TimeMode::RealTime);
        let ctx = eframe::egui::Context::default();
        let delays = Arc::new(Mutex::new(Vec::new()));
        let recorded = Arc::clone(&delays);
        ctx.set_repaint_observer(move |info| {
            recorded.lock().expect("delay list").push(info.delay);
        });

        show_play(&mut app, &ctx, 0.0);
        assert_eq!((app.world.hour, app.world.minute), (0, 0));
        let first = delays.lock().expect("delay list").clone();
        assert!(
            first.contains(&until_next_minute(app.real_seconds)),
            "a real-time frame should ask to draw again, saw {first:?}"
        );

        delays.lock().expect("delay list").clear();
        // Ninety seconds of wall time, and no click. `stable_dt` stays near
        // 1/60 here, because the previous frame did not ask to repaint
        // immediately. The minute moves only if the frame passes `unstable_dt`.
        show_play(&mut app, &ctx, 90.0);
        assert_eq!((app.world.hour, app.world.minute), (0, 1),);
        assert!(
            crate::clock_text(&app.world).contains("00:01"),
            "the heading reads {}",
            crate::clock_text(&app.world)
        );
        let second = delays.lock().expect("delay list").clone();
        assert!(
            second.contains(&until_next_minute(app.real_seconds)),
            "the frame should wake for the leftover seconds, saw {second:?}"
        );
    }

    #[test]
    fn the_play_frame_ignores_elapsed_time_when_turn_based() {
        let mut app = play_app(TimeMode::TurnBased);
        let ctx = eframe::egui::Context::default();
        show_play(&mut app, &ctx, 0.0);
        show_play(&mut app, &ctx, 90.0);
        assert_eq!((app.world.hour, app.world.minute), (0, 0));
        assert_eq!(app.real_seconds, 0.0);
        assert!(crate::clock_text(&app.world).contains("00:00"));
        assert!(crate::clock_text(&app.world).contains("turn-based"));
    }
}
