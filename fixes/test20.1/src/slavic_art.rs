//! Registration of the project's replacement building sheets in map pixels.
use crate::bitmap::Bitmap;

/// Try the INI placement and small translations against the grey cloth
/// painted into the building. Keep the supplied placement on ambiguous art.
pub fn flag_offset(body: &Bitmap, mask: &Bitmap, origin: [i32; 2]) -> [i32; 2] {
    let samples: Vec<_> = mask.rgba.chunks_exact(4).enumerate()
        .filter(|(_,p)| p[3] != 0).step_by(8)
        .map(|(i,_)| ((i % mask.width as usize) as i32, (i / mask.width as usize) as i32)).collect();
    if samples.len() < 12 { return origin; }
    let score = |[dx,dy]: [i32;2]| samples.iter().filter(|&&(x,y)| {
        let (x,y)=(x+dx,y+dy);
        if x<0 || y<0 || x>=body.width as i32 || y>=body.height as i32 { return false; }
        let at=4*(y as usize*body.width as usize+x as usize);
        let p=&body.rgba[at..at+4];
        let hi=p[0].max(p[1]).max(p[2]);
        let lo=p[0].min(p[1]).min(p[2]);
        p[3]!=0 && hi>38 && hi-lo<=18
    }).count();
    let initial=score(origin);
    if initial*100 >= 94*samples.len() { return origin; }
    let mut best=(initial,origin);
    for y in -12..=12 { for x in -36..=36 {
        let offset=[origin[0]+x,origin[1]+y];
        let n=score(offset);
        if n>best.0 { best=(n,offset); }
    }}
    if best.0>=initial+samples.len()/10 {best.1} else {origin}
}

pub fn paint_mask(body: &mut Bitmap, mask: &Bitmap, [dx,dy]:[i32;2]) {
    for (i,p) in mask.rgba.chunks_exact(4).enumerate().filter(|(_,p)| p[3]!=0) {
        let x=dx+(i%mask.width as usize) as i32;
        let y=dy+(i/mask.width as usize) as i32;
        if x>=0 && y>=0 && x<body.width as i32 && y<body.height as i32 {
            let at=(y as usize*body.width as usize+x as usize)*4;
            body.rgba[at..at+4].copy_from_slice(p);
        }
    }
}

pub fn house_grid(w:usize,h:usize,cols:usize,rows:usize)->(usize,usize) {
    if h>=2*w && h%3==0 {(1,3)} else if w>=2*h && w%3==0 {(3,1)} else {(cols,rows)}
}

/// Approved building art, paired with a grayscale owner-colour mask.
pub struct BuildingArt {
    pub class: &'static str,
    pub sprite: &'static str,
    mask: &'static str,
}
macro_rules! building {
    ($name:literal) => { BuildingArt {
        class: $name,
        sprite: concat!("art/slavs/", $name, "_retro.rle"),
        mask: concat!("art/slavs/", $name, "_retro_mask.png"),
    } }
}
pub static BUILDINGS: &[BuildingArt] = &[
    building!("academy"),
    building!("barrack"),
    building!("blacksmith"),
    building!("cathedral"),
    building!("church"),
    building!("farm"),
    building!("harbor"),
    building!("house"),
    building!("lumber"),
    building!("magetower"),
    building!("market"),
    building!("outpost"),
    building!("stable"),
    building!("storage"),
    building!("tavern"),
    building!("tcguild"),
    building!("workshop"),
];

impl BuildingArt {
    /// Finished sprite width, height, left edge and baseline in map pixels,
    /// measured from the north-west cell. The mill has its own 4x4 foundation.
    pub fn layout(&self) -> [i32; 4] {
        match self.class {
            "academy" => [183,185,-10,173], "barrack" => [160,159,0,160],
            "blacksmith" => [138,134,-5,131], "cathedral" => [160,159,0,157],
            "church" => [96,116,3,98], "farm" => [138,143,-5,135],
            "harbor" => [128,123,-14,107], "house" => [96,90,1,93],
            "lumber" => [138,135,-5,131], "magetower" => [159,180,-16,161],
            "market" => [139,152,-6,136], "outpost" => [132,156,-2,130],
            "stable" => [149,139,-10,129], "storage" => [132,128,-2,120],
            "tavern" => [128,127,1,126], "tcguild" => [182,185,-13,168],
            "workshop" => [141,137,-6,133],
            _ => unreachable!("unregistered Slavic building"),
        }
    }
    pub fn for_class(name: &str) -> Option<&'static Self> {
        BUILDINGS.iter().find(|art| art.class == name)
    }
    pub fn picture(&self, assets: &crate::assets::GameAssets, slot: Option<usize>) -> crate::Result<Bitmap> {
        let mut body = crate::sprite::SpriteSheet::decode(crate::runtime_art::bytes(self.sprite))?.frame(0, 0)?;
        // The guild's flag is painted into its sprite, not a separate overlay.
        // Remove the flag cloth and exposed pole, preserving the masonry/roof.
        if self.class == "academy" {
            let mask = Bitmap::decode_png(crate::runtime_art::bytes(self.mask))?;
            if mask.width != body.width || mask.height != body.height {
                return Err(crate::invalid("Slavic guild mask differs in size"));
            }
            let width = body.width as usize;
            for (i,(p,m)) in body.rgba.chunks_exact_mut(4).zip(mask.rgba.chunks_exact(4)).enumerate() {
                let (x,y) = (i % width, i / width);
                if m[0] != 0 || ((113..=119).contains(&x) && y < 43) { p[3] = 0; }
            }
            return Ok(body);
        }
        // The Chram's small owner-colour marker is not a banner; leave it
        // in the original art colours rather than tinting it as a flag.
        if self.class == "magetower" { return Ok(body); }
        if let Some(slot) = slot {
            let mask = Bitmap::decode_png(crate::runtime_art::bytes(self.mask))?;
            if (mask.width, mask.height) != (body.width, body.height) {
                return Err(crate::invalid("Slavic building and owner mask differ in size"));
            }
            let palette = assets.get("IMAGES/PALETTE.BMP")?;
            let start = 14 + crate::u32_at(&palette, 14)? as usize + 4 * (112 + 16 * slot.min(7));
            let ramp = palette.get(start..start + 64).ok_or("Invalid kingdom palette")?;
            for (p, m) in body.rgba.chunks_exact_mut(4).zip(mask.rgba.chunks_exact(4)) {
                if m[0] == 0 || p[3] == 0 { continue; }
                let light = (p[0] as usize + 2*p[1] as usize + p[2] as usize)/4;
                let offset = 4 * (3 + light / 10).min(15);
                let weight = m[0] as u32;
                for c in 0..3 {
                    p[c] = ((p[c] as u32 * (255-weight) + ramp[offset+2-c] as u32 * weight)/255) as u8;
                }
            }
        }
        Ok(body)
    }
}

/// Only completed upgraded Slavic fortifications (and their native wall/gate
/// construction stages) use the new graphics. All other kingdoms and the
/// basic wooden fortifications continue to load the installation's resources.
pub fn fortification(civ: u8, class: usize, section: &str, construction: bool) -> Option<&'static str> {
    use crate::world::{SLAVS, class as c};
    if civ != SLAVS { return None; }
    match class {
        c::TOWER_STONE if !construction => Some("stolp"),
        c::WALL_STONE => match section {
            "walls_stone_tower" => Some("wall_tower"),
            "walls_stone_horz" => Some("wall_horz"),
            "walls_stone_vert" => Some("wall_vert"),
            "walls_stone_tower_dest" => Some("wall_tower_dest"),
            "walls_stone_horz_dest" => Some("wall_horz_dest"),
            "walls_stone_vert_dest" => Some("wall_vert_dest"),
            "walls_stone_build" => Some("wall_build"),
            _ => None,
        },
        c::GATE_STONE => match section {
            "gs0_anim" => Some("gate0_anim"),
            "gs0_build" => Some("gate0_build"),
            "gs1_anim" => Some("gate1_anim"),
            "gs1_build" => Some("gate1_build"),
            "gs2_anim" => Some("gate2_anim"),
            "gs2_build" => Some("gate2_build"),
            "gs3_anim" => Some("gate3_anim"),
            "gs3_build" => Some("gate3_build"),
            _ => None,
        },
        _ => None,
    }
}

pub fn fortification_picture(name: &str, assets: &crate::assets::GameAssets, slot: Option<usize>) -> crate::Result<crate::sprite::SpriteSheet> {
    let raw: &[u8] = match name {
        "gate0_anim" => crate::runtime_art::bytes("art/slavs/fort/gate0_anim.rle"),
        "gate0_build" => crate::runtime_art::bytes("art/slavs/fort/gate0_build.rle"),
        "gate1_anim" => crate::runtime_art::bytes("art/slavs/fort/gate1_anim.rle"),
        "gate1_build" => crate::runtime_art::bytes("art/slavs/fort/gate1_build.rle"),
        "gate2_anim" => crate::runtime_art::bytes("art/slavs/fort/gate2_anim.rle"),
        "gate2_build" => crate::runtime_art::bytes("art/slavs/fort/gate2_build.rle"),
        "gate3_anim" => crate::runtime_art::bytes("art/slavs/fort/gate3_anim.rle"),
        "gate3_build" => crate::runtime_art::bytes("art/slavs/fort/gate3_build.rle"),
        "stolp" => crate::runtime_art::bytes("art/slavs/fort/stolp.rle"),
        "wall_build" => crate::runtime_art::bytes("art/slavs/fort/wall_build.rle"),
        "wall_horz" => crate::runtime_art::bytes("art/slavs/fort/wall_horz.rle"),
        "wall_horz_dest" => crate::runtime_art::bytes("art/slavs/fort/wall_horz_dest.rle"),
        "wall_tower" => crate::runtime_art::bytes("art/slavs/fort/wall_tower.rle"),
        "wall_tower_dest" => crate::runtime_art::bytes("art/slavs/fort/wall_tower_dest.rle"),
        "wall_vert" => crate::runtime_art::bytes("art/slavs/fort/wall_vert.rle"),
        "wall_vert_dest" => crate::runtime_art::bytes("art/slavs/fort/wall_vert_dest.rle"),
        _ => return Err(crate::invalid("Unknown Slavic fortification sheet")),
    };
    let mut sheet = crate::sprite::SpriteSheet::decode(raw)?;
    // RGB555 art uses a blue fabric marker; only that fabric takes the
    // original player's 16-shade ramp, not iron, wood or red embroidery.
    if let Some(slot) = slot {
        let palette = assets.get("IMAGES/PALETTE.BMP")?;
        let start = 14 + crate::u32_at(&palette, 14)? as usize + 4 * (112 + 16 * slot.min(7));
        let ramp = palette.get(start..start + 64).ok_or("Invalid kingdom palette")?;
        for p in sheet.image.rgba.chunks_exact_mut(4) {
            if p[3] == 0 || p[2] as u16 <= p[0] as u16 + 16 || p[2] as u16 <= p[1] as u16 + 8 { continue; }
            let light = (p[0] as usize + 2 * p[1] as usize + p[2] as usize) / 4;
            let at = 4 * (3 + light / 10).min(15);
            p[0] = ramp[at + 2]; p[1] = ramp[at + 1]; p[2] = ramp[at];
        }
    }
    Ok(sheet)
}

#[cfg(test)]
mod fortification_tests {
    use super::*;
    use crate::world::{SLAVS, class as c};
    #[test]
    fn replacements_are_slavs_only_and_only_upgraded_classes() {
        for civ in 0..=3 {
            assert_eq!(fortification(civ, c::TOWER_STONE, "e_tower_stone", false), None);
            assert_eq!(fortification(civ, c::WALL_STONE, "walls_stone_horz", false), None);
            assert_eq!(fortification(civ, c::GATE_STONE, "gs0_anim", false), None);
        }
        assert_eq!(fortification(SLAVS, c::TOWER_STONE, "e_tower_stone", false), Some("stolp"));
        assert_eq!(fortification(SLAVS, c::TOWER_STONE, "build3x3", true), None);
        assert_eq!(fortification(SLAVS, c::WALL_WOOD, "walls_wood_horz", false), None);
        assert_eq!(fortification(SLAVS, c::GATE_WOOD, "gw0_anim", false), None);
        assert_eq!(fortification(SLAVS, c::WALL_STONE, "walls_stone_vert", false), Some("wall_vert"));
        assert_eq!(fortification(SLAVS, c::GATE_STONE, "gs3_anim", false), Some("gate3_anim"));
    }
}

/** Add a narrow row of sharpened wooden stakes to the lower visible edge of
    a native wooden-wall frame. Only the Slavic upgraded wall uses this art. */
pub fn palisade(frame: &Bitmap, damaged: bool) -> Bitmap {
    let mut out = frame.nearest((frame.width * 108 + 50) / 100, (frame.height * 108 + 50) / 100);
    if damaged || out.width < 8 || out.height < 8 { return out; }
    let (w,h) = (out.width as usize,out.height as usize);
    let edges: Vec<_> = (0..w).map(|x| (0..h).rev().find(|&y| {
        let p=&out.rgba[(y*w+x)*4..(y*w+x)*4+4];
        p[3]>200 && p[0]>p[2] && p[1]>=p[2] && p[0]>35
    })).collect();
    out.rgba.resize(w*(h+5)*4,0);
    out.height += 5;
    for x in (3..w.saturating_sub(3)).step_by(5) {
        let Some(y)=edges[x] else {continue};
        if edges[x-1].is_none() || edges[x+1].is_none() {continue;}
        let base=(y*w+x)*4;
        let c=[out.rgba[base],out.rgba[base+1],out.rgba[base+2]];
        for k in 0..7usize {
            let yy=y.saturating_sub(2)+k;
            if yy>=h+5 {break;}
            let radius=if k<4 {1} else {0};
            for xx in x-radius..=x+radius {
                let i=(yy*w+xx)*4;
                let shade=if xx<x {110} else if xx>x {68} else {91};
                for j in 0..3 {out.rgba[i+j]=(c[j] as u16*shade/100).min(255) as u8;}
                out.rgba[i+3]=255;
            }
        }
    }
    out
}
