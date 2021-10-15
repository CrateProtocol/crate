use anchor_lang::prelude::*;

/// Contains the info of a crate token. Immutable.
/// The account associated with this struct is also the mint/freeze authority.
#[account]
#[derive(Copy, Debug, Default, PartialEq, Eq)]
pub struct CrateInfo {
    /// [anchor_spl::token::Mint] of the [CrateInfo].
    pub mint: Pubkey,
    /// Bump.
    pub bump: u8,
    /// Authority that is allowed to issue new shares of the Crate.
    /// This is usually a program that will handle users depositing
    /// tokens into the crate + giving them shares of the crate.
    pub issue_authority: Pubkey,
}
