// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

const fn ff0(x: u32, y: u32, z: u32) -> u32 {
    x ^ y ^ z
}

const fn ff1(x: u32, y: u32, z: u32) -> u32 {
    (x & y) | (x & z) | (y & z)
}

const fn gg0(x: u32, y: u32, z: u32) -> u32 {
    x ^ y ^ z
}

const fn gg1(x: u32, y: u32, z: u32) -> u32 {
    (x & y) | (!x & z)
}

const fn p0(x: u32) -> u32 {
    x ^ x.rotate_left(9) ^ x.rotate_left(17)
}

const fn p1(x: u32) -> u32 {
    x ^ x.rotate_left(15) ^ x.rotate_left(23)
}

const fn get_u32_be(b: &[u8; 64], i: usize) -> u32 {
    let mut buf = [0u8; 4];
    let mut index = 0;
    while index < 4 {
        buf[index] = b[i + index];
        index += 1;
    }
    u32::from_be_bytes(buf)
}

const fn update_digest(buffer: &[u8; 64], digest: &mut [u32; 8]) {
    let mut w0 = [0u32; 68];
    let mut w1 = [0u32; 64];

    let mut i = 0;
    while i < 16 {
        w0[i] = get_u32_be(&buffer, i * 4);
        i += 1;
    }

    i = 16;
    while i < 68 {
        w0[i] = p1(w0[i - 16] ^ w0[i - 9] ^ w0[i - 3].rotate_left(15))
            ^ w0[i - 13].rotate_left(7)
            ^ w0[i - 6];
        i += 1;
    }

    i = 0;
    while i < 64 {
        w1[i] = w0[i] ^ w0[i + 4];
        i += 1;
    }

    let mut ra = digest[0];
    let mut rb = digest[1];
    let mut rc = digest[2];
    let mut rd = digest[3];
    let mut re = digest[4];
    let mut rf = digest[5];
    let mut rg = digest[6];
    let mut rh = digest[7];
    let mut ss1: u32;
    let mut ss2: u32;
    let mut tt1: u32;
    let mut tt2: u32;

    i = 0;
    while i < 16 {
        ss1 = ra
            .rotate_left(12)
            .wrapping_add(re)
            .wrapping_add(0x79cc_4519u32.rotate_left(i as u32))
            .rotate_left(7);
        ss2 = ss1 ^ ra.rotate_left(12);
        tt1 = ff0(ra, rb, rc)
            .wrapping_add(rd)
            .wrapping_add(ss2)
            .wrapping_add(w1[i]);
        tt2 = gg0(re, rf, rg)
            .wrapping_add(rh)
            .wrapping_add(ss1)
            .wrapping_add(w0[i]);
        rd = rc;
        rc = rb.rotate_left(9);
        rb = ra;
        ra = tt1;
        rh = rg;
        rg = rf.rotate_left(19);
        rf = re;
        re = p0(tt2);

        i += 1;
    }

    i = 16;
    while i < 64 {
        ss1 = ra
            .rotate_left(12)
            .wrapping_add(re)
            .wrapping_add(0x7a87_9d8au32.rotate_left(i as u32))
            .rotate_left(7);
        ss2 = ss1 ^ ra.rotate_left(12);
        tt1 = ff1(ra, rb, rc)
            .wrapping_add(rd)
            .wrapping_add(ss2)
            .wrapping_add(w1[i]);
        tt2 = gg1(re, rf, rg)
            .wrapping_add(rh)
            .wrapping_add(ss1)
            .wrapping_add(w0[i]);
        rd = rc;
        rc = rb.rotate_left(9);
        rb = ra;
        ra = tt1;
        rh = rg;
        rg = rf.rotate_left(19);
        rf = re;
        re = p0(tt2);

        i += 1;
    }

    digest[0] ^= ra;
    digest[1] ^= rb;
    digest[2] ^= rc;
    digest[3] ^= rd;
    digest[4] ^= re;
    digest[5] ^= rf;
    digest[6] ^= rg;
    digest[7] ^= rh;
}

const fn pad(bytes: &[u8], padded: &mut [u8]) -> usize {
    padded[0] = 0x80;
    let mut index = 1usize;

    let block_size = 64;
    while (bytes.len() + index) % block_size != 56 {
        padded[index] = 0x00;
        index += 1;
    }

    let len = (bytes.len() << 3) as u64;
    padded[index] = (len >> 56 & 0xff) as u8;
    padded[index + 1] = (len >> 48 & 0xff) as u8;
    padded[index + 2] = (len >> 40 & 0xff) as u8;
    padded[index + 3] = (len >> 32 & 0xff) as u8;
    padded[index + 4] = (len >> 24 & 0xff) as u8;
    padded[index + 5] = (len >> 16 & 0xff) as u8;
    padded[index + 6] = (len >> 8 & 0xff) as u8;
    padded[index + 7] = (len & 0xff) as u8;

    index + 8
}

pub const fn sm3(bytes: &[u8]) -> [u8; 4] {
    let mut digest: [u32; 8] = [
        0x7380_166f,
        0x4914_b2b9,
        0x1724_42d7,
        0xda8a_0600,
        0xa96f_30bc,
        0x1631_38aa,
        0xe38d_ee4d,
        0xb0fb_0e4e,
    ];
    let mut output = [0u8; 4];
    let mut padded = [0u8; 64];

    let padded_len = pad(bytes, &mut padded);
    let len = bytes.len() + padded_len;
    let mut count = 0usize;
    let mut buffer = [0u8; 64];

    while count * 64 != len {
        let mut i = 0;
        while i < 64 {
            let index = count * 64 + i;
            buffer[i] = if index >= bytes.len() {
                padded[index - bytes.len()]
            } else {
                bytes[count * 64 + i]
            };
            i += 1;
        }
        update_digest(&buffer, &mut digest);
        count += 1;
    }

    output[0] = (digest[0] >> 24) as u8;
    output[1] = (digest[0] >> 16) as u8;
    output[2] = (digest[0] >> 8) as u8;
    output[3] = digest[0] as u8;

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selector_0() {
        let signature = "add(uint32)";
        let hash = sm3(signature.as_bytes());
        assert_eq!(hash[0], 0x27);
        assert_eq!(hash[1], 0xd8);
        assert_eq!(hash[2], 0x25);
        assert_eq!(hash[3], 0x07);
    }

    #[test]
    fn selector_1() {
        let signature = "get()";
        let hash = sm3(signature.as_bytes());
        assert_eq!(hash[0], 0x29);
        assert_eq!(hash[1], 0x9f);
        assert_eq!(hash[2], 0x7f);
        assert_eq!(hash[3], 0x9d);
    }
}
