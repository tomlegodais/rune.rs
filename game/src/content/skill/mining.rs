#![cfg_attr(rustfmt, rustfmt::skip)]

use filesystem::config::WearPos;

struct Pickaxe {
    id: u16,
    level: u8,
    seq: u16,
    speed: u16,
    fast_chance: f64,
}

const PICKAXES: &[Pickaxe] = &[
    Pickaxe { id: 1265, level: 1, seq: 625, speed: 8, fast_chance: 0.0 },
    Pickaxe { id: 1267, level: 1, seq: 626, speed: 7, fast_chance: 0.0 },
    Pickaxe { id: 1269, level: 6, seq: 627, speed: 6, fast_chance: 0.0 },
    Pickaxe { id: 1273, level: 21, seq: 629, speed: 5, fast_chance: 0.0 },
    Pickaxe { id: 1271, level: 31, seq: 628, speed: 4, fast_chance: 0.0 },
    Pickaxe { id: 1275, level: 41, seq: 624, speed: 3, fast_chance: 0.0 },
    Pickaxe { id: 13661, level: 41, seq: 10222, speed: 3, fast_chance: 0.0 },
    Pickaxe { id: 15259, level: 61, seq: 12187, speed: 3, fast_chance: 1.0 / 6.0 },
];

struct Rock {
    name: &'static str,
    ore: u16,
    level: u8,
    xp: f64,
    respawn: u16,
    low: u16,
    high: u16,
}

const COPPER: Rock = Rock { name: "copper", ore: 436, level: 1, xp: 17.5, respawn: 4, low: 100, high: 350 };
const TIN: Rock = Rock { name: "tin", ore: 438, level: 1, xp: 17.5, respawn: 4, low: 100, high: 350 };
const IRON: Rock = Rock { name: "iron", ore: 440, level: 15, xp: 35.0, respawn: 9, low: 96, high: 350 };
const SILVER: Rock = Rock { name: "silver", ore: 442, level: 20, xp: 40.0, respawn: 100, low: 25, high: 200 };
const COAL: Rock = Rock { name: "coal", ore: 453, level: 30, xp: 50.0, respawn: 50, low: 16, high: 100 };
const GOLD: Rock = Rock { name: "gold", ore: 444, level: 40, xp: 65.0, respawn: 100, low: 7, high: 75 };
const MITHRIL: Rock = Rock { name: "mithril", ore: 447, level: 55, xp: 80.0, respawn: 200, low: 4, high: 50 };
const ADAMANTITE: Rock = Rock { name: "adamantite", ore: 449, level: 70, xp: 95.0, respawn: 400, low: 2, high: 25 };
const RUNITE: Rock = Rock { name: "runite", ore: 451, level: 85, xp: 125.0, respawn: 1200, low: 1, high: 18 };

fn best_pickaxe(player: &crate::player::Player) -> Option<&'static Pickaxe> {
    let mining = player.stat().level(crate::player::Stat::Mining);
    let wielded = player.worn().slot(WearPos::Weapon).map(|o| o.id);

    PICKAXES
        .iter()
        .rev()
        .find(|p| mining >= p.level && (wielded == Some(p.id) || player.inv().count(p.id) > 0))
}

fn roll_speed(pickaxe: &Pickaxe) -> u16 {
    pickaxe.speed
        - (pickaxe.fast_chance > 0.0 && rand::random::<f64>() < pickaxe.fast_chance) as u16
}

fn success_chance(rock: &Rock, level: u8) -> f64 {
    let lvl = level as f64;
    let numerator = rock.low as f64 * (99.0 - lvl) / 98.0 + rock.high as f64 * (lvl - 1.0) / 98.0;
    ((1.0 + (numerator + 0.5).floor()) / 256.0).clamp(0.0, 1.0)
}

macro_rules! mine_rock {
    ($fn_name:ident, $loc_id:expr, $depleted:expr, $rock:ident) => {
        #[macros::on_loc(id = $loc_id, op = Op1)]
        async fn $fn_name() {
            requires!(stat = Mining, level = $rock.level);
            requires!(inv, slots = 1);

            let Some(pickaxe) = best_pickaxe(&player) else {
                send_message!("You do not have a pickaxe which you have the Mining level to use.");
                return;
            };

            send_message!("You swing your pickaxe at the rock.");

            let seq_id = pickaxe.seq;
            repeat!(delay = roll_speed(pickaxe), seq = seq_id, {
                requires!(loc);
                requires!(inv, slots = 1);

                let Some(pickaxe) = best_pickaxe(&player) else {
                    send_message!("You do not have a pickaxe which you have the Mining level to use.");
                    break;
                };

                let mining_level = crate::player::active_player().stat().level(crate::player::Stat::Mining);
                if failed!(success_chance = success_chance(&$rock, mining_level)) {
                    continue;
                }

                inv_add!(id = $rock.ore);
                give_xp!(stat = Mining, amount = $rock.xp);
                send_message!("You manage to mine some ore.");
                break;
            });

            loc_replace!(replace = $depleted, ticks = $rock.respawn);
            send_message!("The rock has been depleted.");
        }
    };
}

macro_rules! prospect_rock {
    ($fn_name:ident, $loc_id:expr, $rock:ident) => {
        #[macros::on_loc(id = $loc_id, op = Op2)]
        async fn $fn_name() {
            send_message!("You examine the rock for ores...");
            delay!(3);
            if loc_replaced!() {
                send_message!("There is currently no ore available in this rock.");
            } else {
                send_message!("This rock contains {}.", $rock.name);
            }
        }
    };
}

macro_rules! mine_rocks {
    ($rock:ident, $(($id:expr, $depleted:expr)),+ $(,)?) => {
        paste::paste! {
            $(
                mine_rock!([< mine_ $rock:lower _ $id >], $id, $depleted, $rock);
                prospect_rock!([< prospect_ $rock:lower _ $id >], $id, $rock);
            )+
        }
    };
}

mine_rocks!(COPPER,
    (11936, 11552), (11937, 11553), (11938, 11554),
    (11960, 11555), (11961, 11556), (11962, 11557),
    (14906, 14898), (14907, 14899),
    (18991, 19003), (18992, 19004), (18993, 19005),
    (21284, 21296), (21285, 21297), (21286, 21298),
    (29230, 29218), (29231, 29219),
);

mine_rocks!(TIN,
    (11933, 11552), (11934, 11553), (11935, 11554),
    (11957, 11555), (11958, 11556), (11959, 11557),
    (14902, 14898), (14903, 14899),
    (18994, 19003), (18995, 19004), (18996, 19005),
    (19024, 19027), (19025, 19022), (19026, 19029),
    (21293, 21296), (21294, 21297), (21295, 21298),
    (29227, 29218), (29229, 29220),
);

mine_rocks!(IRON,
    (6943, 6947), (6944, 6948),
    (11954, 11555), (11955, 11556), (11956, 11557),
    (14856, 14832), (14857, 14833), (14858, 14834),
    (14913, 14915), (14914, 14916),
    (19000, 19003), (19001, 19004), (19002, 19005),
    (21281, 21296), (21282, 21297), (21283, 21298),
    (29221, 29218), (29222, 29219), (29223, 29220),
    (31071, 31059), (31072, 31060),
    (32441, 33400), (32442, 33401), (32443, 33402),
    (32451, 32447), (32452, 32448),
);

mine_rocks!(SILVER,
    (6945, 6947), (6946, 6948),
    (11948, 11555), (11949, 11556), (11950, 11557),
    (16998, 17007), (16999, 17008), (17000, 17009),
    (29224, 29218), (29225, 29219), (29226, 29220),
    (32444, 33400), (32445, 33401), (32446, 33402),
    (37670, 37669),
);

mine_rocks!(COAL,
    (10948, 10944),
    (11930, 11552), (11931, 11553), (11932, 11554),
    (11963, 11555), (11964, 11556),
    (14850, 14832), (14851, 14833), (14852, 14834),
    (15246, 15249), (15247, 15250), (15248, 15251),
    (18997, 19003), (18998, 19004), (18999, 19005),
    (21287, 21296), (21288, 21297), (21289, 21298),
    (29215, 29218), (29216, 29219), (29217, 29220),
    (31068, 31059), (31069, 31060),
    (32426, 33400), (32427, 33401), (32428, 33402),
    (32449, 32447), (32450, 32448),
);

mine_rocks!(GOLD,
    (2609, 21296), (2610, 21297), (2611, 21298),
    (10574, 19003), (10575, 19004), (10576, 19005),
    (11951, 11555), (11952, 11556), (11953, 11557),
    (17001, 17007), (17002, 17008), (17003, 17009),
    (32432, 33400), (32433, 33401), (32434, 33402),
    (45067, 29218),
);

mine_rocks!(MITHRIL,
    (11942, 11552), (11943, 11553), (11944, 11554),
    (11946, 11556), (11947, 11557),
    (14853, 14832), (14854, 14833), (14855, 14834),
    (19012, 19015), (19013, 19010), (19014, 19017),
    (21278, 21296), (21279, 21297), (21280, 21298),
    (29236, 29218),
    (32438, 33400), (32439, 33401), (32440, 33402),
);

mine_rocks!(ADAMANTITE,
    (11939, 11552), (11941, 11554),
    (14862, 14832), (14863, 14833), (14864, 14834),
    (19018, 19021), (19019, 19022), (19020, 19017),
    (21275, 21296), (21276, 21297), (21277, 21298),
    (29233, 29218), (29235, 29220),
    (32435, 33400), (32436, 33401), (32437, 33402),
);

mine_rocks!(RUNITE,
    (14859, 14832), (14860, 14833),
    (33078, 33400), (33079, 33401),
    (45069, 29218),
);
