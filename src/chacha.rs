use byteorder::ByteOrder;

#[derive(Clone, Debug, PartialEq, Eq)]
struct State {
    x: [u32; 16],
}

impl State {
    fn new(key: [u8; 32], nonce: [u8; 12], counter: u32) -> Self {
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

    fn block(&mut self) -> [u8; 64] {
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
}
