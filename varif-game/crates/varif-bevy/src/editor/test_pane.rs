//! The test-conversation pane on the Dialogue tab: stand the party in front
//! of the person being edited, with attitude, hostility, hour, follower and
//! flags set as asked, and talk to them.
//!
//! The conversation runs in a sandbox world built from the main data text,
//! the people files on disk and the person's file as it is in memory. Nothing
//! here changes the editor's people or the files on disk. The driving
//! logic is plain functions, tested without a screen.

use bevy_egui::egui;
use std::collections::BTreeSet;
use varif_core::{
    sandbox, Command, PersonFile, PersonId, SandboxSetup, World, BASE_ATTITUDE, HOURS_PER_DAY,
};

/// One line of the log.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogLine {
    pub text: String,
    pub error: bool,
}

/// A running test conversation.
#[derive(Clone)]
pub struct Session {
    world: World,
    person: PersonId,
    /// The file the sandbox was built from, to notice later edits.
    started_with: PersonFile,
    pub log: Vec<LogLine>,
}

impl Session {
    /// What the person says now, if a conversation is open.
    pub fn line(&self) -> Option<String> {
        self.world.npc_line().map(str::to_string)
    }

    /// The player's lines on offer now.
    pub fn options(&self) -> Vec<String> {
        self.world
            .dialogue_options(self.person)
            .into_iter()
            .map(|option| option.text)
            .collect()
    }

    /// True when `file` is no longer what the sandbox was built from.
    pub fn is_stale(&self, file: &PersonFile) -> bool {
        &self.started_with != file
    }

    fn run(&mut self, command: Command) {
        match self.world.apply(command) {
            Ok(events) => {
                for event in events {
                    self.log.push(LogLine {
                        text: format!("{event:?}"),
                        error: false,
                    });
                }
            }
            Err(error) => self.log.push(LogLine {
                text: format!("Error: {error}"),
                error: true,
            }),
        }
    }
}

/// Builds the sandbox and starts the conversation. A conversation that
/// cannot open (nothing to say, say) still gives a session with the error
/// in its log; only a sandbox that cannot be built is an `Err`. Starting
/// again with the same inputs is how a restart is done.
pub fn start(
    main: &str,
    disk: &[(String, String)],
    file: &PersonFile,
    setup: &SandboxSetup,
) -> Result<Session, String> {
    let (world, person) = sandbox(main, disk, file, setup).map_err(|error| error.to_string())?;
    let mut session = Session {
        world,
        person,
        started_with: file.clone(),
        log: Vec::new(),
    };
    session.run(Command::Interact { person });
    Ok(session)
}

/// Says the option at `index`, logging what happened.
pub fn say(session: &mut Session, index: usize) {
    session.run(Command::Say { line: index });
}

/// The flags in a text box: separated by commas or new lines, trimmed,
/// blanks dropped.
pub fn parse_flags(text: &str) -> BTreeSet<String> {
    text.split([',', '\n'])
        .map(str::trim)
        .filter(|flag| !flag.is_empty())
        .map(str::to_string)
        .collect()
}

/// What the pane's inputs hold, kept per person.
#[derive(Clone)]
struct PaneState {
    attitude: u32,
    hostility: u32,
    hour: u32,
    follower: bool,
    flags: String,
    session: Option<Session>,
    error: String,
}

impl Default for PaneState {
    fn default() -> Self {
        PaneState {
            attitude: BASE_ATTITUDE,
            hostility: 50,
            hour: 12,
            follower: false,
            flags: String::new(),
            session: None,
            error: String::new(),
        }
    }
}

impl PaneState {
    fn setup(&self) -> SandboxSetup {
        SandboxSetup {
            attitude: self.attitude,
            hostility: self.hostility,
            follower: self.follower,
            flags: parse_flags(&self.flags),
            hour: self.hour,
        }
    }
}

/// Draws the pane. It reads the person and the editor's data and changes
/// neither.
pub fn ui(ui: &mut egui::Ui, file: &PersonFile, main: &str, disk: &[(String, String)]) {
    let id = egui::Id::new(("test_pane", file.key.as_str()));
    let mut pane: PaneState = ui.data_mut(|data| data.get_temp(id)).unwrap_or_default();

    ui.heading("Test conversation");
    egui::Grid::new("test_setup").num_columns(2).show(ui, |ui| {
        ui.label("Attitude");
        ui.add(egui::Slider::new(&mut pane.attitude, 0..=100));
        ui.end_row();
        ui.label("Hostility");
        ui.add(egui::Slider::new(&mut pane.hostility, 0..=100));
        ui.end_row();
        ui.label("Hour");
        ui.add(egui::Slider::new(&mut pane.hour, 0..=HOURS_PER_DAY - 1));
        ui.end_row();
    });
    ui.checkbox(&mut pane.follower, "Follower");
    ui.label("Flags (comma or line separated)");
    ui.add(egui::TextEdit::multiline(&mut pane.flags).desired_rows(2));

    let label = if pane.session.is_some() {
        "Restart"
    } else {
        "Start"
    };
    if ui.button(label).clicked() {
        match start(main, disk, file, &pane.setup()) {
            Ok(session) => {
                pane.session = Some(session);
                pane.error.clear();
            }
            Err(error) => {
                pane.session = None;
                pane.error = error;
            }
        }
    }
    if !pane.error.is_empty() {
        ui.colored_label(egui::Color32::RED, &pane.error);
    }

    let mut say_index = None;
    if let Some(session) = &pane.session {
        if session.is_stale(file) {
            ui.weak("Restart to pick up edits");
        }
        ui.separator();
        match session.line() {
            Some(line) => {
                ui.label(egui::RichText::new(line).strong());
            }
            None => {
                ui.weak("No conversation is open.");
            }
        }
        for (index, text) in session.options().into_iter().enumerate() {
            if ui.button(text).clicked() {
                say_index = Some(index);
            }
        }
        ui.separator();
        egui::ScrollArea::vertical()
            .id_salt("test_log")
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for line in &session.log {
                    let text = egui::RichText::new(&line.text);
                    if line.error {
                        ui.label(text.color(egui::Color32::RED));
                    } else {
                        ui.label(text);
                    }
                }
            });
    }
    if let (Some(index), Some(session)) = (say_index, pane.session.as_mut()) {
        say(session, index);
    }
    ui.data_mut(|data| data.insert_temp(id, pane));
}

#[cfg(test)]
mod tests {
    use super::super::logic::{people_from, scratch_dir, TEST_MAIN};
    use super::super::EditorState;
    use super::*;
    use varif_core::{Choice, Compare, Condition, DialogueTree, Entry, Node};

    fn choice(text: &str, when: Condition) -> Choice {
        Choice {
            text: text.to_string(),
            when,
            actions: Vec::new(),
            next: None,
        }
    }

    fn person_with(entry: Condition) -> PersonFile {
        let mut file = people_from(TEST_MAIN, &[])
            .unwrap()
            .into_iter()
            .find(|p| p.key == "jonnoj")
            .unwrap();
        file.tree = Some(DialogueTree {
            entries: vec![Entry {
                when: entry,
                node: "start".to_string(),
            }],
            nodes: vec![Node {
                key: "start".to_string(),
                line: "Well met.".to_string(),
                sprite: None,
                choices: vec![
                    choice("Hello.", Condition::Always),
                    choice("Trust me.", Condition::Attitude(Compare::AtLeast(70))),
                ],
            }],
        });
        file
    }

    fn setup(attitude: u32) -> SandboxSetup {
        SandboxSetup {
            attitude,
            hostility: 50,
            follower: false,
            flags: BTreeSet::new(),
            hour: 12,
        }
    }

    #[test]
    fn a_gated_choice_shows_only_at_high_attitude() {
        let file = person_with(Condition::Always);
        let low = start(TEST_MAIN, &[], &file, &setup(40)).unwrap();
        assert_eq!(low.line().as_deref(), Some("Well met."));
        assert_eq!(low.options(), vec!["Hello.".to_string()]);
        assert!(!low.log.is_empty());
        assert!(low.log.iter().all(|l| !l.error));

        let high = start(TEST_MAIN, &[], &file, &setup(80)).unwrap();
        assert_eq!(high.options().len(), 2);
    }

    #[test]
    fn saying_a_choice_logs_its_events_and_ends_the_conversation() {
        let file = person_with(Condition::Always);
        let mut session = start(TEST_MAIN, &[], &file, &setup(80)).unwrap();
        let before = session.log.len();
        say(&mut session, 1);
        assert!(session.log.len() > before);
        assert!(session.log.iter().all(|l| !l.error));
        assert!(session.line().is_none());
        assert!(session.options().is_empty());
    }

    #[test]
    fn an_entry_that_does_not_hold_logs_an_error() {
        let file = person_with(Condition::Flag("never".to_string()));
        let session = start(TEST_MAIN, &[], &file, &setup(50)).unwrap();
        let last = session.log.last().unwrap();
        assert!(last.error, "{:?}", session.log);
        assert!(last.text.contains("nothing to say"), "{}", last.text);
    }

    #[test]
    fn a_set_flag_opens_the_entry_and_a_bad_say_logs_an_error() {
        let file = person_with(Condition::Flag("met".to_string()));
        let mut flagged = setup(50);
        flagged.flags = parse_flags("met");
        let mut session = start(TEST_MAIN, &[], &file, &flagged).unwrap();
        assert!(session.log.iter().all(|l| !l.error));
        say(&mut session, 9);
        assert!(session.log.last().unwrap().error);
    }

    #[test]
    fn the_session_notices_edits_and_a_new_start_picks_them_up() {
        let mut file = person_with(Condition::Always);
        let session = start(TEST_MAIN, &[], &file, &setup(50)).unwrap();
        assert!(!session.is_stale(&file));
        file.tree.as_mut().unwrap().nodes[0].line = "Changed.".to_string();
        assert!(session.is_stale(&file));
        let again = start(TEST_MAIN, &[], &file, &setup(50)).unwrap();
        assert_eq!(again.line().as_deref(), Some("Changed."));
    }

    #[test]
    fn testing_leaves_the_editor_state_and_disk_alone() {
        let dir = scratch_dir();
        let (state, _) = EditorState::from_parts(TEST_MAIN.to_string(), dir.to_path_buf()).unwrap();
        let people = state.people.clone();
        let disk = state.disk.clone();
        let file = person_with(Condition::Always);
        let mut session = start(&state.main_text, &state.disk, &file, &setup(80)).unwrap();
        say(&mut session, 0);
        assert_eq!(state.people, people);
        assert_eq!(state.disk, disk);
        assert!(!dir.join("people").exists());
    }

    #[test]
    fn flags_split_on_commas_and_new_lines() {
        let flags = parse_flags(" a, b\n\n c ,, \n");
        let want: BTreeSet<String> = ["a", "b", "c"].iter().map(|s| s.to_string()).collect();
        assert_eq!(flags, want);
        assert!(parse_flags("  \n , ").is_empty());
    }
}
