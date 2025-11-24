pub use iris_crypto::{PrivateKey, PublicKey};
use std::collections::BTreeMap;

#[allow(dead_code)]
struct MasterKey {
    priv_key: PrivateKey,
    // Make sure we only serialize the key ID's, but not the keys themselves
    loaded_children: BTreeMap<u32, (PublicKey, PrivateKey)>,
}

#[allow(dead_code)]
pub struct Keystore {
    #[allow(dead_code)]
    keys: BTreeMap<PublicKey, MasterKey>,
}

impl Keystore {
    // TODO:
    // load(json)
    // store() -> json
    // list_master_pubkeys() -> Vec<Pubkey>
    // list_child_pubkeys(master_pubkey) -> Vec<Pubkey>
    // delete_master_key(master_pubkey)
    // derive_master_key_from_seedphrase(seedphrase) -> derive_master_key(derive_seed(seedphrase))
    // sign(chal, key_handle) -> sig
}

#[allow(dead_code)]
pub struct KeyHandle {
    master_pubkey: PublicKey,
    child_id: u32,
}
