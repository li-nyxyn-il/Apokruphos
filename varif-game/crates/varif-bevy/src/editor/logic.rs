//! The editor's plain logic, kept apart from the egui code so it can be
//! tested without a window: keys, file paths, the problems list and the
//! files on disk.

use std::{
    fs, io,
    path::{Path, PathBuf},
};
use varif_core::{
    check_person_file, load_world_with_people, person_file_of, person_file_text, unreachable_nodes,
    Akstada, Blood, Character, FactionId, Heritage, PersonFile, PersonId, Role, Tongue,
    BASE_ATTITUDE,
};

/// Key given to a new person, before it is made unique.
pub const NEW_KEY: &str = "new-npc";

/// The problems found with one person, errors apart from warnings.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Problems {
    /// These stop a save.
    pub errors: Vec<String>,
    /// These are shown but do not stop a save.
    pub warnings: Vec<String>,
}

/// `base`, or `base-2`, `base-3`, ... when `base` is already taken.
pub fn unique_key(base: &str, taken: &[&str]) -> String {
    if !taken.contains(&base) {
        return base.to_string();
    }
    (2..)
        .map(|n| format!("{base}-{n}"))
        .find(|candidate| !taken.contains(&candidate.as_str()))
        .expect("an unbounded range always finds a free key")
}

/// The key a duplicate of `key` is given.
pub fn copy_key(key: &str, taken: &[&str]) -> String {
    unique_key(&format!("{key}-copy"), taken)
}

/// Why `key` cannot be a file name, if it cannot. Keys become
/// `data/people/<key>.ron`, so they stay to simple lowercase names.
pub fn key_problem(key: &str) -> Option<String> {
    if key.is_empty() {
        return Some("The key is empty.".to_string());
    }
    let simple = key
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_');
    if !simple {
        return Some(format!(
            "The key \"{key}\" may only use lowercase letters, digits, - and _."
        ));
    }
    None
}

/// The folder people files live in, beside the data file.
pub fn people_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("people")
}

/// Where the person with `key` is saved.
pub fn person_path(data_dir: &Path, key: &str) -> PathBuf {
    people_dir(data_dir).join(format!("{key}.ron"))
}

/// True when `data/people/` already holds a file for `key`.
pub fn is_saved(disk: &[(String, String)], key: &str) -> bool {
    disk.iter().any(|(stem, _)| stem == key)
}

/// A sheet for a new ordinary NPC.
pub fn blank_sheet() -> Character {
    Character::new(
        Role::Npc {
            follower: false,
            attitude: BASE_ATTITUDE,
        },
        30,
        Blood::Red,
        Heritage::Shentzi,
        Akstada::Thief,
        vec![Tongue::Shentz],
        "ASSUMPTION: to be written",
        FactionId::Ecumenci,
        Vec::new(),
    )
    .expect("the blank sheet is valid")
}

/// A new person at `place`, with a blank NPC sheet. A place that has sites
/// needs one named, so pass the first of them as `site`.
pub fn blank_person(key: &str, place: &str, site: Option<&str>) -> PersonFile {
    PersonFile {
        key: key.to_string(),
        name: "New NPC".to_string(),
        title: "Resident".to_string(),
        at: place.to_string(),
        site: site.map(str::to_string),
        sheet: Some(blank_sheet()),
        ..PersonFile::default()
    }
}

/// Reads the person with `key` back out of the main data file alone.
pub fn person_from_main(main: &str, key: &str) -> Option<PersonFile> {
    let world = load_world_with_people(main, &[]).ok()?;
    let index = world.people.iter().position(|person| person.key == key)?;
    Some(person_file_of(&world, PersonId(index as u32)))
}

/// Everyone in the world made from the main file and the people files,
/// except the player. A people file replaces a main-file person with the
/// same key.
pub fn people_from(main: &str, disk: &[(String, String)]) -> Result<Vec<PersonFile>, String> {
    let world = load_world_with_people(main, disk).map_err(|error| error.to_string())?;
    Ok(world
        .people
        .iter()
        .enumerate()
        .filter(|(_, person)| !person.is_player())
        .map(|(index, _)| person_file_of(&world, PersonId(index as u32)))
        .collect())
}

/// Checks one person the way a load would, and adds the warnings a load
/// does not raise. `other_keys` are the keys of everyone else in the list.
pub fn find_problems(
    main: &str,
    disk: &[(String, String)],
    file: &PersonFile,
    other_keys: &[&str],
) -> Problems {
    let mut problems = Problems::default();
    if let Some(message) = key_problem(&file.key) {
        problems.errors.push(message);
        return problems;
    }
    if other_keys.contains(&file.key.as_str()) {
        problems.errors.push(format!(
            "Another person in the list already has the key \"{}\".",
            file.key
        ));
    }
    if let Err(error) = check_person_file(main, disk, file) {
        problems.errors.push(error.to_string());
    }
    if let Some(tree) = &file.tree {
        for node in unreachable_nodes(tree) {
            problems
                .warnings
                .push(format!("Unreachable node \"{node}\"."));
        }
    }
    problems
}

/// Writes the person to `data/people/<key>.ron`, making the folder if it
/// is missing, and returns the path written.
pub fn write_person(data_dir: &Path, file: &PersonFile) -> io::Result<PathBuf> {
    if let Some(message) = key_problem(&file.key) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, message));
    }
    fs::create_dir_all(people_dir(data_dir))?;
    let path = person_path(data_dir, &file.key);
    fs::write(&path, person_file_text(file))?;
    Ok(path)
}

/// Removes `data/people/<key>.ron`. Returns false when there was no such
/// file, which is not an error.
pub fn delete_person(data_dir: &Path, key: &str) -> io::Result<bool> {
    if let Some(message) = key_problem(key) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, message));
    }
    match fs::remove_file(person_path(data_dir, key)) {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    const MAIN: &str = include_str!("../../../../data/first_leg.ron");

    /// A fresh empty folder under the system temp dir.
    fn scratch_dir() -> PathBuf {
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let dir = std::env::temp_dir().join(format!(
            "varif-editor-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// The first place and, if it has sites, its first site.
    fn first_place() -> (String, Option<String>) {
        let world = load_world_with_people(MAIN, &[]).unwrap();
        let place = &world.places[0];
        (
            place.key.clone(),
            place.sites.first().map(|site| site.key.clone()),
        )
    }

    fn blank_at_start(key: &str) -> PersonFile {
        let (place, site) = first_place();
        blank_person(key, &place, site.as_deref())
    }

    #[test]
    fn unique_key_counts_up_past_taken_keys() {
        assert_eq!(unique_key("new-npc", &[]), "new-npc");
        assert_eq!(unique_key("new-npc", &["new-npc"]), "new-npc-2");
        assert_eq!(
            unique_key("new-npc", &["new-npc", "new-npc-2", "other"]),
            "new-npc-3"
        );
    }

    #[test]
    fn copy_key_appends_copy_and_stays_unique() {
        assert_eq!(copy_key("jonnoj", &["jonnoj"]), "jonnoj-copy");
        assert_eq!(
            copy_key("jonnoj", &["jonnoj", "jonnoj-copy"]),
            "jonnoj-copy-2"
        );
    }

    #[test]
    fn key_problem_rejects_unsafe_names() {
        assert!(key_problem("jonnoj").is_none());
        assert!(key_problem("new-npc-2").is_none());
        assert!(key_problem("").is_some());
        assert!(key_problem("../evil").is_some());
        assert!(key_problem("a/b").is_some());
        assert!(key_problem("Upper").is_some());
        assert!(key_problem("has space").is_some());
    }

    #[test]
    fn person_path_is_under_people() {
        let path = person_path(Path::new("data"), "jonnoj");
        assert_eq!(path, Path::new("data").join("people").join("jonnoj.ron"));
    }

    #[test]
    fn is_saved_looks_for_the_file_stem() {
        let disk = vec![("jonnoj".to_string(), "()".to_string())];
        assert!(is_saved(&disk, "jonnoj"));
        assert!(!is_saved(&disk, "other"));
    }

    #[test]
    fn main_file_people_exclude_the_player_and_include_jonnoj() {
        let people = people_from(MAIN, &[]).unwrap();
        assert!(people.iter().any(|p| p.key == "jonnoj"));
        assert!(people
            .iter()
            .all(|p| !matches!(p.sheet.as_ref().map(|s| s.role), Some(Role::Player))));
        assert!(person_from_main(MAIN, "jonnoj").is_some());
        assert!(person_from_main(MAIN, "nobody").is_none());
    }

    #[test]
    fn a_blank_person_has_no_problems() {
        let file = blank_at_start(NEW_KEY);
        let problems = find_problems(MAIN, &[], &file, &[]);
        assert_eq!(problems, Problems::default());
        let sheet = file.sheet.unwrap();
        assert_eq!(sheet.age, 30);
        assert_eq!(sheet.background, "ASSUMPTION: to be written");
    }

    #[test]
    fn an_untouched_main_file_person_has_no_problems() {
        let file = person_from_main(MAIN, "jonnoj").unwrap();
        let problems = find_problems(MAIN, &[], &file, &[]);
        assert!(problems.errors.is_empty(), "{:?}", problems.errors);
    }

    #[test]
    fn problems_report_an_unknown_place_and_a_taken_key() {
        let mut file = blank_person(NEW_KEY, "nowhere", None);
        let problems = find_problems(MAIN, &[], &file, &[]);
        assert_eq!(problems.errors.len(), 1, "{:?}", problems.errors);

        file = blank_at_start(NEW_KEY);
        let problems = find_problems(MAIN, &[], &file, &[NEW_KEY]);
        assert_eq!(problems.errors.len(), 1, "{:?}", problems.errors);
        assert!(problems.errors[0].contains("already has the key"));
    }

    #[test]
    fn problems_report_a_bad_key_without_checking_further() {
        let file = blank_at_start("Bad Key");
        let problems = find_problems(MAIN, &[], &file, &[]);
        assert_eq!(problems.errors.len(), 1);
    }

    #[test]
    fn write_then_delete_a_person_file() {
        let dir = scratch_dir();
        let file = blank_at_start("test-writer");

        let path = write_person(&dir, &file).unwrap();
        assert_eq!(path, person_path(&dir, "test-writer"));
        assert_eq!(fs::read_to_string(&path).unwrap(), person_file_text(&file));

        // What was written loads back alongside the main file.
        let disk = varif_core::read_people_dir(&people_dir(&dir)).unwrap();
        assert!(is_saved(&disk, "test-writer"));
        let world = load_world_with_people(MAIN, &disk).unwrap();
        assert!(world.people.iter().any(|p| p.key == "test-writer"));

        assert!(delete_person(&dir, "test-writer").unwrap());
        assert!(!path.exists());
        assert!(!delete_person(&dir, "test-writer").unwrap());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn write_and_delete_refuse_unsafe_keys() {
        let dir = scratch_dir();
        let mut file = blank_at_start("ok");
        file.key = "../escape".to_string();
        assert!(write_person(&dir, &file).is_err());
        assert!(delete_person(&dir, "../escape").is_err());
        let _ = fs::remove_dir_all(&dir);
    }
}
