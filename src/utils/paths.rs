use std::path::PathBuf;

/// Expand `~` to the user's home directory.
pub fn expand_home(path: &str) -> PathBuf {
    if path.starts_with("~/") || path == "~" {
        if let Some(home_dir) = home::home_dir() {
            if path == "~" {
                return home_dir;
            }
            return home_dir.join(&path[2..]);
        }
    }
    PathBuf::from(path)
}

/// Get the default destination path: `~/.agent/skills/`
pub fn get_default_destination() -> PathBuf {
    expand_home("~/.agent/skills")
}
