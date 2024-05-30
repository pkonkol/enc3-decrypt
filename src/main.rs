
const DELTA: i32 = 0x9e3779b9;

// now what was that stupid macro?
fn mx(y: u32, z: u32, sum: u32, key: &[u32], p: u32, e: u32) -> u32 {
    ((z>>5^y<<2) + (y>>3^z<<4)) ^ ((sum^y) + (key[((p&3)^e) as usize] ^ z))
}

fn bdecrypt(buffer &[u8], len i64, k u64) {
    const u32 key: [u32] = [k >> 32, k & 0xFFFFFFFF, 0xDEADDEAD, 0xB00BEEEF]
    let y, z, sum;
    let v: [u32] = [1,2,3];
    let n = (len - len%4) / 4;
    if n < 2 {
        return;
    }
    rounds = 6 + 52/n;
    sum = round * DELTA;
    y = v[0]

}

fn decryptBuffer() {

}

// this should go through all the files in the directory, copy unencrypted and write decrypted 
// to the new outputDirectory so that we get perffect unencrypted mirror
fn createFS() {

}

fn main() {
    println!("Hello, world!");
}
