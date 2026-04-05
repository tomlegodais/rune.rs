use std::collections::HashMap;

use anyhow::{Context, Result, bail};
use serde::Deserialize;

const WIKI_API: &str = "https://oldschool.runescape.wiki/api.php";

#[derive(Deserialize)]
struct ApiResponse {
    parse: ParseResult,
}

#[derive(Deserialize)]
struct ParseResult {
    wikitext: WikiText,
}

#[derive(Deserialize)]
struct WikiText {
    #[serde(rename = "*")]
    content: String,
}

struct ItemStats {
    atk_stab: i16,
    atk_slash: i16,
    atk_crush: i16,
    atk_magic: i16,
    atk_ranged: i16,
    def_stab: i16,
    def_slash: i16,
    def_crush: i16,
    def_magic: i16,
    def_ranged: i16,
    str_bonus: i16,
    ranged_str: i16,
    magic_dmg: i16,
    prayer: i16,
    atk_speed: Option<i16>,
    weapon_category: Option<String>,
    weight: Option<i32>,
}

fn page_name_from_url(url: &str) -> Result<String> {
    if let Some(path) = url.strip_prefix("https://oldschool.runescape.wiki/w/") {
        Ok(path.to_string())
    } else if url.starts_with("http") {
        bail!("Unrecognised wiki URL format: {url}");
    } else {
        Ok(url.to_string())
    }
}

fn parse_infobox(wikitext: &str, template: &str) -> Option<HashMap<String, String>> {
    let needle = format!("{{{{{template}");
    let start = wikitext.find(&needle)?;
    let inner = &wikitext[start..];

    let mut depth = 0usize;
    let mut end = 0usize;
    let chars: Vec<char> = inner.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if i + 1 < chars.len() && chars[i] == '{' && chars[i + 1] == '{' {
            depth += 1;
            i += 2;
        } else if i + 1 < chars.len() && chars[i] == '}' && chars[i + 1] == '}' {
            depth -= 1;
            if depth == 0 {
                end = i + 2;
                break;
            }
            i += 2;
        } else {
            i += 1;
        }
    }

    let block = &inner[..end];
    let mut params = HashMap::new();
    for segment in block.split('|').skip(1) {
        if let Some((k, v)) = segment.split_once('=') {
            let key = k.trim().to_lowercase();
            let val = v.trim().trim_end_matches("}}").trim().to_string();
            params.insert(key, val);
        }
    }
    Some(params)
}

fn parse_bonus(params: &HashMap<String, String>, key: &str) -> i16 {
    params
        .get(key)
        .and_then(|v| v.trim_start_matches('+').parse::<i16>().ok())
        .unwrap_or(0)
}

fn map_weapon_category(cat: &str) -> Option<&'static str> {
    match cat.to_lowercase().replace(' ', "_").as_str() {
        "two-handed_sword" | "2h_sword" | "2hsword" | "two_handed_sword" => Some("two_handed_sword"),
        "axe" => Some("axe"),
        "banner" => Some("banner"),
        "blunt" => Some("blunt"),
        "bludgeon" => Some("bludgeon"),
        "bulwark" => Some("bulwark"),
        "claw" => Some("claw"),
        "egg" => Some("egg"),
        "partisan" => Some("partisan"),
        "pickaxe" => Some("pickaxe"),
        "polearm" => Some("polearm"),
        "polestaff" => Some("polestaff"),
        "scythe" => Some("scythe"),
        "slash_sword" => Some("slash_sword"),
        "spear" => Some("spear"),
        "spiked" => Some("spiked"),
        "stab_sword" => Some("stab_sword"),
        "unarmed" => Some("unarmed"),
        "whip" => Some("whip"),
        "bow" => Some("bow"),
        "blaster" => Some("blaster"),
        "chinchompa" => Some("chinchompa"),
        "crossbow" => Some("crossbow"),
        "gun" => Some("gun"),
        "thrown" => Some("thrown"),
        "bladed_staff" => Some("bladed_staff"),
        "powered_staff" => Some("powered_staff"),
        "staff" => Some("staff"),
        "salamander" => Some("salamander"),
        "multi-style" | "multi_style" => Some("multi_style"),
        _ => None,
    }
}

async fn fetch_stats(page: &str) -> Result<ItemStats> {
    let client = reqwest::Client::builder()
        .user_agent("rune.rs wiki-item tool")
        .build()?;

    let url = format!("{WIKI_API}?action=parse&page={page}&prop=wikitext&redirects=1&format=json");
    let resp: ApiResponse = client
        .get(&url)
        .send()
        .await
        .context("request to wiki API failed")?
        .json()
        .await
        .context("failed to parse wiki API response")?;

    let wikitext = resp.parse.wikitext.content;

    let item_params = parse_infobox(&wikitext, "Infobox Item");
    let weight = item_params
        .as_ref()
        .and_then(|p| p.get("weight"))
        .and_then(|v| v.parse::<f64>().ok())
        .map(|kg| (kg * 1000.0).round() as i32);

    let params =
        parse_infobox(&wikitext, "Infobox Bonuses").context("could not find {{Infobox Bonuses}} on this page")?;

    let weapon_category = params
        .get("combatstyle")
        .and_then(|s| map_weapon_category(s))
        .map(str::to_string);

    let atk_speed = params
        .get("speed")
        .and_then(|v| v.trim_start_matches('+').parse::<i16>().ok());

    Ok(ItemStats {
        atk_stab: parse_bonus(&params, "astab"),
        atk_slash: parse_bonus(&params, "aslash"),
        atk_crush: parse_bonus(&params, "acrush"),
        atk_magic: parse_bonus(&params, "amagic"),
        atk_ranged: parse_bonus(&params, "arange"),
        def_stab: parse_bonus(&params, "dstab"),
        def_slash: parse_bonus(&params, "dslash"),
        def_crush: parse_bonus(&params, "dcrush"),
        def_magic: parse_bonus(&params, "dmagic"),
        def_ranged: parse_bonus(&params, "drange"),
        str_bonus: parse_bonus(&params, "str"),
        ranged_str: parse_bonus(&params, "rstr"),
        magic_dmg: parse_bonus(&params, "mdmg"),
        prayer: parse_bonus(&params, "prayer"),
        atk_speed,
        weapon_category,
        weight,
    })
}

fn print_sql(obj_id: u32, s: &ItemStats) {
    let weapon_cat = s
        .weapon_category
        .as_deref()
        .map(|v| format!("'{v}'"))
        .unwrap_or_else(|| "NULL".to_string());
    let atk_speed = s.atk_speed.map(|v| v.to_string()).unwrap_or_else(|| "NULL".to_string());
    let weight = s.weight.map(|v| v.to_string()).unwrap_or_else(|| "NULL".to_string());

    println!(
        r#"UPDATE obj_configs SET
    weapon_category = {weapon_cat},
    atk_stab        = {}, atk_slash   = {}, atk_crush  = {},
    atk_magic       = {}, atk_ranged  = {},
    def_stab        = {}, def_slash   = {}, def_crush  = {},
    def_magic       = {}, def_ranged  = {},
    str_bonus       = {}, ranged_str  = {},
    magic_dmg       = {}, prayer      = {},
    atk_speed       = {atk_speed},
    weight          = {weight}
WHERE obj_id = {obj_id};"#,
        s.atk_stab,
        s.atk_slash,
        s.atk_crush,
        s.atk_magic,
        s.atk_ranged,
        s.def_stab,
        s.def_slash,
        s.def_crush,
        s.def_magic,
        s.def_ranged,
        s.str_bonus,
        s.ranged_str,
        s.magic_dmg,
        s.prayer,
    );
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        bail!(
            "Usage: wiki-item <wiki-url-or-page-name> <obj-id>\n\nExamples:\n  wiki-item https://oldschool.runescape.wiki/w/Rune_scimitar 1333\n  wiki-item Rune_scimitar 1333"
        );
    }

    let page = page_name_from_url(&args[1])?;
    let obj_id: u32 = args[2].parse().context("obj-id must be a positive integer")?;

    eprintln!("Fetching https://oldschool.runescape.wiki/w/{page} ...");
    let stats = fetch_stats(&page).await?;

    if stats.weapon_category.is_none() {
        eprintln!("Warning: could not map combatstyle to a known WeaponCategory — weapon_category will be NULL.");
    }

    print_sql(obj_id, &stats);
    Ok(())
}
