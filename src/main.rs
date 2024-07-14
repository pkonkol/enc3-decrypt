use std::{io::Read, path::Path};

use flate2::read::ZlibDecoder;

const DELTA: u32 = 0x9e3779b9;

// now what was that stupid macro?
fn mx(y: u32, z: u32, sum: u32, key: &[u32], p: u32, e: u32) -> u32 {
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
    let (mut p, rounds, mut e): (u32, u32, u32);

    // CONVERT &mut [u8] TO &mut [u32]
    // let v: &mut [u32] = &mut [1, 2, 3]; //  uint32_t *v = (uint32_t *) buffer;
    // yeah, we must treat the buffer as &mut [u32] and if there are some dangling bytes just skip it
    // let v = buffer;
    let (prefix, v, suffix): (&mut [u8], &mut [u32], &mut [u8]);
    unsafe {
        let (prefix, v, suffix) = buffer.align_to_mut::<u32>();
    }
    if prefix.len() != 0 {
        panic!("align_to_mut is not exactly what we need then");
    }

    // VERIFY PRECONDITIONS
    let n = v.len() as u32; // let n = (len - len % 4) / 4; lua scripts just aren't u64 big anyway
    if n < 2 {
        return;
    }
    // WORKING ALGORITHM
    let mut rounds = 6 + 52 / n;

    sum = rounds * DELTA;
    y = v[0];
    loop {
        e = (sum >> 2) & 3;
        for p in (1..n).rev() {
            z = v[n as usize - 1];
            v[0] -= mx(y, z, sum, key, p, e);
            y = v[0];
        }
        z = v[n as usize - 1];
        v[0] -= mx(y, z, sum, key, 0, e);
        y = v[0];
        sum -= DELTA;

        if rounds == 0 {
            break;
        }
        rounds -= 1;
    }
}

fn decryptBuffer(buffer: &mut [u8]) -> bool {
    if buffer.len() < 5 {
        return true;
    }
    if buffer[0..4] != b"ENC3"[0..4] {
        // will this comparison work?
        return false;
    }
    // x86 is little endian so le, arm too
    let key: u64 = u64::from_le_bytes(buffer[4..12].try_into().unwrap());
    let compressed_size = u32::from_le_bytes(buffer[12..16].try_into().unwrap());
    let size = u32::from_le_bytes(buffer[16..20].try_into().unwrap());
    let adler = u32::from_le_bytes(buffer[20..24].try_into().unwrap());
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
// to the new outputDirectory so that we get perfect unencrypted mirror
fn createFS(input_path: &Path) {
    // TODO start here next time
}

fn main() {
    let input_path = Path::new("./test");
    createFS(input_path);
}

#[cfg(test)]
mod tests;
