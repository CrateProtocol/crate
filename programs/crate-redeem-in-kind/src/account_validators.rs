//! Validate accounts

use anchor_lang::prelude::*;
use vipers::assert_keys_eq;

use crate::Redeem;
use vipers::validate::Validate;

impl<'info> Validate<'info> for Redeem<'info> {
    fn validate(&self) -> ProgramResult {
        assert_keys_eq!(
            self.withdraw_authority,
            crate::WITHDRAW_AUTHORITY_ADDRESS,
            "withdraw_authority"
        );

        assert_keys_eq!(self.crate_token.mint, self.crate_mint, "crate_token.mint");
        assert_keys_eq!(self.crate_source.mint, self.crate_mint, "crate_source.mint");
        assert_keys_eq!(self.crate_source.owner, self.owner, "crate_source.owner");
        Ok(())
    }
}
