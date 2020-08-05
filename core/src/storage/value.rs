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

use crate::storage::{
    Bind, CachedCell, Flush, You_Should_Use_A_Container_To_Wrap_Your_State_Field_In_Storage,
};
use scale::Encode;

#[derive(Debug)]
pub struct Value<T> {
    cell: CachedCell<T>,
}

impl<T> Bind for Value<T> {
    fn bind_with(key: &[u8]) -> Self {
        Self {
            cell: CachedCell::new(key),
        }
    }
}

impl<T> Flush for Value<T>
where
    T: Encode,
{
    fn flush(&mut self) {
        self.cell.flush();
    }
}

impl<T> Value<T>
where
    T: scale::Codec,
{
    pub fn initialize(&mut self, input: T) {
        if self.cell.get().is_none() {
            self.set(input);
        }
    }

    pub fn set(&mut self, new_val: T) {
        self.cell.set(new_val);
    }

    pub fn get(&self) -> &T {
        self.cell.get().unwrap()
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.cell.get_mut().unwrap()
    }

    pub fn mutate_with<F>(&mut self, f: F) -> &T
    where
        F: FnOnce(&mut T),
    {
        self.cell.mutate_with(f).unwrap()
    }
}

impl<T, R> AsRef<R> for Value<T>
where
    T: AsRef<R> + scale::Codec,
{
    fn as_ref(&self) -> &R {
        self.get().as_ref()
    }
}

impl<T> core::ops::Deref for Value<T>
where
    T: scale::Codec,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T> core::ops::DerefMut for Value<T>
where
    T: scale::Codec,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

macro_rules! impl_ops_for_value {
    ($trait_name:ident; $fn_name:ident) => {
        impl<T> core::ops::$trait_name<T> for &Value<T>
        where
            T: core::ops::$trait_name<T> + Copy + scale::Codec,
        {
            type Output = <T as core::ops::$trait_name>::Output;

            fn $fn_name(self, rhs: T) -> Self::Output {
                (*self.get()).$fn_name(rhs)
            }
        }

        impl<T> core::ops::$trait_name for &Value<T>
        where
            T: core::ops::$trait_name<T> + Copy + scale::Codec,
        {
            type Output = <T as core::ops::$trait_name>::Output;

            fn $fn_name(self, rhs: Self) -> Self::Output {
                (*self.get()).$fn_name((*rhs.get()))
            }
        }

        paste::item! {
            impl<T> core::ops::[<$trait_name Assign>]<T> for Value<T>
            where
                T: core::ops::[<$trait_name Assign>]<T> + scale::Codec,
            {
                fn [<$fn_name _assign>](&mut self, rhs: T) {
                    self.mutate_with(|val| {
                        (*val).[<$fn_name _assign>](rhs);
                    });
                }
            }
        }

        paste::item! {
            impl<T> core::ops::[<$trait_name Assign>]<&Self> for Value<T>
            where
                T: core::ops::[<$trait_name Assign>]<T> + Copy + scale::Codec,
            {
                fn [<$fn_name _assign>](&mut self, rhs: &Self) {
                    self.mutate_with(|val| {
                        (*val).[<$fn_name _assign>](*rhs.get());
                    });
                }
            }
        }
    };
}

impl_ops_for_value!(Add;add);
impl_ops_for_value!(Sub;sub);
impl_ops_for_value!(Mul;mul);
impl_ops_for_value!(Div;div);
impl_ops_for_value!(Rem;rem);
impl_ops_for_value!(BitAnd;bitand);
impl_ops_for_value!(BitOr;bitor);
impl_ops_for_value!(BitXor;bitxor);
impl_ops_for_value!(Shl;shl);
impl_ops_for_value!(Shr;shr);

impl<T> core::ops::Neg for &Value<T>
where
    T: core::ops::Neg + Copy + scale::Codec,
{
    type Output = <T as core::ops::Neg>::Output;

    fn neg(self) -> Self::Output {
        -(*self.get())
    }
}

impl<T> core::ops::Not for &Value<T>
where
    T: core::ops::Not + Copy + scale::Codec,
{
    type Output = <T as core::ops::Not>::Output;

    fn not(self) -> Self::Output {
        !(*self.get())
    }
}

impl<T, I> core::ops::Index<I> for Value<T>
where
    T: core::ops::Index<I> + scale::Codec,
{
    type Output = <T as core::ops::Index<I>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        &((self.get())[index])
    }
}

impl<T, I> core::ops::IndexMut<I> for Value<T>
where
    T: core::ops::IndexMut<I> + scale::Codec,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut (self.get_mut()[index])
    }
}

impl<T> PartialEq<T> for Value<T>
where
    T: PartialEq + scale::Codec,
{
    fn eq(&self, rhs: &T) -> bool {
        self.get().eq(rhs)
    }
}

impl<T> PartialEq for Value<T>
where
    T: PartialEq + scale::Codec,
{
    fn eq(&self, rhs: &Self) -> bool {
        self.get().eq(rhs.get())
    }
}

impl<T> Eq for Value<T> where T: Eq + scale::Codec {}

use core::cmp::Ordering;

impl<T> PartialOrd<T> for Value<T>
where
    T: PartialOrd + scale::Codec,
{
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        self.get().partial_cmp(other)
    }
}

impl<T> PartialOrd for Value<T>
where
    T: PartialOrd + scale::Codec,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.get().partial_cmp(other.get())
    }
}

impl<T> Ord for Value<T>
where
    T: Ord + scale::Codec,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.get().cmp(other.get())
    }
}

#[cfg(test)]
impl<T> core::fmt::Display for Value<T>
where
    T: core::fmt::Display + scale::Codec,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.get().fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_ops {
        ($test_name:ident, $tok:tt, $tok_eq:tt) => {
            #[test]
            fn $test_name() {
                let lhs = 3529;
                let rhs = 7;
                let mut v1 = Value::<i32>::bind_with(b"v1");
                v1.set(lhs);
                let mut v2 = Value::<i32>::bind_with(b"v2");
                v2.set(rhs);

                assert_eq!(v1, lhs);
                assert_eq!(v2, rhs);
                assert_eq!(&v1 $tok rhs, lhs $tok rhs);
                assert_eq!(&v1 $tok &v2, lhs $tok rhs);
            }

            paste::item!{
                #[test]
                fn [<$test_name _assign>]() {
                    let lhs = 3529;
                    let rhs = 7;
                    let mut v1 = Value::<i32>::bind_with(b"v1");
                    v1.set(lhs);
                    let mut v2 = Value::<i32>::bind_with(b"v2");
                    v2.set(rhs);

                    assert_eq!(v1, lhs);
                    assert_eq!(v2, rhs);
                    assert_eq!({
                        v1 $tok_eq rhs;
                        &v1
                    }, &(lhs $tok rhs));
                    assert_eq!({
                        v1 $tok_eq &v2;
                        &v1
                    }, &(lhs $tok rhs $tok rhs));
                }
            }
        };
    }

    test_ops!(test_add, +, +=);
    test_ops!(test_sub, -, -=);
    test_ops!(test_mul, *, *=);
    test_ops!(test_div, /, /=);
    test_ops!(test_rem, %, %=);
    test_ops!(test_bitand, &, &=);
    test_ops!(test_bitor, |, |=);
    test_ops!(test_bitxor, ^, ^=);
    test_ops!(test_shl, <<, <<=);
    test_ops!(test_shr, >>, >>=);

    #[test]
    fn test_neg() {
        let mut v1 = Value::<i32>::bind_with(b"v1");
        v1.set(1);

        assert_eq!(v1, 1);
        assert_eq!(-&v1, -1);
    }

    #[test]
    fn test_not() {
        let mut v1 = Value::<bool>::bind_with(b"v1");
        v1.set(true);

        assert_eq!(v1, true);
        assert_eq!(!&v1, false);
    }

    #[test]
    fn test_eq_ord() {
        let mut v1 = Value::<i32>::bind_with(b"v1");
        v1.set(35);
        let mut v2 = Value::<i32>::bind_with(b"v2");
        v2.set(35);
        let mut v3 = Value::<i32>::bind_with(b"v3");
        v3.set(29);

        // Eq & Ne
        assert!(v1 == v2);
        assert!(v2 != v3);

        // Great-Than
        assert!(!(v1 < v2));
        assert!(v2 > v3);
        assert!(v1 > v3);

        // Great-Than or Eq
        assert!(v1 >= v2);
        assert!(v2 >= v3);
        assert!(v1 >= v3);
    }

    #[test]
    fn test_index() {
        let mut v1 = Value::<Vec<i32>>::bind_with(b"v");
        v1.set(vec![0i32, 1, 2, 3]);
        v1[2] = 5;
        v1.flush();

        let v2 = Value::<Vec<i32>>::bind_with(b"v");
        assert_eq!(v2[2], 5);
    }

    #[test]
    fn test_deref() {
        let mut v1 = Value::<i32>::bind_with(b"v");
        v1.set(2);
        *v1 = 3;
        v1.flush();

        let v2 = Value::<i32>::bind_with(b"v");
        assert_eq!(*v2, 3);
    }
}

impl<T> You_Should_Use_A_Container_To_Wrap_Your_State_Field_In_Storage for Value<T> {}
