//! The first start of an executable without its data (`ReTzar.dat`)
//! beside it, as built from the sources: ReTzar asks for the player's Tzar
//! folder, of any edition, checks that it holds the original game and makes
//! its data pack from it, once, with that edition's content. The release
//! ships the whole pack instead ([`original::require`]).
//! On Windows only: elsewhere (macOS, Linux) the chosen installation is
//! read directly and saved in settings.
use crate::{Result, lang, original, pack, paths, picker};
use std::path::{Path, PathBuf};

/// Whether this start is the first of a portable copy: no game given on
/// the command line or in the environment, and no pack beside the
/// executable.
pub fn applies(args: &[String], exe_dir: &Path) -> bool {
    let _ = (args, exe_dir);
    false
}

/// The installation a chosen folder points to: the folder itself, one of
/// its own (a library of games), or the one above (its `Packs`).
fn installation_at(chosen: &Path) -> Option<PathBuf> {
    if paths::is_installation(chosen) {
        return Some(chosen.to_path_buf());
    }
    let mut inside: Vec<PathBuf> = std::fs::read_dir(chosen)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| paths::is_installation(p))
        .collect();
    inside.sort();
    inside.into_iter().next().or_else(|| {
        chosen
            .parent()
            .filter(|p| paths::is_installation(p))
            .map(Path::to_path_buf)
    })
}

/// Asks for the Tzar folder and makes `ReTzar.dat` beside the executable
/// in `exe_dir`, where the settings also go; the folder is kept as the
/// original copy the game plays beside ([`original::require`]).
#[cfg(windows)]
pub fn prepare(args: &[String], exe_dir: &Path) -> Result<bool> {
    // No portable copy is ever assembled for OpenTzar, even when called
    // from a legacy launch path. The EXE reads original data directly.
    let root = paths::Locations::root_with(args, &|key| std::env::var_os(key), exe_dir);
    let proposed = original::saved(&root).unwrap_or_else(||
        PathBuf::from(r"C:\Program Files\GOG Galaxy\Games\Tzar"));
    original::require_direct(&root, &proposed).map(|_| true)
}

/// macOS reads the original installation directly and remembers it beside
/// the other settings, including when --root or RETZAR_HOME is supplied.
/// Returns false when the player cancels setup.
#[cfg(not(windows))]
pub fn prepare(args: &[String], exe_dir: &Path) -> Result<bool> {
    let root = paths::Locations::root_with(args, &|key| std::env::var_os(key), exe_dir);
    original::speak(&root);
    if !picker::welcome(lang::t(
        "Benvenuto in ReTzar! Scegli la cartella della tua copia di Tzar (GOG, Steam o CD). ReTzar leggerà i suoi file e ricorderà la cartella per i prossimi avvii.",
    )) {
        return Ok(false);
    }
    let mut start = original::saved(&root);
    loop {
        let Some(chosen) = picker::folder(
            lang::t("Scegli la cartella in cui è installato Tzar"),
            start.as_deref(),
        ) else {
            return Ok(false);
        };
        if let Some(found) = installation_at(&chosen).filter(|p| original::holds(p)) {
            std::fs::create_dir_all(&root)?;
            return original::remember(&root, &found).map(|()| true);
        }
        if !picker::confirm(&lang::fill(
            "In {} non c'è una copia originale di Tzar.\n\nScegli la cartella in cui è installato il gioco.",
            &[&chosen.display()],
        )) {
            return Ok(false);
        }
        start = Some(chosen);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The installation is found from the folder itself, from the one that
    /// holds it, or from its `Packs`.
    #[test]
    fn a_chosen_folder_points_to_its_installation() {
        let base = std::env::temp_dir().join(format!("retzar-primo-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let game = base.join("Giochi").join("Tzar");
        for dir in ["Maps", "Packs"] {
            std::fs::create_dir_all(game.join(dir)).unwrap();
        }
        assert_eq!(installation_at(&game), Some(game.clone()));
        assert_eq!(installation_at(&base.join("Giochi")), Some(game.clone()));
        assert_eq!(installation_at(&game.join("Packs")), Some(game.clone()));
        assert_eq!(installation_at(&base), None);
        // A digital release keeps its data loose.
        let loose = base.join("GOG");
        for dir in ["Maps", "DATA", "IMAGES"] {
            std::fs::create_dir_all(loose.join(dir)).unwrap();
        }
        assert_eq!(installation_at(&loose), Some(loose.clone()));
        let mut args = vec!["--source".to_owned(), "x".to_owned()];
        assert!(!applies(&args, &base));
        args.clear();
        if std::env::var_os("TZAR_DATA_ROOT").is_none() {
            assert!(!applies(&args, &base));
            std::fs::write(base.join(pack::FILE), b"").unwrap();
            assert!(!applies(&args, &base));
        }
        std::fs::remove_dir_all(&base).unwrap();
    }
}
