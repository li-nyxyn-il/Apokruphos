//! The seconds a frame hands to the world clock.
//!
//! The core does not read the operating system clock. In real-time mode the
//! host passes elapsed minutes to [`World::advance`]. Those minutes are the
//! frame's real time multiplied by [`World::time_scale`]. Turn-based mode,
//! and a paused clock, ignore that call. Camp, sleep, travel, pack-up, and
//! wait keep using the same function with a different [`AdvanceKind`], and
//! the scale does not shorten them.

use std::time::Duration;

use varif_core::{
    game_minutes, real_ms_until_next_game_minute, AdvanceKind, ClockAdvance, TimeMode, World,
};

/// What one frame does with the seconds since the previous frame.
///
/// `carry_real_ms` is real time not yet a whole game minute. The world's
/// time scale turns it into game minutes before [`World::advance`]. Zero
/// minutes means the frame did not move the clock. Turn-based mode and a
/// paused clock pass nothing and drop the remainder, so that time does not
/// arrive later when the clock starts.
pub(crate) fn apply_elapsed_frame(
    world: &mut World,
    carry_real_ms: &mut u128,
    frame_seconds: f32,
) -> ClockAdvance {
    if !is_running(world) {
        *carry_real_ms = 0;
        return idle();
    }
    if frame_seconds.is_finite() && frame_seconds > 0.0 {
        let added = (frame_seconds * 1000.0).round() as u128;
        *carry_real_ms = carry_real_ms.saturating_add(added);
    }
    let (minutes, rest) = game_minutes(*carry_real_ms, world.time_scale);
    *carry_real_ms = rest;
    if minutes == 0 {
        return idle();
    }
    world.advance(minutes, AdvanceKind::Elapsed)
}

fn idle() -> ClockAdvance {
    ClockAdvance {
        minutes: 0,
        events: Vec::new(),
        stopped: false,
    }
}

/// Real time is running: passive minutes move the clock.
pub(crate) fn is_running(world: &World) -> bool {
    world.time_mode == TimeMode::RealTime && !world.clock_paused
}

/// How long to wait before drawing again, so the next game minute can show.
///
/// At 1× that is the real seconds still short of a minute. At a higher
/// scale the wait is shorter, down to about one real second at 60×. A late
/// wake still counts. The next frame passes the seconds that actually
/// passed, not this estimate.
pub(crate) fn until_next_minute(carry_real_ms: u128, scale: u32) -> Duration {
    let ms = real_ms_until_next_game_minute(carry_real_ms, scale);
    let secs = (ms as f32 / 1000.0).clamp(0.05, 60.0);
    Duration::try_from_secs_f32(secs).unwrap_or(Duration::from_secs(1))
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
        let mut carry = 0;
        for _ in 0..59 {
            let moved = apply_elapsed_frame(&mut world, &mut carry, 1.0);
            assert_eq!(moved.minutes, 0);
        }
        assert_eq!((world.hour, world.minute), (0, 0));

        let moved = apply_elapsed_frame(&mut world, &mut carry, 1.0);
        assert_eq!(moved.minutes, 1);
        assert_eq!((world.hour, world.minute), (0, 1));
        assert_eq!(carry, 0);
    }

    #[test]
    fn one_real_second_at_sixty_times_is_one_game_minute() {
        let mut world = realtime();
        world.apply(Command::SetTimeScale { scale: 60 }).unwrap();
        let mut carry = 0;
        let moved = apply_elapsed_frame(&mut world, &mut carry, 1.0);
        assert_eq!(moved.minutes, 1);
        assert_eq!((world.hour, world.minute), (0, 1));
        assert_eq!(carry, 0);

        let mut slower = realtime();
        let mut carry = 0;
        let moved = apply_elapsed_frame(&mut slower, &mut carry, 1.0);
        assert_eq!(moved.minutes, 0);
        assert_eq!(slower.minute, 0);
    }

    #[test]
    fn a_predicted_frame_slice_is_not_yet_a_minute() {
        let mut world = realtime();
        let mut carry = 0;
        // What `stable_dt` is when the previous frame did not ask to repaint.
        let moved = apply_elapsed_frame(&mut world, &mut carry, 1.0 / 60.0);
        assert_eq!(moved.minutes, 0);
        assert_eq!(world.minute, 0);
    }

    #[test]
    fn ninety_minutes_of_frame_time_rolls_the_same_clock() {
        let mut world = realtime();
        let start = world.date;
        let mut carry = 0;
        let moved = apply_elapsed_frame(&mut world, &mut carry, 90.0 * 60.0);
        assert_eq!(moved.minutes, 90);
        assert_eq!((world.hour, world.minute), (1, 30));
        assert_eq!(carry, 0);
        assert_eq!(world.date, start);
    }

    #[test]
    fn a_day_of_frame_time_rolls_the_date_through_advance() {
        let mut world = realtime();
        let start = world.date;
        let mut carry = 0;
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
        let mut carry = 45_000;
        let moved = apply_elapsed_frame(&mut world, &mut carry, 3600.0);
        assert_eq!(moved.minutes, 0);
        assert!(moved.events.is_empty());
        assert!(!moved.stopped);
        assert_eq!(world, before);
        assert_eq!(carry, 0);

        world.apply(Command::Step).unwrap();
        assert_eq!((world.hour, world.minute), (1, 0));
    }

    #[test]
    fn a_paused_clock_drops_the_gap_instead_of_saving_it() {
        let mut world = realtime();
        world.clock_paused = true;
        let mut carry = 0;
        let moved = apply_elapsed_frame(&mut world, &mut carry, 120.0);
        assert_eq!(moved.minutes, 0);
        assert_eq!((world.hour, world.minute), (0, 0));
        assert_eq!(carry, 0);

        world.clock_paused = false;
        let moved = apply_elapsed_frame(&mut world, &mut carry, 60.0);
        assert_eq!(moved.minutes, 1);
        assert_eq!(world.minute, 1);
    }

    #[test]
    fn the_wake_waits_out_the_seconds_not_yet_a_minute() {
        assert_eq!(until_next_minute(0, 1), Duration::from_secs(60));
        assert_eq!(until_next_minute(30_000, 1), Duration::from_secs(30));
        assert_eq!(until_next_minute(0, 60), Duration::from_secs(1));
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
            first.contains(&until_next_minute(app.real_carry_ms, app.world.time_scale)),
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
            second.contains(&until_next_minute(app.real_carry_ms, app.world.time_scale)),
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
        assert_eq!(app.real_carry_ms, 0);
        assert!(crate::clock_text(&app.world).contains("00:00"));
        assert!(crate::clock_text(&app.world).contains("turn-based"));
    }

    #[test]
    fn the_play_frame_runs_faster_when_the_time_scale_is_higher() {
        let mut app = play_app(TimeMode::RealTime);
        app.world
            .apply(Command::SetTimeScale { scale: 60 })
            .unwrap();
        let ctx = eframe::egui::Context::default();
        show_play(&mut app, &ctx, 0.0);
        show_play(&mut app, &ctx, 2.0);
        assert_eq!(app.world.hour, 0);
        assert!(
            (1..=2).contains(&app.world.minute),
            "two real seconds at 60× should be one or two game minutes, saw {}",
            app.world.minute
        );
        assert!(crate::clock_text(&app.world).contains("60×"));
    }

    #[test]
    fn changing_the_time_scale_drops_the_leftover_real_time() {
        let mut app = play_app(TimeMode::RealTime);
        app.real_carry_ms = 30_000;
        assert!(app.send(Command::SetTimeScale { scale: 10 }));
        assert_eq!(app.world.time_scale, 10);
        assert_eq!(app.real_carry_ms, 0);

        app.real_carry_ms = 30_000;
        assert!(!app.send(Command::SetTimeScale { scale: 0 }));
        assert_eq!(app.world.time_scale, 10);
        assert_eq!(app.real_carry_ms, 30_000);
    }
}
