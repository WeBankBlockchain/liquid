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

const BITS: usize = 256;
const RATE: usize = 200 - BITS / 4;
const DELIM: u8 = 0x01;
const ROUNDS: usize = 24;

const RC: [u64; ROUNDS] = [
    1u64,
    0x8082u64,
    0x800000000000808au64,
    0x8000000080008000u64,
    0x808bu64,
    0x80000001u64,
    0x8000000080008081u64,
    0x8000000000008009u64,
    0x8au64,
    0x88u64,
    0x80008009u64,
    0x8000000au64,
    0x8000808bu64,
    0x800000000000008bu64,
    0x8000000000008089u64,
    0x8000000000008003u64,
    0x8000000000008002u64,
    0x8000000000000080u64,
    0x800au64,
    0x800000008000000au64,
    0x8000000080008081u64,
    0x8000000000008080u64,
    0x80000001u64,
    0x8000000080008008u64,
];

const WORD: usize = 25;
const BYTE: usize = 8 * WORD;

const RHO: [u32; 24] = [
    1, 3, 6, 10, 15, 21, 28, 36, 45, 55, 2, 14, 27, 41, 56, 8, 25, 43, 62, 18, 39, 61,
    20, 44,
];

const PI: [usize; 24] = [
    10, 7, 11, 17, 18, 3, 5, 16, 8, 21, 24, 4, 15, 23, 19, 13, 12, 2, 20, 14, 22, 9, 6, 1,
];

const fn keccak(a: &mut [u64; WORD]) {
    let mut i = 0;
    while i < ROUNDS {
        let mut array: [u64; 5] = [0; 5];

        // Theta
        let mut x = 0;
        while x < 5 {
            let mut y_count = 0;
            while y_count < 5 {
                let y = y_count * 5;
                array[x] ^= a[x + y];
                y_count += 1;
            }
            x += 1;
        }

        let mut x = 0;
        while x < 5 {
            let mut y_count = 0;
            while y_count < 5 {
                let y = y_count * 5;
                a[y + x] ^= array[(x + 4) % 5] ^ array[(x + 1) % 5].rotate_left(1);
                y_count += 1;
            }
            x += 1;
        }

        // Rho and pi
        let mut last = a[1];
        let mut x = 0;
        while x < 24 {
            array[0] = a[PI[x]];
            a[PI[x]] = last.rotate_left(RHO[x]);
            last = array[0];
            x += 1;
        }

        // Chi
        let mut y_step = 0;
        while y_step < 5 {
            let y = y_step * 5;

            let mut x = 0;
            while x < 5 {
                array[x] = a[y + x];
                x += 1;
            }

            let mut x = 0;
            while x < 5 {
                a[y + x] = array[x] ^ ((!array[(x + 1) % 5]) & (array[(x + 2) % 5]));
                x += 1;
            }
            y_step += 1;
        }

        // Iota
        a[0] ^= RC[i];
        i += 1;
    }
}

const fn xorin(a: &mut [u8], b: &[u8], len: usize) {
    let mut i = 0;
    while i < len {
        a[i] ^= b[i];
        i += 1;
    }
}

const fn fold(a: &mut [u64; WORD], input: &[u8]) -> usize {
    let mut ip = 0;
    let mut l = input.len();
    while l >= RATE {
        let temp_input = convert_bytes_to_bytes(input, ip, RATE);
        let mut temp_a = convert_words_to_bytes(a);
        xorin(&mut temp_a, &temp_input, RATE);
        let temp = convert_bytes_to_words(&temp_a);

        let mut i = 0;
        while i < WORD {
            a[i] = temp[i];
            i += 1;
        }

        keccak(a);
        ip += RATE;
        l -= RATE;
    }

    let temp_input = convert_bytes_to_bytes(input, ip, l);
    let mut temp_a = convert_words_to_bytes(a);
    xorin(&mut temp_a, &temp_input, l);
    let temp = convert_bytes_to_words(&temp_a);

    let mut i = 0;
    while i < WORD {
        a[i] = temp[i];
        i += 1;
    }
    l
}

const fn convert_bytes_to_bytes(b: &[u8], start: usize, len: usize) -> [u8; RATE] {
    let mut ret = [0u8; RATE];
    let mut i = 0;
    while i < RATE && i < len {
        ret[i] = b[start + i];
        i += 1;
    }
    ret
}

const fn convert_words_to_bytes(w: &[u64; WORD]) -> [u8; BYTE] {
    let mut ret = [0u8; BYTE];
    let mut i = 0;
    while i < WORD {
        let bytes = w[i].to_le_bytes();
        let mut j = 0;
        while j < 8 {
            ret[i * 8 + j] = bytes[j];
            j += 1;
        }
        i += 1;
    }
    ret
}

const fn convert_bytes_to_words(b: &[u8; BYTE]) -> [u64; WORD] {
    let mut ret = [0u64; WORD];
    let mut i = 0;
    while i < WORD {
        let mut bytes = [0u8; 8];
        let mut j = 0;
        while j < 8 {
            bytes[j] = b[i * 8 + j];
            j += 1;
        }
        ret[i] = u64::from_le_bytes(bytes);
        i += 1;
    }
    ret
}

pub const fn keccak256(input: &[u8]) -> [u8; 32] {
    let mut output = [0u8; 32];
    let mut buffer = [0u64; WORD];

    let offset = fold(&mut buffer, input);
    let mut buffer_temp = convert_words_to_bytes(&buffer);

    buffer_temp[offset] ^= DELIM;
    buffer_temp[RATE - 1] ^= 0x80;

    buffer = convert_bytes_to_words(&buffer_temp);
    keccak(&mut buffer);
    buffer_temp = convert_words_to_bytes(&buffer);

    let mut i = 0;
    while i < 32 {
        output[i] = buffer_temp[i];
        i += 1;
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selector_0() {
        let signature = "add(uint32,string)";
        let hash = keccak256(signature.as_bytes());
        assert_eq!(hash[0], 0x66);
        assert_eq!(hash[1], 0x89);
        assert_eq!(hash[2], 0xaa);
        assert_eq!(hash[3], 0x15);
    }

    #[test]
    fn selector_1() {
        let signature = "get()";
        let hash = keccak256(signature.as_bytes());
        assert_eq!(hash[0], 0x6d);
        assert_eq!(hash[1], 0x4c);
        assert_eq!(hash[2], 0xe6);
        assert_eq!(hash[3], 0x3c);
    }
}
