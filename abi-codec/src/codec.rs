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

use core::mem;
use liquid_macro::seq;
use liquid_prelude::{
    string::String,
    vec::{from_elem, Vec},
};

#[cfg(feature = "std")]
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Error(&'static str);

#[cfg(not(feature = "std"))]
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Error;

impl From<&'static str> for Error {
    #[cfg(feature = "std")]
    fn from(s: &'static str) -> Error {
        Error(s)
    }

    #[cfg(not(feature = "std"))]
    fn from(_: &'static str) -> Error {
        Error
    }
}

#[cfg(feature = "std")]
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub const WORD_SIZE: usize = 32;
pub type Word = [u8; WORD_SIZE];

/// Trait that allows reading of data into a slice.
pub trait Input {
    /// Read the exact number of words required to fill the given buffer.
    fn read_words(&mut self, into: &mut [Word]) -> Result<(), Error>;

    /// Read the exact number of bytes required to fill the given buffer.
    fn read_bytes(&mut self, into: &mut [u8]) -> Result<(), Error>;

    fn read_byte(&mut self) -> Result<u8, Error>;

    /// Should return the remaining length of the input data.
    fn remaining_len(&self) -> usize;
}

impl Input for &[u8] {
    fn read_words(&mut self, into: &mut [Word]) -> Result<(), Error> {
        let len = into.len();
        if len * WORD_SIZE > self.len() {
            return Err("Not enough data to fill buffer".into());
        }

        for word in into.iter_mut() {
            word.copy_from_slice(&self[..WORD_SIZE]);
            *self = &self[WORD_SIZE..];
        }
        Ok(())
    }

    fn read_bytes(&mut self, into: &mut [u8]) -> Result<(), Error> {
        let len = into.len();
        if into.len() > self.len() {
            return Err("Not enough data to fill buffer".into());
        }

        into.copy_from_slice(&self[..into.len()]);
        *self = &self[len..];
        Ok(())
    }

    fn read_byte(&mut self) -> Result<u8, Error> {
        let mut buf: [u8; 1] = Default::default();
        self.read_bytes(&mut buf[..])?;
        Ok(buf[0])
    }

    fn remaining_len(&self) -> usize {
        self.len()
    }
}

/// Trait that allows writing of data into self.
pub trait Output: Sized {
    /// Write to the output
    fn write(&mut self, bytes: &[u8]);
}

impl Output for Vec<u8> {
    fn write(&mut self, bytes: &[u8]) {
        self.extend_from_slice(bytes);
    }
}

pub trait TypeInfo {
    /// Indicate the type is whether a certain dynamic type or not.
    #[inline(always)]
    fn is_dynamic() -> bool {
        false
    }

    /// If the type is a certain static type, this method will return
    /// the size after the type being encoded.
    /// Please **DO NOT** use its default implementation if the type is a
    /// certain dynamic type.
    #[inline(always)]
    fn size_hint() -> u32 {
        WORD_SIZE as u32
    }
}

pub enum Mediate {
    Raw(Vec<Word>),
    Prefixed(Vec<Word>),
    RawTuple(Vec<Mediate>),
    PrefixedTuple(Vec<Mediate>),
    PrefixedArrayWithLength(Vec<Mediate>),
}

fn u32_to_word(value: u32) -> Word {
    let mut buf = [0x00; WORD_SIZE];
    buf[28..].copy_from_slice(&value.to_be_bytes());
    buf
}

impl Mediate {
    fn head_len(&self) -> usize {
        match *self {
            Mediate::Raw(ref raw) => raw.len() * WORD_SIZE,
            Mediate::Prefixed(_)
            | Mediate::PrefixedTuple(_)
            | Mediate::PrefixedArrayWithLength(_) => WORD_SIZE,
            Mediate::RawTuple(ref mediates) => mediates.len() * WORD_SIZE,
        }
    }

    fn tail_len(&self) -> usize {
        match *self {
            Mediate::Raw(_) | Mediate::RawTuple(_) => 0,
            Mediate::Prefixed(ref prefixed) => prefixed.len() * WORD_SIZE,
            Mediate::PrefixedTuple(ref mediates) => mediates
                .iter()
                .fold(0, |acc, m| acc + m.head_len() + m.tail_len()),
            Mediate::PrefixedArrayWithLength(ref mediates) => mediates
                .iter()
                .fold(WORD_SIZE, |acc, m| acc + m.head_len() + m.tail_len()),
        }
    }

    fn head(&self, suffix_offset: u32) -> Vec<Word> {
        match *self {
            Mediate::Raw(ref raw) => raw.clone(),
            Mediate::Prefixed(_)
            | Mediate::PrefixedTuple(_)
            | Mediate::PrefixedArrayWithLength(_) => {
                [u32_to_word(suffix_offset)].to_vec()
            }
            Mediate::RawTuple(ref raw) => raw
                .iter()
                .map(|mediate| mediate.head(0))
                .flatten()
                .collect(),
        }
    }

    fn tail(&self) -> Vec<Word> {
        match *self {
            Mediate::Raw(_) | Mediate::RawTuple(_) => Vec::new(),
            Mediate::Prefixed(ref raw) => raw.clone(),
            Mediate::PrefixedTuple(ref mediates) => encode_head_tail(mediates),
            Mediate::PrefixedArrayWithLength(ref mediates) => {
                // + `WORD_SIZE` added to offset represents len of the array prepanded to tail
                let mut result = [u32_to_word(mediates.len() as u32)].to_vec();
                let head_tail = encode_head_tail(mediates);
                result.extend(head_tail);
                result
            }
        }
    }
}

pub trait MediateEncode {
    fn encode(&self) -> Mediate;
}

pub struct DecodeResult<T: Sized> {
    pub value: T,
    pub new_offset: usize,
}

pub trait MediateDecode: Sized {
    fn decode(slices: &[Word], offset: usize) -> Result<DecodeResult<Self>, Error>;
}

impl MediateEncode for bool {
    fn encode(&self) -> Mediate {
        let mut buf = [0x00; WORD_SIZE];
        if *self {
            buf[WORD_SIZE - 1] = 1;
        }
        Mediate::Raw([buf].to_vec())
    }
}

impl TypeInfo for bool {}

pub fn peek(slices: &[Word], position: usize) -> Result<&Word, Error> {
    match slices.get(position) {
        Some(word) => Ok(word),
        None => Err("Unable to peek slices".into()),
    }
}

#[allow(dead_code)]
struct BytesTaken {
    bytes: Vec<u8>,
    new_offset: usize,
}

fn take(slices: &[Word], position: usize, len: usize) -> Result<BytesTaken, Error> {
    let words_len = (len + WORD_SIZE - 1) / WORD_SIZE;

    let mut words = Vec::with_capacity(words_len);
    for i in 0..words_len {
        let slice = peek(slices, position + i)?;
        words.push(slice);
    }

    let bytes = words
        .into_iter()
        .flat_map(|slice| slice.to_vec())
        .take(len)
        .collect();

    Ok(BytesTaken {
        bytes,
        new_offset: position + words_len,
    })
}

impl MediateDecode for bool {
    fn decode(slices: &[Word], offset: usize) -> Result<DecodeResult<Self>, Error> {
        let slice = peek(slices, offset)?;

        if !slice[..(WORD_SIZE - 1)].iter().all(|x| *x == 0) {
            Err("Invalid boolean representation".into())
        } else {
            let new_offset = offset + 1;
            match slice[WORD_SIZE - 1] {
                0 => Ok(DecodeResult {
                    value: false,
                    new_offset,
                }),
                1 => Ok(DecodeResult {
                    value: true,
                    new_offset,
                }),
                _ => Err("Invalid boolean representation".into()),
            }
        }
    }
}

macro_rules! from_word_to_integer {
    ($( $t:ty ),*) => { $(
        paste::item! {
            pub fn [<as_ $t>] (buf: &Word) -> Result<$t, Error> {
                const TYPE_SIZE: usize = mem::size_of::<$t>();
                let error_info: &'static str = concat!("Invalid ", stringify!($t), " representation");
                let signed = (buf[WORD_SIZE - TYPE_SIZE] & 0x80u8) != 0;
                if !buf[..(WORD_SIZE - TYPE_SIZE)].iter().all(|x| {
                    // unused comparisons will be optimized by compiler
                    #[allow(unused_comparisons)]
                    if <$t>::MIN < 0 {
                        if signed {
                            *x == 0xffu8
                        } else {
                            *x == 0
                        }
                    } else {
                        *x == 0
                    }
                }) {
                    return Err(error_info.into());
                }

                let mut slice: [u8; TYPE_SIZE] = Default::default();
                slice.clone_from_slice(&buf[(WORD_SIZE - TYPE_SIZE)..]);
                let res: $t = <$t>::from_be_bytes(slice);

                Ok(res)
            }
        })*
    };
}

from_word_to_integer!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);

macro_rules! impl_integer {
    ($( $t:ty ),*) => { $(
        paste::item! {
            impl MediateEncode for $t {
                fn encode(&self) -> Mediate {
                    // unused comparisons will be optimized by compiler
                    #[allow(unused_comparisons)]
                    let mut buf = if *self >=0 {
                        [0x00; WORD_SIZE]
                    } else {
                        [0xff; WORD_SIZE]
                    };

                    let be = self.to_be_bytes();
                    buf[(WORD_SIZE - be.len())..].copy_from_slice(&be);
                    Mediate::Raw([buf].to_vec())
                }
            }

            impl MediateDecode for $t {
                fn decode(slices: &[Word], offset: usize) -> Result<DecodeResult<Self>, Error> {
                    let slice = peek(slices, offset)?;
                    let value = [< as_ $t >](slice)?;
                    Ok(DecodeResult{
                        value,
                        new_offset: offset + 1
                    })
                }
            }

            impl TypeInfo for $t {}
        })*
    };
}

impl_integer!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);

fn encode_bytes(bytes: &[u8]) -> Vec<Word> {
    let mut res = [u32_to_word(bytes.len() as u32)].to_vec();
    res.extend(pad_bytes(bytes));
    res
}

fn pad_bytes(bytes: &[u8]) -> Vec<Word> {
    let len = (bytes.len() + WORD_SIZE - 1) / WORD_SIZE;
    let mut res = Vec::with_capacity(len);
    for i in 0..len {
        let offset = i * WORD_SIZE;
        let mut padded = [0x00; WORD_SIZE];
        let copy_end = if i != len - 1 {
            WORD_SIZE
        } else {
            match bytes.len() % WORD_SIZE {
                0 => WORD_SIZE,
                copy_end => copy_end,
            }
        };

        padded[..copy_end].copy_from_slice(&bytes[offset..(offset + copy_end)]);
        res.push(padded);
    }

    res
}

impl MediateEncode for String {
    fn encode(&self) -> Mediate {
        Mediate::Prefixed(encode_bytes(self.as_bytes()))
    }
}

impl MediateDecode for String {
    fn decode(slices: &[Word], offset: usize) -> Result<DecodeResult<Self>, Error> {
        let offset_slice = peek(slices, offset)?;
        let len_offset = (as_u32(offset_slice)? / (WORD_SIZE as u32)) as usize;

        let len_slice = peek(slices, len_offset)?;
        let len = as_u32(len_slice)? as usize;

        let taken = take(slices, len_offset + 1, len)?;
        Ok(DecodeResult {
            value: String::from_utf8_lossy(taken.bytes.as_slice()).into_owned(),
            new_offset: offset + 1,
        })
    }
}

impl TypeInfo for String {
    #[inline(always)]
    fn is_dynamic() -> bool {
        true
    }

    #[inline(always)]
    fn size_hint() -> u32 {
        unreachable!()
    }
}

impl<T> MediateEncode for Vec<T>
where
    T: MediateEncode,
{
    fn encode(&self) -> Mediate {
        let mediates = self.iter().map(|elem| elem.encode()).collect::<_>();
        Mediate::PrefixedArrayWithLength(mediates)
    }
}

impl<T> MediateDecode for Vec<T>
where
    T: MediateDecode,
{
    fn decode(slices: &[Word], offset: usize) -> Result<DecodeResult<Self>, Error> {
        let offset_slice = peek(slices, offset)?;
        let len_offset = (as_u32(offset_slice)? / (WORD_SIZE as u32)) as usize;
        let len_slice = peek(slices, len_offset)?;
        let len = as_u32(len_slice)? as usize;

        let tail = &slices[len_offset + 1..];
        let mut ret = Vec::with_capacity(len);
        let mut new_offset = 0;

        for _ in 0..len {
            let elem = <T as MediateDecode>::decode(&tail, new_offset)?;
            new_offset = elem.new_offset;
            ret.push(elem.value);
        }

        Ok(DecodeResult {
            value: ret,
            new_offset: offset + 1,
        })
    }
}

impl<T> TypeInfo for Vec<T> {
    #[inline(always)]
    fn is_dynamic() -> bool {
        true
    }

    #[inline(always)]
    fn size_hint() -> u32 {
        unreachable!()
    }
}

pub trait Encode {
    fn encode_to<T: Output>(&self, dest: &mut T) {
        dest.write(&self.encode());
    }

    fn encode(&self) -> Vec<u8>;
}

pub trait Decode: Sized {
    fn decode<I: Input>(input: &mut I) -> Result<Self, Error>;
}

pub trait Codec: Encode + Decode {}

pub fn encode_head_tail(mediates: &[Mediate]) -> Vec<Word> {
    let heads_len = mediates.iter().fold(0, |acc, m| acc + m.head_len());

    let (mut result, len) = mediates.iter().fold(
        (Vec::with_capacity(heads_len), heads_len),
        |(mut acc, offset), m| {
            acc.extend(m.head(offset as u32));
            (acc, offset + m.tail_len())
        },
    );

    let tails =
        mediates
            .iter()
            .fold(Vec::with_capacity(len - heads_len), |mut acc, m| {
                acc.extend(m.tail());
                acc
            });

    result.extend(tails);
    result
}

macro_rules! impl_tuple {
    (
        $first_ty:ident,
    ) => {
        impl<$first_ty: MediateEncode> Encode for ($first_ty,) {
            fn encode(&self) -> Vec<u8> {
                encode_head_tail(&[<$first_ty as MediateEncode>::encode(&self.0)]).iter().flat_map(|word| word.to_vec()).collect()
            }
        }

        impl<$first_ty: MediateEncode> Encode for $first_ty {
            fn encode(&self) -> Vec<u8> {
                encode_head_tail(&[<$first_ty as MediateEncode>::encode(&self)]).iter().flat_map(|word| word.to_vec()).collect()
            }
        }

        impl<$first_ty: MediateDecode> Decode for ($first_ty,) {
            fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
                let size = input.remaining_len();
                if size & (WORD_SIZE - 1) > 0 {
                    return Err("Invalid data size, 
                    which should be multiples of WORD_SIZE".into());
                }

                let len = size / WORD_SIZE;
                let mut buf = from_elem([0u8; WORD_SIZE], len);
                input.read_words(buf.as_mut_slice())?;
                match <$first_ty as MediateDecode>::decode(buf.as_slice(), 0) {
                    Ok(r) => Ok((r.value,)),
                    Err(e) => Err(e),
                }
            }
        }

        impl<$first_ty: MediateDecode> Decode for $first_ty {
            fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
                let size = input.remaining_len();
                if size & (WORD_SIZE - 1) > 0 {
                    return Err("Invalid data size, 
                    which should be multiples of WORD_SIZE".into());
                }

                let len = size / WORD_SIZE;
                let mut buf = from_elem([0u8; WORD_SIZE], len);
                input.read_words(buf.as_mut_slice())?;
                match <$first_ty as MediateDecode>::decode(buf.as_slice(), 0) {
                    Ok(r) => Ok(r.value),
                    Err(e) => Err(e),
                }
            }
        }
    };
    (
        $first_ty:ident, $( $rest_ty:ident, )+
    ) => {
        impl<$first_ty: MediateEncode, $( $rest_ty: MediateEncode),+> Encode for ($first_ty, $( $rest_ty ),+) {
            fn encode(&self) -> Vec<u8> {
                let mut mediates: Vec<Mediate> = Vec::new();
                let (
                    ref $first_ty,
                    $( ref $rest_ty ),+
                ) = *self;

                mediates.push($first_ty.encode());
                $( mediates.push($rest_ty.encode()); )+
                encode_head_tail(&mediates).iter().flat_map(|word| word.to_vec()).collect()
            }
        }

        #[allow(unused_assignments)]
        impl<$first_ty: MediateDecode, $( $rest_ty: MediateDecode ),+> Decode for ($first_ty, $( $rest_ty ),+) {
            fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
                let size = input.remaining_len();
                debug_assert!(size % WORD_SIZE == 0);

                let len = size / WORD_SIZE;
                let mut buf = from_elem([0u8; WORD_SIZE], len);
                input.read_words(buf.as_mut_slice())?;
                let mut offset = 0;

                let $first_ty = match <$first_ty>::decode(buf.as_slice(), offset) {
                    Ok(r) => r,
                    Err(e) => return Err(e),
                };
                offset = $first_ty.new_offset;

                $(
                    let $rest_ty = match <$rest_ty>::decode(buf.as_slice(), offset) {
                        Ok(r) => r,
                        Err(e) => return Err(e),
                    };
                    offset = $rest_ty.new_offset;
                )+

                Ok(
                    (
                        $first_ty.value,
                        $( $rest_ty.value ),+
                    )
                )
            }
        }

        impl<$first_ty: TypeInfo, $( $rest_ty: TypeInfo ),+> TypeInfo for ($first_ty, $( $rest_ty ),+) {
            #[inline(always)]
            fn is_dynamic() -> bool {
                $first_ty::is_dynamic() $( || $rest_ty::is_dynamic() )+
            }

            #[inline]
            fn size_hint() -> u32 {
                if Self::is_dynamic() {
                    unreachable!();
                } else {
                    $first_ty::size_hint() $( + $rest_ty::size_hint() )+
                }
            }
        }

        impl_tuple!( $( $rest_ty, )+ );
    };
}

#[allow(non_snake_case)]
mod inner_impl_tuple {
    use super::*;

    seq!(N in 0..16 {
        impl_tuple! {
            #(T#N,)*
        }
    });
}

impl Encode for () {
    #[inline(always)]
    fn encode(&self) -> Vec<u8> {
        Vec::new()
    }
}

impl Decode for () {
    #[inline(always)]
    fn decode<I: Input>(_: &mut I) -> Result<(), Error> {
        Ok(())
    }
}

impl TypeInfo for () {
    #[inline(always)]
    fn is_dynamic() -> bool {
        false
    }

    #[inline(always)]
    fn size_hint() -> u32 {
        0
    }
}

impl Encode for ((),) {
    #[inline(always)]
    fn encode(&self) -> Vec<u8> {
        Vec::new()
    }
}

impl Decode for ((),) {
    #[inline(always)]
    fn decode<I: Input>(_: &mut I) -> Result<((),), Error> {
        Ok(((),))
    }
}

use liquid_primitives::types::{i256, u256, Address, ADDRESS_LENGTH};

impl TypeInfo for Address {}

impl MediateEncode for Address {
    fn encode(&self) -> Mediate {
        let mut buf = [0x00; WORD_SIZE];
        buf[(WORD_SIZE - ADDRESS_LENGTH)..].copy_from_slice(&self.0);
        Mediate::Raw([buf].to_vec())
    }
}

impl MediateDecode for Address {
    fn decode(slices: &[Word], offset: usize) -> Result<DecodeResult<Self>, Error> {
        let slice = peek(slices, offset)?;

        if !slice[..(WORD_SIZE - ADDRESS_LENGTH)]
            .iter()
            .all(|x| *x == 0)
        {
            Err("Invalid address representation".into())
        } else {
            let new_offset = offset + 1;
            let mut address = [0u8; ADDRESS_LENGTH];
            address[..].copy_from_slice(&slice[(WORD_SIZE - ADDRESS_LENGTH)..]);
            Ok(DecodeResult {
                value: Self(address),
                new_offset,
            })
        }
    }
}

impl TypeInfo for i256 {}

impl MediateEncode for i256 {
    fn encode(&self) -> Mediate {
        let be = self.0.to_signed_bytes_be();
        let mut buf = match self.0.sign() {
            num_bigint::Sign::Plus | num_bigint::Sign::NoSign => [0x00; WORD_SIZE],
            _ => [0xff; WORD_SIZE],
        };

        buf[(WORD_SIZE - be.len())..].copy_from_slice(&be);
        Mediate::Raw([buf].to_vec())
    }
}

impl MediateDecode for i256 {
    fn decode(slices: &[Word], offset: usize) -> Result<DecodeResult<Self>, Error> {
        let slice = peek(slices, offset)?;
        let value = num_bigint::BigInt::from_signed_bytes_be(slice);
        Ok(DecodeResult {
            value: i256(value),
            new_offset: offset + 1,
        })
    }
}

impl TypeInfo for u256 {}

impl MediateEncode for u256 {
    fn encode(&self) -> Mediate {
        let be = self.0.to_bytes_be();
        let mut buf: Word = [0x00; WORD_SIZE];

        buf[(WORD_SIZE - be.len())..].copy_from_slice(&be);
        Mediate::Raw([buf].to_vec())
    }
}

impl MediateDecode for u256 {
    fn decode(slices: &[Word], offset: usize) -> Result<DecodeResult<Self>, Error> {
        let slice = peek(slices, offset)?;
        let value = u256::from_bytes_be(slice);
        Ok(DecodeResult {
            value,
            new_offset: offset + 1,
        })
    }
}
