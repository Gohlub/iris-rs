/// Hashable DSL for converting Rust types to tip5-hashable noun structures.
use nbx_nockchain_math::{
    belt::Belt,
    tip5::hash::{hash_10, hash_belt, hash_hash_list, Digest},
};

/// Hashable DSL for tip5 hashing
#[derive(Debug, Clone)]
pub enum Hashable {
    Leaf(Belt),
    Hash(Digest),
    List(Vec<Hashable>),
    Cell(Box<Hashable>, Box<Hashable>),
}

impl Hashable {
    pub fn cell(left: Hashable, right: Hashable) -> Self {
        Hashable::Cell(Box::new(left), Box::new(right))
    }

    pub fn hash(&self) -> Digest {
        match self {
            Hashable::Hash(h) => *h,
            Hashable::Leaf(belt) => hash_belt(*belt),
            Hashable::List(elements) => {
                hash_hash_list(&mut elements.into_iter().map(|e| e.hash()).collect::<Vec<_>>())
            }
            Hashable::Cell(l, r) => {
                let mut belts = Vec::<Belt>::with_capacity(10);
                belts.extend(l.hash().0);
                belts.extend(r.hash().0);
                hash_10(&mut belts)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nbx_nockchain_math::tip5::hash::{digest_to_bytes, from_b58};

    #[test]
    fn test_hashable_vectors() {
        let leaf = Hashable::Leaf(Belt(42));
        assert_eq!(
            digest_to_bytes(leaf.hash()).to_vec(),
            from_b58("mhVFxh4yzHZWzLENL4FDu6WKynrgcyx3p6kJbJ9Cg7m9DPbSEvZMMf"),
        );
        let cell = Hashable::cell(Hashable::Leaf(Belt(42)), Hashable::Leaf(Belt(69)));
        assert_eq!(
            digest_to_bytes(cell.hash()).to_vec(),
            from_b58("4D62tFybemZW3YX4w16jFwT5pNUaGgYz3zyx32wMsuwtrZuYUnNCeGQ"),
        );
        let mut list = Vec::new();
        list.push(Hashable::Leaf(Belt(42)));
        list.push(Hashable::Leaf(Belt(69)));
        list.push(Hashable::Leaf(Belt(88)));
        assert_eq!(
            digest_to_bytes(Hashable::List(list).hash()).to_vec(),
            from_b58("uANkACbninAKJgsKMr2jKaP2Qskqfvpbk2agiB45VDq8sxXf7NW9eT"),
        );
    }
}
