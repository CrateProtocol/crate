use anchor_lang::{prelude::*, solana_program::pubkey::PUBKEY_BYTES};
use num_traits::ToPrimitive;
use vipers::unwrap_int;

/// Contains the info of a crate token. Immutable.
/// The account associated with this struct is also the mint/freeze authority.
#[account]
#[derive(Copy, Debug, Default, PartialEq, Eq)]
pub struct CrateToken {
    /// [anchor_spl::token::Mint] of the [CrateToken].
    pub mint: Pubkey,
    /// Bump.
    pub bump: u8,

    /// Authority that can modify the [CrateToken]'s fees.
    pub fee_setter_authority: Pubkey,
    /// Authority that can modify who can change the fees.
    pub fee_to_setter: Pubkey,
    /// Authority that is allowed to issue new shares of the Crate.
    /// This is usually a program that will handle users depositing
    /// tokens into the crate + giving them shares of the crate.
    pub issue_authority: Pubkey,
    /// Authority that is allowed to withdraw any token from the Crate.
    /// Withdrawals may be subject to fees.
    pub withdraw_authority: Pubkey,

    /// Account which is the recipient of issue/withdraw ("author") fees.
    /// If fees do not exist, this is unused.
    pub author_fee_to: Pubkey,

    /// The issuance fee in bps.
    /// [crate::ISSUE_FEE_BPS] of this fee goes to the Crate DAO.
    pub issue_fee_bps: u16,
    /// The issuance fee in bps.
    /// [crate::WITHDRAW_FEE_BPS] of this fee goes to the Crate DAO.
    pub withdraw_fee_bps: u16,
}

impl CrateToken {
    pub const LEN: usize = PUBKEY_BYTES + 1 + PUBKEY_BYTES * 5 + 2 + 2;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Fees {
    pub amount: u64,
    /// Fee to the Crate's author.
    pub author_fee: u64,
    /// Fee to the Crate protocol.
    pub protocol_fee: u64,
}

fn apply_bps(amount: u64, bps: u16) -> Result<(u64, u64)> {
    let bps = unwrap_int!((amount)
        .checked_mul(bps.into())
        .and_then(|v| v.checked_div(10_000))
        .and_then(|v| v.to_u64()));
    Ok((unwrap_int!(amount.checked_sub(bps)), bps))
}

impl CrateToken {
    /// Applies the issuance fee.
    pub fn apply_issue_fee(&self, amount: u64) -> Result<Fees> {
        let (amount, issue_fee) = apply_bps(amount, self.issue_fee_bps)?;
        let (author_fee, protocol_fee) = apply_bps(issue_fee, crate::ISSUE_FEE_BPS)?;
        Ok(Fees {
            amount,
            author_fee,
            protocol_fee,
        })
    }

    /// Applies the withdraw fee.
    pub fn apply_withdraw_fee(&self, amount: u64) -> Result<Fees> {
        let (amount, withdraw_fee) = apply_bps(amount, self.withdraw_fee_bps)?;
        let (author_fee, protocol_fee) = apply_bps(withdraw_fee, crate::WITHDRAW_FEE_BPS)?;
        Ok(Fees {
            amount,
            author_fee,
            protocol_fee,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crate_token_len() {
        use crate::CrateToken;
        assert_eq!(
            CrateToken::LEN,
            CrateToken::default().try_to_vec().unwrap().len()
        );
    }
}
