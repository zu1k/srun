use base64::{
    alphabet::Alphabet,
    engine::{GeneralPurpose, self}, Engine,
};
use lazy_static::lazy_static;

const BASE64_ALPHABET: &str = "LVoJPiCN2R8G90yg+hmFHuacZ1OWMnrsSTXkYpUq/3dlbfKwv6xztjI7DeBE45QA";
lazy_static! {
    static ref BASE64_ENGINE: GeneralPurpose = {
        let alphabet = Alphabet::new(BASE64_ALPHABET).unwrap();
        engine::GeneralPurpose::new(&alphabet, engine::GeneralPurposeConfig::new())
    };
}

fn mix(buffer: &[u8], append_size: bool) -> Vec<u32> {
    let mut res: Vec<u32> = buffer
        .chunks(4)
        .map(|chunk| {
            u32::from_le_bytes(chunk.try_into().unwrap_or_else(|_| {
                let mut last_chunk = [0u8, 0, 0, 0];
                last_chunk[..chunk.len()].clone_from_slice(chunk);
                last_chunk
            }))
        })
        .collect();
    if append_size {
        res.push(buffer.len() as u32);
    }
    res
}

fn splite(buffer: Vec<u32>, include_size: bool) -> Vec<u8> {
    let len = buffer.len();
    let size_record = buffer[len - 1];
    if include_size {
        let size = ((len - 1) * 4) as u32;
        if size_record < size - 3 || size_record > size {
            return "".into();
        }
    }

    let mut buffer: Vec<u8> = buffer.iter().flat_map(|i| i.to_le_bytes()).collect();
    if include_size {
        buffer.truncate(size_record as usize);
    }
    buffer
}

fn x_encode(msg: &str, key: &str) -> Vec<u8> {
    if msg.is_empty() {
        return vec![];
    }
    let mut msg = mix(msg.as_bytes(), true);
    let key = mix(key.as_bytes(), false);

    let len = msg.len();
    let last = len - 1;
    let mut right = msg[last];
    let c: u32 = 0x9e3779b9; // 0x9e3779b9 = 0x86014019 | 0x183639A0
    let mut d: u32 = 0;

    let count = 6 + 52 / msg.len();
    for _ in 0..count {
        d = d.wrapping_add(c);
        let e = d >> 2 & 3;
        for p in 0..=last {
            let left = msg[(p + 1) % len];
            right = ((right >> 5) ^ (left << 2))
                .wrapping_add((left >> 3 ^ right << 4) ^ (d ^ left))
                .wrapping_add(key[(p & 3) ^ e as usize] ^ right)
                .wrapping_add(msg[p]);
            msg[p] = right;
        }
    }
    splite(msg, false)
}

pub fn param_i(username: &str, password: &str, ip: &str, acid: i32, token: &str) -> String {
    let info = serde_json::json!({
        "username": username,
        "password": password,
        "ip": ip,
        "acid": acid,
        "enc_ver": "srun_bx1",
    })
    .to_string();
    let xen = x_encode(info.as_str(), token);
    String::from("{SRBX1}") + BASE64_ENGINE.encode(xen).as_str()
}
