//! Where the game writes its journal. All functions are pure: the caller passes the environment in.
//! Folder layout: <https://elite-journal.readthedocs.io/en/latest/File%20Format.html>

use std::path::{Path, PathBuf};

/// Operating systems the game (or Proton) runs on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    /// Native Windows; `home` is the user profile.
    Windows,
    /// Linux, running the game through Steam Proton.
    Linux,
}

/// Steam app id of Elite Dangerous; names its Proton prefix.
const APP_ID: &str = "359320";

fn saved_games(root: &Path) -> PathBuf {
    root.join("Saved Games")
        .join("Frontier Developments")
        .join("Elite Dangerous")
}

/// Candidate journal folders, best guess first.
pub fn candidates(platform: Platform, home: &Path, xdg_data_home: Option<&Path>) -> Vec<PathBuf> {
    match platform {
        Platform::Windows => vec![saved_games(home)],
        Platform::Linux => {
            let data_home = xdg_data_home
                .map(Path::to_path_buf)
                .unwrap_or_else(|| home.join(".local/share"));
            // Native Steam, the legacy symlink, then the Flatpak sandbox.
            // https://github.com/ValveSoftware/Proton/wiki/Proton-FAQ (prefix per app id under compatdata)
            [
                data_home.join("Steam"),
                home.join(".steam/steam"),
                home.join(".var/app/com.valvesoftware.Steam/.local/share/Steam"),
            ]
            .into_iter()
            .map(|steam| {
                saved_games(
                    &steam
                        .join("steamapps/compatdata")
                        .join(APP_ID)
                        .join("pfx/drive_c/users/steamuser"),
                )
            })
            .collect()
        }
    }
}

/// First candidate that exists. `exists` is a parameter so tests need no disk.
pub fn detect(candidates: &[PathBuf], exists: impl Fn(&Path) -> bool) -> Option<PathBuf> {
    candidates.iter().find(|p| exists(p)).cloned()
}

/// The platform this binary runs on, if the game is supported there.
pub fn current_platform() -> Option<Platform> {
    if cfg!(windows) {
        Some(Platform::Windows)
    } else if cfg!(target_os = "linux") {
        Some(Platform::Linux)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TAIL: &str = "Saved Games/Frontier Developments/Elite Dangerous";

    #[test]
    fn windows_uses_saved_games() {
        let c = candidates(Platform::Windows, Path::new("/u/me"), None);
        assert_eq!(c, vec![Path::new("/u/me").join(TAIL)]);
    }

    #[test]
    fn linux_proton_default_order() {
        let c = candidates(Platform::Linux, Path::new("/home/me"), None);
        let prefix = "steamapps/compatdata/359320/pfx/drive_c/users/steamuser";
        assert_eq!(
            c,
            vec![
                Path::new("/home/me/.local/share/Steam")
                    .join(prefix)
                    .join(TAIL),
                Path::new("/home/me/.steam/steam").join(prefix).join(TAIL),
                Path::new("/home/me/.var/app/com.valvesoftware.Steam/.local/share/Steam")
                    .join(prefix)
                    .join(TAIL),
            ]
        );
    }

    #[test]
    fn linux_honours_xdg_data_home() {
        let c = candidates(
            Platform::Linux,
            Path::new("/home/me"),
            Some(Path::new("/data")),
        );
        assert!(c[0].starts_with("/data/Steam"));
        assert_eq!(c.len(), 3);
    }

    #[test]
    fn detect_picks_first_existing() {
        let c = candidates(Platform::Linux, Path::new("/home/me"), None);
        let second = c[1].clone();
        let third = c[2].clone();
        assert_eq!(
            detect(&c, |p| p == second || p == third),
            Some(second.clone())
        );
        assert_eq!(detect(&c, |_| false), None);
    }
}
