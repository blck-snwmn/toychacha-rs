struct State {
    x: [u32; 16],
}

impl State {
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
}
