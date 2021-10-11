use crate::posit::Posit;
use std::fmt;
use std::cmp::{max, min};

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Quire<const N: u8, const ES: u8>(pub u128);

impl<const N: u8, const ES: u8> Quire<N, ES> {
    #[inline]
    pub const fn new(i: u128) -> Self {
        Self(i)
    }

    pub const SIGN_MASK:u128 = 1 << N - 1;
    pub const BIAS: i32 = 2i32.pow(ES as u32 + 1) * (N  as i32 - 2);
}

impl<const N: u8, const ES: u8> fmt::Binary for Quire<N, ES> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Binary::fmt(&self.0, f)
    }
}

impl<const N: u8, const ES: u8> std::convert::From<Posit<N, ES>> for Quire<N, ES> {
    fn from(item: Posit<N, ES>) -> Self {

        if item.is_zero() {
            return Self(0x0);
        }

        let (_, s, rc, r, e, f) = item.encode();

        let rg = if rc {(!r).wrapping_add(1)} else {r};

        let ex = (rg as i32) * 2i32.pow(ES as u32) + (e as i32);

        let shift_f = max(Self::BIAS + ex, 0);

        let mut quire: u128 = ((f >> (32 - (N - ES - 2))) as u128) << shift_f;

        quire = if s {(!quire).wrapping_add(1)} else {quire};

        quire = ((quire as i128) >> (N - ES - 4)) as u128;

        Self(quire)
    }
}

impl<const N: u8, const ES: u8> std::convert::From<Quire<N, ES>> for Posit<N, ES> {
    fn from(item: Quire<N, ES>) -> Self {

        if item.0 == 0 {
            return Self::zero();
        }

        let s = (1 << 127) & item.0 != 0;
        let quire = if s {(!item.0).wrapping_add(1)} else {item.0};
        
        let lod = quire.leading_zeros() as i32 - (128 - 51 - 1);
        let f = quire << lod + (128 - 51 - 1);

        let quire_e = (51 - 1 - lod - Quire::<N, ES>::BIAS) as u32;

        let e_o = quire_e & (0xffffffffu32 >> (32 - ES));
        let r_o = min(if quire_e & (1 << (Self::RS + ES + 1)) == 0 {(quire_e >> ES) + 1} else {(!quire_e >> ES).wrapping_add(1)}, (N - 1).into());

        let e_msb = (quire_e << (31 - Self::RS - ES - 1)) & 0x80000000u32;

        let mut rem = ((f << 1) >> 128 - 32) as u32;

        rem = (e_o << 32 - ES) | (rem >> ES);

        rem = e_msb | (rem >> 1);

        rem = rem >> N;

        rem = if e_msb == 0 {(0xffffffffu32 << (32 - N)) | rem} else {rem};

        rem = rem >> r_o;

        let mut p: u32 = (rem << N) >> (32 - N + 1);

        let l = (0x80000000u32 >> (2 * N - 2)) & rem != 0;
        let g = (0x80000000u32 >> (2 * N - 1)) & rem != 0;
        let r = (0x80000000u32 >> (2 * N)) & rem != 0;
        let st = (rem << (2 * N)) != 0;

        let ulp = if (g & (r | st)) | (l & g & !(r | st)) {1} else {0};

        p = if r_o < (N - 1).into() {p + ulp} else {p};

        if p == 0 {
            return if s {
                - Self::MINPOS
            } else {
                Self::MINPOS
            };
        }

        p = if s {((!p).wrapping_add(1) & Self::BODY_MASK) | 0x1u32 << N - 1} else {p};

        Self(p)
    }
}

use std::ops::{Neg, Add, Sub, Mul, Div};
use num_traits::identities::{One, Zero};

impl<const N: u8, const ES: u8> Neg for Quire<N, ES> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(
            (!self.0).wrapping_add(1)
        )
    }
}

impl<const N: u8, const ES: u8> Add for Quire<N, ES> {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        if self.is_zero() {
            other
        }
        else if other.is_zero() {
            self
        }
        else {
            Self(self.0.wrapping_add(other.0))
        }
    }
}

impl<const N: u8, const ES: u8> Sub for Quire<N, ES> {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        self + (- other)
    }
}

impl<const N: u8, const ES: u8> Mul for Quire<N, ES> {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        if self.is_zero() || other.is_zero() {
            Self::zero()
        }
        else if self.is_one() {
            other
        }
        else if other.is_one() {
            self
        }
        else {
            let c = ((self.0.wrapping_mul(other.0)) as i128 >> Self::BIAS + 1) as u128;
            Self(c)
        }
    }
}

impl<const N: u8, const ES: u8> Div for Quire<N, ES> {
    type Output = Self;

    // TODO
    fn div(self, _other: Self) -> Self::Output {
        Self(0x0)
    }
}

impl<const N: u8, const ES: u8> Zero for Quire<N, ES> {

    fn zero() -> Self {
        Self(0x0)
    }

    fn is_zero(self:&Self) -> bool {
        self.0 == 0
    }

}

impl<const N: u8, const ES: u8> One for Quire<N, ES> {

    fn one() -> Self {
        Self::new(1 << Self::BIAS + 1)
    }

    fn is_one(self:&Self) -> bool {
        self.0 == (1 << Self::BIAS + 1)
    }
}