//! Crate events

use anchor_lang::prelude::*;

/// Emitted when crate tokens are redeemed.
#[event]
pub struct RedeemEvent {
    #[index]
    pub crate_key: Pubkey,
    pub source: Pubkey,
    pub amount: u64,
}
