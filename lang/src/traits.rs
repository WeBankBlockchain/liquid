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
use liquid_primitives::{types::*, Selector};

cfg_if! {
    if #[cfg(feature = "solidity-compatible")] {
        use liquid_abi_codec::{Decode, Encode};
    } else {
        use scale::{Decode, Encode};
    }
}

pub trait FnInput {
    type Input: Decode + 'static;
}

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

        pub trait AcquireSigners {
            fn acquire_signers(&self) -> liquid_prelude::collections::BTreeSet<Address>;
        }

        #[allow(non_camel_case_types)]
        pub trait Can_Not_Select_Any_Account_Address_From_It {
            fn acquire_addrs(self) -> Vec<Address>;
        }

        impl<'a, T> Can_Not_Select_Any_Account_Address_From_It for T
        where
            T: IntoIterator<Item = &'a Address>
        {
            fn acquire_addrs(self) -> Vec<Address> {
                self.into_iter().copied().collect()
            }
        }

        pub trait ContractName {
            const CONTRACT_NAME: &'static str;
        }

        pub trait ContractType: Sized {
            type T = Self;
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
pub trait You_Should_Use_An_Valid_Return_Type: Sized {
    type T = Self;
}

#[allow(non_camel_case_types)]
pub trait You_Should_Use_An_Valid_Input_Type: Sized {
    type T = Self;
}

cfg_if! {
    if #[cfg(feature = "solidity-compatible")] {
        #[allow(non_camel_case_types)]
        pub trait You_Should_Use_An_Valid_Element_Type: Sized {
            type T = Self;
        }

        #[allow(non_camel_case_types)]
        pub trait You_Should_Use_An_Valid_InOut_Type: Sized {
            type T = Self;
        }

        #[allow(non_camel_case_types)]
        pub trait You_Should_Use_An_Valid_State_Type: Sized {
            type T = Self;
        }
    } else {
        #[allow(non_camel_case_types)]
        pub trait You_Should_Use_An_Valid_Field_Type: Sized {
            type T = Self;
        }
    }
}

#[allow(non_camel_case_types)]
pub trait You_Should_Use_An_Valid_Event_Data_Type: Sized {
    type T = Self;
}

#[allow(non_camel_case_types)]
pub trait You_Should_Use_An_Valid_Event_Topic_Type: Sized {
    type T = Self;

    fn topic(&self) -> Hash
    where
        Self: Encode,
    {
        self.encode().into()
    }
}

macro_rules! gen_basic_type_notations {
    ($t:ty) => {
        cfg_if! {
            if #[cfg(feature = "solidity-compatible")] {
                impl You_Should_Use_An_Valid_InOut_Type for $t {}
                impl You_Should_Use_An_Valid_State_Type for $t {}
                impl You_Should_Use_An_Valid_Element_Type for $t {}
            } else {
                impl You_Should_Use_An_Valid_Field_Type for $t {}
            }
        }
        impl You_Should_Use_An_Valid_Event_Data_Type for $t {}
        impl You_Should_Use_An_Valid_Return_Type for $t {}
        impl You_Should_Use_An_Valid_Input_Type for $t {}
    };
}

macro_rules! gen_type_notations {
    ($t:ty) => {
        gen_basic_type_notations!($t);

        impl You_Should_Use_An_Valid_Event_Topic_Type for $t {}
    };
}

macro_rules! impl_for_primitives {
    ($($t:ty),*) => {
        $(
            gen_type_notations!($t);
        )*
    };
}

impl_for_primitives!(
    u8, u16, u32, u64, u128, u256, i8, i16, i32, i64, i128, i256, bool, Address
);

gen_basic_type_notations!(Bytes);
seq!(N in 1..=32 {
    #(
        gen_type_notations!(Bytes#N);
    )*
});

gen_basic_type_notations!(String);
impl You_Should_Use_An_Valid_Event_Topic_Type for String {
    type T = Self;
    fn topic(&self) -> Hash {
        liquid_primitives::hash::hash(self.as_bytes()).into()
    }
}

cfg_if! {
    if #[cfg(feature = "solidity-compatible")] {
        impl<T> You_Should_Use_An_Valid_Element_Type for Vec<T> where
        T: You_Should_Use_An_Valid_Element_Type
        {
        }
        impl<T> You_Should_Use_An_Valid_Return_Type for Vec<T> where
            T: You_Should_Use_An_Valid_Element_Type
        {
        }
        impl<T> You_Should_Use_An_Valid_Input_Type for Vec<T> where
            T: You_Should_Use_An_Valid_Element_Type
        {
        }
        impl<T> You_Should_Use_An_Valid_Event_Data_Type for Vec<T> where
            T: You_Should_Use_An_Valid_Element_Type
        {
        }
        impl<T> You_Should_Use_An_Valid_InOut_Type for Vec<T> where
            T: You_Should_Use_An_Valid_Element_Type
        {
        }
        impl<T> You_Should_Use_An_Valid_State_Type for Vec<T> where
        T: You_Should_Use_An_Valid_Element_Type
        {
        }
        impl<T, const N: usize> You_Should_Use_An_Valid_Element_Type for [T; N] where
            T: You_Should_Use_An_Valid_Element_Type
        {
        }
        impl<T, const N: usize> You_Should_Use_An_Valid_Return_Type for [T; N] where
            T: You_Should_Use_An_Valid_Element_Type
        {
        }
        impl<T, const N: usize> You_Should_Use_An_Valid_Input_Type for [T; N] where
            T: You_Should_Use_An_Valid_Element_Type
        {
        }
        impl<T, const N: usize> You_Should_Use_An_Valid_InOut_Type for [T; N] where
            T: You_Should_Use_An_Valid_Element_Type
        {
        }
        impl<T, const N: usize> You_Should_Use_An_Valid_State_Type for [T; N] where
        T: You_Should_Use_An_Valid_Element_Type
        {
        }
        impl<T, const N: usize> You_Should_Use_An_Valid_Event_Data_Type for [T; N] where
            T: You_Should_Use_An_Valid_Element_Type
        {
        }
    } else {
        impl<T> You_Should_Use_An_Valid_Return_Type for Vec<T> where
            T: You_Should_Use_An_Valid_Return_Type
        {
        }
        impl<T> You_Should_Use_An_Valid_Input_Type for Vec<T> where
            T: You_Should_Use_An_Valid_Input_Type
        {
        }
        impl<T> You_Should_Use_An_Valid_Event_Data_Type for Vec<T> where
            T: You_Should_Use_An_Valid_Event_Data_Type
        {
        }
        impl<T> You_Should_Use_An_Valid_Field_Type for Vec<T> where
            T: You_Should_Use_An_Valid_Field_Type
        {
        }
        impl<T, const N: usize> You_Should_Use_An_Valid_Return_Type for [T; N] where
            T: You_Should_Use_An_Valid_Return_Type
        {
        }
        impl<T, const N: usize> You_Should_Use_An_Valid_Input_Type for [T; N] where
            T: You_Should_Use_An_Valid_Input_Type
        {
        }
        impl<T, const N: usize> You_Should_Use_An_Valid_Field_Type for [T; N] where
            T: You_Should_Use_An_Valid_Field_Type
        {
        }
        impl<T, const N: usize> You_Should_Use_An_Valid_Event_Data_Type for [T; N] where
            T: You_Should_Use_An_Valid_Event_Data_Type
        {
        }
    }
}

cfg_if! {
    if #[cfg(feature = "solidity-compatible")] {
        /// `()` can be used to indicate returning nothing.
        impl You_Should_Use_An_Valid_Return_Type for () {}
    } else {
        gen_basic_type_notations!(());
    }
}

cfg_if! {
    if #[cfg(feature = "contract")] {
        /// `__LIQUID_GETTER_INDEX_PLACEHOLDER` can only be used in getter for
        /// `liquid_lang::storage::Value`
        use liquid_primitives::__LIQUID_GETTER_INDEX_PLACEHOLDER;
        impl You_Should_Use_An_Valid_Input_Type for __LIQUID_GETTER_INDEX_PLACEHOLDER {}
    }
}

/// For tuple types, implement `You_Should_Use_An_Valid_Return_Type` only.
/// Due to that tuple types can only be used in return value of a contract's method.
macro_rules! impl_for_tuple {
    ($first:tt,) => {
        cfg_if! {
            if #[cfg(feature = "solidity-compatible")] {
                impl<$first> You_Should_Use_An_Valid_Return_Type for ($first,)
                where
                    $first: You_Should_Use_An_Valid_Element_Type
                {
                }
            } else {
                impl<$first> You_Should_Use_An_Valid_Return_Type for ($first,)
                where
                    $first: You_Should_Use_An_Valid_Return_Type
                {
                }
                impl<$first> You_Should_Use_An_Valid_Input_Type for ($first,)
                where
                    $first: You_Should_Use_An_Valid_Input_Type
                {
                }
                impl<$first> You_Should_Use_An_Valid_Field_Type for ($first,)
                where
                    $first: You_Should_Use_An_Valid_Field_Type
                {
                }
                impl<$first> You_Should_Use_An_Valid_Event_Data_Type for ($first,)
                where $first: You_Should_Use_An_Valid_Event_Data_Type
                {
                }
            }
        }
    };
    ($first:tt, $($rest:tt,)+) => {
        cfg_if! {
            if #[cfg(feature = "solidity-compatible")] {
                impl<$first, $($rest),+> You_Should_Use_An_Valid_Return_Type for ($first, $($rest),+)
                where
                    $first: You_Should_Use_An_Valid_Element_Type,
                    $($rest: You_Should_Use_An_Valid_Element_Type),+
                {
                }
            } else {
                impl<$first, $($rest),+> You_Should_Use_An_Valid_Return_Type for ($first, $($rest),+)
                where
                    $first: You_Should_Use_An_Valid_Return_Type,
                    $($rest: You_Should_Use_An_Valid_Return_Type),+
                {
                }
                impl<$first, $($rest),+> You_Should_Use_An_Valid_Input_Type for ($first, $($rest),+)
                where
                    $first: You_Should_Use_An_Valid_Input_Type,
                    $($rest: You_Should_Use_An_Valid_Input_Type),+
                {
                }
                impl<$first, $($rest),+> You_Should_Use_An_Valid_Field_Type for ($first, $($rest),+)
                where
                    $first: You_Should_Use_An_Valid_Field_Type,
                    $($rest: You_Should_Use_An_Valid_Field_Type),+
                {
                }
                impl<$first, $($rest),+> You_Should_Use_An_Valid_Event_Data_Type for ($first, $($rest),+)
                where
                    $first: You_Should_Use_An_Valid_Event_Data_Type,
                    $($rest: You_Should_Use_An_Valid_Event_Data_Type),+
                {
                }
            }
        }

        impl_for_tuple!($($rest,)+);
    };
}

// The max number of outputs of a contract's method is 16.
seq! (N in 0..16 {
    impl_for_tuple!(#(T#N,)*);
});

cfg_if! {
    if #[cfg(not(feature = "solidity-compatible"))] {
        impl<T> You_Should_Use_An_Valid_Return_Type for Option<T>
        where
            T: You_Should_Use_An_Valid_Return_Type
        {
        }
        impl<T> You_Should_Use_An_Valid_Input_Type for Option<T>
        where
            T: You_Should_Use_An_Valid_Input_Type
        {
        }
        impl<T> You_Should_Use_An_Valid_Field_Type for Option<T>
        where
            T: You_Should_Use_An_Valid_Field_Type
        {
        }
        impl<T> You_Should_Use_An_Valid_Event_Data_Type for Option<T>
        where
            T: You_Should_Use_An_Valid_Event_Data_Type
        {
        }

        impl<T, E> You_Should_Use_An_Valid_Return_Type for Result<T, E>
        where
            T: You_Should_Use_An_Valid_Return_Type,
            E: You_Should_Use_An_Valid_Return_Type,
        {
        }
        impl<T, E> You_Should_Use_An_Valid_Input_Type for Result<T, E>
        where
            T: You_Should_Use_An_Valid_Input_Type,
            E: You_Should_Use_An_Valid_Input_Type,
        {
        }
        impl<T, E> You_Should_Use_An_Valid_Field_Type for Result<T, E>
        where
            T: You_Should_Use_An_Valid_Field_Type,
            E: You_Should_Use_An_Valid_Field_Type,
        {
        }
        impl<T, E> You_Should_Use_An_Valid_Event_Data_Type for Result<T, E>
        where
            T: You_Should_Use_An_Valid_Event_Data_Type,
            E: You_Should_Use_An_Valid_Event_Data_Type
        {
        }
    }
}

cfg_if! {
    if #[cfg(feature = "contract")] {
        #[cfg(feature = "liquid-abi-gen")]
        pub trait GenerateABI {
            fn generate_abi() -> liquid_abi_gen::ContractABI;
        }
    } else if #[cfg(feature = "collaboration")] {
        #[cfg(feature = "liquid-abi-gen")]
        pub trait GenerateABI {
            fn generate_abi() -> liquid_abi_gen::CollaborationABI;
        }
    }
}
