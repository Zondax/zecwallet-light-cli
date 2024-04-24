use std::convert::TryFrom;

pub fn vec_to_array<'a, T, const N: usize>(vec: &'a Vec<T>) -> &'a [T; N] {
    <&[T; N]>::try_from(&vec[..]).unwrap()
}
