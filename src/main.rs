use core::panic;
use std::{
    fs,
    io::{self, ErrorKind, Read},
    num::Wrapping,
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use clap::{arg, command, value_parser};
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
    let header_n = (len - len % 4) / 4;
    let n = v.len() as u32; // let n = (len - len % 4) / 4; lua scripts just aren't u64 big anyway
    if header_n != n as isize {
        println!("n from the header: {header_n}, actual buffer n: {n}");
    }
    if n < 2 {
        return;
    }

    rounds = Wrapping(6 + 52 / n);

    sum = rounds * Wrapping(DELTA);

    y = Wrapping(v[0]);
    loop {
        e = (sum >> 2) & Wrapping(3);
        for p in (1..n).rev().map(|i| Wrapping(i)) {
            z = Wrapping(v[p.0 as usize - 1]);
            let pmx_result = mx(y, z, sum, key, p, e).0;
            v[p.0 as usize] = v[p.0 as usize].wrapping_sub(pmx_result);
            y = Wrapping(v[p.0 as usize]);
        }
        z = Wrapping(v[n as usize - 1]);
        let mx_result = mx(y, z, sum, key, Wrapping(0), e).0;
        v[0] = v[0].wrapping_sub(mx_result);
        y = Wrapping(v[0]);
        sum -= DELTA;

        rounds -= 1;
        if rounds.0 == 0 {
            break;
        }
    }
}

#[derive(thiserror::Error, Debug)]
enum DecryptErr {
    #[error("buffer is too short")]
    TooShort,
    #[error("not enc3 encrypted")]
    NotENC3,
    #[error("compress size does not match buffer len")]
    WrongCompressedSize,
    #[error("failed to decode zlib")]
    ZlibDecode(#[from] io::Error),
}

fn decrypt_buffer(buffer: &mut [u8]) -> Result<String, DecryptErr> {
    if buffer.len() < 5 {
        // not really < 24? Then I could ignore the unwraps below
        return Err(DecryptErr::TooShort);
    }
    if buffer[0..4] != b"ENC3"[0..4] {
        return Err(DecryptErr::NotENC3);
    }

    let key: u64 = u64::from_le_bytes(buffer[4..12].try_into().unwrap());
    let compressed_size = u32::from_le_bytes(buffer[12..16].try_into().unwrap());
    // may this be somewhat different from full buffer and cause problesm?
    let size = u32::from_le_bytes(buffer[16..20].try_into().unwrap());
    // let adler = u32::from_le_bytes(buffer[20..24].try_into().unwrap());
    if (compressed_size as usize) != buffer.len() - 24 {
        return Err(DecryptErr::WrongCompressedSize);
    }

    bdecrypt(&mut buffer[24..], compressed_size as isize, key);

    let mut z = ZlibDecoder::new(&buffer[24..]);
    let mut s = String::new();
    z.read_to_string(&mut s)?;
    // println!("output buffer\n{s}");
    return Ok(s);
}

// this should go through all the files in the directory, copy unencrypted and write decrypted
// to the new outputDirectory so that we get perfect unencrypted mirror
fn scan_dir(input_path: &Path, output_path: &Path) -> anyhow::Result<()> {
    println!("scanning dir: {input_path:?} and outputting it to: {output_path:?}");
    // CREATE EMPTY OUTPUT DIRECTORY
    if let Err(e) = fs::create_dir(output_path) {
        match e.kind() {
            ErrorKind::AlreadyExists => {
                println!("dir already exists");
            }
            _ => return Err(e.into()),
        }
    }

    // READ THE INPUT DIRECTORY
    let res = fs::read_dir(input_path)?;
    for f in res {
        println!("------------- dir entry result: {:?}", f);
        if let Ok(entry) = f {
            if let Ok(ft) = entry.file_type() {
                if ft.is_file() {
                    println!("found file {:?}, {:?}", entry.path(), entry.metadata());
                    if entry.file_name().to_str().unwrap().ends_with(".lua") {
                        let mut buffer = fs::read(entry.path())?;
                        println!(
                            "decrypting file {:?}, len: {}, buffer contents start: {:?}",
                            entry.path(),
                            buffer.len(),
                            String::from_utf8_lossy(&buffer[..4]),
                        );
                        match decrypt_buffer(&mut buffer) {
                            Ok(decrypted) => {
                                fs::write(output_path.join(entry.file_name()), decrypted)?;
                            }
                            Err(DecryptErr::NotENC3) | Err(DecryptErr::TooShort) => {
                                // fs.copy or something like that
                                println!("{:?} not ENC3 encrypted, copying", entry.path());
                                fs::copy(entry.path(), output_path.join(entry.file_name()))?;
                            }
                            Err(DecryptErr::ZlibDecode(inner)) => {
                                println!(
                                    "!!!!!{:?} got zlib decode error, inner as if that even matters: {inner}",entry.path()
                                )
                            }
                            Err(DecryptErr::WrongCompressedSize) => {
                                println!(
                                    "!!!!!!{:?} skipping due to wrong compression size",
                                    entry.path()
                                );
                            }
                        }
                    }
                } else if ft.is_dir() {
                    println!("found dir {:?}, {:?}", entry.path(), entry.metadata());
                    // recreate such dir in output_path
                    // let pbuf = entry.path();
                    // let fname = pbuf
                    //     .file_name()
                    //     .expect("file in scanned dirs lacks filename, lol");
                    scan_dir(
                        &entry.path(),
                        &output_path.join(
                            entry.path().file_name().ok_or(anyhow::anyhow!(
                                "file in scanned dirs lacks filename, lol"
                            ))?,
                        ),
                    )?;
                    // scan recursively
                    // guess I could do it imperatively but i'd have to keep some queue with tuple or struct
                } else {
                    println!(
                        "!!!!! found neither dir nor file {:?}, {:?}",
                        entry.path(),
                        entry.metadata()?
                    );
                }
            }
        }
    }

    Ok(())
}

fn main() {
    let matches = command!()
        .arg(arg!([input_dir] "input directory containing ENC3 encrypted files").required(true).value_parser(value_parser!(PathBuf)))
        .arg(arg!([output_dir] "output directory which will mirror input_dir but with decrypted files").required(true).value_parser(value_parser!(PathBuf)))
        .get_matches();
    let i = matches.ids();
    println!("ids: {i:?}");
    let input_path = matches
        .get_one::<PathBuf>("input_dir")
        .expect("passed wrong input path");
    let output_path = matches
        .get_one::<PathBuf>("output_dir")
        .expect("passed wrong output path");
    scan_dir(input_path, output_path).unwrap();
}

#[cfg(test)]
mod tests;
// TODO start here next, get correct test data from c++ ver or generate it myself
