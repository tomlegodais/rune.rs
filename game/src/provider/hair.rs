use std::collections::HashMap;

use filesystem::ParamMap;
use macros::data_provider;
use once_cell::sync::OnceCell;

use crate::provider::{ProviderContext, get_enum_type, get_struct_type};

const LOOK_HAIR_MALE: u32 = 2338;
const LOOK_HAIR_FEMALE: u32 = 2341;
const BODY_LOOK_INDEX: u32 = 789;
const BODY_LOOK_FLAT_LOW: u32 = 790;
const BODY_LOOK_FLAT_MID: u32 = 791;

pub struct HairLookup {
    pub mid: HashMap<u16, u16>,
    pub low: HashMap<u16, u16>,
}

pub struct HairReplacements {
    pub male: HairLookup,
    pub female: HairLookup,
}

static INSTANCE: OnceCell<HairReplacements> = OnceCell::new();

#[data_provider(priority = 1)]
async fn load_hair_replacements(_ctx: &ProviderContext) -> anyhow::Result<()> {
    INSTANCE
        .set(HairReplacements {
            male: build_lookup(LOOK_HAIR_MALE),
            female: build_lookup(LOOK_HAIR_FEMALE),
        })
        .map_err(|_| anyhow::anyhow!("hair replacements already loaded"))
}

fn build_lookup(look_hair_id: u32) -> HairLookup {
    let entries: Vec<_> = get_enum_type(look_hair_id)
        .into_iter()
        .flat_map(|e| {
            e.values
                .keys()
                .filter_map(|&k| e.int_value(k).and_then(|id| get_struct_type(id as u32)))
        })
        .filter_map(|s| s.params.int_param(BODY_LOOK_INDEX).map(|i| (i as u16, s)))
        .collect();

    HairLookup {
        mid: entries
            .iter()
            .filter_map(|(k, s)| s.params.int_param(BODY_LOOK_FLAT_MID).map(|v| (*k, v as u16)))
            .collect(),
        low: entries
            .iter()
            .filter_map(|(k, s)| s.params.int_param(BODY_LOOK_FLAT_LOW).map(|v| (*k, v as u16)))
            .collect(),
    }
}

pub fn get_hair_mid(look: u16, male: bool) -> u16 {
    INSTANCE
        .get()
        .map(|r| if male { &r.male } else { &r.female })
        .and_then(|l| l.mid.get(&look).copied())
        .unwrap_or(look)
}

pub fn get_hair_low(look: u16, male: bool) -> u16 {
    INSTANCE
        .get()
        .map(|r| if male { &r.male } else { &r.female })
        .and_then(|l| l.low.get(&look).copied())
        .unwrap_or(look)
}
