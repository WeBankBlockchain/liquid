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

use cfg_if::cfg_if;
use liquid_macro::seq;
use liquid_prelude::{string::String, vec::Vec};
use liquid_primitives::{
    types::{hash::HASH_LENGTH, *},
    Selector,
};
use scale::Encode;

pub trait FnOutput {
    type Output: Encode + 'static;
}

pub trait FnSelector {
    const SELECTOR: Selector;
}

pub trait FnMutability {
    const IS_MUT: bool;
}

pub trait Env {
    fn env(&self) -> crate::EnvAccess {
        crate::EnvAccess {}
    }
}

cfg_if! {
    if #[cfg(feature = "collaboration")] {
        #[allow(non_camel_case_types)]
        pub trait You_Should_Use_An_Valid_Contract_Type: Sized {
            type T = Self;
        }

        /// Every contract needs to implement this trait
        /// to get signers from runtime.
        pub trait AcquireSigners {
            fn acquire_signers(&self) -> liquid_prelude::collections::BTreeSet<&Address>;
        }

        #[allow(non_camel_case_types)]
        pub trait Parties_Should_Be_Address_Or_Address_Collection<'a>
        {
            fn acquire_addrs(self) -> Vec<&'a Address>;
        }

        impl<'a, T: 'a> Parties_Should_Be_Address_Or_Address_Collection<'a> for T
        where
            T: IntoIterator<Item = &'a Address>
        {
            fn acquire_addrs(self) -> Vec<&'a Address> {
                self.into_iter().collect()
            }
        }

        /// This function is used for type inference, because that in Rust there is no
        /// equivalent of `decltype` in C++ for now. With the help of this function, the
        /// error info can be simplified when the user want to acquire addresses from an
        /// invalid data structure.
        pub fn acquire_addrs<'a, T>(t: T) -> Vec<&'a Address>
        where
            T: Parties_Should_Be_Address_Or_Address_Collection<'a>,
        {
            <T as Parties_Should_Be_Address_Or_Address_Collection<'a>>::acquire_addrs(t)
        }

        pub trait ContractName {
            const CONTRACT_NAME: &'static str;
        }

        pub trait ContractType {
            type T;
        }

        pub trait ContractVisitor
        {
            type Contract: ContractName;
            type ContractId;

            fn fetch(&self) -> Self::Contract;
            fn sign_new_contract(contract: Self::Contract) -> Self::ContractId;

            fn inexistent_error(id: u32) {
                let mut error_info = String::from("the contract `");
                error_info.push_str(Self::Contract::CONTRACT_NAME);
                error_info.push_str("` with id `");
                use liquid_prelude::string::ToString;
                error_info.push_str(&id.to_string());
                error_info.push_str("` is not exist");
                crate::env::revert(&error_info);
            }

            fn abolished_error(id: u32) {
                let mut error_info = String::from("the contract `");
                error_info.push_str(Self::Contract::CONTRACT_NAME);
                error_info.push_str("` with id `");
                use liquid_prelude::string::ToString;
                error_info.push_str(&id.to_string());
                error_info.push_str("` had been abolished already");
                crate::env::revert(&error_info);
            }
        }
    }
}

#[allow(non_camel_case_types)]
pub trait You_Should_Use_An_Valid_Input_Type: Sized {
    type T = Self;
}

#[allow(non_camel_case_types)]
pub trait You_Should_Use_An_Valid_Output_Type: Sized {
    type T = Self;
}

#[cfg(feature = "contract")]
#[allow(non_camel_case_types)]
pub trait You_Should_Use_An_Valid_State_Type: Sized {
    type T = Self;
}

#[allow(non_camel_case_types)]
pub trait You_Should_Use_An_Valid_Topic_Type: Sized {
    type T = Self;

    fn topic(&self) -> Hash
    where
        Self: Encode,
    {
        let encoded = self.encode();
        let mut hash = [0u8; HASH_LENGTH];
        hash[(HASH_LENGTH - encoded.len())..].copy_from_slice(&encoded);
        hash.into()
    }
}

macro_rules! impl_basic_trait {
    ($($t:ty),*) => {
        $(
            impl You_Should_Use_An_Valid_Input_Type for $t {}
            impl You_Should_Use_An_Valid_Output_Type for $t {}
            #[cfg(feature = "contract")]
            impl You_Should_Use_An_Valid_State_Type for $t {}
        )*
    };
}

// The primary reason why we implement `You_Should_Use_An_Valid_Input_Type` for each valid type manually instead of using
// `impl<T> You_Should_Use_An_Valid_Input_Type for T where T: Encode` directly, is that if we choose latter one,
// the error information would be like following:
// ```
// error[E0277]: the trait bound `f32: parity_scale_codec::codec::WrapperTypeDecode` is not satisfied
//    |
// 12 |         pub fn noop(&self, value: f32) {}
//    |                                   ^^^ the trait `parity_scale_codec::codec::WrapperTypeDecode` is not implemented for `f32`
//    |
// = note: required because of the requirements on the impl of `Decode` for `f32`
// = note: required because of the requirements on the impl of `You_Should_Use_An_Valid_Input_Type` for `f32`
// ```
// The hint is some a little puzzling. On the contrary, if we choose the former way, the error
// information would be like following:
// ```
// error[E0277]: the trait bound `f32: You_Should_Use_An_Valid_Input_Type` is not satisfied
//    |
// 12 |         pub fn noop(&self, value: f32) {}
//    |                                   ^^^ the trait `You_Should_Use_An_Valid_Input_Type` is not implemented for `f32`
// ```
// For programmers the hint is straightforward enough. Hence, we decide to use former way for better
// readability.
// Another reason is that the event struct will also implement `Encode` trait, and if we adopt
// latter way, the event struct will be regard as an valid output type. That's not something we want to see
// happen.
impl_basic_trait! {
    u8,
    u16,
    u32,
    u64,
    u128,
    u256,
    i8,
    i16,
    i32,
    i64,
    i128,
    i256,
    bool,
    Address,
    String,
    Hash,
    Bytes,
    ()
}

impl<T> You_Should_Use_An_Valid_Input_Type for Vec<T> where
    T: You_Should_Use_An_Valid_Input_Type
{
}

impl<T> You_Should_Use_An_Valid_Output_Type for Vec<T> where
    T: You_Should_Use_An_Valid_Output_Type
{
}

#[cfg(feature = "contract")]
impl<T> You_Should_Use_An_Valid_State_Type for Vec<T> where
    T: You_Should_Use_An_Valid_State_Type
{
}

impl<T, const N: usize> You_Should_Use_An_Valid_Input_Type for [T; N] where
    T: You_Should_Use_An_Valid_Input_Type
{
}

impl<T, const N: usize> You_Should_Use_An_Valid_Output_Type for [T; N] where
    T: You_Should_Use_An_Valid_Output_Type
{
}

#[cfg(feature = "contract")]
impl<T, const N: usize> You_Should_Use_An_Valid_State_Type for [T; N] where
    T: You_Should_Use_An_Valid_State_Type
{
}

impl<T> You_Should_Use_An_Valid_Input_Type for Option<T> where
    T: You_Should_Use_An_Valid_Input_Type
{
}

impl<T> You_Should_Use_An_Valid_Output_Type for Option<T> where
    T: You_Should_Use_An_Valid_Output_Type
{
}

#[cfg(feature = "contract")]
impl<T> You_Should_Use_An_Valid_State_Type for Option<T> where
    T: You_Should_Use_An_Valid_State_Type
{
}

impl<T, E> You_Should_Use_An_Valid_Input_Type for Result<T, E>
where
    T: You_Should_Use_An_Valid_Input_Type,
    E: You_Should_Use_An_Valid_Input_Type,
{
}

impl<T, E> You_Should_Use_An_Valid_Output_Type for Result<T, E>
where
    T: You_Should_Use_An_Valid_Output_Type,
    E: You_Should_Use_An_Valid_Output_Type,
{
}

#[cfg(feature = "contract")]
impl<T, E> You_Should_Use_An_Valid_State_Type for Result<T, E>
where
    T: You_Should_Use_An_Valid_State_Type,
    E: You_Should_Use_An_Valid_State_Type,
{
}

macro_rules! impl_topic_trait {
    ($($t:ty),*) => {
        $(
            impl You_Should_Use_An_Valid_Topic_Type for $t {}
        )*
    };
}

impl_topic_trait!(
    u8,
    u16,
    u32,
    u64,
    u128,
    u256,
    i8,
    i16,
    i32,
    i64,
    i128,
    i256,
    bool,
    Address,
    ()
);

seq!(N in 1..=32 {
    #(
        impl_topic_trait!(Bytes#N);
        impl_basic_trait!(Bytes#N);
    )*
});

impl You_Should_Use_An_Valid_Topic_Type for String {
    fn topic(&self) -> Hash {
        liquid_primitives::hash::hash(self.as_bytes()).into()
    }
}

cfg_if! {
    if #[cfg(feature = "contract")] {
        // `__Liquid_Getter_Index_Placeholder` can only be used in getter for
        // `liquid_lang::storage::Value`
        use liquid_primitives::__Liquid_Getter_Index_Placeholder;
        impl You_Should_Use_An_Valid_Input_Type for __Liquid_Getter_Index_Placeholder {}

        #[cfg(feature = "contract-abi-gen")]
        pub trait GenerateAbi {
            fn generate_abi() -> liquid_abi_gen::ContractAbi;
        }
    } else if #[cfg(feature = "collaboration")] {
        #[cfg(feature = "collaboration-abi-gen")]
        pub trait GenerateAbi {
            fn generate_abi() -> liquid_abi_gen::CollaborationAbi;
        }
    }
}

macro_rules! impl_traits_for_tuple {
    ($first:tt,) => {
        impl<$first> You_Should_Use_An_Valid_Input_Type for ($first,)
        where
            $first: You_Should_Use_An_Valid_Input_Type
        {
        }

        impl<$first> You_Should_Use_An_Valid_Output_Type for ($first,)
        where
            $first: You_Should_Use_An_Valid_Output_Type
        {
        }

        #[cfg(feature = "contract")]
        impl<$first> You_Should_Use_An_Valid_State_Type for ($first,)
        where
            $first: You_Should_Use_An_Valid_State_Type
        {
        }
    };
    ($first:tt, $($rest:tt,)+) => {
        impl<$first, $($rest),+> You_Should_Use_An_Valid_Input_Type for ($first, $($rest),+)
        where
            $first: You_Should_Use_An_Valid_Input_Type,
            $($rest: You_Should_Use_An_Valid_Input_Type),+
        {
        }

        impl<$first, $($rest),+> You_Should_Use_An_Valid_Output_Type for ($first, $($rest),+)
        where
            $first: You_Should_Use_An_Valid_Output_Type,
            $($rest: You_Should_Use_An_Valid_Output_Type),+
        {
        }

        #[cfg(feature = "contract")]
        impl<$first, $($rest),+> You_Should_Use_An_Valid_State_Type for ($first, $($rest),+)
        where
            $first: You_Should_Use_An_Valid_Output_Type,
            $($rest: You_Should_Use_An_Valid_Output_Type),+
        {
        }

        impl_traits_for_tuple!($($rest,)+);
    };
}

// The max number of outputs of a contract's method is 18.
seq! (N in 0..18 {
    impl_traits_for_tuple!(#(T#N,)*);
});
