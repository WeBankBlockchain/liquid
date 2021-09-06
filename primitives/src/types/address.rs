// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::Error;
use core::{
    convert::AsRef,
    ops::{Deref, DerefMut},
};
use liquid_prelude::{
    str::FromStr,
    string::{String, ToString},
};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, scale::Decode, scale::Encode, Hash)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Address(String);

impl Address {
    #[allow(dead_code)]
    pub fn empty() -> Self {
        Self(String::new())
    }
}

impl Default for Address {
    fn default() -> Self {
        Self::empty()
    }
}

impl Deref for Address {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Address {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<String> for Address {
    fn from(addr: String) -> Self {
        Self(addr)
    }
}

impl<S> From<&S> for Address
where
    S: AsRef<str> + ?Sized,
{
    fn from(addr: &S) -> Self {
        Self(addr.as_ref().to_string())
    }
}

#[allow(dead_code)]
pub struct AddressIter {
    ptr: core::ptr::NonNull<Address>,
    end: *const Address,
}

impl FromStr for Address {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl<'a> IntoIterator for &'a Address {
    type Item = &'a Address;
    type IntoIter = core::slice::Iter<'a, Address>;

    fn into_iter(self) -> Self::IntoIter {
        unsafe {
            let ptr: *const Address = self;
            let iter = AddressIter {
                ptr: core::ptr::NonNull::new_unchecked(ptr as *mut Address),
                end: ptr.add(1),
            };
            core::mem::transmute::<AddressIter, Self::IntoIter>(iter)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let _: Address = "/usr/bin/".into();

        let addr_str = String::from("/usr/bin/");
        let _: Address = addr_str.into();

        let addr_str = String::from("/usr/bin/");
        let addr: Address = (&addr_str).into();
        assert_eq!(addr.as_bytes(), addr_str.as_bytes());

        assert_eq!(Address::empty().len(), 0);
    }
}
