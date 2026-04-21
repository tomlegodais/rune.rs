use filesystem::{WeaponCategory, WearPos};

use crate::provider;

const SHIELD_BLOCK_SEQ: u16 = 1156;

fn weapon_type(player: &crate::player::Player) -> Option<&'static filesystem::ObjType> {
    player
        .worn()
        .slot(WearPos::Weapon)
        .and_then(|obj| provider::get_obj_type(obj.id as u32))
}

pub fn attack(player: &crate::player::Player) -> u16 {
    let style = player.combat().combat_style() as usize;
    if let Some(t) = weapon_type(player)
        && !t.atk_seq.is_empty()
    {
        return t
            .atk_seq
            .get(style)
            .or_else(|| t.atk_seq.first())
            .copied()
            .unwrap_or(422);
    }

    let cat = player.worn().weapon_category().unwrap_or(WeaponCategory::Unarmed);
    let seqs = attack_seqs(cat);
    seqs.get(style).or_else(|| seqs.first()).copied().unwrap_or(422)
}

fn attack_seqs(cat: WeaponCategory) -> &'static [u16] {
    match cat {
        WeaponCategory::Unarmed => &[422, 423, 422],         // punch, kick, block
        WeaponCategory::StabSword => &[386, 386, 390, 388],  // stab, lunge, slash, block
        WeaponCategory::SlashSword => &[390, 390, 386, 388], // chop, slash, lunge, block
        WeaponCategory::TwoHandedSword => &[407, 407, 406, 407], // chop, slash, smash, block
        WeaponCategory::Axe => &[395, 395, 401, 395],        // chop, hack, smash, block
        WeaponCategory::Pickaxe => &[401, 401, 400, 401],    // spike, impale, smash, block
        WeaponCategory::Blunt | WeaponCategory::Spiked => &[401, 401, 401], // pound, pummel, block
        WeaponCategory::Bludgeon => &[401, 401, 401],        // pound, pummel, smash
        WeaponCategory::Spear | WeaponCategory::Polearm => &[400, 400, 400], // lunge, swipe, fend
        WeaponCategory::Whip => &[1658, 1658, 1658],         // flick, lash, deflect
        WeaponCategory::Scythe => &[8056, 8056, 8056, 8056], // reap, chop, jab, block
        WeaponCategory::Claw => &[393, 393, 386, 393],       // chop, slash, lunge, block
        WeaponCategory::Staff | WeaponCategory::BladedStaff => &[419, 419, 419, 419, 419], // bash, pound, focus, spell, spell
        WeaponCategory::Partisan => &[386, 386, 401, 386],                                 // stab, lunge, pound, block
        WeaponCategory::Banner => &[386, 390, 401, 386],                                   // lunge, swipe, pound, block
        WeaponCategory::Bulwark => &[401, 401],                                            // pummel, block
        WeaponCategory::Polestaff => &[419, 419, 419],                                     // bash, pound, block
        WeaponCategory::Bow => &[426, 426, 426],                                           // accurate, rapid, longrange
        WeaponCategory::Crossbow | WeaponCategory::Gun | WeaponCategory::Blaster => &[4230, 4230, 4230],
        WeaponCategory::Thrown => &[806, 806, 806],
        WeaponCategory::Chinchompa => &[7618, 7618, 7618],
        _ => &[422],
    }
}

pub fn block(player: &crate::player::Player) -> u16 {
    if player.worn().slot(WearPos::Shield).is_some() {
        return SHIELD_BLOCK_SEQ;
    }
    if let Some(seq) = weapon_type(player).and_then(|t| t.block_seq) {
        return seq;
    }
    let cat = player.worn().weapon_category().unwrap_or(WeaponCategory::Unarmed);
    match cat {
        WeaponCategory::Unarmed => 424,
        WeaponCategory::StabSword | WeaponCategory::SlashSword => 388,
        WeaponCategory::TwoHandedSword => 7050,
        WeaponCategory::Axe => 397,
        WeaponCategory::Pickaxe => 403,
        WeaponCategory::Blunt | WeaponCategory::Spiked => 403,
        WeaponCategory::Spear | WeaponCategory::Polearm => 400,
        WeaponCategory::Whip => 1659,
        WeaponCategory::Scythe => 8058,
        WeaponCategory::Claw => 397,
        WeaponCategory::Staff | WeaponCategory::BladedStaff => 420,
        WeaponCategory::Bulwark => 7512,
        _ => 424,
    }
}
