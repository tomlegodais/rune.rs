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

struct NpcStats {
    name: String,
    max_hp: u32,
    atk_level: u16,
    str_level: u16,
    def_level: u16,
    atk_bonus: i16,
    str_bonus: i16,
    def_stab: i16,
    def_slash: i16,
    def_crush: i16,
    def_magic: i16,
    def_ranged: i16,
    atk_speed: u16,
    max_hit: u16,
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

fn parse_stat(params: &HashMap<String, String>, key: &str) -> u16 {
    params
        .get(key)
        .and_then(|v| v.trim_start_matches('+').parse::<u16>().ok())
        .unwrap_or(1)
}

fn parse_bonus(params: &HashMap<String, String>, key: &str) -> i16 {
    params
        .get(key)
        .and_then(|v| v.trim_start_matches('+').parse::<i16>().ok())
        .unwrap_or(0)
}

async fn fetch_stats(page: &str) -> Result<NpcStats> {
    let client = reqwest::Client::builder().user_agent("rune.rs wiki-npc tool").build()?;

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
    let params =
        parse_infobox(&wikitext, "Infobox Monster").context("could not find {{Infobox Monster}} on this page")?;

    let name = params.get("name").cloned().unwrap_or_else(|| page.replace('_', " "));
    let max_hp = params.get("hitpoints").and_then(|v| v.parse::<u32>().ok()).unwrap_or(1);
    let max_hit = params.get("max hit").and_then(|v| v.parse::<u16>().ok()).unwrap_or(1);
    let atk_speed = params
        .get("attack speed")
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(4);

    Ok(NpcStats {
        name,
        max_hp,
        atk_level: parse_stat(&params, "att"),
        str_level: parse_stat(&params, "str"),
        def_level: parse_stat(&params, "def"),
        atk_bonus: parse_bonus(&params, "attbns"),
        str_bonus: parse_bonus(&params, "strbns"),
        def_stab: parse_bonus(&params, "dstab"),
        def_slash: parse_bonus(&params, "dslash"),
        def_crush: parse_bonus(&params, "dcrush"),
        def_magic: parse_bonus(&params, "dmagic"),
        def_ranged: parse_bonus(&params, "drange"),
        atk_speed,
        max_hit,
    })
}

fn print_sql(npc_id: u32, s: &NpcStats) {
    println!(
        r#"-- {name}
INSERT INTO npc_configs (npc_id, max_hp, atk_level, str_level, def_level, atk_bonus, str_bonus, def_stab, def_slash, def_crush, def_magic, def_ranged, atk_speed, max_hit)
VALUES ({npc_id}, {max_hp}, {atk_level}, {str_level}, {def_level}, {atk_bonus}, {str_bonus}, {def_stab}, {def_slash}, {def_crush}, {def_magic}, {def_ranged}, {atk_speed}, {max_hit})
ON CONFLICT (npc_id) DO UPDATE SET
    max_hp = EXCLUDED.max_hp, atk_level = EXCLUDED.atk_level, str_level = EXCLUDED.str_level,
    def_level = EXCLUDED.def_level, atk_bonus = EXCLUDED.atk_bonus, str_bonus = EXCLUDED.str_bonus,
    def_stab = EXCLUDED.def_stab, def_slash = EXCLUDED.def_slash, def_crush = EXCLUDED.def_crush,
    def_magic = EXCLUDED.def_magic, def_ranged = EXCLUDED.def_ranged,
    atk_speed = EXCLUDED.atk_speed, max_hit = EXCLUDED.max_hit;"#,
        name = s.name,
        max_hp = s.max_hp,
        atk_level = s.atk_level,
        str_level = s.str_level,
        def_level = s.def_level,
        atk_bonus = s.atk_bonus,
        str_bonus = s.str_bonus,
        def_stab = s.def_stab,
        def_slash = s.def_slash,
        def_crush = s.def_crush,
        def_magic = s.def_magic,
        def_ranged = s.def_ranged,
        atk_speed = s.atk_speed,
        max_hit = s.max_hit,
    );
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        bail!(
            "Usage: wiki-npc <wiki-url-or-page-name> <npc-id>\n\n\
             Examples:\n  \
             wiki-npc https://oldschool.runescape.wiki/w/White_Knight 3348\n  \
             wiki-npc General_Graardor 2215\n  \
             wiki-npc Black_demon 4702"
        );
    }

    let page = page_name_from_url(&args[1])?;
    let npc_id: u32 = args[2].parse().context("npc-id must be a positive integer")?;

    eprintln!("Fetching https://oldschool.runescape.wiki/w/{page} ...");
    let stats = fetch_stats(&page).await?;

    eprintln!(
        "  {} — HP:{} ATK:{} STR:{} DEF:{} MaxHit:{} Speed:{}",
        stats.name, stats.max_hp, stats.atk_level, stats.str_level, stats.def_level, stats.max_hit, stats.atk_speed
    );

    print_sql(npc_id, &stats);
    Ok(())
}
