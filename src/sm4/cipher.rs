// Copyright 2018 Cryptape Technology LLC.
//
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

use crate::sm4::error::{Sm4Error, Sm4Result};

static SBOX: [u8; 256] = [
    0xd6, 0x90, 0xe9, 0xfe, 0xcc, 0xe1, 0x3d, 0xb7, 0x16, 0xb6, 0x14, 0xc2, 0x28, 0xfb, 0x2c, 0x05,
    0x2b, 0x67, 0x9a, 0x76, 0x2a, 0xbe, 0x04, 0xc3, 0xaa, 0x44, 0x13, 0x26, 0x49, 0x86, 0x06, 0x99,
    0x9c, 0x42, 0x50, 0xf4, 0x91, 0xef, 0x98, 0x7a, 0x33, 0x54, 0x0b, 0x43, 0xed, 0xcf, 0xac, 0x62,
    0xe4, 0xb3, 0x1c, 0xa9, 0xc9, 0x08, 0xe8, 0x95, 0x80, 0xdf, 0x94, 0xfa, 0x75, 0x8f, 0x3f, 0xa6,
    0x47, 0x07, 0xa7, 0xfc, 0xf3, 0x73, 0x17, 0xba, 0x83, 0x59, 0x3c, 0x19, 0xe6, 0x85, 0x4f, 0xa8,
    0x68, 0x6b, 0x81, 0xb2, 0x71, 0x64, 0xda, 0x8b, 0xf8, 0xeb, 0x0f, 0x4b, 0x70, 0x56, 0x9d, 0x35,
    0x1e, 0x24, 0x0e, 0x5e, 0x63, 0x58, 0xd1, 0xa2, 0x25, 0x22, 0x7c, 0x3b, 0x01, 0x21, 0x78, 0x87,
    0xd4, 0x00, 0x46, 0x57, 0x9f, 0xd3, 0x27, 0x52, 0x4c, 0x36, 0x02, 0xe7, 0xa0, 0xc4, 0xc8, 0x9e,
    0xea, 0xbf, 0x8a, 0xd2, 0x40, 0xc7, 0x38, 0xb5, 0xa3, 0xf7, 0xf2, 0xce, 0xf9, 0x61, 0x15, 0xa1,
    0xe0, 0xae, 0x5d, 0xa4, 0x9b, 0x34, 0x1a, 0x55, 0xad, 0x93, 0x32, 0x30, 0xf5, 0x8c, 0xb1, 0xe3,
    0x1d, 0xf6, 0xe2, 0x2e, 0x82, 0x66, 0xca, 0x60, 0xc0, 0x29, 0x23, 0xab, 0x0d, 0x53, 0x4e, 0x6f,
    0xd5, 0xdb, 0x37, 0x45, 0xde, 0xfd, 0x8e, 0x2f, 0x03, 0xff, 0x6a, 0x72, 0x6d, 0x6c, 0x5b, 0x51,
    0x8d, 0x1b, 0xaf, 0x92, 0xbb, 0xdd, 0xbc, 0x7f, 0x11, 0xd9, 0x5c, 0x41, 0x1f, 0x10, 0x5a, 0xd8,
    0x0a, 0xc1, 0x31, 0x88, 0xa5, 0xcd, 0x7b, 0xbd, 0x2d, 0x74, 0xd0, 0x12, 0xb8, 0xe5, 0xb4, 0xb0,
    0x89, 0x69, 0x97, 0x4a, 0x0c, 0x96, 0x77, 0x7e, 0x65, 0xb9, 0xf1, 0x09, 0xc5, 0x6e, 0xc6, 0x84,
    0x18, 0xf0, 0x7d, 0xec, 0x3a, 0xdc, 0x4d, 0x20, 0x79, 0xee, 0x5f, 0x3e, 0xd7, 0xcb, 0x39, 0x48,
];

fn split(input: u32) -> [u8; 4] {
    let i4: u8 = input as u8;
    let i3: u8 = (input >> 8) as u8;
    let i2: u8 = (input >> 16) as u8;
    let i1: u8 = (input >> 24) as u8;

    [i1, i2, i3, i4]
}

fn combine(input: &[u8]) -> u32 {
    let out: u32 = u32::from(input[3]);
    let out = out | (u32::from(input[2]) << 8);
    let out = out | (u32::from(input[1]) << 16);
    out | (u32::from(input[0]) << 24)
}

fn split_block(input: &[u8]) -> Sm4Result<[u32; 4]> {
    if input.len() != 16 {
        return Err(Sm4Error::ErrorBlockSize);
    }
    let mut out: [u32; 4] = [0; 4];
    for (i, v) in out.iter_mut().enumerate().take(4) {
        let start = 4 * i;
        let end = 4 * i + 4;
        *v = combine(&input[start..end])
    }
    Ok(out)
}

fn combine_block(input: &[u32]) -> Sm4Result<[u8; 16]> {
    let mut out: [u8; 16] = [0; 16];
    for i in 0..4 {
        let outi = split(input[i]);
        for j in 0..4 {
            out[i * 4 + j] = outi[j];
        }
    }
    Ok(out)
}

#[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
#[target_feature(enable = "sse")]
#[target_feature(enable = "sse2")]
#[target_feature(enable = "sse3")]
#[target_feature(enable = "aes")]
unsafe fn sm4_crypt_affine_ni(key: &Vec<u32>, sin: &[u8; 64], out: &mut [u8; 64], enc: i32) {
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::*;

    let c0f: __m128i =
        core::mem::transmute_copy(&[0x0F0F0F0F0F0F0F0F as u64, 0x0F0F0F0F0F0F0F0F as u64]);
    let flp: __m128i =
        core::mem::transmute_copy(&[0x0405060700010203 as u64, 0x0C0D0E0F08090A0B as u64]);
    let shr: __m128i =
        core::mem::transmute_copy(&[0x0B0E0104070A0D00 as u64, 0x0306090C0F020508 as u64]);
    let m1l: __m128i =
        core::mem::transmute_copy(&[0x9197E2E474720701 as u64, 0xC7C1B4B222245157 as u64]);
    let m1h: __m128i =
        core::mem::transmute_copy(&[0xE240AB09EB49A200 as u64, 0xF052B91BF95BB012 as u64]);
    let m2l: __m128i =
        core::mem::transmute_copy(&[0x5B67F2CEA19D0834 as u64, 0xEDD14478172BBE82 as u64]);
    let m2h: __m128i =
        core::mem::transmute_copy(&[0xAE7201DD73AFDC00 as u64, 0x11CDBE62CC1063BF as u64]);
    let r08: __m128i =
        core::mem::transmute_copy(&[0x0605040702010003 as u64, 0x0E0D0C0F0A09080B as u64]);
    let r16: __m128i =
        core::mem::transmute_copy(&[0x0504070601000302 as u64, 0x0D0C0F0E09080B0A as u64]);
    let r24: __m128i =
        core::mem::transmute_copy(&[0x0407060500030201 as u64, 0x0C0F0E0D080B0A09 as u64]);

    let mut t0: __m128i;
    let mut t1: __m128i;
    let mut t2: __m128i;
    let mut t3: __m128i;

    let p32: [i32; 16] = core::mem::transmute_copy(sin);

    t0 = _mm_set_epi32(p32[12], p32[8], p32[4], p32[0]);
    t0 = _mm_shuffle_epi8(t0, flp);

    t1 = _mm_set_epi32(p32[13], p32[9], p32[5], p32[1]);
    t1 = _mm_shuffle_epi8(t1, flp);

    t2 = _mm_set_epi32(p32[14], p32[10], p32[6], p32[2]);
    t2 = _mm_shuffle_epi8(t2, flp);

    t3 = _mm_set_epi32(p32[15], p32[11], p32[7], p32[3]);
    t3 = _mm_shuffle_epi8(t3, flp);

    let mut x: __m128i;
    let mut y: __m128i;
    let mut t4: __m128i;

    for i in 0..8 {
        let k = if enc == 0 { i * 4 } else { 31 - i * 4 };
        let k1 = key[k];
        t4 = core::mem::transmute_copy(&[k1, k1, k1, k1]);
        x = _mm_xor_si128(_mm_xor_si128(_mm_xor_si128(t1, t2), t3), t4);

        y = _mm_and_si128(x, c0f);
        y = _mm_shuffle_epi8(m1l, y);
        x = _mm_srli_epi64(x, 4);
        x = _mm_and_si128(x, c0f);
        x = _mm_xor_si128(_mm_shuffle_epi8(m1h, x), y);
        x = _mm_shuffle_epi8(x, shr);
        x = _mm_aesenclast_si128(x, c0f);
        y = _mm_andnot_si128(x, c0f);
        y = _mm_shuffle_epi8(m2l, y);
        x = _mm_srli_epi64(x, 4);
        x = _mm_and_si128(x, c0f);
        x = _mm_xor_si128(_mm_shuffle_epi8(m2h, x), y);
        y = _mm_xor_si128(
            _mm_xor_si128(x, _mm_shuffle_epi8(x, r08)),
            _mm_shuffle_epi8(x, r16),
        );
        y = _mm_xor_si128(_mm_slli_epi32(y, 2), _mm_srli_epi32(y, 30));
        x = _mm_xor_si128(_mm_xor_si128(x, y), _mm_shuffle_epi8(x, r24));

        t0 = _mm_xor_si128(t0, x);

        let k = if enc == 0 { i * 4 + 1 } else { 30 - i * 4 };
        let k2 = key[k];
        t4 = core::mem::transmute_copy(&[k2, k2, k2, k2]);
        x = _mm_xor_si128(_mm_xor_si128(_mm_xor_si128(t0, t2), t3), t4);

        y = _mm_and_si128(x, c0f);
        y = _mm_shuffle_epi8(m1l, y);
        x = _mm_srli_epi64(x, 4);
        x = _mm_and_si128(x, c0f);
        x = _mm_xor_si128(_mm_shuffle_epi8(m1h, x), y);
        x = _mm_shuffle_epi8(x, shr);
        x = _mm_aesenclast_si128(x, c0f);
        y = _mm_andnot_si128(x, c0f);
        y = _mm_shuffle_epi8(m2l, y);
        x = _mm_srli_epi64(x, 4);
        x = _mm_and_si128(x, c0f);
        x = _mm_xor_si128(_mm_shuffle_epi8(m2h, x), y);
        y = _mm_xor_si128(
            _mm_xor_si128(x, _mm_shuffle_epi8(x, r08)),
            _mm_shuffle_epi8(x, r16),
        );
        y = _mm_xor_si128(_mm_slli_epi32(y, 2), _mm_srli_epi32(y, 30));
        x = _mm_xor_si128(_mm_xor_si128(x, y), _mm_shuffle_epi8(x, r24));

        t1 = _mm_xor_si128(t1, x);

        let k = if enc == 0 { i * 4 + 2 } else { 29 - i * 4 };
        let k3 = key[k];
        t4 = core::mem::transmute_copy(&[k3, k3, k3, k3]);
        x = _mm_xor_si128(_mm_xor_si128(_mm_xor_si128(t0, t1), t3), t4);

        y = _mm_and_si128(x, c0f);
        y = _mm_shuffle_epi8(m1l, y);
        x = _mm_srli_epi64(x, 4);
        x = _mm_and_si128(x, c0f);
        x = _mm_xor_si128(_mm_shuffle_epi8(m1h, x), y);
        x = _mm_shuffle_epi8(x, shr);
        x = _mm_aesenclast_si128(x, c0f);
        y = _mm_andnot_si128(x, c0f);
        y = _mm_shuffle_epi8(m2l, y);
        x = _mm_srli_epi64(x, 4);
        x = _mm_and_si128(x, c0f);
        x = _mm_xor_si128(_mm_shuffle_epi8(m2h, x), y);
        y = _mm_xor_si128(
            _mm_xor_si128(x, _mm_shuffle_epi8(x, r08)),
            _mm_shuffle_epi8(x, r16),
        );
        y = _mm_xor_si128(_mm_slli_epi32(y, 2), _mm_srli_epi32(y, 30));
        x = _mm_xor_si128(_mm_xor_si128(x, y), _mm_shuffle_epi8(x, r24));

        t2 = _mm_xor_si128(t2, x);

        let k = if enc == 0 { i * 4 + 3 } else { 28 - i * 4 };
        let k4 = key[k];
        t4 = core::mem::transmute_copy(&[k4, k4, k4, k4]);
        x = _mm_xor_si128(_mm_xor_si128(_mm_xor_si128(t0, t1), t2), t4);

        y = _mm_and_si128(x, c0f);
        y = _mm_shuffle_epi8(m1l, y);
        x = _mm_srli_epi64(x, 4);
        x = _mm_and_si128(x, c0f);
        x = _mm_xor_si128(_mm_shuffle_epi8(m1h, x), y);
        x = _mm_shuffle_epi8(x, shr);
        x = _mm_aesenclast_si128(x, c0f);
        y = _mm_andnot_si128(x, c0f);
        y = _mm_shuffle_epi8(m2l, y);
        x = _mm_srli_epi64(x, 4);
        x = _mm_and_si128(x, c0f);
        x = _mm_xor_si128(_mm_shuffle_epi8(m2h, x), y);
        y = _mm_xor_si128(
            _mm_xor_si128(x, _mm_shuffle_epi8(x, r08)),
            _mm_shuffle_epi8(x, r16),
        );
        y = _mm_xor_si128(_mm_slli_epi32(y, 2), _mm_srli_epi32(y, 30));
        x = _mm_xor_si128(_mm_xor_si128(x, y), _mm_shuffle_epi8(x, r24));

        t3 = _mm_xor_si128(t3, x);
    }

    let mut res: [u32; 16] = [0; 16];
    let vr: [u32; 4];

    let mut v: __m128i = _mm_set_epi64x(0x0, 0x0);
    let v_prt: *mut __m128i = &mut v;
    _mm_store_si128(v_prt, _mm_shuffle_epi8(t3, flp));
    vr = core::mem::transmute_copy(&v);
    res[0] = vr[0];
    res[4] = vr[1];
    res[8] = vr[2];
    res[12] = vr[3];

    let mut v: __m128i = _mm_set_epi64x(0x0, 0x0);
    let v_prt: *mut __m128i = &mut v;
    _mm_store_si128(v_prt, _mm_shuffle_epi8(t2, flp));

    let vr: [u32; 4];
    vr = core::mem::transmute_copy(&v);
    res[1] = vr[0];
    res[5] = vr[1];
    res[9] = vr[2];
    res[13] = vr[3];

    let mut v: __m128i = _mm_set_epi64x(0x0, 0x0);
    let v_prt: *mut __m128i = &mut v;
    _mm_store_si128(v_prt, _mm_shuffle_epi8(t1, flp));

    let vr: [u32; 4];
    vr = core::mem::transmute_copy(&v);
    res[2] = vr[0];
    res[6] = vr[1];
    res[10] = vr[2];
    res[14] = vr[3];

    let mut v: __m128i = _mm_set_epi64x(0x0, 0x0);
    let v_prt: *mut __m128i = &mut v;
    _mm_store_si128(v_prt, _mm_shuffle_epi8(t0, flp));

    let vr: [u32; 4];
    vr = core::mem::transmute_copy(&v);
    res[3] = vr[0];
    res[7] = vr[1];
    res[11] = vr[2];
    res[15] = vr[3];

    *out = core::mem::transmute_copy(&res);
}

fn tau_trans(input: u32) -> u32 {
    let input = split(input);
    let mut out: [u8; 4] = [0; 4];
    for i in 0..4 {
        out[i] = SBOX[input[i] as usize];
    }
    combine(&out)
}

fn l_rotate(x: u32, i: u32) -> u32 {
    (x << (i % 32)) | (x >> (32 - (i % 32)))
}

fn l_trans(input: u32) -> u32 {
    let b = input;
    b ^ l_rotate(b, 2) ^ l_rotate(b, 10) ^ l_rotate(b, 18) ^ l_rotate(b, 24)
}

fn t_trans(input: u32) -> u32 {
    l_trans(tau_trans(input))
}

pub struct Sm4Cipher {
    // round key
    rk: Vec<u32>,
}

static FK: [u32; 4] = [0xa3b1bac6, 0x56aa3350, 0x677d9197, 0xb27022dc];

static CK: [u32; 32] = [
    0x00070E15, 0x1C232A31, 0x383F464D, 0x545B6269, 0x70777E85, 0x8C939AA1, 0xA8AFB6BD, 0xC4CBD2D9,
    0xE0E7EEF5, 0xFC030A11, 0x181F262D, 0x343B4249, 0x50575E65, 0x6C737A81, 0x888F969D, 0xA4ABB2B9,
    0xC0C7CED5, 0xDCE3EAF1, 0xF8FF060D, 0x141B2229, 0x30373E45, 0x4C535A61, 0x686F767D, 0x848B9299,
    0xA0A7AEB5, 0xBCC3CAD1, 0xD8DFE6ED, 0xF4FB0209, 0x10171E25, 0x2C333A41, 0x484F565D, 0x646B7279,
];

fn sm4_key_sub(input: u32) -> u32 {
    let t = sm4_t_non_lin_sub(input);

    t ^ rotl(t, 13) ^ rotl(t, 23)
}

fn sm4_t_non_lin_sub(x: u32) -> u32 {
    let mut t: u32 = 0;

    t |= ((SBOX[((x >> 24) as u8) as usize]) as u32) << 24;
    t |= ((SBOX[((x >> 16) as u8) as usize]) as u32) << 16;
    t |= ((SBOX[((x >> 8) as u8) as usize]) as u32) << 8;
    t |= (SBOX[(x as u8) as usize]) as u32;

    t
}

fn rotl(a: u32, n: u32) -> u32 {
    (a << n) | (a >> (32 - n))
}

impl Sm4Cipher {
    pub fn new(key: &[u8]) -> Result<Sm4Cipher, Sm4Error> {
        let mut k: [u32; 4] = split_block(key)?;
        let mut cipher = Sm4Cipher { rk: Vec::new() };
        for i in 0..4 {
            k[i] ^= FK[i];
        }
        for i in 0..8 {
            k[0] ^= sm4_key_sub(k[1] ^ k[2] ^ k[3] ^ CK[i * 4]);
            k[1] ^= sm4_key_sub(k[2] ^ k[3] ^ k[0] ^ CK[i * 4 + 1]);
            k[2] ^= sm4_key_sub(k[3] ^ k[0] ^ k[1] ^ CK[i * 4 + 2]);
            k[3] ^= sm4_key_sub(k[0] ^ k[1] ^ k[2] ^ CK[i * 4 + 3]);
            cipher.rk.push(k[0]);
            cipher.rk.push(k[1]);
            cipher.rk.push(k[2]);
            cipher.rk.push(k[3]);
        }

        Ok(cipher)
    }

    pub fn encrypt(&self, block_in: &[u8]) -> Result<[u8; 16], Sm4Error> {
        let mut x: [u32; 4] = split_block(block_in)?;
        let rk = &self.rk;
        for i in 0..8 {
            x[0] ^= t_trans(x[1] ^ x[2] ^ x[3] ^ rk[i * 4]);
            x[1] ^= t_trans(x[2] ^ x[3] ^ x[0] ^ rk[i * 4 + 1]);
            x[2] ^= t_trans(x[3] ^ x[0] ^ x[1] ^ rk[i * 4 + 2]);
            x[3] ^= t_trans(x[0] ^ x[1] ^ x[2] ^ rk[i * 4 + 3]);
        }
        let y = [x[3], x[2], x[1], x[0]];
        combine_block(&y)
    }

    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64")))]
    pub fn encrypt_sm4ni(&self, block_in: &[u8; 64]) -> Result<[u8; 64], Sm4Error> {
        let rk = &self.rk;

        let mut res: [u8; 64] = [0; 64];
        if is_x86_feature_detected!("sse")
            && is_x86_feature_detected!("sse2")
            && is_x86_feature_detected!("sse3")
            && is_x86_feature_detected!("aes")
        {
            unsafe { sm4_crypt_affine_ni(rk, block_in, &mut res, 0) };
        } else {
            for i in 0..4 {
                let tmp_res = self.encrypt(&block_in[i * 16..i * 16 + 16])?;
                for z in 0..16 {
                    res[i * 16 + z] = tmp_res[z];
                }
            }
        }

        Ok(res)
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    pub fn encrypt_sm4ni(&self, block_in: &[u8; 64]) -> Result<[u8; 64], Sm4Error> {
        let rk = &self.rk;

        let mut res: [u8; 64] = [0; 64];
        for i in 0..4 {
            let tmp_res = self.encrypt(&block_in[i * 16..i * 16 + 16])?;
            for z in 0..16 {
                res[i * 16 + z] = tmp_res[z];
            }
        }
        Ok(res)
    }

    pub fn decrypt(&self, block_in: &[u8]) -> Result<[u8; 16], Sm4Error> {
        let mut x: [u32; 4] = split_block(block_in)?;
        let rk = &self.rk;
        for i in 0..8 {
            x[0] ^= t_trans(x[1] ^ x[2] ^ x[3] ^ rk[31 - i * 4]);
            x[1] ^= t_trans(x[2] ^ x[3] ^ x[0] ^ rk[31 - (i * 4 + 1)]);
            x[2] ^= t_trans(x[3] ^ x[0] ^ x[1] ^ rk[31 - (i * 4 + 2)]);
            x[3] ^= t_trans(x[0] ^ x[1] ^ x[2] ^ rk[31 - (i * 4 + 3)]);
        }
        let y = [x[3], x[2], x[1], x[0]];
        combine_block(&y)
    }
}

// Tests below

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setup_cipher() {
        let key: [u8; 16] = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54,
            0x32, 0x10,
        ];
        let cipher = Sm4Cipher::new(&key).unwrap();
        let rk = &cipher.rk;
        assert_eq!(rk[0], 0xf121_86f9);
        assert_eq!(rk[31], 0x9124_a012);
    }

    #[test]
    fn enc_and_dec() {
        let key: [u8; 16] = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54,
            0x32, 0x10,
        ];
        let cipher = Sm4Cipher::new(&key).unwrap();

        let data: [u8; 16] = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54,
            0x32, 0x10,
        ];
        let ct = cipher.encrypt(&data).unwrap();
        let standard_ct: [u8; 16] = [
            0x68, 0x1e, 0xdf, 0x34, 0xd2, 0x06, 0x96, 0x5e, 0x86, 0xb3, 0xe9, 0x4f, 0x53, 0x6e,
            0x42, 0x46,
        ];

        // Check the example cipher text
        for i in 0..16 {
            assert_eq!(standard_ct[i], ct[i]);
        }

        // Check the result of decryption
        let pt = cipher.decrypt(&ct).unwrap();
        for i in 0..16 {
            assert_eq!(pt[i], data[i]);
        }
    }

    #[test]
    fn enc_and_dec_sm4ni() {
        let key: [u8; 16] = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54,
            0x32, 0x10,
        ];
        let cipher = Sm4Cipher::new(&key).unwrap();

        let data: [u8; 64] = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54,
            0x32, 0x10, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0xfe, 0xdc, 0xba, 0x98,
            0x76, 0x54, 0x32, 0x10, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0xfe, 0xdc,
            0xba, 0x98, 0x76, 0x54, 0x32, 0x10, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
            0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10,
        ];
        let ct = cipher.encrypt_sm4ni(&data).unwrap();
        let standard_ct: [u8; 64] = [
            0x68, 0x1e, 0xdf, 0x34, 0xd2, 0x06, 0x96, 0x5e, 0x86, 0xb3, 0xe9, 0x4f, 0x53, 0x6e,
            0x42, 0x46, 0x68, 0x1e, 0xdf, 0x34, 0xd2, 0x06, 0x96, 0x5e, 0x86, 0xb3, 0xe9, 0x4f,
            0x53, 0x6e, 0x42, 0x46, 0x68, 0x1e, 0xdf, 0x34, 0xd2, 0x06, 0x96, 0x5e, 0x86, 0xb3,
            0xe9, 0x4f, 0x53, 0x6e, 0x42, 0x46, 0x68, 0x1e, 0xdf, 0x34, 0xd2, 0x06, 0x96, 0x5e,
            0x86, 0xb3, 0xe9, 0x4f, 0x53, 0x6e, 0x42, 0x46,
        ];

        // Check the example cipher text
        for i in 0..64 {
            assert_eq!(standard_ct[i], ct[i]);
        }
    }
}
