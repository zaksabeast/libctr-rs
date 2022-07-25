use super::Button;
use std::ops::BitAnd;

/// An interface for parsing and doing bit math with io bits.
/// This is an experimental interface that may be removed,
/// depending on whether `PressedButtons` or `InterfaceDevice::is_just_pressed`
/// is used more naturally.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PressedButtons {
    io_bits: u32,
}

impl PressedButtons {
    pub fn new(io_bits: u32) -> Self {
        Self { io_bits }
    }

    pub fn a(&self) -> bool {
        (self.io_bits & Button::A) != 0
    }

    pub fn b(&self) -> bool {
        (self.io_bits & Button::B) != 0
    }

    pub fn select(&self) -> bool {
        (self.io_bits & Button::Select) != 0
    }

    pub fn start(&self) -> bool {
        (self.io_bits & Button::Start) != 0
    }

    pub fn dright(&self) -> bool {
        (self.io_bits & Button::Dright) != 0
    }

    pub fn dleft(&self) -> bool {
        (self.io_bits & Button::Dleft) != 0
    }

    pub fn dup(&self) -> bool {
        (self.io_bits & Button::Dup) != 0
    }

    pub fn ddown(&self) -> bool {
        (self.io_bits & Button::Ddown) != 0
    }

    pub fn r(&self) -> bool {
        (self.io_bits & Button::R) != 0
    }

    pub fn l(&self) -> bool {
        (self.io_bits & Button::L) != 0
    }

    pub fn x(&self) -> bool {
        (self.io_bits & Button::X) != 0
    }

    pub fn y(&self) -> bool {
        (self.io_bits & Button::Y) != 0
    }

    pub fn none(&self) -> bool {
        self.io_bits == 0
    }
}

impl BitAnd<PressedButtons> for u32 {
    type Output = u32;

    fn bitand(self, rhs: PressedButtons) -> Self::Output {
        self & rhs.io_bits
    }
}

impl BitAnd<u32> for PressedButtons {
    type Output = u32;

    fn bitand(self, rhs: u32) -> Self::Output {
        self.io_bits & rhs
    }
}

impl PartialEq<PressedButtons> for u32 {
    fn eq(&self, other: &PressedButtons) -> bool {
        *self == other.io_bits
    }
}
