use derive_more::{BitAnd, Eq, From, Into, Shr, Sub};
use std::{
    fmt::{Debug, Formatter, Result},
    ops::{Add, AddAssign},
};

#[derive(Clone, Copy, From, Into, Shr, BitAnd, Hash, PartialEq, Eq, Sub)]
pub struct Address(pub u32);

impl Debug for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_tuple("Address")
            .field(&format_args!("0x{:08X}", self.0))
            .finish()
    }
}

impl Add<usize> for Address {
    type Output = Address;
    fn add(self, offset: usize) -> Address {
        Address(self.0 + offset as u32)
    }
}

impl Add<u32> for Address {
    type Output = Address;
    fn add(self, offset: u32) -> Address {
        Address(self.0 + offset)
    }
}

impl AddAssign<usize> for Address {
    fn add_assign(&mut self, offset: usize) {
        self.0 += offset as u32;
    }
}

impl From<Address> for usize {
    fn from(addr: Address) -> usize {
        addr.0 as usize
    }
}

impl From<Address> for i32 {
    fn from(value: Address) -> Self {
        value.0 as i32
    }
}
