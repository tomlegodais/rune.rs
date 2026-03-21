use std::sync::OnceLock;

static INSTANCE: OnceLock<Huffman> = OnceLock::new();

pub struct Huffman {
    codes: Vec<u32>,
    lengths: Vec<u8>,
    tree: Vec<i32>,
}

impl Huffman {
    pub fn init(table: &[u8]) {
        INSTANCE.get_or_init(|| Self::build(table));
    }

    pub fn get() -> &'static Huffman {
        INSTANCE.get().expect("huffman not initialized")
    }

    fn build(table: &[u8]) -> Self {
        let len = table.len();
        let mut codes = vec![0u32; len];
        let mut tree = vec![0i32; 8];
        let mut positions = [0u32; 33];

        for (i, &bit_len) in table.iter().enumerate() {
            if bit_len == 0 {
                continue;
            }

            let bit_len = bit_len as usize;
            let boundary = 1u32 << (32 - bit_len);
            let code = positions[bit_len];
            codes[i] = code;

            let next = if code & boundary == 0 {
                for depth in (1..bit_len).rev() {
                    if positions[depth] != code {
                        break;
                    }
                    let half = 1u32 << (32 - depth);
                    if positions[depth] & half != 0 {
                        positions[depth] = positions[depth - 1];
                        break;
                    }
                    positions[depth] |= half;
                }
                code | boundary
            } else {
                positions[bit_len - 1]
            };

            positions[bit_len] = next;
            for depth in (bit_len + 1)..=32 {
                if positions[depth] == code {
                    positions[depth] = next;
                }
            }

            let mut node = 0usize;
            for bit in 0..bit_len {
                let mask = 0x80000000u32 >> bit;
                if code & mask == 0 {
                    node += 1;
                } else {
                    if tree[node] == 0 {
                        tree[node] = tree.len() as i32;
                    }
                    node = tree[node] as usize;
                }
                if node >= tree.len() {
                    tree.resize(node + 1, 0);
                }
            }
            tree[node] = !(i as i32);
        }

        Self {
            codes,
            lengths: table.to_vec(),
            tree,
        }
    }

    pub fn decode(&self, data: &[u8], text_len: usize) -> String {
        if text_len == 0 {
            return String::new();
        }

        let mut out = Vec::with_capacity(text_len);
        let mut node = 0usize;
        let mut byte_idx = 0;

        'outer: loop {
            let byte = data[byte_idx];

            for bit in (0..8).rev() {
                if byte & (1 << bit) == 0 {
                    node += 1;
                } else {
                    node = self.tree[node] as usize;
                }

                let val = self.tree[node];
                if val < 0 {
                    out.push((!val) as u8);
                    if out.len() >= text_len {
                        break 'outer;
                    }
                    node = 0;
                }
            }

            byte_idx += 1;
        }

        String::from_utf8_lossy(&out).into_owned()
    }

    pub fn encode_into(&self, text: &str, buf: &mut [u8], buf_offset: usize) -> usize {
        let message = text.as_bytes();
        let mut bit_pos = (buf_offset << 3) as i32;

        for &ch in message {
            let idx = ch as usize;
            let code = self.codes[idx];
            let num_bits = self.lengths[idx] as i32;
            if num_bits == 0 {
                continue;
            }

            let mut byte_pos = (bit_pos >> 3) as usize;
            let bit_offset = bit_pos & 7;
            let last_byte = ((num_bits + bit_offset - 1) >> 3) as usize + byte_pos;
            let mut shift = 24 + bit_offset;

            buf[byte_pos] &= (0xFFu32 << (8 - bit_offset)) as u8;
            buf[byte_pos] |= (code >> shift) as u8;

            if byte_pos < last_byte {
                byte_pos += 1;
                shift -= 8;
                buf[byte_pos] = (code >> shift) as u8;

                if byte_pos < last_byte {
                    byte_pos += 1;
                    shift -= 8;
                    buf[byte_pos] = (code >> shift) as u8;

                    if byte_pos < last_byte {
                        byte_pos += 1;
                        shift -= 8;
                        buf[byte_pos] = (code << -shift) as u8;
                    }
                }
            }

            bit_pos += num_bits;
        }

        ((bit_pos + 7) >> 3) as usize - buf_offset
    }

    pub fn encode(&self, text: &str) -> Vec<u8> {
        let mut buf = vec![0u8; text.len() * 2 + 1];
        let len = self.encode_into(text, &mut buf, 0);
        buf.truncate(len);
        buf
    }
}