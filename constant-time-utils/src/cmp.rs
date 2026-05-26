use subtle::ConstantTimeEq;

pub fn ct_eq<const N: usize>(a: &[u8; N], b: &[u8; N]) -> bool {
    let mut diff = 0u8;
    for i in 0..N {
        diff |= a[i] ^ b[i];
    }
    diff.ct_eq(&0).into()
}

pub fn ct_lt<const N: usize>(a: &[u8; N], b: &[u8; N]) -> bool {
    let mut lt = 0u8;
    let mut eq = 1u8;

    for i in 0..N {
        let x = a[i];
        let y = b[i];

        let x_lt_y = (((x as u16).wrapping_sub(y as u16)) >> 8) as u8 & 1;
        let x_eq_y = (x ^ y).ct_eq(&0u8).unwrap_u8();
        lt |= x_lt_y & eq;
        eq &= x_eq_y;
    }

    lt == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equals_long_in_constant_time() {
        let a = [7u8; 32];
        let b = [7u8; 32];
        let mut c = [7u8; 32];
        c[0] = 0;

        assert!(ct_eq(&a, &b));
        assert!(!ct_eq(&a, &c));
    }

    #[test]
    fn equals_short_in_constant_time() {
        let a = [0u8; 8];
        let mut b = [0u8; 8];
        b[7] = 1;

        assert!(ct_eq(&a, &a));
        assert!(!ct_eq(&a, &b));
    }

    #[test]
    fn less_than_long_in_constant_time() {
        let mut a = [1u8; 32];
        a[31] = 2;
        let mut b = [1u8; 32];
        b[31] = 3;
        let mut c = [1u8; 32];
        c[31] = 1;

        assert!(ct_lt(&a, &b));
        assert!(!ct_lt(&a, &c)); // equal
        assert!(!ct_lt(&b, &a));
    }

    #[test]
    fn less_than_short_in_constant_time() {
        let mut a = [5u8; 8];
        a[0] = 6;
        let mut b = [5u8; 8];
        b[0] = 7;
        let mut c = [5u8; 8];
        c[0] = 4;

        assert!(ct_lt(&a, &b));
        assert!(!ct_lt(&a, &a));
        assert!(!ct_lt(&a, &c));
        assert!(ct_lt(&c, &a)); // reversed order captures greater case
    }

    #[test]
    fn less_than_short_handles_large_byte_differences() {
        let zero = 0u64.to_be_bytes();
        let large = 3_000_000_000u64.to_be_bytes();

        assert!(ct_lt(&zero, &large));
        assert!(!ct_lt(&large, &zero));
    }
}
