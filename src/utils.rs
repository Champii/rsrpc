use super::byteorder::{LittleEndian, ByteOrder};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

pub fn prepend_u64 (num: u64, vec: Vec<u8>) -> Vec<u8> {
  let mut s_id_vec = [0u8; 8].to_vec();

  LittleEndian::write_u64(&mut s_id_vec, num);

  let data_iter = s_id_vec.into_iter().chain(vec.into_iter());

  data_iter.collect()
}

pub fn extract_u64_head(vec: Vec<u8>) -> (u64, Vec<u8>) {
  let num = LittleEndian::read_u64(&vec);

  let vec: Vec<u8> = vec.into_iter().skip(8).collect();

  (num, vec)
}

pub fn hash_ident_fn(id: &str) -> usize {
  let id = id.to_string();

  let mut hasher = DefaultHasher::new();

  hasher.write(&id.into_bytes());
  hasher.finish() as usize
}

#[macro_export]
macro_rules! hash_ident {
  ($x:ident) => ( $crate::utils::hash_ident_fn(stringify!($x)) )
}

