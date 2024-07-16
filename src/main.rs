use std::{fs, io::Read, num::Wrapping, path::Path};

use flate2::read::ZlibDecoder;

const DELTA: u32 = 0x9e3779b9;

// now what was that stupid macro?
fn mx(
    y: Wrapping<u32>,
    z: Wrapping<u32>,
    sum: Wrapping<u32>,
    key: &[Wrapping<u32>],
    p: Wrapping<u32>,
    e: Wrapping<u32>,
) -> Wrapping<u32> {
    ((z >> 5 ^ y << 2) + (y >> 3 ^ z << 4))
        ^ ((sum ^ y) + (key[((p & Wrapping(3)) ^ e).0 as usize] ^ z))
}

fn bdecrypt(buffer: &mut [u8], len: isize, k: u64) {
    let key: &[Wrapping<u32>] = &[
        Wrapping((k >> 32) as u32),
        Wrapping((k & 0xFFFFFFFF) as u32),
        Wrapping(0xDEADDEAD),
        Wrapping(0xB00BEEEF),
    ];
    let (mut y, mut z, mut sum): (Wrapping<u32>, Wrapping<u32>, Wrapping<u32>);
    let (mut p, mut rounds, mut e): (Wrapping<u32>, Wrapping<u32>, Wrapping<u32>);

    // CONVERT &mut [u8] TO &mut [u32]
    // let v: &mut [u32] = &mut [1, 2, 3]; //  uint32_t *v = (uint32_t *) buffer;
    // yeah, we must treat the buffer as &mut [u32] and if there are some dangling bytes just skip it
    // let v = buffer;
    let (prefix, v, suffix): (&mut [u8], &mut [u32], &mut [u8]);
    unsafe {
        (prefix, v, suffix) = buffer.align_to_mut::<u32>();
    }
    if prefix.len() != 0 {
        panic!("align_to_mut is not exactly what we need then");
    }
    if suffix.len() != 0 {
        println!("oh cool, suffix={:?}", suffix);
    };

    // VERIFY PRECONDITIONS
    let n = v.len() as u32; // let n = (len - len % 4) / 4; lua scripts just aren't u64 big anyway
    if n < 2 {
        return;
    }
    // WORKING ALGORITHM
    rounds = Wrapping(6 + 52 / n);

    sum = rounds * Wrapping(DELTA);
    // (sum, _) = rounds.overflowing_mul(DELTA);
    y = Wrapping(v[0]);
    loop {
        e = (sum >> 2) & Wrapping(3);
        for p in (1..n).rev().map(|i| Wrapping(i)) {
            z = Wrapping(v[n as usize - 1]);
            v[0] = v[0].wrapping_sub(mx(y, z, sum, key, p, e).0);
            y = Wrapping(v[0]);
        }
        z = Wrapping(v[n as usize - 1]);
        v[0] = v[0].wrapping_sub(mx(y, z, sum, key, Wrapping(0), e).0);
        y = Wrapping(v[0]);
        sum -= DELTA;

        if rounds.0 == 0 {
            break;
        }
        rounds -= 1;
    }
}

// what should this actually return?
fn decrypt_buffer(buffer: &mut [u8]) -> Option<String> {
    println!(
        "buffer.len() {}, first 24bytes: {:?}",
        buffer.len(),
        String::from_utf8_lossy(&buffer[..24]),
    );
    if buffer.len() < 5 {
        return None; // why was true here and false in other early returns?
                     // does it mean changed buffer or something?
    }
    if buffer[0..4] != b"ENC3"[0..4] {
        // will this comparison work?
        return None;
    }
    // x86 is little endian so le, arm too
    let key: u64 = u64::from_le_bytes(buffer[4..12].try_into().unwrap());
    let compressed_size = u32::from_le_bytes(buffer[12..16].try_into().unwrap());
    let size = u32::from_le_bytes(buffer[16..20].try_into().unwrap());
    let adler = u32::from_le_bytes(buffer[20..24].try_into().unwrap());
    // it was < but any discrepancy seems to be bad
    println!("key: {key}, compressed_size: {compressed_size}, size: {size}, adler: {adler}\n");
    if (compressed_size as usize) != buffer.len() - 24 {
        return None;
    }
    bdecrypt(buffer, compressed_size as isize, key);

    let mut z = ZlibDecoder::new(&buffer[24..]);
    let mut s = String::new();
    z.read_to_string(&mut s).unwrap();
    println!("output buffer\n{s}");
    return Some(s);
}

// this should go through all the files in the directory, copy unencrypted and write decrypted
// to the new outputDirectory so that we get perfect unencrypted mirror
fn scan_dir(input_path: &Path, output_path: &Path) -> anyhow::Result<()> {
    // CREATE EMPTY OUTPUT DIRECTORY
    fs::create_dir(output_path)?;

    // READ THE INPUT DIRECTORY
    let res = fs::read_dir(input_path)?;
    for f in res {
        if let Ok(entry) = f {
            if let Ok(ft) = entry.file_type() {
                if ft.is_file() {
                    println!("found file {:?}, {:?}", entry.path(), entry.metadata());
                    if entry.file_name().to_str().unwrap().ends_with(".lua") {
                        let mut buffer = fs::read(entry.path())?;
                        if let Some(decrypted) = decrypt_buffer(&mut buffer) {
                            fs::write(output_path.join(entry.file_name()), decrypted)?;
                        }
                        // should I skip unencrypted or just copy them?
                    }
                } else if ft.is_dir() {
                    println!("found dir {:?}, {:?}", entry.path(), entry.metadata());
                    // recreate such dir in output_path
                } else {
                    println!("found idk {:?}, {:?}", entry.path(), entry.metadata()?);
                    continue;
                }
            }
        }
    }

    Ok(())
}

fn main() {
    let input_path = Path::new("./test");
    let output_path = Path::new("./output");
    scan_dir(input_path, output_path);
}

#[cfg(test)]
mod tests;
// TODO start here next, get correct test data from c++ ver or generate it myself
