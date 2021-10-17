//! Validate accounts

use anchor_lang::prelude::*;
use vipers::assert_keys;

use crate::Redeem;
use vipers::validate::Validate;

impl<'info> Validate<'info> for Redeem<'info> {
    fn validate(&self) -> ProgramResult {
        assert_keys!(
            self.withdraw_authority,
            crate::WITHDRAW_AUTHORITY_ADDRESS,
            "withdraw_authority"
        );

        assert_keys!(self.crate_token.mint, self.crate_mint, "crate_info.mint");
        assert_keys!(self.crate_source.mint, self.crate_mint, "crate_source.mint");
        assert_keys!(self.crate_source.owner, self.owner, "crate_source.owner");
        Ok(())
    }
}
