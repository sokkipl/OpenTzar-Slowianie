pub fn label(key:&str)->Option<&'static str>{Some(match key {
    "tcguild"|"sl_tcguild"=>"Gildia kupców",
    "tribal"|"sl_tribal"=>"Woj plemienny",
    "slinger"|"sl_slinger"=>"Procarz",
    "shieldman"|"sl_shieldman"=>"Tarczownik",
    "rider"|"sl_rider"=>"Konny",
    "lancer"|"sl_lancer"=>"Włócznik konny",
    "retainer"|"sl_retainer"=>"Drużynnik grodowy",
    "heavyhorse"|"sl_heavyhorse"=>"Ciężka konnica",
    "javelin"|"sl_javelin"=>"Oszczepnik",
    "watcher"|"sl_watcher"=>"Zwiadowca",
    "scouting"|"sl_scouting"|"te_sl_scouting"=>"Zwiadowca",
    "eye"|"sl_eye"|"te_sl_eye"=>"Oko Świętowita",
    "tips"|"sl_tips"|"te_sl_tips"=>"Hartowane groty",
    "throwing"|"sl_throwing"|"te_sl_throwing"=>"Szkolenie oszczepników",
    "lances"|"sl_lances"|"te_sl_lances"=>"Włócznicy konni",
    "retinue"|"sl_retinue"|"te_sl_retinue"=>"Drużyna grodowa",
    "heavy"|"sl_heavy"|"te_sl_heavy"=>"Ciężka konnica",
    "cathedral"=>"Kącina Świętowita",
    "church"|"e_church"=>"Kapliczka",
    "priest"|"sl_priest"=>"Kapłan",
    "te_sl_reaction"|"sl_reaction"=>"Reakcja pogańska",
    "magicworkshop"=>"Święty las",
    "e_magetower"=>"Chram Welesa",
    "magetower"=>"Chram Welesa",
    "grove"=>"Święty las",
    "mage"=>"Wołchw",
    "leszy"=>"Leszy",
    "wolf"=>"Wilk",
    "boar"=>"Dzik",
    "raven"=>"Kruk",
    "bear"=>"Niedźwiedź",
    "perun"=>"Piorun Peruna",
    "mist"=>"Mgła Nawii",
    "roots"=>"Korzenie",
    "sl_magicworkshop"=>"Święty las",
    "sl_e_magetower"=>"Chram Welesa",
    "sl_magetower"=>"Chram Welesa",
    "sl_grove"=>"Święty las",
    "sl_mage"=>"Wołchw",
    "sl_leszy"=>"Przywołaj Leszego",
    "sl_wolf"=>"Wilk",
    "sl_boar"=>"Dzik",
    "sl_raven"=>"Kruk",
    "sl_bear"=>"Niedźwiedź",
    "sl_perun"=>"Piorun Peruna",
    "sl_mist"=>"Mgła Nawii",
    "sl_roots"=>"Korzenie",
    "te_sl_leszy"=>"Badanie: Leszy",
    "te_sl_grove"=>"Badanie: Święty las",
    "te_sl_perun"=>"Badanie: Piorun Peruna",
    "te_sl_mist"=>"Badanie: Mgła Nawii",
_=>return None,
})}

pub fn cloth_kind(path: &str) -> Option<bool> {
    let path = path.to_ascii_lowercase();
    if path.ends_with("_s.bmp") { return None; }
    if path.starts_with("images/chram/mage_") { Some(false) }
    else if path.starts_with("images/chram/priest_") { Some(true) } else { None }
}

pub fn tint_cloth(sheet: &mut crate::sprite::SpriteSheet, priest: bool, slot: usize, palette: &[u8]) -> crate::Result<()> {
    let start = 14 + crate::u32_at(palette, 14)? as usize + 4*(112+16*slot.min(7));
    let ramp = palette.get(start..start+64).ok_or("Missing kingdom palette")?;
    let width = sheet.image.width as usize;
    for frame in sheet.image.rgba.chunks_exact_mut(4*width*sheet.frame_height) {
        let mut min = [width,sheet.frame_height];
        let mut max = [0,0];
        for (i,_) in frame.chunks_exact(4).enumerate().filter(|(_,p)| p[3]!=0) {
            let (x,y) = (i%width,i/width);
            min = [min[0].min(x),min[1].min(y)]; max = [max[0].max(x),max[1].max(y)];
        }
        if min[0]>max[0] { continue; }
        for (i,p) in frame.chunks_exact_mut(4).enumerate().filter(|(_,p)| p[3]!=0) {
            let (x,y)=(i%width,i/width);
            let height=max[1]-min[1]+1;
            if (y-min[1])*100 < height*24 || (y-min[1])*100 > height*82 ||
                (2*x).abs_diff(min[0]+max[0])*100 > 76*(max[0]-min[0]+1) {continue;}
            let [r,g,b]=[p[0] as i32,p[1] as i32,p[2] as i32];
            let fabric=if priest { (r-g<25 && r>65) || r*10>g*19 } else { r-g<27 && g-b>=5 };
            if fabric {
                let shade=((r+g+b)/32).clamp(3,15) as usize*4;
                p[..3].copy_from_slice(&[ramp[shade+2],ramp[shade+1],ramp[shade]]);
            }
        }
    }
    Ok(())
}
