use crate::posit::Posit;
use std::cmp::min;

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct FLQuire<const N: u8, const ES: u8, const SIZE: u8> {
    pub quire:u128,
    pub sf:i32,
}

impl<const N: u8, const ES: u8, const SIZE: u8> FLQuire<N, ES, SIZE> {
    #[inline]
    pub const fn new(quire: u128, sf: i32) -> Self {
        Self{quire, sf}
    }

    pub const BIAS: i32 = 2i32.pow(ES as u32 + 1) * (N  as i32 - 2);
}

impl<const N: u8, const ES: u8, const SIZE: u8> std::convert::From<Posit<N, ES>> for FLQuire<N, ES, SIZE> {
    fn from(item: Posit<N, ES>) -> Self {

        if item.is_zero() {
            return Self{quire:0x0, sf:0x0};
        }

        let (_, s, rc, r, e, f) = item.encode();

        let rg = if rc {(!r).wrapping_add(1)} else {r};

        let sf = (rg as i32) * 2i32.pow(ES as u32) + (e as i32);

        let mut quire: u128 = (f >> (32 - (N - ES - 2))) as u128;

        quire = quire << (SIZE as u32  - 2 - (N as u32 - ES as u32 - 2));

        quire = if s {(!quire).wrapping_add(1)} else {quire};

        Self{quire, sf}
    }
}

impl<const N: u8, const ES: u8, const SIZE: u8> std::convert::From<FLQuire<N, ES, SIZE>> for Posit<N, ES> {
    fn from(item: FLQuire<N, ES, SIZE>) -> Self {

        if item.quire == 0 {
            return Self::zero();
        }

        let s = (1 << 127) & item.quire != 0;
        let quire = if s {(!item.quire).wrapping_add(1)} else {item.quire};
        
        let lod = SIZE as u32  - (128 - quire.leading_zeros());

        let f = quire << lod;

        let quire_e = (item.sf  - (lod as i32 + 1) + 3) as u32;

        let e_o = quire_e & (0xffffffffu32 >> (32 - ES));
        let r_o = min(if quire_e & (1 << (Self::RS + ES + 1)) == 0 {(quire_e >> ES) + 1} else {(!quire_e >> ES).wrapping_add(1)}, (N - 1).into());

        let e_msb = (quire_e << (31 - Self::RS - ES - 1)) & 0x80000000u32;

        let mut rem = (f << (32 - SIZE + 1)) as u32;

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

use std::ops::{Neg, Add, Sub, Mul, Div, AddAssign};
use num_traits::identities::{One, Zero};

impl<const N: u8, const ES: u8, const SIZE: u8> Neg for FLQuire<N, ES, SIZE> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(
            (!self.quire).wrapping_add(1),
            self.sf
        )
    }
}

impl<const N: u8, const ES: u8, const SIZE: u8> Add for FLQuire<N, ES, SIZE> {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {

        if self.is_zero() {
            return other;
        }

        if other.is_zero() {
            return self;
        }

        let (a, b) = if self.sf > other.sf {(self, other)} else {(other, self)};

        let mut quire = a.quire + (b.quire as i128 >> (a.sf - b.sf)) as u128;

        let s = (quire & (1 << SIZE - 1)) != 0;
        let g = (quire & (1 << SIZE - 2)) != 0;

        let ovf = s ^ g;

        let sf = if ovf {
            quire = (quire as i128 >> 1) as u128;
            a.sf + 1

        } else {
            a.sf
        };

        Self{quire, sf}
    }
}

impl<const N: u8, const ES: u8, const SIZE: u8> AddAssign for FLQuire<N, ES, SIZE> {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl<const N: u8, const ES: u8, const SIZE: u8> Sub for FLQuire<N, ES, SIZE> {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        self + (- other)
    }
}

impl<const N: u8, const ES: u8, const SIZE: u8> Mul for FLQuire<N, ES, SIZE> {
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
            let mut quire = ((self.quire * other.quire) as i128 >> SIZE - 3) as u128;

            let s = (quire & (1 << SIZE - 1)) != 0;
            let g = (quire & (1 << SIZE - 2)) != 0;

            let ovf = s ^ g;

            let sf = if ovf {
                quire = (quire as i128 >> 1) as u128;
                self.sf + other.sf + 1

            } else {
                self.sf + other.sf
            };

            Self{quire, sf}
        }
    }
}

impl<const N: u8, const ES: u8, const SIZE: u8> Div for FLQuire<N, ES, SIZE> {
    type Output = Self;

    fn div(self, _other: Self) -> Self::Output {
        Self::zero()
    }
}

impl<const N: u8, const ES: u8, const SIZE: u8> Zero for FLQuire<N, ES, SIZE> {

    fn zero() -> Self {
        Self::new(0x0, 0x0)
    }

    fn is_zero(self:&Self) -> bool {
        self.quire == 0
    }

}

impl<const N: u8, const ES: u8, const SIZE: u8> One for FLQuire<N, ES, SIZE> {

    fn one() -> Self {
        Self::new(1 << SIZE - 2, 0x0)
    }

    fn is_one(self:&Self) -> bool {
        self.quire == 1 << SIZE - 2 && self.sf == 0
    }
}