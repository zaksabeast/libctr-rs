use core::ops::{Add, BitAnd, BitOr};

/// The different buttons that can be represented.
/// Each button represents one bit in a u32.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button {
    A = 1,
    B = 2,
    Select = 4,
    Start = 8,
    Dright = 16,
    Dleft = 32,
    Dup = 64,
    Ddown = 128,
    R = 256,
    L = 512,
    X = 1024,
    Y = 2048,
}

impl BitAnd<Button> for u32 {
    type Output = u32;

    fn bitand(self, rhs: Button) -> Self::Output {
        self & (rhs as u32)
    }
}

impl BitOr for Button {
    type Output = u32;

    fn bitor(self, rhs: Self) -> Self::Output {
        (self as u32) | (rhs as u32)
    }
}

impl Add for Button {
    type Output = u32;

    fn add(self, rhs: Self) -> Self::Output {
        (self as u32) + (rhs as u32)
    }
}
