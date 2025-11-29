pub fn decode_base37(mut value: i64) -> String {
    if value == 0 {
        return String::new();
    }

    const MAX_LEN: usize = 12;
    let mut chars = Vec::with_capacity(MAX_LEN);

    while value != 0 && chars.len() < MAX_LEN {
        let remainder = (value % 37) as u8;
        value /= 37;

        let c = match remainder {
            1..=26 => (b'a' + (remainder - 1)) as char,
            27..=36 => (b'0' + (remainder - 27)) as char,
            _ => '_',
        };

        chars.push(c);
    }

    chars.reverse();
    chars.into_iter().collect()
}
