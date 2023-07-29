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
}
