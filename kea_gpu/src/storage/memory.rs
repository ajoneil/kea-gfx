use num_traits::{PrimInt, Unsigned};

pub fn align<T: PrimInt + Unsigned + From<u8>>(size_or_address: T, alignment: T) -> T {
    (size_or_address + (alignment - <T as From<u8>>::from(1)))
        & !(alignment - <T as From<u8>>::from(1))
}
