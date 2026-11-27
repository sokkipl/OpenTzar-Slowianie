//! Version-local OpenTzar artwork. The GOG archive loader is unchanged.
//! Each registered file has an embedded fallback for standalone builds.
use std::{collections::HashMap, fs, path::{Path, PathBuf}, sync::{Mutex, OnceLock}};
use crate::{Result, invalid};

// A closed list, not arbitrary paths. Every entry is an existing mod resource.
static ART: &[(&str, &[u8])] = &[
    ("art/army/eye.png", include_bytes!("../art/army/eye.png")),
    ("art/army/heavy.png", include_bytes!("../art/army/heavy.png")),
    ("art/army/heavyhorse.png", include_bytes!("../art/army/heavyhorse.png")),
    ("art/army/heavyhorse_attack.rle", include_bytes!("../art/army/heavyhorse_attack.rle")),
    ("art/army/heavyhorse_die.rle", include_bytes!("../art/army/heavyhorse_die.rle")),
    ("art/army/heavyhorse_idle.rle", include_bytes!("../art/army/heavyhorse_idle.rle")),
    ("art/army/heavyhorse_walk.rle", include_bytes!("../art/army/heavyhorse_walk.rle")),
    ("art/army/javelin.png", include_bytes!("../art/army/javelin.png")),
    ("art/army/javelin_attack.rle", include_bytes!("../art/army/javelin_attack.rle")),
    ("art/army/javelin_die.rle", include_bytes!("../art/army/javelin_die.rle")),
    ("art/army/javelin_idle.rle", include_bytes!("../art/army/javelin_idle.rle")),
    ("art/army/javelin_walk.rle", include_bytes!("../art/army/javelin_walk.rle")),
    ("art/army/lancer.png", include_bytes!("../art/army/lancer.png")),
    ("art/army/lancer_attack.rle", include_bytes!("../art/army/lancer_attack.rle")),
    ("art/army/lancer_die.rle", include_bytes!("../art/army/lancer_die.rle")),
    ("art/army/lancer_idle.rle", include_bytes!("../art/army/lancer_idle.rle")),
    ("art/army/lancer_walk.rle", include_bytes!("../art/army/lancer_walk.rle")),
    ("art/army/lances.png", include_bytes!("../art/army/lances.png")),
    ("art/army/retainer.png", include_bytes!("../art/army/retainer.png")),
    ("art/army/retainer_attack.rle", include_bytes!("../art/army/retainer_attack.rle")),
    ("art/army/retainer_die.rle", include_bytes!("../art/army/retainer_die.rle")),
    ("art/army/retainer_idle.rle", include_bytes!("../art/army/retainer_idle.rle")),
    ("art/army/retainer_walk.rle", include_bytes!("../art/army/retainer_walk.rle")),
    ("art/army/retinue.png", include_bytes!("../art/army/retinue.png")),
    ("art/army/rider.png", include_bytes!("../art/army/rider.png")),
    ("art/army/rider_attack.rle", include_bytes!("../art/army/rider_attack.rle")),
    ("art/army/rider_die.rle", include_bytes!("../art/army/rider_die.rle")),
    ("art/army/rider_idle.rle", include_bytes!("../art/army/rider_idle.rle")),
    ("art/army/rider_walk.rle", include_bytes!("../art/army/rider_walk.rle")),
    ("art/army/scouting.png", include_bytes!("../art/army/scouting.png")),
    ("art/army/shieldman.png", include_bytes!("../art/army/shieldman.png")),
    ("art/army/shieldman_attack.rle", include_bytes!("../art/army/shieldman_attack.rle")),
    ("art/army/shieldman_die.rle", include_bytes!("../art/army/shieldman_die.rle")),
    ("art/army/shieldman_idle.rle", include_bytes!("../art/army/shieldman_idle.rle")),
    ("art/army/shieldman_walk.rle", include_bytes!("../art/army/shieldman_walk.rle")),
    ("art/army/slinger.png", include_bytes!("../art/army/slinger.png")),
    ("art/army/slinger_attack.rle", include_bytes!("../art/army/slinger_attack.rle")),
    ("art/army/slinger_die.rle", include_bytes!("../art/army/slinger_die.rle")),
    ("art/army/slinger_idle.rle", include_bytes!("../art/army/slinger_idle.rle")),
    ("art/army/slinger_walk.rle", include_bytes!("../art/army/slinger_walk.rle")),
    ("art/army/throwing.png", include_bytes!("../art/army/throwing.png")),
    ("art/army/tips.png", include_bytes!("../art/army/tips.png")),
    ("art/army/tribal.png", include_bytes!("../art/army/tribal.png")),
    ("art/army/tribal_attack.rle", include_bytes!("../art/army/tribal_attack.rle")),
    ("art/army/tribal_attack_s.rle", include_bytes!("../art/army/tribal_attack_s.rle")),
    ("art/army/tribal_die.rle", include_bytes!("../art/army/tribal_die.rle")),
    ("art/army/tribal_die_s.rle", include_bytes!("../art/army/tribal_die_s.rle")),
    ("art/army/tribal_idle.rle", include_bytes!("../art/army/tribal_idle.rle")),
    ("art/army/tribal_idle_s.rle", include_bytes!("../art/army/tribal_idle_s.rle")),
    ("art/army/tribal_walk.rle", include_bytes!("../art/army/tribal_walk.rle")),
    ("art/army/tribal_walk_s.rle", include_bytes!("../art/army/tribal_walk_s.rle")),
    ("art/army/watcher.png", include_bytes!("../art/army/watcher.png")),
    ("art/army/watcher_attack.rle", include_bytes!("../art/army/watcher_attack.rle")),
    ("art/army/watcher_die.rle", include_bytes!("../art/army/watcher_die.rle")),
    ("art/army/watcher_idle.rle", include_bytes!("../art/army/watcher_idle.rle")),
    ("art/army/watcher_walk.rle", include_bytes!("../art/army/watcher_walk.rle")),
    ("art/chram/drumstick.png", include_bytes!("../art/chram/drumstick.png")),
    ("art/chram/grove_retro.rle", include_bytes!("../art/chram/grove_retro.rle")),
    ("art/chram/icon_boar.png", include_bytes!("../art/chram/icon_boar.png")),
    ("art/chram/icon_egg.png", include_bytes!("../art/chram/icon_egg.png")),
    ("art/chram/icon_grove.png", include_bytes!("../art/chram/icon_grove.png")),
    ("art/chram/icon_idol.png", include_bytes!("../art/chram/icon_idol.png")),
    ("art/chram/icon_leszy.png", include_bytes!("../art/chram/icon_leszy.png")),
    ("art/chram/icon_mage.png", include_bytes!("../art/chram/icon_mage.png")),
    ("art/chram/icon_mist.png", include_bytes!("../art/chram/icon_mist.png")),
    ("art/chram/icon_perun.png", include_bytes!("../art/chram/icon_perun.png")),
    ("art/chram/icon_priest.png", include_bytes!("../art/chram/icon_priest.png")),
    ("art/chram/icon_raven.png", include_bytes!("../art/chram/icon_raven.png")),
    ("art/chram/icon_reaction.png", include_bytes!("../art/chram/icon_reaction.png")),
    ("art/chram/icon_roots.png", include_bytes!("../art/chram/icon_roots.png")),
    ("art/chram/icon_wolf.png", include_bytes!("../art/chram/icon_wolf.png")),
    ("art/chram/mist_cycle.png", include_bytes!("../art/chram/mist_cycle.png")),
    ("art/chram/motion/boar_attack.rle", include_bytes!("../art/chram/motion/boar_attack.rle")),
    ("art/chram/motion/boar_attack_s.rle", include_bytes!("../art/chram/motion/boar_attack_s.rle")),
    ("art/chram/motion/boar_die.rle", include_bytes!("../art/chram/motion/boar_die.rle")),
    ("art/chram/motion/boar_die_s.rle", include_bytes!("../art/chram/motion/boar_die_s.rle")),
    ("art/chram/motion/boar_idle.rle", include_bytes!("../art/chram/motion/boar_idle.rle")),
    ("art/chram/motion/boar_idle_s.rle", include_bytes!("../art/chram/motion/boar_idle_s.rle")),
    ("art/chram/motion/boar_walk.rle", include_bytes!("../art/chram/motion/boar_walk.rle")),
    ("art/chram/motion/boar_walk_s.rle", include_bytes!("../art/chram/motion/boar_walk_s.rle")),
    ("art/chram/motion/leszy_attack.rle", include_bytes!("../art/chram/motion/leszy_attack.rle")),
    ("art/chram/motion/leszy_attack_s.rle", include_bytes!("../art/chram/motion/leszy_attack_s.rle")),
    ("art/chram/motion/leszy_die.rle", include_bytes!("../art/chram/motion/leszy_die.rle")),
    ("art/chram/motion/leszy_die_s.rle", include_bytes!("../art/chram/motion/leszy_die_s.rle")),
    ("art/chram/motion/leszy_idle.rle", include_bytes!("../art/chram/motion/leszy_idle.rle")),
    ("art/chram/motion/leszy_idle_s.rle", include_bytes!("../art/chram/motion/leszy_idle_s.rle")),
    ("art/chram/motion/leszy_walk.rle", include_bytes!("../art/chram/motion/leszy_walk.rle")),
    ("art/chram/motion/leszy_walk_s.rle", include_bytes!("../art/chram/motion/leszy_walk_s.rle")),
    ("art/chram/motion/mage_attack.rle", include_bytes!("../art/chram/motion/mage_attack.rle")),
    ("art/chram/motion/mage_attack_s.rle", include_bytes!("../art/chram/motion/mage_attack_s.rle")),
    ("art/chram/motion/mage_die.rle", include_bytes!("../art/chram/motion/mage_die.rle")),
    ("art/chram/motion/mage_die_s.rle", include_bytes!("../art/chram/motion/mage_die_s.rle")),
    ("art/chram/motion/mage_idle.rle", include_bytes!("../art/chram/motion/mage_idle.rle")),
    ("art/chram/motion/mage_idle_s.rle", include_bytes!("../art/chram/motion/mage_idle_s.rle")),
    ("art/chram/motion/mage_walk.rle", include_bytes!("../art/chram/motion/mage_walk.rle")),
    ("art/chram/motion/mage_walk_s.rle", include_bytes!("../art/chram/motion/mage_walk_s.rle")),
    ("art/chram/motion/priest_attack.rle", include_bytes!("../art/chram/motion/priest_attack.rle")),
    ("art/chram/motion/priest_attack_s.rle", include_bytes!("../art/chram/motion/priest_attack_s.rle")),
    ("art/chram/motion/priest_cast.rle", include_bytes!("../art/chram/motion/priest_cast.rle")),
    ("art/chram/motion/priest_cast_s.rle", include_bytes!("../art/chram/motion/priest_cast_s.rle")),
    ("art/chram/motion/priest_die.rle", include_bytes!("../art/chram/motion/priest_die.rle")),
    ("art/chram/motion/priest_die_s.rle", include_bytes!("../art/chram/motion/priest_die_s.rle")),
    ("art/chram/motion/priest_idle.rle", include_bytes!("../art/chram/motion/priest_idle.rle")),
    ("art/chram/motion/priest_idle_s.rle", include_bytes!("../art/chram/motion/priest_idle_s.rle")),
    ("art/chram/motion/priest_walk.rle", include_bytes!("../art/chram/motion/priest_walk.rle")),
    ("art/chram/motion/priest_walk_s.rle", include_bytes!("../art/chram/motion/priest_walk_s.rle")),
    ("art/chram/motion/raven_attack.rle", include_bytes!("../art/chram/motion/raven_attack.rle")),
    ("art/chram/motion/raven_attack_s.rle", include_bytes!("../art/chram/motion/raven_attack_s.rle")),
    ("art/chram/motion/raven_die.rle", include_bytes!("../art/chram/motion/raven_die.rle")),
    ("art/chram/motion/raven_die_s.rle", include_bytes!("../art/chram/motion/raven_die_s.rle")),
    ("art/chram/motion/raven_idle.rle", include_bytes!("../art/chram/motion/raven_idle.rle")),
    ("art/chram/motion/raven_idle_s.rle", include_bytes!("../art/chram/motion/raven_idle_s.rle")),
    ("art/chram/motion/raven_walk.rle", include_bytes!("../art/chram/motion/raven_walk.rle")),
    ("art/chram/motion/raven_walk_s.rle", include_bytes!("../art/chram/motion/raven_walk_s.rle")),
    ("art/chram/motion/wolf_attack.rle", include_bytes!("../art/chram/motion/wolf_attack.rle")),
    ("art/chram/motion/wolf_attack_s.rle", include_bytes!("../art/chram/motion/wolf_attack_s.rle")),
    ("art/chram/motion/wolf_die.rle", include_bytes!("../art/chram/motion/wolf_die.rle")),
    ("art/chram/motion/wolf_die_s.rle", include_bytes!("../art/chram/motion/wolf_die_s.rle")),
    ("art/chram/motion/wolf_idle.rle", include_bytes!("../art/chram/motion/wolf_idle.rle")),
    ("art/chram/motion/wolf_idle_s.rle", include_bytes!("../art/chram/motion/wolf_idle_s.rle")),
    ("art/chram/motion/wolf_walk.rle", include_bytes!("../art/chram/motion/wolf_walk.rle")),
    ("art/chram/motion/wolf_walk_s.rle", include_bytes!("../art/chram/motion/wolf_walk_s.rle")),
    ("art/slavs/academy_retro.rle", include_bytes!("../art/slavs/academy_retro.rle")),
    ("art/slavs/academy_retro_mask.png", include_bytes!("../art/slavs/academy_retro_mask.png")),
    ("art/slavs/barrack_retro.rle", include_bytes!("../art/slavs/barrack_retro.rle")),
    ("art/slavs/barrack_retro_mask.png", include_bytes!("../art/slavs/barrack_retro_mask.png")),
    ("art/slavs/blacksmith_retro.rle", include_bytes!("../art/slavs/blacksmith_retro.rle")),
    ("art/slavs/blacksmith_retro_mask.png", include_bytes!("../art/slavs/blacksmith_retro_mask.png")),
    ("art/slavs/castle_flags.bmp", include_bytes!("../art/slavs/castle_flags.bmp")),
    ("art/slavs/castle_retro.rle", include_bytes!("../art/slavs/castle_retro.rle")),
    ("art/slavs/cathedral_retro.rle", include_bytes!("../art/slavs/cathedral_retro.rle")),
    ("art/slavs/cathedral_retro_mask.png", include_bytes!("../art/slavs/cathedral_retro_mask.png")),
    ("art/slavs/church_retro.rle", include_bytes!("../art/slavs/church_retro.rle")),
    ("art/slavs/church_retro_mask.png", include_bytes!("../art/slavs/church_retro_mask.png")),
    ("art/slavs/egg.png", include_bytes!("../art/slavs/egg.png")),
    ("art/slavs/farm_retro.rle", include_bytes!("../art/slavs/farm_retro.rle")),
    ("art/slavs/farm_retro_mask.png", include_bytes!("../art/slavs/farm_retro_mask.png")),
    ("art/slavs/fort/gate0_anim.rle", include_bytes!("../art/slavs/fort/gate0_anim.rle")),
    ("art/slavs/fort/gate0_build.rle", include_bytes!("../art/slavs/fort/gate0_build.rle")),
    ("art/slavs/fort/gate1_anim.rle", include_bytes!("../art/slavs/fort/gate1_anim.rle")),
    ("art/slavs/fort/gate1_build.rle", include_bytes!("../art/slavs/fort/gate1_build.rle")),
    ("art/slavs/fort/gate2_anim.rle", include_bytes!("../art/slavs/fort/gate2_anim.rle")),
    ("art/slavs/fort/gate2_build.rle", include_bytes!("../art/slavs/fort/gate2_build.rle")),
    ("art/slavs/fort/gate3_anim.rle", include_bytes!("../art/slavs/fort/gate3_anim.rle")),
    ("art/slavs/fort/gate3_build.rle", include_bytes!("../art/slavs/fort/gate3_build.rle")),
    ("art/slavs/fort/stolp.rle", include_bytes!("../art/slavs/fort/stolp.rle")),
    ("art/slavs/fort/wall_build.rle", include_bytes!("../art/slavs/fort/wall_build.rle")),
    ("art/slavs/fort/wall_horz.rle", include_bytes!("../art/slavs/fort/wall_horz.rle")),
    ("art/slavs/fort/wall_horz_dest.rle", include_bytes!("../art/slavs/fort/wall_horz_dest.rle")),
    ("art/slavs/fort/wall_tower.rle", include_bytes!("../art/slavs/fort/wall_tower.rle")),
    ("art/slavs/fort/wall_tower_dest.rle", include_bytes!("../art/slavs/fort/wall_tower_dest.rle")),
    ("art/slavs/fort/wall_vert.rle", include_bytes!("../art/slavs/fort/wall_vert.rle")),
    ("art/slavs/fort/wall_vert_dest.rle", include_bytes!("../art/slavs/fort/wall_vert_dest.rle")),
    ("art/slavs/harbor_retro.rle", include_bytes!("../art/slavs/harbor_retro.rle")),
    ("art/slavs/harbor_retro_mask.png", include_bytes!("../art/slavs/harbor_retro_mask.png")),
    ("art/slavs/hen_icon.png", include_bytes!("../art/slavs/hen_icon.png")),
    ("art/slavs/hen_walk.png", include_bytes!("../art/slavs/hen_walk.png")),
    ("art/slavs/house_retro.rle", include_bytes!("../art/slavs/house_retro.rle")),
    ("art/slavs/house_retro_mask.png", include_bytes!("../art/slavs/house_retro_mask.png")),
    ("art/slavs/lumber_retro.rle", include_bytes!("../art/slavs/lumber_retro.rle")),
    ("art/slavs/lumber_retro_mask.png", include_bytes!("../art/slavs/lumber_retro_mask.png")),
    ("art/slavs/magetower_retro.rle", include_bytes!("../art/slavs/magetower_retro.rle")),
    ("art/slavs/magetower_retro_mask.png", include_bytes!("../art/slavs/magetower_retro_mask.png")),
    ("art/slavs/market_retro.rle", include_bytes!("../art/slavs/market_retro.rle")),
    ("art/slavs/market_retro_mask.png", include_bytes!("../art/slavs/market_retro_mask.png")),
    ("art/slavs/outpost_retro.rle", include_bytes!("../art/slavs/outpost_retro.rle")),
    ("art/slavs/outpost_retro_mask.png", include_bytes!("../art/slavs/outpost_retro_mask.png")),
    ("art/slavs/stable_retro.rle", include_bytes!("../art/slavs/stable_retro.rle")),
    ("art/slavs/stable_retro_mask.png", include_bytes!("../art/slavs/stable_retro_mask.png")),
    ("art/slavs/storage_retro.rle", include_bytes!("../art/slavs/storage_retro.rle")),
    ("art/slavs/storage_retro_mask.png", include_bytes!("../art/slavs/storage_retro_mask.png")),
    ("art/slavs/tavern_retro.rle", include_bytes!("../art/slavs/tavern_retro.rle")),
    ("art/slavs/tavern_retro_mask.png", include_bytes!("../art/slavs/tavern_retro_mask.png")),
    ("art/slavs/tcguild_retro.rle", include_bytes!("../art/slavs/tcguild_retro.rle")),
    ("art/slavs/tcguild_retro_mask.png", include_bytes!("../art/slavs/tcguild_retro_mask.png")),
    ("art/slavs/workshop_retro.rle", include_bytes!("../art/slavs/workshop_retro.rle")),
    ("art/slavs/workshop_retro_mask.png", include_bytes!("../art/slavs/workshop_retro_mask.png")),
    ("art/sound/bear.wav", include_bytes!("../art/sound/bear.wav")),
    ("art/sound/boar.wav", include_bytes!("../art/sound/boar.wav")),
    ("art/sound/hen_egg.wav", include_bytes!("../art/sound/hen_egg.wav")),
    ("art/sound/hen_order.wav", include_bytes!("../art/sound/hen_order.wav")),
    ("art/sound/hen_select.wav", include_bytes!("../art/sound/hen_select.wav")),
    ("art/sound/leszy.wav", include_bytes!("../art/sound/leszy.wav")),
    ("art/sound/raven.wav", include_bytes!("../art/sound/raven.wav")),
    ("art/sound/wolf.wav", include_bytes!("../art/sound/wolf.wav")),
    ("src/army/motion.ini", include_bytes!("../src/army/motion.ini")),
    ("src/chram/motion.ini", include_bytes!("../src/chram/motion.ini")),
    ("art/army/heavyhorse_attack_s.rle", include_bytes!("../art/army/heavyhorse_attack_s.rle")),
    ("art/army/heavyhorse_die_s.rle", include_bytes!("../art/army/heavyhorse_die_s.rle")),
    ("art/army/heavyhorse_idle_s.rle", include_bytes!("../art/army/heavyhorse_idle_s.rle")),
    ("art/army/heavyhorse_walk_s.rle", include_bytes!("../art/army/heavyhorse_walk_s.rle")),
    ("art/army/javelin_attack_s.rle", include_bytes!("../art/army/javelin_attack_s.rle")),
    ("art/army/javelin_die_s.rle", include_bytes!("../art/army/javelin_die_s.rle")),
    ("art/army/javelin_idle_s.rle", include_bytes!("../art/army/javelin_idle_s.rle")),
    ("art/army/javelin_projectile.rle", include_bytes!("../art/army/javelin_projectile.rle")),
    ("art/army/javelin_walk_s.rle", include_bytes!("../art/army/javelin_walk_s.rle")),
    ("art/army/lancer_attack_s.rle", include_bytes!("../art/army/lancer_attack_s.rle")),
    ("art/army/lancer_die_s.rle", include_bytes!("../art/army/lancer_die_s.rle")),
    ("art/army/lancer_idle_s.rle", include_bytes!("../art/army/lancer_idle_s.rle")),
    ("art/army/lancer_walk_s.rle", include_bytes!("../art/army/lancer_walk_s.rle")),
    ("art/army/retainer_attack_s.rle", include_bytes!("../art/army/retainer_attack_s.rle")),
    ("art/army/retainer_die_s.rle", include_bytes!("../art/army/retainer_die_s.rle")),
    ("art/army/retainer_idle_s.rle", include_bytes!("../art/army/retainer_idle_s.rle")),
    ("art/army/retainer_walk_s.rle", include_bytes!("../art/army/retainer_walk_s.rle")),
    ("art/army/rider_attack_s.rle", include_bytes!("../art/army/rider_attack_s.rle")),
    ("art/army/rider_die_s.rle", include_bytes!("../art/army/rider_die_s.rle")),
    ("art/army/rider_idle_s.rle", include_bytes!("../art/army/rider_idle_s.rle")),
    ("art/army/rider_walk_s.rle", include_bytes!("../art/army/rider_walk_s.rle")),
    ("art/army/shieldman_attack_s.rle", include_bytes!("../art/army/shieldman_attack_s.rle")),
    ("art/army/shieldman_die_s.rle", include_bytes!("../art/army/shieldman_die_s.rle")),
    ("art/army/shieldman_idle_s.rle", include_bytes!("../art/army/shieldman_idle_s.rle")),
    ("art/army/shieldman_walk_s.rle", include_bytes!("../art/army/shieldman_walk_s.rle")),
    ("art/army/slinger_attack_s.rle", include_bytes!("../art/army/slinger_attack_s.rle")),
    ("art/army/slinger_die_s.rle", include_bytes!("../art/army/slinger_die_s.rle")),
    ("art/army/slinger_idle_s.rle", include_bytes!("../art/army/slinger_idle_s.rle")),
    ("art/army/slinger_walk_s.rle", include_bytes!("../art/army/slinger_walk_s.rle")),
    ("art/army/watcher_attack_s.rle", include_bytes!("../art/army/watcher_attack_s.rle")),
    ("art/army/watcher_die_s.rle", include_bytes!("../art/army/watcher_die_s.rle")),
    ("art/army/watcher_idle_s.rle", include_bytes!("../art/army/watcher_idle_s.rle")),
    ("art/army/watcher_walk_s.rle", include_bytes!("../art/army/watcher_walk_s.rle")),
    ("art/chram/motion/leszy_roots.rle", include_bytes!("../art/chram/motion/leszy_roots.rle")),
    ("art/chram/motion/leszy_roots_s.rle", include_bytes!("../art/chram/motion/leszy_roots_s.rle")),
    ("art/chram/roots_ground.rle", include_bytes!("../art/chram/roots_ground.rle")),
];
static SOURCE_ROOT: OnceLock<Option<PathBuf>> = OnceLock::new();
static CACHE: OnceLock<Mutex<HashMap<String, &'static [u8]>>> = OnceLock::new();
const MAX_BYTES: u64 = 32 * 1024 * 1024;

fn source_root() -> Option<&'static Path> {
    SOURCE_ROOT.get_or_init(|| Some(std::env::current_exe().ok()?.parent()?.join("src"))).as_deref()
}
fn fallback(path: &str) -> Option<&'static [u8]> {
    ART.iter().find(|(name, _)| *name == path).map(|(_, data)| *data)
}
fn read(path: &Path) -> Result<Vec<u8>> {
    use std::io::Read;
    let file = fs::File::open(path)?;
    if file.metadata()?.len() > MAX_BYTES { return Err(invalid("Za duży zasób")); }
    let mut data = Vec::new();
    file.take(MAX_BYTES + 1).read_to_end(&mut data)?;
    if data.is_empty() || data.len() as u64 > MAX_BYTES { return Err(invalid("Nieprawidłowy rozmiar zasobu")); }
    Ok(data)
}
fn is_motion(path: &str) -> bool {
    path.starts_with("art/army/") || path.starts_with("art/chram/motion/")
}
fn validate_file(path: &str, data: &[u8], original: &[u8]) -> Result<()> {
    if path.ends_with(".rle") {
        let sheet = crate::sprite::SpriteSheet::decode(data)?;
        if !is_motion(path) || path == "art/army/javelin_projectile.rle" {
            // Fixed-layout buildings and fortifications keep their established geometry.
            if data.get(4..12) != original.get(4..12) || data.get(20..26) != original.get(20..26) {
                return Err(invalid("Zmiana układu lub formatu arkusza wymaga poprawki kodu"));
            }
        } else if sheet.columns != 8 || sheet.rows > 128 || sheet.rows == 0 || data.get(24..26) != original.get(24..26) {
            return Err(invalid("Animacja musi zachować 8 kierunków i format RLE"));
        }
    } else if path.ends_with(".png") {
        let image = crate::bitmap::Bitmap::decode_png(data)?;
        let old = crate::bitmap::Bitmap::decode_png(original)?;
        if (image.width, image.height) != (old.width, old.height) {
            return Err(invalid("Zmiana wymiarów PNG wymaga poprawki kodu"));
        }
    } else if path.ends_with(".bmp") {
        let image = crate::bitmap::Bitmap::decode(data, true)?;
        let old = crate::bitmap::Bitmap::decode(original, true)?;
        if (image.width, image.height) != (old.width, old.height) {
            return Err(invalid("Zmiana wymiarów BMP wymaga poprawki kodu"));
        }
    } else if path.ends_with(".wav") {
        crate::audio::Pcm::decode(data)?;
    } else if path.ends_with("/motion.ini") {
        // Only existing animation sections and fields may be edited as data.
        let ini = crate::assets::Ini::parse(data);
        let old = crate::assets::Ini::parse(original);
        let sections: Vec<_> = ini.section_names().collect();
        if sections != old.section_names().collect::<Vec<_>>() {
            return Err(invalid("Dodanie/usunięcie sekcji animacji wymaga poprawki kodu"));
        }
        for section in sections {
            let keys: Vec<_> = ini.entries(section).iter().map(|(k,_)| k.as_str()).collect();
            if keys != old.entries(section).iter().map(|(k,_)| k.as_str()).collect::<Vec<_>>() || ini.get(section,"image") != old.get(section,"image") {
                return Err(invalid("Nie można zmieniać mapowania plików animacji w szybkiej poprawce"));
            }
            for (key,value) in ini.entries(section) {
                if key == "image" { continue; }
                let parts: Option<Vec<i32>> = value.split(',').map(|v|v.trim().parse::<i32>().ok()).collect();
                let parts = parts.ok_or_else(||invalid("Nieprawidłowa wartość animacji"))?;
                if matches!(key.as_str(), "object" | "selection") {
                    if parts.len()!=4 || parts.iter().any(|&v| !(-2048..=2048).contains(&v)) || parts[2]<=0 || parts[3]<=0 {return Err(invalid("Nieprawidłowa geometria animacji"));}
                } else if parts.len()!=1 || !(1..=60000).contains(&parts[0]) {return Err(invalid("Nieprawidłowy parametr animacji"));}
            }
        }
    } else { return Err(invalid("Nieobsługiwany typ zasobu")); }
    Ok(())
}

/// The game's existing call sites still receive exactly the same byte slices.
/// Cache one immutable copy per asset, for the lifetime of this game process.
/// A new game process reads the newly installed version; no GOG writes.
pub fn bytes(path: &str) -> &'static [u8] {
    let original = fallback(path).expect("unregistered OpenTzar art path");
    let Some(root) = source_root() else { return original; };
    let mut cache = CACHE.get_or_init(|| Mutex::new(HashMap::new())).lock().unwrap_or_else(|e|e.into_inner());
    if let Some(data) = cache.get(path) { return *data; }
    let file = root.join(path);
    let data: &'static [u8] = match read(&file) {
        Ok(data) if data.as_slice() == original => original,
        Ok(data) => match validate_file(path,&data,original) {
            Ok(()) => &*Box::leak(data.into_boxed_slice()),
            Err(e) => { eprintln!("[ZASOBY] {}: {e}; używam kopii w EXE",file.display()); original }
        },
        Err(_) => original,
    };
    cache.insert(path.to_owned(),data);
    data
}

/// Validate using the game's own decoders, without starting a game or GOG.
/// Called by Studio before the new version is made active.
pub fn check_installation() -> Result<()> {
    let root = source_root().ok_or_else(||invalid("Brak katalogu EXE"))?;
    let parent = root.parent().ok_or_else(||invalid("Brak katalogu wersji"))?;
    let marker = parent.join("runtime-art-v1.ok");
    if marker.exists() { fs::remove_file(&marker)?; }
    let mut total_bytes = 0usize;
    for &(path, original) in ART {
        let data = read(&root.join(path)).map_err(|e|invalid(format!("{path}: {e}")))?;
        total_bytes += data.len();
        if total_bytes > 128 * 1024 * 1024 { return Err(invalid("Zasoby przekraczają 128 MB")); }
        validate_file(path,&data,original).map_err(|e|invalid(format!("{path}: {e}")))?;
    }
    // Match each motion INI's declared frame grid to its actual RLE header.
    for (ini_path,folder) in [("src/army/motion.ini","art/army"),("src/chram/motion.ini","art/chram/motion")] {
        let data=read(&root.join(ini_path))?;
        let ini=crate::assets::Ini::parse(&data);
        for section in ini.section_names() {
            let image=ini.get(section,"image").ok_or_else(||invalid("Brak image"))?;
            let basename=image.rsplit('/').next().and_then(|s|s.strip_suffix(".bmp")).ok_or_else(||invalid("Zła nazwa animacji"))?;
            let path=format!("{folder}/{basename}.rle");
            if fallback(&path).is_none() {return Err(invalid(format!("Nieznana animacja: {path}")));}
            let rle=read(&root.join(&path))?;
            let cols=crate::u32_at(&rle,4)? as usize;
            let rows=crate::u32_at(&rle,8)? as usize;
            let number=|key|ini.get(section,key).and_then(|s|s.parse::<usize>().ok());
            if number("cols")!=Some(cols) || number("rows")!=Some(rows) || number("actionrow").is_none_or(|v|v>rows) {
                return Err(invalid(format!("{section}: niezgodne klatki RLE i INI")));
            }
            if basename.ends_with("_s") {
                let body=format!("{folder}/{}.rle",basename.trim_end_matches("_s"));
                let body=read(&root.join(body))?;
                if rle.get(4..12)!=body.get(4..12) {return Err(invalid(format!("{section}: cień i postać mają różną liczbę klatek")));}
            }
        }
    }
    let list=ART.iter().map(|(path,_)|*path).collect::<Vec<_>>().join("\n");
    fs::write(&marker, list)?;
    let _=fs::remove_file(parent.join("runtime-art-v1.error"));
    println!("[ZASOBY] Zweryfikowano {} zasobów; oryginalna instalacja nie była otwierana.",ART.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn embedded_registry_is_unique_and_decodes() {
        let mut names=std::collections::BTreeSet::new();
        for &(name,data) in ART {
            assert!(names.insert(name),"duplicate {name}");
            validate_file(name,data,data).unwrap_or_else(|e|panic!("{name}: {e}"));
        }
    }
    #[test]
    fn invalid_resource_is_rejected() {
        let name="art/army/tribal_walk.rle";
        assert!(validate_file(name,b"invalid",fallback(name).unwrap()).is_err());
        assert!(fallback("../Packs/images.wdt").is_none());
    }
}
