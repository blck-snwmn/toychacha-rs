use byteorder::ByteOrder;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    x: [u32; 16],
}

impl State {
    pub fn new(key: [u8; 32], nonce: [u8; 12], counter: u32) -> Self {
        // TODO change args type to reference
        let tkey: [u32; 8] = unsafe { std::mem::transmute(key) };
        let tnonce: [u32; 3] = unsafe { std::mem::transmute(nonce) };
        State {
            x: [
                0x61707865, 0x3320646e, 0x79622d32, 0x6b206574, // constants
                tkey[0], tkey[1], tkey[2], tkey[3], tkey[4], tkey[5], tkey[6], tkey[7], // key
                counter, //counter
                tnonce[0], tnonce[1], tnonce[2], // nonce
            ],
        }
    }

    fn quarter_round(&mut self, ai: usize, bi: usize, ci: usize, di: usize) {
        let a = self.x[ai];
        let b = self.x[bi];
        let c = self.x[ci];
        let d = self.x[di];

        let (a, b, c, d) = quarter_round(a, b, c, d);

        self.x[ai] = a;
        self.x[bi] = b;
        self.x[ci] = c;
        self.x[di] = d;
    }

    fn inner_block(&mut self) {
        // column rounds
        self.quarter_round(0, 4, 8, 12);
        self.quarter_round(1, 5, 9, 13);
        self.quarter_round(2, 6, 10, 14);
        self.quarter_round(3, 7, 11, 15);

        // diagonal rounds
        self.quarter_round(0, 5, 10, 15);
        self.quarter_round(1, 6, 11, 12);
        self.quarter_round(2, 7, 8, 13);
        self.quarter_round(3, 4, 9, 14);
    }

    fn add(&mut self, other: &State) {
        for i in 0..16 {
            self.x[i] = self.x[i].wrapping_add(other.x[i]);
        }
    }

    pub fn block(&mut self) -> [u8; 64] {
        let mut state = self.clone();
        for _ in 0..10 {
            state.inner_block();
        }
        self.add(&state);

        self.serialize()
    }

    fn serialize(&self) -> [u8; 64] {
        let mut out = [0u8; 64];
        for i in 0..16 {
            byteorder::LittleEndian::write_u32(&mut out[4 * i..4 * (i + 1)], self.x[i]);
        }
        out
    }
}

fn rotation_n(n: usize, x: u32) -> u32 {
    (x << n) | (x >> (32 - n))
}

fn quarter_round(a: u32, b: u32, c: u32, d: u32) -> (u32, u32, u32, u32) {
    let a = a.wrapping_add(b);
    let d = rotation_n(16, d ^ a);

    let c = c.wrapping_add(d);
    let b = rotation_n(12, b ^ c);

    let a = a.wrapping_add(b);
    let d = rotation_n(8, d ^ a);

    let c = c.wrapping_add(d);
    let b = rotation_n(7, b ^ c);

    (a, b, c, d)
}

pub struct ChaCha20 {
    key: [u8; 32],
    nonce: [u8; 12],
}

impl ChaCha20 {
    pub fn new(key: [u8; 32], nonce: [u8; 12]) -> Self {
        ChaCha20 { key, nonce }
    }

    pub fn encrypt(&self, plaintext: &mut [u8], counter: u32) {
        let mut counter = counter;

        let mut hp = plaintext;

        while !hp.is_empty() {
            let mut state = State::new(self.key, self.nonce, counter); // TODO reuse state
            let block = state.block();

            let len = std::cmp::min(hp.len(), 64);
            for i in 0..len {
                hp[i] ^= block[i];
            }

            counter += 1;
            hp = &mut hp[len..];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotation_n() {
        assert_eq!(rotation_n(7, 0x7998bfda), 0xcc5fed3c);
    }

    #[test]
    fn test_quarter_round() {
        let a = 0x11111111;
        let b = 0x01020304;
        let c = 0x9b8d6f43;
        let d = 0x01234567;

        let (a, b, c, d) = quarter_round(a, b, c, d);

        assert_eq!(a, 0xea2a92f4);
        assert_eq!(b, 0xcb1cf8ce);
        assert_eq!(c, 0x4581472e);
        assert_eq!(d, 0x5881c4bb);
    }

    #[test]
    fn test_state_quarter_round() {
        let mut s = State {
            x: [
                0x879531e0, 0xc5ecf37d, 0x516461b1, 0xc9a62f8a, 0x44c20ef3, 0x3390af7f, 0xd9fc690b,
                0x2a5f714c, 0x53372767, 0xb00a5631, 0x974c541a, 0x359e9963, 0x5c971061, 0x3d631689,
                0x2098d9d6, 0x91dbd320,
            ],
        };

        s.quarter_round(2, 7, 8, 13);

        let want = State {
            x: [
                0x879531e0, 0xc5ecf37d, 0xbdb886dc, 0xc9a62f8a, 0x44c20ef3, 0x3390af7f, 0xd9fc690b,
                0xcfacafd2, 0xe46bea80, 0xb00a5631, 0x974c541a, 0x359e9963, 0x5c971061, 0xccc07c79,
                0x2098d9d6, 0x91dbd320,
            ],
        };
        assert_eq!(s.x, want.x);
    }

    #[test]
    fn test_inner_block() {
        let mut s = State {
            x: [
                0x61707865, 0x3320646e, 0x79622d32, 0x6b206574, 0x03020100, 0x07060504, 0x0b0a0908,
                0x0f0e0d0c, 0x13121110, 0x17161514, 0x1b1a1918, 0x1f1e1d1c, 0x00000001, 0x09000000,
                0x4a000000, 0x00000000,
            ],
        };

        for _ in 0..10 {
            s.inner_block();
        }

        let want = State {
            x: [
                0x837778ab, 0xe238d763, 0xa67ae21e, 0x5950bb2f, 0xc4f2d0c7, 0xfc62bb2f, 0x8fa018fc,
                0x3f5ec7b7, 0x335271c2, 0xf29489f3, 0xeabda8fc, 0x82e46ebd, 0xd19c12b4, 0xb04e16de,
                0x9e83d0cb, 0x4e3c50a2,
            ],
        };
        assert_eq!(s.x, want.x);
    }

    #[test]
    fn test_state_new() {
        let c = State::new(
            [
                0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
                0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b,
                0x1c, 0x1d, 0x1e, 0x1f,
            ],
            [
                0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00, 0x4a, 0x00, 0x00, 0x00, 0x00,
            ],
            1,
        );
        let want = State {
            x: [
                0x61707865, 0x3320646e, 0x79622d32, 0x6b206574, 0x03020100, 0x07060504, 0x0b0a0908,
                0x0f0e0d0c, 0x13121110, 0x17161514, 0x1b1a1918, 0x1f1e1d1c, 0x00000001, 0x09000000,
                0x4a000000, 0x00000000,
            ],
        };
        assert_eq!(c.x, want.x);
    }

    #[test]
    fn test_state_block() {
        let mut c = State::new(
            [
                0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
                0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b,
                0x1c, 0x1d, 0x1e, 0x1f,
            ],
            [
                0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00, 0x4a, 0x00, 0x00, 0x00, 0x00,
            ],
            1,
        );
        let out = c.block();

        let want = State {
            x: [
                0xe4e7f110, 0x15593bd1, 0x1fdd0f50, 0xc47120a3, 0xc7f4d1c7, 0x0368c033, 0x9aaa2204,
                0x4e6cd4c3, 0x466482d2, 0x09aa9f07, 0x05d7c214, 0xa2028bd9, 0xd19c12b5, 0xb94e16de,
                0xe883d0cb, 0x4e3c50a2,
            ],
        };
        assert_eq!(c.x, want.x);

        let want = [
            0x10, 0xf1, 0xe7, 0xe4, 0xd1, 0x3b, 0x59, 0x15, 0x50, 0x0f, 0xdd, 0x1f, 0xa3, 0x20,
            0x71, 0xc4, 0xc7, 0xd1, 0xf4, 0xc7, 0x33, 0xc0, 0x68, 0x03, 0x04, 0x22, 0xaa, 0x9a,
            0xc3, 0xd4, 0x6c, 0x4e, 0xd2, 0x82, 0x64, 0x46, 0x07, 0x9f, 0xaa, 0x09, 0x14, 0xc2,
            0xd7, 0x05, 0xd9, 0x8b, 0x02, 0xa2, 0xb5, 0x12, 0x9c, 0xd1, 0xde, 0x16, 0x4e, 0xb9,
            0xcb, 0xd0, 0x83, 0xe8, 0xa2, 0x50, 0x3c, 0x4e,
        ];

        assert_eq!(out, want)
    }

    #[test]
    fn test_chacha_encrypt() {
        {
            let key: [u8; 32] = [
                0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
                0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b,
                0x1c, 0x1d, 0x1e, 0x1f,
            ];
            let nonce: [u8; 12] = [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x4a, 0x00, 0x00, 0x00, 0x00,
            ];

            let mut plaintext = [
                0x4c, 0x61, 0x64, 0x69, 0x65, 0x73, 0x20, 0x61, 0x6e, 0x64, 0x20, 0x47, 0x65, 0x6e,
                0x74, 0x6c, 0x65, 0x6d, 0x65, 0x6e, 0x20, 0x6f, 0x66, 0x20, 0x74, 0x68, 0x65, 0x20,
                0x63, 0x6c, 0x61, 0x73, 0x73, 0x20, 0x6f, 0x66, 0x20, 0x27, 0x39, 0x39, 0x3a, 0x20,
                0x49, 0x66, 0x20, 0x49, 0x20, 0x63, 0x6f, 0x75, 0x6c, 0x64, 0x20, 0x6f, 0x66, 0x66,
                0x65, 0x72, 0x20, 0x79, 0x6f, 0x75, 0x20, 0x6f, 0x6e, 0x6c, 0x79, 0x20, 0x6f, 0x6e,
                0x65, 0x20, 0x74, 0x69, 0x70, 0x20, 0x66, 0x6f, 0x72, 0x20, 0x74, 0x68, 0x65, 0x20,
                0x66, 0x75, 0x74, 0x75, 0x72, 0x65, 0x2c, 0x20, 0x73, 0x75, 0x6e, 0x73, 0x63, 0x72,
                0x65, 0x65, 0x6e, 0x20, 0x77, 0x6f, 0x75, 0x6c, 0x64, 0x20, 0x62, 0x65, 0x20, 0x69,
                0x74, 0x2e,
            ];

            let c = ChaCha20::new(key, nonce);
            c.encrypt(&mut plaintext, 1);

            let want = [
                0x6e, 0x2e, 0x35, 0x9a, 0x25, 0x68, 0xf9, 0x80, 0x41, 0xba, 0x07, 0x28, 0xdd, 0x0d,
                0x69, 0x81, 0xe9, 0x7e, 0x7a, 0xec, 0x1d, 0x43, 0x60, 0xc2, 0x0a, 0x27, 0xaf, 0xcc,
                0xfd, 0x9f, 0xae, 0x0b, 0xf9, 0x1b, 0x65, 0xc5, 0x52, 0x47, 0x33, 0xab, 0x8f, 0x59,
                0x3d, 0xab, 0xcd, 0x62, 0xb3, 0x57, 0x16, 0x39, 0xd6, 0x24, 0xe6, 0x51, 0x52, 0xab,
                0x8f, 0x53, 0x0c, 0x35, 0x9f, 0x08, 0x61, 0xd8, 0x07, 0xca, 0x0d, 0xbf, 0x50, 0x0d,
                0x6a, 0x61, 0x56, 0xa3, 0x8e, 0x08, 0x8a, 0x22, 0xb6, 0x5e, 0x52, 0xbc, 0x51, 0x4d,
                0x16, 0xcc, 0xf8, 0x06, 0x81, 0x8c, 0xe9, 0x1a, 0xb7, 0x79, 0x37, 0x36, 0x5a, 0xf9,
                0x0b, 0xbf, 0x74, 0xa3, 0x5b, 0xe6, 0xb4, 0x0b, 0x8e, 0xed, 0xf2, 0x78, 0x5e, 0x42,
                0x87, 0x4d,
            ];

            assert_eq!(want, plaintext)
        }
        {
            let key: [u8; 32] = [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00,
            ];
            let nonce: [u8; 12] = [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ];

            let mut plaintext = [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ];

            let c = ChaCha20::new(key, nonce);
            c.encrypt(&mut plaintext, 0);

            let want = [
                0x76, 0xb8, 0xe0, 0xad, 0xa0, 0xf1, 0x3d, 0x90, 0x40, 0x5d, 0x6a, 0xe5, 0x53, 0x86,
                0xbd, 0x28, 0xbd, 0xd2, 0x19, 0xb8, 0xa0, 0x8d, 0xed, 0x1a, 0xa8, 0x36, 0xef, 0xcc,
                0x8b, 0x77, 0x0d, 0xc7, 0xda, 0x41, 0x59, 0x7c, 0x51, 0x57, 0x48, 0x8d, 0x77, 0x24,
                0xe0, 0x3f, 0xb8, 0xd8, 0x4a, 0x37, 0x6a, 0x43, 0xb8, 0xf4, 0x15, 0x18, 0xa1, 0x1c,
                0xc3, 0x87, 0xb6, 0x69, 0xb2, 0xee, 0x65, 0x86,
            ];

            assert_eq!(want, plaintext)
        }
        {
            let key: [u8; 32] = [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x01,
            ];
            let nonce: [u8; 12] = [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
            ];

            let mut plaintext = [
                0x41, 0x6e, 0x79, 0x20, 0x73, 0x75, 0x62, 0x6d, 0x69, 0x73, 0x73, 0x69, 0x6f, 0x6e,
                0x20, 0x74, 0x6f, 0x20, 0x74, 0x68, 0x65, 0x20, 0x49, 0x45, 0x54, 0x46, 0x20, 0x69,
                0x6e, 0x74, 0x65, 0x6e, 0x64, 0x65, 0x64, 0x20, 0x62, 0x79, 0x20, 0x74, 0x68, 0x65,
                0x20, 0x43, 0x6f, 0x6e, 0x74, 0x72, 0x69, 0x62, 0x75, 0x74, 0x6f, 0x72, 0x20, 0x66,
                0x6f, 0x72, 0x20, 0x70, 0x75, 0x62, 0x6c, 0x69, 0x63, 0x61, 0x74, 0x69, 0x6f, 0x6e,
                0x20, 0x61, 0x73, 0x20, 0x61, 0x6c, 0x6c, 0x20, 0x6f, 0x72, 0x20, 0x70, 0x61, 0x72,
                0x74, 0x20, 0x6f, 0x66, 0x20, 0x61, 0x6e, 0x20, 0x49, 0x45, 0x54, 0x46, 0x20, 0x49,
                0x6e, 0x74, 0x65, 0x72, 0x6e, 0x65, 0x74, 0x2d, 0x44, 0x72, 0x61, 0x66, 0x74, 0x20,
                0x6f, 0x72, 0x20, 0x52, 0x46, 0x43, 0x20, 0x61, 0x6e, 0x64, 0x20, 0x61, 0x6e, 0x79,
                0x20, 0x73, 0x74, 0x61, 0x74, 0x65, 0x6d, 0x65, 0x6e, 0x74, 0x20, 0x6d, 0x61, 0x64,
                0x65, 0x20, 0x77, 0x69, 0x74, 0x68, 0x69, 0x6e, 0x20, 0x74, 0x68, 0x65, 0x20, 0x63,
                0x6f, 0x6e, 0x74, 0x65, 0x78, 0x74, 0x20, 0x6f, 0x66, 0x20, 0x61, 0x6e, 0x20, 0x49,
                0x45, 0x54, 0x46, 0x20, 0x61, 0x63, 0x74, 0x69, 0x76, 0x69, 0x74, 0x79, 0x20, 0x69,
                0x73, 0x20, 0x63, 0x6f, 0x6e, 0x73, 0x69, 0x64, 0x65, 0x72, 0x65, 0x64, 0x20, 0x61,
                0x6e, 0x20, 0x22, 0x49, 0x45, 0x54, 0x46, 0x20, 0x43, 0x6f, 0x6e, 0x74, 0x72, 0x69,
                0x62, 0x75, 0x74, 0x69, 0x6f, 0x6e, 0x22, 0x2e, 0x20, 0x53, 0x75, 0x63, 0x68, 0x20,
                0x73, 0x74, 0x61, 0x74, 0x65, 0x6d, 0x65, 0x6e, 0x74, 0x73, 0x20, 0x69, 0x6e, 0x63,
                0x6c, 0x75, 0x64, 0x65, 0x20, 0x6f, 0x72, 0x61, 0x6c, 0x20, 0x73, 0x74, 0x61, 0x74,
                0x65, 0x6d, 0x65, 0x6e, 0x74, 0x73, 0x20, 0x69, 0x6e, 0x20, 0x49, 0x45, 0x54, 0x46,
                0x20, 0x73, 0x65, 0x73, 0x73, 0x69, 0x6f, 0x6e, 0x73, 0x2c, 0x20, 0x61, 0x73, 0x20,
                0x77, 0x65, 0x6c, 0x6c, 0x20, 0x61, 0x73, 0x20, 0x77, 0x72, 0x69, 0x74, 0x74, 0x65,
                0x6e, 0x20, 0x61, 0x6e, 0x64, 0x20, 0x65, 0x6c, 0x65, 0x63, 0x74, 0x72, 0x6f, 0x6e,
                0x69, 0x63, 0x20, 0x63, 0x6f, 0x6d, 0x6d, 0x75, 0x6e, 0x69, 0x63, 0x61, 0x74, 0x69,
                0x6f, 0x6e, 0x73, 0x20, 0x6d, 0x61, 0x64, 0x65, 0x20, 0x61, 0x74, 0x20, 0x61, 0x6e,
                0x79, 0x20, 0x74, 0x69, 0x6d, 0x65, 0x20, 0x6f, 0x72, 0x20, 0x70, 0x6c, 0x61, 0x63,
                0x65, 0x2c, 0x20, 0x77, 0x68, 0x69, 0x63, 0x68, 0x20, 0x61, 0x72, 0x65, 0x20, 0x61,
                0x64, 0x64, 0x72, 0x65, 0x73, 0x73, 0x65, 0x64, 0x20, 0x74, 0x6f,
            ];

            let c = ChaCha20::new(key, nonce);
            c.encrypt(&mut plaintext, 1);

            let want = [
                0xa3, 0xfb, 0xf0, 0x7d, 0xf3, 0xfa, 0x2f, 0xde, 0x4f, 0x37, 0x6c, 0xa2, 0x3e, 0x82,
                0x73, 0x70, 0x41, 0x60, 0x5d, 0x9f, 0x4f, 0x4f, 0x57, 0xbd, 0x8c, 0xff, 0x2c, 0x1d,
                0x4b, 0x79, 0x55, 0xec, 0x2a, 0x97, 0x94, 0x8b, 0xd3, 0x72, 0x29, 0x15, 0xc8, 0xf3,
                0xd3, 0x37, 0xf7, 0xd3, 0x70, 0x05, 0x0e, 0x9e, 0x96, 0xd6, 0x47, 0xb7, 0xc3, 0x9f,
                0x56, 0xe0, 0x31, 0xca, 0x5e, 0xb6, 0x25, 0x0d, 0x40, 0x42, 0xe0, 0x27, 0x85, 0xec,
                0xec, 0xfa, 0x4b, 0x4b, 0xb5, 0xe8, 0xea, 0xd0, 0x44, 0x0e, 0x20, 0xb6, 0xe8, 0xdb,
                0x09, 0xd8, 0x81, 0xa7, 0xc6, 0x13, 0x2f, 0x42, 0x0e, 0x52, 0x79, 0x50, 0x42, 0xbd,
                0xfa, 0x77, 0x73, 0xd8, 0xa9, 0x05, 0x14, 0x47, 0xb3, 0x29, 0x1c, 0xe1, 0x41, 0x1c,
                0x68, 0x04, 0x65, 0x55, 0x2a, 0xa6, 0xc4, 0x05, 0xb7, 0x76, 0x4d, 0x5e, 0x87, 0xbe,
                0xa8, 0x5a, 0xd0, 0x0f, 0x84, 0x49, 0xed, 0x8f, 0x72, 0xd0, 0xd6, 0x62, 0xab, 0x05,
                0x26, 0x91, 0xca, 0x66, 0x42, 0x4b, 0xc8, 0x6d, 0x2d, 0xf8, 0x0e, 0xa4, 0x1f, 0x43,
                0xab, 0xf9, 0x37, 0xd3, 0x25, 0x9d, 0xc4, 0xb2, 0xd0, 0xdf, 0xb4, 0x8a, 0x6c, 0x91,
                0x39, 0xdd, 0xd7, 0xf7, 0x69, 0x66, 0xe9, 0x28, 0xe6, 0x35, 0x55, 0x3b, 0xa7, 0x6c,
                0x5c, 0x87, 0x9d, 0x7b, 0x35, 0xd4, 0x9e, 0xb2, 0xe6, 0x2b, 0x08, 0x71, 0xcd, 0xac,
                0x63, 0x89, 0x39, 0xe2, 0x5e, 0x8a, 0x1e, 0x0e, 0xf9, 0xd5, 0x28, 0x0f, 0xa8, 0xca,
                0x32, 0x8b, 0x35, 0x1c, 0x3c, 0x76, 0x59, 0x89, 0xcb, 0xcf, 0x3d, 0xaa, 0x8b, 0x6c,
                0xcc, 0x3a, 0xaf, 0x9f, 0x39, 0x79, 0xc9, 0x2b, 0x37, 0x20, 0xfc, 0x88, 0xdc, 0x95,
                0xed, 0x84, 0xa1, 0xbe, 0x05, 0x9c, 0x64, 0x99, 0xb9, 0xfd, 0xa2, 0x36, 0xe7, 0xe8,
                0x18, 0xb0, 0x4b, 0x0b, 0xc3, 0x9c, 0x1e, 0x87, 0x6b, 0x19, 0x3b, 0xfe, 0x55, 0x69,
                0x75, 0x3f, 0x88, 0x12, 0x8c, 0xc0, 0x8a, 0xaa, 0x9b, 0x63, 0xd1, 0xa1, 0x6f, 0x80,
                0xef, 0x25, 0x54, 0xd7, 0x18, 0x9c, 0x41, 0x1f, 0x58, 0x69, 0xca, 0x52, 0xc5, 0xb8,
                0x3f, 0xa3, 0x6f, 0xf2, 0x16, 0xb9, 0xc1, 0xd3, 0x00, 0x62, 0xbe, 0xbc, 0xfd, 0x2d,
                0xc5, 0xbc, 0xe0, 0x91, 0x19, 0x34, 0xfd, 0xa7, 0x9a, 0x86, 0xf6, 0xe6, 0x98, 0xce,
                0xd7, 0x59, 0xc3, 0xff, 0x9b, 0x64, 0x77, 0x33, 0x8f, 0x3d, 0xa4, 0xf9, 0xcd, 0x85,
                0x14, 0xea, 0x99, 0x82, 0xcc, 0xaf, 0xb3, 0x41, 0xb2, 0x38, 0x4d, 0xd9, 0x02, 0xf3,
                0xd1, 0xab, 0x7a, 0xc6, 0x1d, 0xd2, 0x9c, 0x6f, 0x21, 0xba, 0x5b, 0x86, 0x2f, 0x37,
                0x30, 0xe3, 0x7c, 0xfd, 0xc4, 0xfd, 0x80, 0x6c, 0x22, 0xf2, 0x21,
            ];

            assert_eq!(want, plaintext)
        }
        {
            let key: [u8; 32] = [
                0x1c, 0x92, 0x40, 0xa5, 0xeb, 0x55, 0xd3, 0x8a, 0xf3, 0x33, 0x88, 0x86, 0x04, 0xf6,
                0xb5, 0xf0, 0x47, 0x39, 0x17, 0xc1, 0x40, 0x2b, 0x80, 0x09, 0x9d, 0xca, 0x5c, 0xbc,
                0x20, 0x70, 0x75, 0xc0,
            ];
            let nonce: [u8; 12] = [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
            ];

            let mut plaintext = [
                0x27, 0x54, 0x77, 0x61, 0x73, 0x20, 0x62, 0x72, 0x69, 0x6c, 0x6c, 0x69, 0x67, 0x2c,
                0x20, 0x61, 0x6e, 0x64, 0x20, 0x74, 0x68, 0x65, 0x20, 0x73, 0x6c, 0x69, 0x74, 0x68,
                0x79, 0x20, 0x74, 0x6f, 0x76, 0x65, 0x73, 0x0a, 0x44, 0x69, 0x64, 0x20, 0x67, 0x79,
                0x72, 0x65, 0x20, 0x61, 0x6e, 0x64, 0x20, 0x67, 0x69, 0x6d, 0x62, 0x6c, 0x65, 0x20,
                0x69, 0x6e, 0x20, 0x74, 0x68, 0x65, 0x20, 0x77, 0x61, 0x62, 0x65, 0x3a, 0x0a, 0x41,
                0x6c, 0x6c, 0x20, 0x6d, 0x69, 0x6d, 0x73, 0x79, 0x20, 0x77, 0x65, 0x72, 0x65, 0x20,
                0x74, 0x68, 0x65, 0x20, 0x62, 0x6f, 0x72, 0x6f, 0x67, 0x6f, 0x76, 0x65, 0x73, 0x2c,
                0x0a, 0x41, 0x6e, 0x64, 0x20, 0x74, 0x68, 0x65, 0x20, 0x6d, 0x6f, 0x6d, 0x65, 0x20,
                0x72, 0x61, 0x74, 0x68, 0x73, 0x20, 0x6f, 0x75, 0x74, 0x67, 0x72, 0x61, 0x62, 0x65,
                0x2e,
            ];

            let c = ChaCha20::new(key, nonce);
            c.encrypt(&mut plaintext, 42);

            let want = [
                0x62, 0xe6, 0x34, 0x7f, 0x95, 0xed, 0x87, 0xa4, 0x5f, 0xfa, 0xe7, 0x42, 0x6f, 0x27,
                0xa1, 0xdf, 0x5f, 0xb6, 0x91, 0x10, 0x04, 0x4c, 0x0d, 0x73, 0x11, 0x8e, 0xff, 0xa9,
                0x5b, 0x01, 0xe5, 0xcf, 0x16, 0x6d, 0x3d, 0xf2, 0xd7, 0x21, 0xca, 0xf9, 0xb2, 0x1e,
                0x5f, 0xb1, 0x4c, 0x61, 0x68, 0x71, 0xfd, 0x84, 0xc5, 0x4f, 0x9d, 0x65, 0xb2, 0x83,
                0x19, 0x6c, 0x7f, 0xe4, 0xf6, 0x05, 0x53, 0xeb, 0xf3, 0x9c, 0x64, 0x02, 0xc4, 0x22,
                0x34, 0xe3, 0x2a, 0x35, 0x6b, 0x3e, 0x76, 0x43, 0x12, 0xa6, 0x1a, 0x55, 0x32, 0x05,
                0x57, 0x16, 0xea, 0xd6, 0x96, 0x25, 0x68, 0xf8, 0x7d, 0x3f, 0x3f, 0x77, 0x04, 0xc6,
                0xa8, 0xd1, 0xbc, 0xd1, 0xbf, 0x4d, 0x50, 0xd6, 0x15, 0x4b, 0x6d, 0xa7, 0x31, 0xb1,
                0x87, 0xb5, 0x8d, 0xfd, 0x72, 0x8a, 0xfa, 0x36, 0x75, 0x7a, 0x79, 0x7a, 0xc1, 0x88,
                0xd1,
            ];

            assert_eq!(want, plaintext)
        }
    }
}
