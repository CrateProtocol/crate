//! Validate accounts

use anchor_lang::prelude::*;
use vipers::{assert_keys, invariant};

use crate::{Issue, NewCrate, Redeem};
use anchor_lang::Key;
use vipers::validate::Validate;

impl<'info> Validate<'info> for NewCrate<'info> {
    fn validate(&self) -> ProgramResult {
        assert_keys!(
            self.crate_mint.mint_authority.unwrap(),
            self.crate_info.key(),
            "crate_mint.mint_authority"
        );
        assert_keys!(
            self.crate_mint.freeze_authority.unwrap(),
            self.crate_info.key(),
            "crate_mint.mint_authority"
        );
        invariant!(self.crate_mint.supply == 0, "supply must be zero");
        Ok(())
    }
}

impl<'info> Validate<'info> for Issue<'info> {
    fn validate(&self) -> ProgramResult {
        assert_keys!(
            self.crate_info.mint,
            self.crate_mint.key(),
            "crate_info.mint"
        );
        assert_keys!(
            self.crate_info.issue_authority,
            self.issue_authority,
            "crate_info.issue_authority"
        );
        assert_keys!(
            self.mint_destination.mint,
            self.crate_info.mint,
            "mint_destination.mint"
        );
        Ok(())
    }
}

impl<'info> Validate<'info> for Redeem<'info> {
    fn validate(&self) -> ProgramResult {
        assert_keys!(
            self.crate_info.mint,
            self.crate_mint.key(),
            "crate_info.mint"
        );
        assert_keys!(self.crate_source.mint, self.crate_mint, "crate_source.mint");
        assert_keys!(self.crate_source.owner, self.owner, "crate_source.owner");
        Ok(())
    }
}
