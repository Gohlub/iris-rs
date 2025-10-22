use alloc::{vec, vec::Vec};

use crate::belt::{Belt, PRIME};
use crate::tip5::hash::{hash_fixed, hash_varlen};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Digest(pub [Belt; 5]);

impl From<[u64; 5]> for Digest {
    fn from(belts: [u64; 5]) -> Self {
        Digest(belts.map(|b| Belt(b)))
    }
}

impl Digest {
    pub fn to_bytes(&self) -> [u8; 40] {
        use ibig::UBig;

        let p = UBig::from(PRIME);
        let p2 = &p * &p;
        let p3 = &p * &p2;
        let p4 = &p * &p3;

        let [a, b, c, d, e] = self.0.map(|b| UBig::from(b.0));
        let res = a + b * &p + c * p2 + d * p3 + e * p4;

        let mut bytes = [0u8; 40];
        let res_bytes = res.to_be_bytes();
        bytes[40 - res_bytes.len()..].copy_from_slice(&res_bytes);
        bytes
    }
}

pub fn to_b58(bytes: &[u8]) -> Vec<u8> {
    bs58::encode(bytes).into_vec()
}

pub fn from_b58(s: &str) -> Vec<u8> {
    bs58::decode(s).into_vec().unwrap()
}

pub fn hash_noun(leaves: &[Belt], dyck: &[Belt]) -> Digest {
    let mut combined = Vec::with_capacity(1 + leaves.len() + dyck.len());
    combined.push(Belt(leaves.len() as u64));
    combined.extend_from_slice(leaves);
    combined.extend_from_slice(dyck);
    Digest(hash_varlen(&mut combined).map(|u| Belt(u)))
}

pub fn hash_belt(input: Belt) -> Digest {
    hash_noun(&vec![input], &vec![])
}

pub fn hash_belt_list(input: &[Belt]) -> Digest {
    let mut leaves = Vec::with_capacity(input.len() + 1);
    leaves.extend_from_slice(input);
    leaves.push(Belt(0));

    let mut dyck = Vec::new();
    for _ in input {
        dyck.push(Belt(0));
        dyck.push(Belt(1));
    }

    hash_noun(&leaves, &dyck)
}

pub fn hash_hash_list(input: &[Digest]) -> Digest {
    let mut leaves = Vec::new();
    for h in input {
        leaves.extend_from_slice(&h.0);
    }
    leaves.push(Belt(0));

    let mut dyck = Vec::new();
    for _ in input {
        dyck.push(Belt(0));
        for _ in 0..4 {
            dyck.push(Belt(0));
            dyck.push(Belt(1));
        }
        dyck.push(Belt(1));
    }

    hash_noun(&leaves, &dyck)
}

pub trait Hashable {
    fn hash(&self) -> Digest;
}

impl Hashable for Belt {
    fn hash(&self) -> Digest {
        hash_belt(*self)
    }
}

impl<A: Hashable, B: Hashable> Hashable for (A, B) {
    fn hash(&self) -> Digest {
        let mut belts = Vec::<Belt>::with_capacity(10);
        belts.extend_from_slice(&self.0.hash().0);
        belts.extend_from_slice(&self.1.hash().0);
        Digest(hash_fixed(&mut belts).map(|u| Belt(u)))
    }
}

impl Hashable for u64 {
    fn hash(&self) -> Digest {
        Belt(*self).hash()
    }
}

impl Hashable for usize {
    fn hash(&self) -> Digest {
        (*self as u64).hash()
    }
}

impl Hashable for i32 {
    fn hash(&self) -> Digest {
        (*self as u64).hash()
    }
}

impl Hashable for bool {
    fn hash(&self) -> Digest {
        (if *self { 1 } else { 0 }).hash()
    }
}

impl Hashable for Digest {
    fn hash(&self) -> Digest {
        *self
    }
}

impl<T: Hashable> Hashable for &T {
    fn hash(&self) -> Digest {
        (**self).hash()
    }
}

impl<T: Hashable> Hashable for Option<T> {
    fn hash(&self) -> Digest {
        match self {
            None => 0.hash(),
            Some(v) => (&0, v).hash(),
        }
    }
}

impl<T: Hashable> Hashable for &[T] {
    fn hash(&self) -> Digest {
        fn build_tree<T: Hashable>(items: &[T], i: usize) -> Digest {
            if i >= items.len() {
                return 0.hash();
            }
            let left_tree = build_tree(items, i * 2 + 1);
            let right_tree = build_tree(items, i * 2 + 2);
            ((&items[i], left_tree), right_tree).hash()
        }
        build_tree(self, 0)
    }
}

impl<T: Hashable> Hashable for Vec<T> {
    fn hash(&self) -> Digest {
        hash_hash_list(&mut self.iter().map(|e| e.hash()).collect::<Vec<_>>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hashable_vectors() {
        assert_eq!(
            to_b58(&42.hash().to_bytes()),
            "mhVFxh4yzHZWzLENL4FDu6WKynrgcyx3p6kJbJ9Cg7m9DPbSEvZMMf".as_bytes(),
        );
        assert_eq!(
            to_b58(&(42, 69).hash().to_bytes()),
            "4D62tFybemZW3YX4w16jFwT5pNUaGgYz3zyx32wMsuwtrZuYUnNCeGQ".as_bytes(),
        );
        assert_eq!(
            to_b58(&vec![42, 69, 88].hash().to_bytes()),
            "uANkACbninAKJgsKMr2jKaP2Qskqfvpbk2agiB45VDq8sxXf7NW9eT".as_bytes(),
        );
    }
}
