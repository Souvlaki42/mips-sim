use derive_more::{BitAnd, Eq, From, Into, Rem, Shr};
use std::{
    fmt::{Debug, Formatter, Result},
    ops::{Add, AddAssign, Sub},
};

#[derive(Clone, Copy, From, Into, Shr, BitAnd, Hash, Eq, Rem, PartialEq, PartialOrd, Ord)]
pub struct Address(pub u32);

impl Debug for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "0x{:08X}", self.0)
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

impl Sub<Address> for i32 {
    type Output = i32;
    fn sub(self, rhs: Address) -> Self::Output {
        self - rhs.0 as i32
    }
}

impl Sub<Address> for u32 {
    type Output = u32;
    fn sub(self, rhs: Address) -> Self::Output {
        self - rhs.0
    }
}

impl Sub<Address> for Address {
    type Output = Address;
    fn sub(self, rhs: Address) -> Self::Output {
        Address(self.0 - rhs.0)
    }
}

impl Sub<Address> for usize {
    type Output = usize;
    fn sub(self, rhs: Address) -> Self::Output {
        self - rhs.0 as usize
    }
}

impl Add<usize> for &Address {
    type Output = Address;
    fn add(self, offset: usize) -> Address {
        Address(self.0 + offset as u32)
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

impl From<Address> for i16 {
    fn from(value: Address) -> Self {
        value.0 as i16
    }
}

impl From<Address> for u16 {
    fn from(value: Address) -> Self {
        value.0 as u16
    }
}

impl PartialEq<i32> for Address {
    fn eq(&self, other: &i32) -> bool {
        self.0 as i32 == *other
    }
}
