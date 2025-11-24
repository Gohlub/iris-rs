use crate::draft::{Coins, Draft, LockIntent, Recipient};

#[allow(dead_code)]
struct TxEngine {
    // per pubkey:
    // available notes
    // spent notes
    // transactions
}

impl TxEngine {
    #[allow(dead_code)]
    pub fn create_tx(
        &self,
        _fee: Coins,
        _recipients: &[Recipient],
        _lock_intent: LockIntent,
    ) -> Draft {
        todo!()
    }
}
