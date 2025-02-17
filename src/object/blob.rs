use crate::object::Hash;

pub(crate) struct Blob {
    size: usize,
    hash: Hash,
}