use std::io::Read;

use flate2::read::ZlibDecoder;

const DELTA: u32 = 0x9e3779b9;

// now what was that stupid macro?
fn mx(y: u32, z: u32, sum: u32, key: &[u32], p: usize, e: usize) -> u32 {
    ((z >> 5 ^ y << 2) + (y >> 3 ^ z << 4)) ^ ((sum ^ y) + (key[((p & 3) ^ e) as usize] ^ z))
}

fn bdecrypt(buffer: &mut [u8], len: isize, k: u64) {
    let key: &[u32] = &[
        (k >> 32) as u32,
        (k & 0xFFFFFFFF) as u32,
        0xDEADDEAD,
        0xB00BEEEF,
    ];
    let (mut y, mut z, mut sum): (u32, u32, u32);
    // v should be basically the pointer to the buffer
    // let v: &mut [u32] = &mut [1, 2, 3]; //  uint32_t *v = (uint32_t *) buffer;
    // what does the type transform from uint8_t to uint32_t and how to do it in rust
    let v = buffer;

    let (mut p, rounds, mut e): (usize, usize, usize);
    let n = (len - len % 4) / 4;
    if n < 2 {
        return;
    }
    let mut rounds = (6 + 52 / n) as u32;

    sum = rounds * DELTA;
    // TODO read and access more than u8 from the buffer
    y = v[0..4];
    loop {
        e = ((sum >> 2) & 3) as usize;
        p = n as usize - 1;
        while p > 0 {
            z = v[n as usize - 1];
            // y = v[0] -= mx(y, z, sum, key, p, e);
            v[0] -= mx(y, z, sum, key, p, e);
            y = v[0];
            p -= 1;
        }
        z = v[n as usize - 1];
        // this should be how c++ executes it in order
        // y = (v[0] -= mx(y, z, sum, key, p, e));
        v[0] -= mx(y, z, sum, key, p, e);
        y = v[0];
        sum -= DELTA;
        if rounds == 0 {
            break;
        }
        rounds -= 1;
    }
}

fn decryptBuffer(buffer: &[u8]) -> bool {
    if buffer.len() < 5 {
        return true;
    }
    if buffer[0..4] != b"ENC3"[0..4] {
        // will this comparison work?
        return false;
    }
    // should it be be or le bytes?
    let key: u64 = u64::from_be_bytes(buffer[4..12].try_into().unwrap());
    let compressed_size = u32::from_be_bytes(buffer[12..16].try_into().unwrap());
    let size = u32::from_be_bytes(buffer[16..20].try_into().unwrap());
    let adler = u32::from_be_bytes(buffer[20..24].try_into().unwrap());
    if (compressed_size as usize) < buffer.len() - 24 {
        return false;
    }
    bdecrypt(buffer, compressed_size as isize, key);

    let mut z = ZlibDecoder::new(&buffer[24..]);
    let mut s = String::new();
    z.read_to_string(&mut s).unwrap();
    println!(s);
    // let mut new_buffer = String::new();
    // new_buffer.reserve(size as usize);
    return false;
}

// this should go through all the files in the directory, copy unencrypted and write decrypted
// to the new outputDirectory so that we get perffect unencrypted mirror
fn createFS() {}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests;
