pub fn format_display_name(name: &str) -> String {
    name.split('_')
        .map(|w| w.split_at(1))
        .map(|(h, t)| format!("{}{}", h.to_uppercase(), t))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn format_sentence(text: &str) -> String {
    text.chars()
        .scan(true, |cap, c| {
            let out = match c {
                '.' | '!' | '?' => {
                    *cap = true;
                    c
                }
                ' ' if *cap => c,
                _ if *cap => {
                    *cap = false;
                    c.to_ascii_uppercase()
                }
                _ => c.to_ascii_lowercase(),
            };
            Some(out)
        })
        .collect()
}
