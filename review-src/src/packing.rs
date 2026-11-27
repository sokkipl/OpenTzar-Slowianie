//! The portable copy's pack made from an installation of any edition
//! (`retzar-tools --crea-pacchetto`): its archives (or, for a digital
//! release that keeps its data loose, archives made of those folders under
//! the names the engine reads), maps, campaigns and `Tzar.ini`, the
//! American release's campaign, what FX's Spanish edition adds ([`SPANISH`])
//! and the movies turned into MP4: the Italian and Spanish intros and the
//! American DDV, decoded with the tables of an installed `Tzar.exe` that is
//! not encrypted, which stays out of the pack. The pack is read back whole
//! before it is kept: every file must give back its bytes.
use crate::{
    Result,
    archive::Archive,
    assets::SPANISH_SCREENS,
    ddv,
    hpfs::Container,
    invalid,
    media::{self, Kind, Source},
    pack::{self, Codec, Folder, FolderEntry, Item, Node, Pack, PackWriter},
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

/// The quality of the movies' pictures (of 100): their own blocks, no new
/// ones of the encoder.
const QUALITY: u32 = 90;

/// What went into a pack.
#[derive(Debug, Default)]
pub struct Summary {
    pub files: usize,
    pub resources: usize,
    pub original: u64,
    pub packed: u64,
    /// Bytes saved by deflate, PNG and FLAC.
    pub saved: [u64; 3],
}

/// Records for `items`, the new ones encoded on every core.
fn add_all(w: &mut PackWriter, items: &[(String, Vec<u8>)]) -> Result<Vec<u32>> {
    let mut todo: Vec<usize> = Vec::new();
    let mut fresh = std::collections::HashSet::new();
    for (i, (_, bytes)) in items.iter().enumerate() {
        if w.known(bytes).is_none() && fresh.insert(*blake3::hash(bytes).as_bytes()) {
            todo.push(i);
        }
    }
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let mut encoded: Vec<Option<(Codec, Vec<u8>)>> = vec![None; items.len()];
    std::thread::scope(|scope| {
        let chunks: Vec<_> = todo
            .chunks(todo.len().div_ceil(threads).max(1))
            .map(|part| {
                scope.spawn(move || {
                    part.iter()
                        .map(|&i| (i, pack::encode(&items[i].0, &items[i].1)))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        for c in chunks {
            for (i, e) in c.join().expect("codifica") {
                encoded[i] = Some(e);
            }
        }
    });
    let mut ids = Vec::with_capacity(items.len());
    for ((_, bytes), e) in items.iter().zip(encoded) {
        ids.push(match e {
            Some((codec, payload)) => w.add_encoded(bytes, codec, payload)?,
            None => w.known(bytes).ok_or_else(|| invalid("Record perso"))?,
        });
    }
    Ok(ids)
}

fn folder(w: &mut PackWriter, c: &Container, summary: &mut Summary) -> Result<Folder> {
    // Each entry: a file, the next of `items`, or a container of maps.
    let mut items = Vec::new();
    let mut inner = Vec::with_capacity(c.entries.len());
    for e in &c.entries {
        let nested = if e.name.to_ascii_lowercase().ends_with(".wmp") {
            c.open_nested(&e.name).ok()
        } else {
            None
        };
        if nested.is_none() {
            items.push((e.name.clone(), c.read(e.inode)?));
        }
        inner.push(nested);
    }
    summary.resources += items.len();
    summary.original += items.iter().map(|(_, b)| b.len() as u64).sum::<u64>();
    let mut ids = add_all(w, &items)?.into_iter();
    let mut entries = Vec::with_capacity(c.entries.len());
    for (e, nested) in c.entries.iter().zip(inner) {
        let item = match nested {
            Some(n) => Item::Nested(folder(w, &n, summary)?),
            None => Item::Record(ids.next().ok_or_else(|| invalid("Ordine perso"))?),
        };
        entries.push(FolderEntry {
            name: e.name.clone(),
            inode: e.inode,
            size: e.size,
            item,
        });
    }
    Ok(Folder {
        block_size: c.header.block_size,
        declared_blocks: c.header.declared_blocks,
        bytes: c.header.bytes,
        entries,
    })
}

/// The files of `dir` with `extension`, sorted; none when it is missing.
fn files_in(dir: &Path, extension: &str) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.is_file()
                && p.extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case(extension))
        })
        .collect();
    files.sort();
    files
}
fn name_of(path: &Path) -> String {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

/// The folders a digital release keeps loose, under the names of the
/// archives that hold them in the other editions.
const LOOSE: [(&str, &[&str]); 5] = [
    ("data.wdt", &["DATA"]),
    ("help.wdt", &["HELP"]),
    ("images.wdt", &["IMAGES"]),
    ("local.wdt", &["LOCAL"]),
    ("sounds.wdt", &["SOUND", "MUSIC"]),
];

/// The archives of an installation, by their name in the pack: those of its
/// `Packs` but the movies' or, when it keeps its data loose, archives made
/// of those folders.
fn archives(source: &Path) -> Result<Vec<(String, Archive)>> {
    let mut out = Vec::new();
    for path in files_in(&source.join("Packs"), "wdt") {
        let name = name_of(&path);
        if !name.eq_ignore_ascii_case("movies.wdt") {
            let archive = Archive::open(&path).map_err(|e| format!("{name}: {e}"))?;
            out.push((name, archive));
        }
    }
    if !out
        .iter()
        .any(|(n, _)| n.eq_ignore_ascii_case("images.wdt"))
    {
        for (name, tops) in LOOSE {
            if out.iter().any(|(n, _)| n.eq_ignore_ascii_case(name)) {
                continue;
            }
            let archive = Archive::folder(source, tops)?;
            if !archive.entries.is_empty() {
                out.push((name.to_owned(), archive));
            }
        }
    }
    if !out
        .iter()
        .any(|(n, _)| n.eq_ignore_ascii_case("images.wdt"))
    {
        return Err(invalid(format!(
            "{}: Tzar non trovato (mancano Packs, o DATA e IMAGES)",
            source.display()
        )));
    }
    Ok(out)
}

/// Where the pack keeps what FX's Spanish edition adds: its campaigns that
/// the pack's own are, voiced in Spanish, its localized screens and its
/// launcher's intro.
pub const SPANISH: &str = "Spanish";

fn add_archive(
    w: &mut PackWriter,
    name: &str,
    archive: &Archive,
    summary: &mut Summary,
) -> Result<()> {
    let items: Vec<(String, Vec<u8>)> = archive
        .entries
        .iter()
        .map(|e| Ok((e.name.clone(), archive.read(e)?)))
        .collect::<Result<_>>()?;
    summary.resources += items.len();
    summary.original += items.iter().map(|(_, b)| b.len() as u64).sum::<u64>();
    let ids = add_all(w, &items)?;
    let entries = items.into_iter().map(|(n, _)| n).zip(ids).collect();
    w.file(name, Node::Archive(entries));
    summary.files += 1;
    Ok(())
}

/// The campaigns of FX's Spanish edition `es` that are the pack's own
/// campaigns (the same maps) with recorded voices, by their name in the
/// pack.
fn spanish_campaigns(es: &Path, own: &[(String, PathBuf)]) -> Result<Vec<(String, PathBuf)>> {
    let wmp = |name: &str| name.to_ascii_lowercase().ends_with(".wmp");
    let mut maps = std::collections::HashSet::new();
    for (_, path) in own.iter().filter(|(n, _)| n.starts_with("Campaign/")) {
        let c = Container::open(path)?;
        maps.extend(
            c.entries
                .iter()
                .filter(|e| wmp(&e.name))
                .map(|e| e.name.to_ascii_lowercase()),
        );
    }
    let mut out = Vec::new();
    for path in files_in(&es.join("Campaign"), "wcm") {
        let c = Container::open(&path)?;
        let ours = c
            .entries
            .iter()
            .filter(|e| wmp(&e.name) && maps.contains(&e.name.to_ascii_lowercase()))
            .collect::<Vec<_>>();
        let voiced = ours.iter().any(|e| {
            c.open_nested(&e.name).is_ok_and(|m| {
                m.entries
                    .iter()
                    .any(|x| x.name.to_ascii_lowercase().ends_with(".wav"))
            })
        });
        if voiced {
            out.push((format!("{SPANISH}/Campaign/{}", name_of(&path)), path));
        }
    }
    Ok(out)
}

/// Writes the pack at `out` from the installation `source`, of any edition,
/// and, when given, the American CD's `Tzar` folder `american` (its
/// campaign and its movies) and FX's Spanish edition `spanish` (its own
/// campaigns, [`crate::scenario::SPANISH_CAMPAIGNS`], and its voices, screens
/// and intro, [`SPANISH`]); `work` holds the movies while they are converted.
pub fn build(
    source: &Path,
    american: Option<&Path>,
    spanish: Option<&Path>,
    out: &Path,
    work: &Path,
    log: &mut dyn FnMut(String),
) -> Result<Summary> {
    let _ = (source, american, spanish, out, work, log);
    Err(invalid("OpenTzar: tworzenie ReTzar.dat jest wyłączone"))
}

fn same_folder(pack: &Pack, f: &Folder, c: &Container, name: &str) -> Result<()> {
    let differs = || invalid(format!("{name}: diverso dall'originale"));
    if f.entries.len() != c.entries.len()
        || (f.block_size, f.declared_blocks, f.bytes)
            != (
                c.header.block_size,
                c.header.declared_blocks,
                c.header.bytes,
            )
    {
        return Err(differs());
    }
    for (p, o) in f.entries.iter().zip(&c.entries) {
        if (p.name.as_str(), p.inode, p.size) != (o.name.as_str(), o.inode, o.size) {
            return Err(differs());
        }
        match &p.item {
            Item::Record(id) => {
                if pack.record(*id)? != c.read(o.inode)? {
                    return Err(invalid(format!("{name}/{}: byte diversi", o.name)));
                }
            }
            Item::Nested(n) => same_folder(pack, n, &c.open_nested(&o.name)?, &o.name)?,
        }
    }
    Ok(())
}

/// Every file of the pack against the original, and every movie decoded.
pub fn verify(
    path: &Path,
    source: &Path,
    american: Option<&Path>,
    spanish: Option<&Path>,
) -> Result<()> {
    let pack = Pack::open(path)?;
    let original = |name: &str| -> PathBuf {
        if let (Some(es), Some(rest)) = (spanish, name.strip_prefix(&format!("{SPANISH}/"))) {
            return es.join(rest);
        }
        let at = source.join(name);
        if at.exists() {
            return at;
        }
        // The American campaign, then the Spanish edition's own.
        [american, spanish]
            .into_iter()
            .flatten()
            .map(|dir| dir.join(name))
            .find(|p| p.exists())
            .unwrap_or(at)
    };
    let archives = archives(source)?;
    let screens = spanish
        .map(|es| Archive::open(&es.join("Packs/images2.wdt")))
        .transpose()
        .ok()
        .flatten();
    for (name, node) in pack.files() {
        match node {
            Node::Archive(entries) => {
                let short = name.rsplit('/').next().unwrap_or(name);
                let a = if name.eq_ignore_ascii_case(SPANISH_SCREENS) {
                    screens.as_ref()
                } else {
                    archives
                        .iter()
                        .find(|(n, _)| n.eq_ignore_ascii_case(short))
                        .map(|(_, a)| a)
                }
                .ok_or_else(|| invalid(format!("{name}: archivio sconosciuto")))?;
                if entries.len() != a.entries.len() {
                    return Err(invalid(format!("{name}: voci diverse")));
                }
                for ((n, id), e) in entries.iter().zip(&a.entries) {
                    if *n != e.name || pack.record(*id)? != a.read(e)? {
                        return Err(invalid(format!("{name}/{n}: byte diversi")));
                    }
                }
            }
            Node::Container(f) => same_folder(&pack, f, &Container::open(&original(name))?, name)?,
            Node::File(id) if name.to_ascii_lowercase().ends_with(".mp4") => {
                let mut m = media::Media::open(&pack.record(*id)?, Kind::Mp4)?;
                if m.width == 0 || m.next_frame()?.is_none() || m.audio.is_none() {
                    return Err(invalid(format!("{name}: filmato illeggibile")));
                }
            }
            Node::File(id) => {
                if pack.record(*id)? != std::fs::read(original(name))? {
                    return Err(invalid(format!("{name}: byte diversi")));
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Of FX's Spanish edition, only the campaigns that the pack's own are
    /// and that speak go in: Excalibur and Genghis Khan, not the American
    /// campaign nor the old tutorial, which have no voices.
    #[test]
    #[ignore = "Requires the original installation and the Spanish edition"]
    fn the_spanish_campaigns_in_the_pack_are_the_voiced_ones() {
        let tzar = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let spanish = tzar.join("Anthology");
        if !spanish.is_dir() {
            return;
        }
        let mut own: Vec<(String, PathBuf)> = files_in(&tzar.join("Campaign"), "wcm")
            .into_iter()
            .map(|p| (format!("Campaign/{}", name_of(&p)), p))
            .collect();
        own.push((
            "Campaign/Tzar.wcm".into(),
            tzar.join("USA/CD/SETUP/Tzar/Campaign/Tzar.wcm"),
        ));
        let mut names: Vec<String> = spanish_campaigns(&spanish, &own)
            .unwrap()
            .into_iter()
            .map(|(n, _)| n)
            .collect();
        names.sort();
        assert_eq!(
            names,
            [
                "Spanish/Campaign/El Imperio de Gengis Khan.wcm",
                "Spanish/Campaign/Excalibur y el rey Arturo.wcm",
            ]
        );
    }
}
