use alloc::vec;
use alloc::vec::Vec;
use nbx_crypto::PrivateKey;
use nbx_ztd::{Digest, Hashable as HashableTrait};

use super::note::Note;
use super::tx::{Seed, Seeds, Spend, SpendCondition, Spends, Witness};
use crate::{LockPrimitive, LockTim, Nicks, Pkh, RawTx};

pub struct TxBuilder {
    notes: Vec<Note>,
    recipient: Digest,
    gift: Nicks,
    fee: Nicks,
    refund_pkh: Digest,
}

impl TxBuilder {
    pub fn new_simple(
        notes: Vec<Note>,
        recipient: Digest,
        gift: Nicks,
        fee: Nicks,
        refund_pkh: Digest,
    ) -> Self {
        Self {
            notes,
            recipient,
            gift,
            fee,
            refund_pkh,
        }
    }

    pub fn sign(self, signing_key: &PrivateKey) -> Result<RawTx, BuildError> {
        if self.gift.0 == 0 {
            return Err(BuildError::ZeroGift);
        }

        let spends = self.create_spends_v1(signing_key)?;

        Ok(RawTx::new(spends))
    }

    fn create_spends_v1(self, signing_key: &PrivateKey) -> Result<Spends, BuildError> {
        let pkh = signing_key.public_key().hash();

        let mut spends_vec = Vec::new();
        let mut remaining_gift = self.gift.0;
        let mut remaining_fee = self.fee.0;

        for note in self.notes {
            let simple_pkh = Pkh::single(pkh);
            let gift_portion = if remaining_gift == 0 {
                0
            } else {
                remaining_gift.min(note.assets.0)
            };

            let available_for_fee = note.assets.0.saturating_sub(gift_portion);
            let fee_portion = if remaining_fee == 0 {
                0
            } else {
                remaining_fee.min(available_for_fee)
            };

            let refund = note.assets.0.saturating_sub(gift_portion + fee_portion);

            // Skip if no seeds would be created (protocol requires >=1 seed)
            if gift_portion == 0 && refund == 0 {
                continue;
            }

            if note.assets.0 != gift_portion + fee_portion + refund {
                return Err(BuildError::AccountingMismatch);
            }

            remaining_gift = remaining_gift.saturating_sub(gift_portion);
            remaining_fee = remaining_fee.saturating_sub(fee_portion);

            let mut seeds_vec = Vec::new();

            // Add refund seed if refund > 0
            if refund > 0 {
                seeds_vec.push(Seed::new_single_pkh(
                    self.refund_pkh,
                    refund.into(),
                    note.hash(),
                ));
            }

            // Add gift seed if gift_portion > 0
            if gift_portion > 0 {
                seeds_vec.push(Seed::new_single_pkh(
                    self.recipient,
                    gift_portion.into(),
                    note.hash(),
                ));
            }

            if seeds_vec.is_empty() {
                return Err(BuildError::NoSeeds);
            }

            let mut spend = Spend::new(
                Witness::new(SpendCondition(vec![
                    LockPrimitive::Pkh(simple_pkh),
                    LockPrimitive::Tim(LockTim::coinbase()),
                ])),
                Seeds(seeds_vec),
                fee_portion.into(),
            );
            spend.add_signature(
                signing_key.public_key(),
                signing_key.sign(&spend.sig_hash()),
            );
            spends_vec.push((note.name.clone(), spend));
        }

        if remaining_gift > 0 || remaining_fee > 0 {
            return Err(BuildError::InsufficientFunds);
        }

        Ok(Spends(spends_vec))
    }
}

#[derive(Debug)]
pub enum BuildError {
    ZeroGift,
    InsufficientFunds,
    NoSeeds,
    AccountingMismatch,
}

impl core::fmt::Display for BuildError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            BuildError::ZeroGift => write!(f, "Cannot create a transaction with zero gift"),
            BuildError::InsufficientFunds => write!(f, "Insufficient funds to pay fee and gift"),
            BuildError::NoSeeds => write!(f, "No seeds were provided"),
            BuildError::AccountingMismatch => {
                write!(f, "Assets in must equal gift + fee + refund")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Name, Version};
    use alloc::{string::ToString, vec};
    use bip39::Mnemonic;
    use nbx_crypto::derive_master_key;

    #[test]
    fn test_vector() {
        let mnemonic = Mnemonic::parse("dice domain inspire horse time initial monitor nature mass impose tone benefit vibrant dash kiss mosquito rice then color ribbon agent method drop fat").unwrap();
        let private_key = derive_master_key(&mnemonic.to_seed(""))
            .private_key
            .unwrap();

        let note = Note {
            version: Version::V1,
            origin_page: 13.into(),
            name: Name::new(
                "2H7WHTE9dFXiGgx4J432DsCLuMovNkokfcnCGRg7utWGM9h13PgQvsH".into(),
                "7yMzrJjkb2Xu8uURP7YB3DFcotttR8dKDXF1tSp2wJmmXUvLM7SYzvM".into(),
            ),
            note_data_hash: 0.hash(),
            assets: 4294967296.into(),
        };

        let tx = TxBuilder::new_simple(
            vec![note],
            "6psXufjYNRxffRx72w8FF9b5MYg8TEmWq2nEFkqYm51yfqsnkJu8XqX".into(),
            1234567.into(),
            2850816.into(),
            "6psXufjYNRxffRx72w8FF9b5MYg8TEmWq2nEFkqYm51yfqsnkJu8XqX".into(),
        )
        .sign(&private_key)
        .unwrap();

        assert!(tx.id.to_string() == "3j4vkn72mcpVtQrTgNnYyoF3rDuYax3aebT5axu3Qe16jm9x2wLtepW");
    }
}
