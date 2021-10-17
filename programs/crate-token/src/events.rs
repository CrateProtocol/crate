//! Crate events

use anchor_lang::prelude::*;

/// Emitted when a crate is created.
#[event]
pub struct NewCrateEvent {
    #[index]
    pub crate_key: Pubkey,
    #[index]
    pub issue_authority: Pubkey,
    #[index]
    pub withdraw_authority: Pubkey,
}

/// Emitted when crate tokens are issued.
#[event]
pub struct IssueEvent {
    #[index]
    pub crate_key: Pubkey,
    pub destination: Pubkey,
    pub amount: u64,
    pub author_fee: u64,
    pub protocol_fee: u64,
}

/// Emitted when crate tokens are withdrawn.
#[event]
pub struct WithdrawEvent {
    #[index]
    pub crate_key: Pubkey,
    pub token: Pubkey,
    pub destination: Pubkey,
    pub amount: u64,
    pub author_fee: u64,
    pub protocol_fee: u64,
}
