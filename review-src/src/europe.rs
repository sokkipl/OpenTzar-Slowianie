//! Optional compatibility aliases for the old Polish GOG layout.
//! All other editions use the selected installation's native images and INI.
//! Never require an additional archive, or read outside the selected installation.
use crate::{Result, archive, archive::Archive};
use std::{collections::HashMap, fs, path::Path, sync::OnceLock};

const GOG_SIZE: usize = 62145886;
const GOG_CRC: u32 = 0xb5998b64;
static PICTURES: OnceLock<HashMap<String, Vec<u8>>> = OnceLock::new();

pub fn ready() -> bool { PICTURES.get().is_some() }

pub fn picture(path: &str) -> Option<&'static [u8]> {
    let key = archive::key(path);
    let relative = key.strip_prefix("IMAGES/EU_BASE/")?;
    PICTURES.get()?.get(relative).map(Vec::as_slice)
}

/// The old hardcoded geometry only applies to this exact original archive.
/// A different size/checksum is a normal edition difference, not an error.
fn legacy_pictures(source: &Path) -> Result<Option<HashMap<String, Vec<u8>>>> {
    let path = source.join("Packs/images.wdt");
    if !fs::metadata(&path).is_ok_and(|m| m.is_file() && m.len() == GOG_SIZE as u64) {
        return Ok(None);
    }
    let data = fs::read(&path)?;
    if data.len() != GOG_SIZE || crc32fast::hash(&data) != GOG_CRC {
        return Ok(None);
    }
    let archive = Archive::from_bytes(&data)?;
    let mut pictures = HashMap::new();
    for entry in &archive.entries {
        let key = archive::key(&entry.name);
        let Some(short) = key.strip_prefix("IMAGES/") else { continue };
        if short.starts_with("BUILDING/EURO ") || short.starts_with("BUILDING/MASK/EURO ")
            || short.starts_with("UNIT/EURO PEASANT") || short == "ICONS/ICONS.BMP" {
            pictures.insert(short.to_owned(), archive.read(entry)?);
        }
    }
    Ok((!pictures.is_empty()).then_some(pictures))
}

pub fn prepare(source: &Path, root: &Path) -> Result<()> {
    if PICTURES.get().is_some() { return Ok(()); }
    // The main loader already validated the selected physical installation.
    // Native geometry is the safe fallback for other versions (also loose GOG).
    let mode = match legacy_pictures(source) {
        Ok(Some(pictures)) => {
            let count = pictures.len();
            let _ = PICTURES.set(pictures);
            format!("PL WDT: zachowano zgodny uklad Europejczykow ({count} zasobow)")
        }
        Ok(None) => "NATIVE: grafiki i geometria z wybranej instalacji".to_owned(),
        Err(error) => format!("NATIVE: pominieto opcjonalne aliasy PL ({error})"),
    };
    let language = crate::lang::installation().map_or("unknown", |l| l.code());
    let report = format!("OpenTzar TEST19.1\nZrodlo: {}\nFormat: {}\nJezyk zasobow: {language}\nEuropa: {mode}\nBez dodatkowego images.wdt / Packs.zip.\n", source.display(), crate::paths::installation_format(source));
    eprintln!("{report}");
    if let Err(error) = crate::write_atomic(&root.join("GOG_COMPAT.txt"), report.as_bytes()) {
        eprintln!("Nie zapisano GOG_COMPAT.txt: {error}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loose_and_other_wdt_editions_do_not_require_polish_archive() {
        let root = std::env::temp_dir().join(format!("ot-gog-europe-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        for dir in ["DATA", "IMAGES", "Maps"] { fs::create_dir_all(root.join(dir)).unwrap(); }
        assert!(legacy_pictures(&root).unwrap().is_none());
        fs::create_dir_all(root.join("Packs")).unwrap();
        fs::write(root.join("Packs/images.wdt"), b"different edition").unwrap();
        assert!(legacy_pictures(&root).unwrap().is_none());
        fs::remove_dir_all(root).unwrap();
    }
}
