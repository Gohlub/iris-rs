extern crate alloc;

use alloc::{vec, vec::Vec};

use super::*;
use crate::based;
use crate::belt::{montify, Belt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Digest(pub [Belt; 5]);

// assert that input is made of base field elements
pub fn assert_all_based(vecbelt: &Vec<Belt>) {
    vecbelt.iter().for_each(|b| based!(b.0));
}

// calc q and r for vecbelt, based on RATE
pub fn tip5_calc_q_r(input_vec: &Vec<Belt>) -> (usize, usize) {
    let lent_input = input_vec.len();
    let (q, r) = (lent_input / RATE, lent_input % RATE);
    (q, r)
}

// pad vecbelt with ~[1 0 ... 0] to be a multiple of rate
pub fn tip5_pad_vecbelt(input_vec: &mut Vec<Belt>, r: usize) {
    input_vec.push(Belt(1));
    for _i in 0..(RATE - r) - 1 {
        input_vec.push(Belt(0));
    }
}

// monitify vecbelt (bring into montgomery space)
pub fn tip5_montify_vecbelt(input_vec: &mut Vec<Belt>) {
    for i in 0..input_vec.len() {
        input_vec[i] = Belt(montify(input_vec[i].0));
    }
}

// calc digest
pub fn tip5_calc_digest(sponge: &[u64; 16]) -> Digest {
    let mut digest = [Belt(0); DIGEST_LENGTH];
    for i in 0..DIGEST_LENGTH {
        digest[i] = Belt(mont_reduction(sponge[i] as u128));
    }
    Digest(digest)
}

// absorb complete input
pub fn tip5_absorb_input(input_vec: &mut Vec<Belt>, sponge: &mut [u64; 16], q: usize) {
    let mut cnt_q = q;
    let mut input_to_absorb = input_vec.as_slice();
    loop {
        let (scag_input, slag_input) = input_to_absorb.split_at(RATE);
        tip5_absorb_rate(sponge, scag_input);

        if cnt_q == 0 {
            break;
        }
        cnt_q -= 1;
        input_to_absorb = slag_input;
    }
}

// absorb one part of input (size RATE)
pub fn tip5_absorb_rate(sponge: &mut [u64; 16], input: &[Belt]) {
    assert_eq!(input.len(), RATE);

    for copy_pos in 0..RATE {
        sponge[copy_pos] = input[copy_pos].0;
    }

    permute(sponge);
}

pub fn hash_varlen(input_vec: &mut Vec<Belt>) -> Digest {
    let mut sponge = create_init_sponge_variable();

    // assert that input is made of base field elements
    assert_all_based(input_vec);

    // pad input with ~[1 0 ... 0] to be a multiple of rate
    let (q, r) = tip5_calc_q_r(input_vec);
    tip5_pad_vecbelt(input_vec, r);

    // bring input into montgomery space
    tip5_montify_vecbelt(input_vec);

    // process input in batches of size RATE
    tip5_absorb_input(input_vec, &mut sponge, q);

    // calc digest
    tip5_calc_digest(&sponge)
}

pub fn hash_10(input_vec: &mut Vec<Belt>) -> Digest {
    // check input
    let (q, r) = tip5_calc_q_r(input_vec);
    assert_eq!(q, 1);
    assert_eq!(r, 0);
    assert_all_based(input_vec);

    // bring input into montgomery space
    tip5_montify_vecbelt(input_vec);

    // create init sponge (%fixed)
    let mut sponge = create_init_sponge_fixed();

    // process input (q=1, so one batch only)
    //tip5_absorb_input(&mut input_vec, &mut sponge, q);
    tip5_absorb_rate(&mut sponge, input_vec.as_slice());

    //  calc digest
    tip5_calc_digest(&sponge)
}

pub fn create_init_sponge_variable() -> [u64; STATE_SIZE] {
    [0u64; STATE_SIZE]
}

pub fn create_init_sponge_fixed() -> [u64; STATE_SIZE] {
    let mut sponge = [0u64; STATE_SIZE];
    for i in 10..STATE_SIZE {
        sponge[i] = 4294967295u64;
    }
    sponge
}

pub fn digest_to_bytes(digest: Digest) -> [u8; 40] {
    use ibig::UBig;

    let p = UBig::from(crate::belt::PRIME);
    let p2 = &p * &p;
    let p3 = &p * &p2;
    let p4 = &p * &p3;

    let [a, b, c, d, e] = digest.0.map(|b| UBig::from(b.0));
    let res = a + b * &p + c * p2 + d * p3 + e * p4;

    let mut bytes = [0u8; 40];
    let res_bytes = res.to_be_bytes();
    bytes[40 - res_bytes.len()..].copy_from_slice(&res_bytes);

    bytes
}

pub fn hash_noun(leaves: &[Belt], dyck: &[Belt]) -> Digest {
    let mut combined = Vec::with_capacity(1 + leaves.len() + dyck.len());
    combined.push(Belt(leaves.len() as u64));
    combined.extend_from_slice(leaves);
    combined.extend_from_slice(dyck);
    hash_varlen(&mut combined)
}

pub fn hash_belt(input: Belt) -> Digest {
    hash_noun(&vec![input], &vec![])
}

pub fn hash_belt_list(input: &[Belt]) -> Digest {
    let mut leaves = Vec::with_capacity(input.len() + 1);
    leaves.extend(input);
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
        leaves.extend(h.0.iter());
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

pub fn to_b58(digest: Digest) -> Vec<u8> {
    bs58::encode(digest_to_bytes(digest)).into_vec()
}

pub fn from_b58(s: &str) -> Vec<u8> {
    bs58::decode(s).into_vec().unwrap()
}
