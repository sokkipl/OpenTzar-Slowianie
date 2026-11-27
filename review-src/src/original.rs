//! ReTzar ships the whole game, every edition's campaigns, voices and
//! languages, and starts beside any copy of Tzar. The folder the player
//! chooses must hold some of the original game's files, known here only by
//! a digest of their names: any edition passes, in any language and modded
//! or not, with its archives or its loose files wherever they lie in the
//! folder, and a folder without the game does not.
use crate::{
    Result,
    archive::Archive,
    invalid,
    lang::{self, Language},
};
use std::{
    collections::{BTreeSet, VecDeque},
    path::{Path, PathBuf},
};

/// Digests ([`digest`]) of the names of the files looked for.
const KNOWN: [[u8; 16]; 8] = [
    [
        203, 101, 216, 159, 254, 96, 194, 121, 92, 104, 61, 84, 33, 136, 145, 185,
    ],
    [
        45, 57, 243, 83, 42, 134, 153, 242, 116, 122, 37, 212, 240, 123, 209, 233,
    ],
    [
        179, 5, 94, 189, 201, 134, 35, 151, 247, 11, 154, 17, 141, 223, 140, 190,
    ],
    [
        53, 15, 251, 85, 51, 242, 204, 184, 50, 146, 112, 49, 144, 63, 74, 142,
    ],
    [
        151, 30, 236, 161, 149, 85, 245, 78, 102, 29, 210, 75, 151, 75, 106, 81,
    ],
    [
        248, 250, 210, 51, 133, 14, 46, 243, 167, 7, 104, 19, 207, 244, 159, 101,
    ],
    [
        63, 72, 242, 115, 227, 231, 32, 100, 149, 48, 128, 200, 179, 202, 38, 18,
    ],
    [
        231, 165, 7, 73, 11, 138, 142, 10, 135, 57, 103, 144, 31, 206, 85, 127,
    ],
];
/// How many of them the folder must hold.
const NEEDED: usize = 4;
/// How deep below the chosen folder, and through how many files, the search
/// goes before giving up.
const DEPTH: usize = 4;
const FILES: usize = 50_000;
/// Where the folder is kept, in `settings.json`.
const KEY: &str = "original_folder";

/// The digest of a file's name, without its folders and in lower case.
fn digest(path: &str) -> [u8; 16] {
    let name = path.rsplit(['/', '\\']).next().unwrap_or(path);
    let hash = blake3::Hasher::new_derive_key("ReTzar original file name")
        .update(name.to_ascii_lowercase().as_bytes())
        .finalize();
    let mut out = [0; 16];
    out.copy_from_slice(&hash.as_bytes()[..16]);
    out
}

/// Notes the file named `path` among those found when its name is known.
fn note(found: &mut BTreeSet<usize>, path: &str) {
    let d = digest(path);
    if let Some(i) = KNOWN.iter().position(|k| *k == d) {
        found.insert(i);
    }
}

/// Whether `folder` holds a copy of Tzar: the folders nearest to it first,
/// the names inside its archives and those of its loose files.
pub fn holds(folder: &Path) -> bool {
    let mut found = BTreeSet::new();
    let mut queue = VecDeque::from([(folder.to_path_buf(), 0)]);
    let mut seen = 0;
    while let Some((dir, depth)) = queue.pop_front() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(meta) = std::fs::metadata(&path) else {
                continue;
            };
            if meta.is_dir() {
                if depth < DEPTH {
                    queue.push_back((path, depth + 1));
                }
                continue;
            }
            seen += 1;
            if seen > FILES {
                return false;
            }
            let archive = path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("wdt"));
            if archive && let Ok(a) = Archive::open(&path) {
                for e in &a.entries {
                    note(&mut found, &e.name);
                }
            } else {
                note(&mut found, &entry.file_name().to_string_lossy());
            }
            if found.len() >= NEEDED {
                return true;
            }
        }
    }
    false
}

fn settings(root: &Path) -> serde_json::Value {
    std::fs::read(root.join("settings.json"))
        .ok()
        .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok())
        .filter(serde_json::Value::is_object)
        .unwrap_or_else(|| serde_json::json!({}))
}
/// The folder chosen before.
pub(crate) fn saved(root: &Path) -> Option<PathBuf> {
    settings(root).get(KEY)?.as_str().map(PathBuf::from)
}
/// Keeps the chosen folder in `settings.json`, with the rest of it.
pub fn remember(root: &Path, folder: &Path) -> Result<()> {
    let mut s = settings(root);
    s[KEY] = folder.to_string_lossy().into_owned().into();
    crate::write_atomic(&root.join("settings.json"), &serde_json::to_vec_pretty(&s)?)
}
/// The questions in the player's language, before the game has started:
/// the one of `settings.json`, else the default.
pub(crate) fn speak(root: &Path) {
    let language = settings(root)
        .get("language")
        .and_then(serde_json::Value::as_u64)
        .filter(|_| settings(root).get("opentzar_language_migrated")
            .and_then(serde_json::Value::as_bool).unwrap_or(false))
        .map_or(crate::menu::Settings::default().language, |i| i as usize);
    lang::set(Language::from_index(language));
}
/// Why the game does not start when no folder is chosen.
pub(crate) fn refused() -> Box<dyn std::error::Error + Send + Sync> {
    invalid(lang::t(
        "Per giocare con ReTzar serve una copia originale di Tzar (dal CD, da GOG o da Steam): alla prossima partenza scegli la cartella in cui è installata.",
    ))
}

/// Lets the game start only beside a copy of Tzar: the folder chosen
/// before, or one the player chooses now and that is kept. Closing a
/// window ends the game with the reason.
pub fn require(root: &Path) -> Result<()> {
    let before = saved(root);
    if before.as_deref().is_some_and(holds) {
        return Ok(());
    }
    speak(root);
    let mut start = before;
    loop {
        let chosen = crate::picker::folder(
            lang::t("Scegli la cartella in cui è installato Tzar"),
            start.as_deref(),
        )
        .ok_or_else(refused)?;
        if holds(&chosen) {
            remember(root, &chosen)?;
            return Ok(());
        }
        if !crate::picker::confirm(&lang::fill(
            "In {} non c'è una copia originale di Tzar.\n\nScegli la cartella in cui è installato il gioco.",
            &[&chosen.display()],
        )) {
            return Err(refused());
        }
        start = Some(chosen);
    }
}

/// TEST3: verify the physical GOG (or other retail) installation each run.
/// This gate is invoked from main, including a direct Explorer double-click.
/// No game data is copied into the mod directory.
pub fn require_direct(root: &Path, candidate: &Path) -> Result<PathBuf> {
    speak(root);
    let saved = saved(root);
    let confirmed = settings(root)
        .get("opentzar_installation_confirmed")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    if confirmed && saved.as_deref() == Some(candidate) && verify_direct(candidate).is_ok() {
        return Ok(candidate.to_path_buf());
    }
    let mut start = candidate.to_path_buf();
    let mut auto = crate::paths::is_installation(candidate) && crate::picker::welcome(&format!(
        "Wykryto instalację Tzara:\n\n{}\n\nOK — zatwierdź; Anuluj — wybierz inny folder.",
        candidate.display()
    ));
    let chosen = loop {
        let path = if auto {
            auto = false;
            start.clone()
        } else {
            crate::picker::folder("Wybierz folder z zainstalowaną grą Tzar (WDT lub GOG offline)", Some(&start))
                .ok_or_else(|| invalid("Nie wybrano instalacji Tzara. OpenTzar nie uruchomi się bez oryginalnej gry."))?
        };
        // Selecting Packs/DATA/IMAGES rather than the game root is common.
        let path = if crate::paths::is_installation(&path) {
            path
        } else if let Some(parent) = path.parent().filter(|p| crate::paths::is_installation(p)) {
            parent.to_path_buf()
        } else { path };
        match verify_direct(&path) {
            Ok(()) => break path,
            Err(error) => {
                eprintln!("[TEST19.1] {error}");
                if !crate::picker::confirm(&format!("{error}\n\nCzy chcesz wskazać inny folder?")) {
                    return Err(error);
                }
                start = path;
            }
        }
    };
    let mut value = settings(root);
    value["original_folder"] = chosen.to_string_lossy().into_owned().into();
    value["opentzar_installation_confirmed"] = true.into();
    crate::write_atomic(&root.join("settings.json"), &serde_json::to_vec_pretty(&value)?)?;
    Ok(chosen)
}

pub fn verify_direct(folder: &Path) -> Result<()> {
    if !folder.is_dir() || !crate::paths::is_installation(folder) {
        return Err(invalid(format!(
            "Nie znaleziono prawidłowej instalacji Tzara w: {}.\n\nOpenTzar wymaga instalacji gry (GOG/CD/Steam). Nie używa ReTzar.dat. Obsługiwane są Packs (WDT) oraz rozpakowane DATA i IMAGES.\nAby zmienić folder, uruchom z --source ŚCIEŻKA.",
            folder.display()
        )));
    }
    let assets = crate::assets::GameAssets::open(folder).map_err(|e|
        invalid(format!("Nie można otworzyć danych gry w {}: {e}", folder.display())))?;
    for name in ["DATA/IMAGES.INI", "DATA/ICONS.INI", "DATA/SOUNDS.INI"] {
        let bytes = assets.original(name).map_err(|e|
            invalid(format!("Błąd czytania zasobu {} z {}: {e}", name, folder.display())))?;
        if bytes.is_empty() || bytes.len() > 16 * 1024 * 1024 {
            return Err(invalid(format!("Uszkodzony lub nieprawidłowy zasób {name}")));
        }
        eprintln!("[TEST3][instalacja] {} OK: {} bajtów", name, bytes.len());
    }
    eprintln!("[TEST19.1][instalacja] zatwierdzono {} ({})", folder.display(), crate::paths::installation_format(folder));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("retzar-original-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// A name is the same in any folder and in any case.
    #[test]
    fn a_name_is_known_whatever_its_folder_and_case() {
        assert_eq!(
            digest("IMAGES\\UNIT\\Some File.RLE"),
            digest("some file.rle")
        );
        assert_eq!(digest("a/b/SOME FILE.rle"), digest("Some File.rle"));
        assert_ne!(digest("some file.rle"), digest("some file.wav"));
    }

    /// A folder without the game holds no copy, archives or not; the files
    /// known outnumber those asked for.
    #[test]
    fn a_folder_without_the_game_holds_no_copy() {
        assert!(NEEDED <= KNOWN.len());
        let dir = scratch("vuota");
        for k in 0..KNOWN.len() {
            let folder = dir.join("a").join(k.to_string());
            std::fs::create_dir_all(&folder).unwrap();
            std::fs::write(folder.join("x.wdt"), vec![0u8; 64]).unwrap();
            std::fs::write(dir.join(format!("{k}.bin")), vec![7u8; 64]).unwrap();
        }
        assert!(!holds(&dir));
        assert!(!holds(&dir.join("assente")));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// Small synthetic archives, not copyrighted original game data.
    fn fixture_wdt(path: &Path, entries: &[(&str, &[u8])]) {
        let mut bytes = vec![0u8; 0x28];
        bytes[..15].copy_from_slice(b"HMMSYS PackFile");
        bytes[0x20..0x24].copy_from_slice(&(entries.len() as u32).to_le_bytes());
        let mut offset = 0x28 + entries.iter().map(|(n, _)| 10 + n.len()).sum::<usize>();
        for (name, data) in entries {
            bytes.extend_from_slice(&[name.len() as u8, 0]);
            bytes.extend_from_slice(name.as_bytes());
            bytes.extend_from_slice(&(offset as u32).to_le_bytes());
            bytes.extend_from_slice(&(data.len() as u32).to_le_bytes());
            offset += data.len();
        }
        for (_, data) in entries { bytes.extend_from_slice(data); }
        std::fs::write(path, bytes).unwrap();
    }

    #[test]
    fn direct_loader_accepts_loose_packed_and_mixed_installations() {
        let base = scratch("gog-compat");
        let data: &[(&str, &[u8])] = &[
            ("DATA/IMAGES.INI", b"[images]"),
            ("DATA/ICONS.INI", b"[icons]"),
            ("DATA/SOUNDS.INI", b"[sounds]"),
        ];
        for mode in ["loose", "packed", "mixed"] {
            let root = base.join(mode);
            std::fs::create_dir_all(root.join("Maps")).unwrap();
            if mode != "packed" {
                std::fs::create_dir_all(root.join("DATA")).unwrap();
                std::fs::create_dir_all(root.join("IMAGES/BUILDING")).unwrap();
                for (name, bytes) in data { std::fs::write(root.join(name), bytes).unwrap(); }
                std::fs::write(root.join("IMAGES/BUILDING/EURO HOUSE.BMP"), b"loose art").unwrap();
            }
            if mode != "loose" {
                std::fs::create_dir_all(root.join("Packs")).unwrap();
                fixture_wdt(&root.join("Packs/data.wdt"), data);
                fixture_wdt(&root.join("Packs/images.wdt"), &[("IMAGES/BUILDING/EURO HOUSE.BMP", b"packed art")]);
            }
            verify_direct(&root).unwrap();
            let assets = crate::assets::GameAssets::open(&root).unwrap();
            let expected: &[u8] = if mode == "loose" { b"loose art" } else { b"packed art" };
            assert_eq!(assets.original("images/building/euro house.bmp").unwrap().as_slice(), expected);
            assert_eq!(crate::paths::installation_format(&root), match mode {
                "loose" => "LOOSE (katalogi)", "packed" => "WDT", _ => "MIXED (WDT + katalogi)"
            });
        }
        assert!(verify_direct(&base).is_err());
        std::fs::remove_dir_all(&base).unwrap();
    }

    /// Every edition at hand passes with its archives (the Italian one, the
    /// American CD, the Spanish Anthology with its mods), and so do the
    /// files taken out loose, empty and under any folder, as other editions
    /// and mods keep them; fewer than those asked for do not.
    #[test]
    #[ignore = "Requires the original installation"]
    fn any_edition_passes_with_archives_or_loose_files() {
        let tzar = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf();
        assert!(holds(&tzar));
        for other in ["USA/CD/SETUP/Tzar", "Anthology"] {
            if tzar.join(other).is_dir() {
                assert!(holds(&tzar.join(other)), "{other}");
            }
        }
        let mut names = BTreeSet::new();
        for entry in std::fs::read_dir(tzar.join("Packs")).unwrap().flatten() {
            let Ok(a) = Archive::open(&entry.path()) else {
                continue;
            };
            for e in &a.entries {
                if KNOWN.contains(&digest(&e.name)) {
                    names.insert(e.name.rsplit(['/', '\\']).next().unwrap().to_owned());
                }
            }
        }
        assert_eq!(names.len(), KNOWN.len());
        let dir = scratch("sciolti");
        for (k, name) in names.iter().enumerate() {
            let folder = dir.join("Gioco").join(format!("cartella {}", k % 3));
            std::fs::create_dir_all(&folder).unwrap();
            std::fs::write(folder.join(name.to_ascii_lowercase()), b"").unwrap();
            assert_eq!(holds(&dir), k + 1 >= NEEDED, "{} file", k + 1);
        }
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
