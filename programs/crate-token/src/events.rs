//! Crate events

use anchor_lang::prelude::*;

/// Emitted when a crate is created.
#[event]
pub struct NewCrateEvent {
    #[index]
    pub crate_key: Pubkey,
    #[index]
    pub issue_authority: Pubkey,
}

/// Emitted when crate tokens are issued.
#[event]
pub struct IssueEvent {
    #[index]
    pub crate_key: Pubkey,
    pub destination: Pubkey,
    pub amount: u64,
}

/// Emitted when crate tokens are redeemed.
#[event]
pub struct RedeemEvent {
    #[index]
    pub crate_key: Pubkey,
    pub source: Pubkey,
    pub amount: u64,
}
