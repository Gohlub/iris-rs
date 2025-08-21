use std::collections::BTreeMap;

pub struct PublicKey(u128);
pub struct Signature;

struct PrivateKey(u128);

impl PrivateKey {
    fn derive_public_key(&self) -> PublicKey {
        todo!()
    }

    // sign 5 belts
    fn sign(&self, m: &[u64; 5]) -> Signature {
        // TODO: impl sign:affine:belt-schnorr:cheetah
        todo!()
    }
}

struct MasterKey {
    priv_key: PrivateKey,
    // Make sure we only serialize the key ID's, but not the keys themselves
    loaded_children: BTreeMap<u32, (PublicKey, PrivateKey)>,
}

pub struct Keystore {
    keys: BTreeMap<PublicKey, MasterKey>,
}

impl Keystore {
    // TODO:
    // load(json)
    // store() -> json
    // list_master_pubkeys() -> Vec<Pubkey>
    // list_child_pubkeys(master_pubkey) -> Vec<Pubkey>
    // delete_master_key(master_pubkey)
    // derive_master_key(seed) -> Pubkey
    // derive_master_key_from_seedphrase(seedphrase) -> derive_master_key(derive_seed(seedphrase))
    // sign(chal, key_handle) -> sig
}

pub struct KeyHandle {
    master_pubkey: PublicKey,
    child_id: u32,
}
