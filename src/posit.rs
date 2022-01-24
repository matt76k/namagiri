use std::fmt;
use std::mem;
use std::cmp::min;

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Debug)]
pub struct Posit<const N: u8, const ES: u8>(pub u32);

impl<const N: u8, const ES: u8> fmt::Binary for Posit<N, ES> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Binary::fmt(&self.0, f)
    }
}

impl<const N: u8, const ES: u8> fmt::Display for Posit<N, ES> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:b} n:{} es:{} f32:{}", self, N, ES, f32::from(*self))
    }
}

use std::cmp::Ordering;
impl<const N: u8, const ES: u8> PartialOrd for Posit<N, ES> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.is_nar() || other.is_nar() {
            None
        }
        else {
            let a = (if self.is_negative() {Self::SIGN_MASK | self.0} else {self.0}) as i32;
            let b = (if other.is_negative() {Self::SIGN_MASK | other.0} else {other.0}) as i32;

            Some(a.cmp(&b))
        }
    }
}

impl<const N: u8, const ES: u8> std::convert::From<Posit<N, ES>> for f32 {
    fn from(item: Posit<N, ES>) -> Self {
        if item.is_zero() {
            return 0f32;
        }

        if item.is_nar() {
            return f32::INFINITY;
        }

        let (_, s, rc, r, e, f) = item.encode();

        let rg = if rc {(!r).wrapping_add(1)} else {r};

        let exp = (rg as i32) * 2i32.pow(ES as u32) + (e as i32) - (N - ES - 3) as i32;

        let frac = f >> (32 - (N - ES - 2));
        let s_frac = (if s {(!frac).wrapping_add(1)} else {frac}) as i32;

        (s_frac as f32) * 2f32.powf(exp as f32)
    }
}

fn decode(e:u32, f:u32, n:u8, es:u8, rs:u8) -> u32 {
    let e_sign = e & (1 << (rs + es + 1)) != 0;
    let e_o = e & (0xffffffffu32 >> (32 - es));
    let r_o = min(if e_sign {(!e >> es).wrapping_add(1)} else {(e >> es) + 1}, (n - 1).into());

    let mut rem:u128 = ((f << 1) as u128) << 128 - 32;
    
    rem = ((e_o as u128) << 128 - es) | (rem >> es);
    
    rem = (if e_sign {1 << 127} else {0x0}) | (rem >> 1);

    rem = rem >> n;
    rem = if e_sign {rem} else {(!(0x0 as u128) << (128 - n)) | rem};
    rem = rem >> r_o;

    let p: u32 = ((rem << n) >> (128 - n + 1)) as u32;

    let l = ((1 << 127) >> (2 * n - 2)) & rem != 0;
    let g = ((1 << 127) >> (2 * n - 1)) & rem != 0;
    let r = ((1 << 127) >> (2 * n)) & rem != 0;
    let st = (rem << (2 * n)) != 0;

    let ulp = if (g & (r | st)) | (l & g & !(r | st)) {1} else {0};

    if r_o < (n - 1).into() {p + ulp} else {p}
}

use std::num::FpCategory;
impl<const N: u8, const ES: u8> std::convert::From<f32> for Posit<N, ES> {
    fn from(item: f32) -> Self {
        if item.classify() == FpCategory::Zero {
            return Self::zero();
        }

        if item.classify() == FpCategory::Infinite || item.classify() == FpCategory::Nan {
            return Self::NAR;
        }

        let bits:u32 = item.to_bits();

        let s = bits & 0x80000000u32 != 0;
        let e = (((bits >> 23) & 0xff) as i32 - 127) as u32;
        let f = 0x80000000u32 | (bits << 8);

        let mut p = decode(e, f, N, ES, Self::RS);

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

impl<const N: u8, const ES: u8> Posit<N, ES> {
    #[inline]
    pub const fn new(i: u32) -> Self {
        Self(i)
    }
    
    pub const SIGN_BIT:u32 = 1 << N - 1;
    pub const SIGN_MASK:u32 = 0xffffffffu32 << N - 1;
    pub const MASK:u32 = 0xffffffffu32 >> (32 - N);
    pub const BODY_MASK:u32 = 0xffffffffu32 >> (32 - (N - 1));
    pub const RS: u8 = 7u8 - N.leading_zeros() as u8;

    pub const NAR: Self = Self::new(Self::SIGN_BIT);
    pub const MINPOS: Self = Self::new(1);
    pub const MAXPOS: Self = Self::new(!Self::SIGN_BIT);

    pub fn encode(self) -> (u32, bool, bool, u32, u32, u32) {
        let s = self.0 & Self::SIGN_BIT != 0;

        let xin = if s {(!self.0).wrapping_add(1)}  else  {self.0} << (32 - N + 1);

        let rc =  (xin & 0x80000000u32) == 0;

        let r = if rc {xin.leading_zeros()} else {xin.leading_ones() - 1};

        let xin_tmp = xin << (if rc {r + 1} else {r + 2});

        let e = (xin_tmp & !(0xffffffffu32 >> ES)) >> (32 - ES);

        let frac = ((xin_tmp << ES) >> 1) | 0x80000000u32;

        return (xin, s, rc, r, e, frac);
    }


    #[inline]
    pub fn is_nar(self) -> bool {
        self == Self::NAR
    }
}

use std::ops::{Neg, Add, Sub, Mul, Div, Rem, AddAssign};
use num_traits::identities::{One, Zero};
use num_traits::sign::Signed;

impl<const N: u8, const ES: u8> num_traits::Num for Posit<N, ES> {
    type FromStrRadixErr = num_traits::ParseFloatError;
    fn from_str_radix(src: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        Ok(Self::from(f32::from_str_radix(src, radix)?))
    }
}

impl<const N: u8, const ES: u8> Signed for Posit<N, ES> {
    #[inline]
    fn abs(&self) -> Self {
        if self.is_negative() { -*self } else { *self }
    }

    #[inline]
    fn abs_sub(&self, other: &Self) -> Self {
        if *self <= *other { Self::zero() } else { *self - *other }
    }

    #[inline]
    fn signum(&self) -> Self {
        match (*self).0 {
            n if n == Self::NAR.0 => Self::NAR,
            n if n < Self::SIGN_BIT => Self::one(),
            0 => Self::zero(),
            _ => -Self::one(),
        }
    }

    #[inline]
    fn is_positive(&self) -> bool {(*self).0 & Self::SIGN_BIT == 0}

    #[inline]
    fn is_negative(&self) -> bool {(*self).0 & Self::SIGN_BIT != 0}
}

impl<const N: u8, const ES: u8> Neg for Posit<N, ES> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(
            (!self.0).wrapping_add(1) & Self::MASK
        )
    }
}

impl<const N: u8, const ES: u8> Add for Posit<N, ES> {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        if self.is_nar() || other.is_nar() {
            Self::NAR
        }
        else if self.is_zero() {
            other
        }
        else if other.is_zero() {
            self
        }
        else if self == -other {
            Self::zero()
        }
        else {
            let (xin1, mut s1, mut rc1, mut r1, mut e1, mut frac1) = self.encode();
            let (xin2, mut s2, mut rc2, mut r2, mut e2, mut frac2) = other.encode();

            if xin1 < xin2 {
                mem::swap(&mut s1, &mut s2);
                mem::swap(&mut rc1, &mut rc2);
                mem::swap(&mut r1, &mut r2);
                mem::swap(&mut e1, &mut e2);
                mem::swap(&mut frac1, &mut frac2);
            }

            let rv1 = if rc1 {(!r1).wrapping_add(1)} else {r1};
            let rv2 = if rc2 {(!r2).wrapping_add(1)} else {r2};

            let diff = min((rv1 << ES | e1).wrapping_sub(rv2 << ES | e2), (N - 1).into());

            let right_out = frac2 >> diff;

            let add_m = if s1 ^ s2 {(frac1 >> 1) - (right_out >> 1)} else {(frac1 >> 1) + (right_out >> 1)};

            let movf = (add_m & 0x80000000u32) >> 31;
            let add_m = if movf == 0 {add_m << 1} else {add_m};

            let nshift = add_m.leading_zeros();
            let add_m = add_m << nshift;

            let le_o = ((rv1 << ES | e1).wrapping_add(movf)).wrapping_sub(nshift);

            let mut p = decode(le_o, add_m, N, ES, Self::RS);

            p = if s1 {((!p).wrapping_add(1) & Self::BODY_MASK) | 0x1u32 << N - 1} else {p};

            Self(p)
        }
    }
}

impl<const N: u8, const ES: u8> AddAssign for Posit<N, ES> {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl<const N: u8, const ES: u8> Sub for Posit<N, ES> {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        if self.is_nar() || other.is_nar() {
            Self::NAR
        }
        else if self.is_zero() {
            other.neg()
        }
        else if other.is_zero() {
            self
        }
        else if self == other {
            Self::zero()
        }
        else {
            self + (- other)
        }
    }
}

impl<const N: u8, const ES: u8> Mul for Posit<N, ES> {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        if self.is_nar() || other.is_nar() {
            Self::NAR
        }
        else if self.is_zero() || other.is_zero() {
            Self::zero()
        }
        else if self.is_one() {
            other
        }
        else if other.is_one() {
            self
        }
        else {
            let (_, s1, rc1, r1, e1, frac1) = self.encode();
            let (_, s2, rc2, r2, e2, frac2) = other.encode();

            let s = s1 ^ s2;

            let d = N - ES - 2;

            let f:u32 = ((frac1 >> (32 -  d)) * (frac2 >> (32 - d))) << (32 - 2 * d);

            let movf = (f & 0x80000000u32) >> 31;

            let f = if movf == 0 {f << 1} else {f};

            let rg1 = if rc1 {(!r1).wrapping_add(1)} else {r1};
            let rg2 = if rc2 {(!r2).wrapping_add(1)} else {r2};

            let e = ((rg1 << ES) | e1) + ((rg2 << ES) | e2) + movf;

            let mut p = decode(e, f, N, ES, Self::RS);

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
}

impl<const N: u8, const ES: u8> Div for Posit<N, ES> {
    type Output = Self;

    fn div(self, other: Self) -> Self::Output {
        if self.is_nar() || other.is_nar() {
            Self::NAR
        }
        else if self.is_zero() {
            Self::zero()
        }
        else if other.is_zero() {
            Self::NAR
        }
        else if self.is_one() {
            other
        }
        else if other.is_one() {
            self
        }
        else {
            // TODO
            Self::zero()
        }
    }
}

impl<const N: u8, const ES: u8> Rem for Posit<N, ES> {
    type Output = Self;

    fn rem(self, _other: Self) -> Self::Output {
        // TODO
        Self::zero()
    }
}

impl<const N: u8, const ES: u8> Zero for Posit<N, ES> {

    fn zero() -> Self {
        Self::new(0)
    }

    fn is_zero(self:&Self) -> bool {
        self.0 == 0
    }

}

impl<const N: u8, const ES: u8> One for Posit<N, ES> {

    fn one() -> Self {
        Self::new(1 << (N - 2))
    }

    fn is_one(self:&Self) -> bool {
        self.0 == 1 << (N - 2)
    }
}

use ndarray::ScalarOperand;
impl<const N: u8, const ES: u8> ScalarOperand for Posit<N, ES> {}