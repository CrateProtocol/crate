//! Crate events
#![deny(missing_docs)]

use anchor_lang::prelude::*;

/// Emitted when a crate is created.
#[event]
pub struct NewCrateEvent {
    /// Key of the created crate.
    #[index]
    pub crate_key: Pubkey,
    /// Issue authority.
    #[index]
    pub issue_authority: Pubkey,
    /// Withdraw authority.
    #[index]
    pub withdraw_authority: Pubkey,
}

/// Emitted when crate tokens are issued.
#[event]
pub struct IssueEvent {
    /// Key of the created crate.
    #[index]
    pub crate_key: Pubkey,
    /// Destination token account.
    pub destination: Pubkey,
    /// Amount of tokens issued.
    pub amount: u64,
    /// Author fee.
    pub author_fee: u64,
    /// Protocol fee.
    pub protocol_fee: u64,
}

/// Emitted when crate tokens are withdrawn.
#[event]
pub struct WithdrawEvent {
    /// Key of the crate withdrawn from.
    #[index]
    pub crate_key: Pubkey,
    /// Mint of the withdrawn token.
    pub token: Pubkey,
    /// Destination of tokens.
    pub destination: Pubkey,
    /// Amount of tokens withdrawn.
    pub amount: u64,
    /// Author fee.
    pub author_fee: u64,
    /// Protocol fee.
    pub protocol_fee: u64,
}
