struct state {
    x: [u32; 16],
}

impl state {
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
        let mut s = state {
            x: [
                0x879531e0, 0xc5ecf37d, 0x516461b1, 0xc9a62f8a, 0x44c20ef3, 0x3390af7f, 0xd9fc690b,
                0x2a5f714c, 0x53372767, 0xb00a5631, 0x974c541a, 0x359e9963, 0x5c971061, 0x3d631689,
                0x2098d9d6, 0x91dbd320,
            ],
        };

        s.quarter_round(2, 7, 8, 13);

        let want = state {
            x: [
                0x879531e0, 0xc5ecf37d, 0xbdb886dc, 0xc9a62f8a, 0x44c20ef3, 0x3390af7f, 0xd9fc690b,
                0xcfacafd2, 0xe46bea80, 0xb00a5631, 0x974c541a, 0x359e9963, 0x5c971061, 0xccc07c79,
                0x2098d9d6, 0x91dbd320,
            ],
        };
        assert_eq!(s.x, want.x);
    }
}
