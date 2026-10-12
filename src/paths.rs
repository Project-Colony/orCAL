//! Where orCAL keeps its files: `<platform root>/Colony/orCAL/`.
//!
//! This is the table of Project-Colony-Resources `design/filesystem.md`, the
//! one `colony_ui::paths` implements for the iced programs. orCAL is a Tauri
//! program and does not link colony-ui, so the table is reproduced here with
//! `dirs`, rule for rule:
//!
//! | Kind | Linux | Windows | macOS |
//! |---|---|---|---|
//! | config | `~/.config/Colony/orCAL/` | `%LOCALAPPDATA%\Colony\orCAL\` | `~/Library/Application Support/Colony/orCAL/` |
//! | data | `~/.local/share/Colony/orCAL/` | `%LOCALAPPDATA%\Colony\orCAL\` | `~/Library/Application Support/Colony/orCAL/` |
//! | cache | `~/.cache/Colony/orCAL/` | `%LOCALAPPDATA%\Colony\orCAL\cache\` | `~/Library/Caches/Colony/orCAL/` |
//!
//! Windows uses `AppData\Local`, never `Roaming`, which is why this uses
//! `config_local_dir` and not `config_dir` (or Tauri's `app_config_dir`, which
//! is Roaming). Config and data share a root on Windows and macOS, so they are
//! kept apart by sub-directory, never by relying on the roots differing.

use std::ffi::OsStr;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// The organisation directory every Colony program nests under.
const VENDOR: &str = "Colony";

/// orCAL's directory name: the display name, capitalised as orCAL spells it.
pub const PROGRAM: &str = "orCAL";

/// `<data>/Colony/apps/` is the shared install root of the Colony launcher, so
/// no program may be called that.
const APPS: &str = "apps";

/// Windows has no cache root of its own: `dirs::cache_dir()` is LocalAppData,
/// the config and data root. This component keeps "clear the cache" from
/// taking the preferences with it.
const CACHE: &str = "cache";

/// `program` as a single path component that cannot escape the Colony tree
/// (`..`, separators, an empty name), alias another folder (Windows strips a
/// trailing dot or space) or land on the shared `apps` root (in any case, as
/// Windows and macOS file systems ignore it).
fn component(program: &str) -> io::Result<&str> {
    let invalid = program.is_empty()
        || program.contains(['/', '\\', '\0'])
        || program.ends_with(['.', ' '])
        || program.to_uppercase().eq_ignore_ascii_case(APPS)
        || Path::new(program).file_name() != Some(OsStr::new(program));
    if invalid {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{program:?} cannot name a Colony program directory"),
        ));
    }
    Ok(program)
}

/// `<base>/Colony/<program>`, or why there is none.
fn under(base: Option<PathBuf>, kind: &str, program: &str) -> io::Result<PathBuf> {
    let base = base.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("this platform has no {kind} directory"),
        )
    })?;
    Ok(base.join(VENDOR).join(component(program)?))
}

/// The cache directory for a program root, with the `cache` component Windows
/// needs.
fn cache_in(program_root: PathBuf, windows: bool) -> PathBuf {
    if windows {
        program_root.join(CACHE)
    } else {
        program_root
    }
}

/// The same directories, without creating them: for showing a path (the About
/// page) without bringing it into existence.
pub mod locate {
    use super::*;

    /// What the user chose: preferences and the window position.
    pub fn config_dir() -> io::Result<PathBuf> {
        under(dirs::config_local_dir(), "config", PROGRAM)
    }

    /// What orCAL produced: the webview profile.
    pub fn data_dir() -> io::Result<PathBuf> {
        under(dirs::data_local_dir(), "data", PROGRAM)
    }

    /// What can be rebuilt. orCAL keeps nothing there yet; the About page
    /// still shows where it would go.
    pub fn cache_dir() -> io::Result<PathBuf> {
        Ok(cache_in(
            under(dirs::cache_dir(), "cache", PROGRAM)?,
            cfg!(windows),
        ))
    }
}

/// [`locate::config_dir`], created if missing.
pub fn config_dir() -> io::Result<PathBuf> {
    let dir = locate::config_dir()?;
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// [`locate::data_dir`], created if missing.
pub fn data_dir() -> io::Result<PathBuf> {
    let dir = locate::data_dir()?;
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Replace `path` with `contents` in one step. The bytes go to a sibling
/// temporary file, reach the disk, then a rename swaps it in: a crash or a full
/// disk mid-write leaves the previous file, never half of a new one.
pub fn write_atomic(path: &Path, contents: &[u8]) -> io::Result<()> {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    let written = fs::File::create(&tmp).and_then(|mut file| {
        file.write_all(contents)?;
        file.sync_all()
    });
    match written.and_then(|()| fs::rename(&tmp, path)) {
        Ok(()) => Ok(()),
        Err(error) => {
            let _ = fs::remove_file(&tmp);
            Err(error)
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A fresh directory under the system temp dir, unique to this test run.
    pub(crate) fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("orcal-test-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn every_root_nests_under_colony_then_orcal() {
        for path in [
            locate::config_dir().unwrap(),
            locate::data_dir().unwrap(),
            cache_in(under(dirs::cache_dir(), "cache", PROGRAM).unwrap(), false),
        ] {
            assert!(path.ends_with(Path::new("Colony").join("orCAL")), "{path:?}");
        }
    }

    #[test]
    fn a_program_name_cannot_escape_or_alias_a_folder() {
        let base = Some(PathBuf::from("base"));
        for bad in [
            "", ".", "..", "../etc", "a/b", "a\\b", "nul\0", "orCAL.", "orCAL ", "apps", "Apps",
        ] {
            assert!(under(base.clone(), "config", bad).is_err(), "{bad:?}");
        }
        for good in ["orCAL", "SAM - Colony Edition"] {
            assert!(under(base.clone(), "config", good).is_ok(), "{good:?}");
        }
        assert!(under(None, "config", "orCAL").is_err());
    }

    #[test]
    fn windows_keeps_the_cache_in_its_own_component() {
        let root = PathBuf::from("LocalAppData").join("Colony").join("orCAL");
        assert_eq!(cache_in(root.clone(), true), root.join("cache"));
        assert_eq!(cache_in(root.clone(), false), root);
    }

    /// Deleting the cache must never take config or data with it, including on
    /// Windows, where all three roots are the same LocalAppData folder.
    #[test]
    fn the_cache_never_is_or_contains_config_or_data() {
        let shared = Some(PathBuf::from("LocalAppData"));
        let config = under(shared.clone(), "config", PROGRAM).unwrap();
        let cache = cache_in(under(shared, "cache", PROGRAM).unwrap(), true);
        assert_ne!(cache, config);
        assert!(!config.starts_with(&cache));

        let cache = locate::cache_dir().unwrap();
        for other in [locate::config_dir().unwrap(), locate::data_dir().unwrap()] {
            assert_ne!(cache, other);
            assert!(!other.starts_with(&cache), "{other:?} inside {cache:?}");
        }
    }

    #[test]
    fn an_atomic_write_replaces_the_file_and_leaves_no_temporary() {
        let dir = temp_dir("atomic");
        let file = dir.join("state.json");
        write_atomic(&file, b"one").unwrap();
        write_atomic(&file, b"two").unwrap();
        assert_eq!(fs::read(&file).unwrap(), b"two");
        assert!(!dir.join("state.json.tmp").exists());
        fs::remove_dir_all(dir).unwrap();
    }
}
