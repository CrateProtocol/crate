//! Validate accounts

use anchor_lang::prelude::*;
use vipers::{assert_keys_eq, invariant};

use crate::{Issue, NewCrate, SetFeeTo, SetFeeToSetter, SetFees, Withdraw};
use anchor_lang::Key;
use vipers::validate::Validate;

impl<'info> Validate<'info> for NewCrate<'info> {
    fn validate(&self) -> ProgramResult {
        assert_keys_eq!(
            self.crate_mint.mint_authority.unwrap(),
            self.crate_token,
            "crate_mint.mint_authority"
        );

        let freeze_authority = self.crate_mint.freeze_authority.unwrap();
        invariant!(
            freeze_authority == self.crate_token.key()
                || freeze_authority == self.issue_authority.key(),
            InvalidFreezeAuthority
        );

        invariant!(self.crate_mint.supply == 0, "supply must be zero");
        Ok(())
    }
}

impl<'info> Validate<'info> for SetFees<'info> {
    fn validate(&self) -> ProgramResult {
        assert_keys_eq!(
            self.crate_token.fee_setter_authority,
            self.fee_setter,
            "crate_token.fee_setter_authority"
        );
        Ok(())
    }
}

impl<'info> Validate<'info> for SetFeeTo<'info> {
    fn validate(&self) -> ProgramResult {
        assert_keys_eq!(
            self.crate_token.fee_to_setter,
            self.fee_to_setter,
            "crate_token.fee_to_setter"
        );
        Ok(())
    }
}

impl<'info> Validate<'info> for SetFeeToSetter<'info> {
    fn validate(&self) -> ProgramResult {
        assert_keys_eq!(
            self.crate_token.fee_to_setter,
            self.fee_to_setter,
            "crate_token.fee_to_setter"
        );
        Ok(())
    }
}

impl<'info> Validate<'info> for Issue<'info> {
    fn validate(&self) -> ProgramResult {
        assert_keys_eq!(
            self.crate_token.mint,
            self.crate_mint.key(),
            "crate_token.mint"
        );
        assert_keys_eq!(
            self.crate_token.issue_authority,
            self.issue_authority,
            "crate_token.issue_authority"
        );

        assert_keys_eq!(
            self.mint_destination.mint,
            self.crate_token.mint,
            "mint_destination.mint"
        );

        // only validate fee destinations if there are fees
        if self.crate_token.issue_fee_bps != 0 {
            assert_keys_eq!(
                self.author_fee_destination.mint,
                self.crate_token.mint,
                "author_fee_destination.mint"
            );
            assert_keys_eq!(
                self.author_fee_destination.owner,
                self.crate_token.author_fee_to,
                "author_fee_destination.owner"
            );
            assert_keys_eq!(
                self.protocol_fee_destination.mint,
                self.crate_token.mint,
                "protocol_fee_destination.mint"
            );
            assert_keys_eq!(
                self.protocol_fee_destination.owner,
                crate::FEE_TO_ADDRESS,
                "fee to mismatch"
            );
        }

        Ok(())
    }
}

impl<'info> Validate<'info> for Withdraw<'info> {
    fn validate(&self) -> ProgramResult {
        assert_keys_eq!(
            self.crate_underlying.owner,
            self.crate_token,
            "crate_underlying.owner"
        );
        assert_keys_eq!(
            self.withdraw_authority,
            self.crate_token.withdraw_authority,
            "withdraw_authority"
        );

        assert_keys_eq!(
            self.withdraw_destination.mint,
            self.crate_underlying.mint,
            "withdraw_destination.mint"
        );

        // only validate fee destinations if there are fees
        if self.crate_token.withdraw_fee_bps != 0 {
            assert_keys_eq!(
                self.author_fee_destination.mint,
                self.crate_underlying.mint,
                "author_fee_destination.mint"
            );
            assert_keys_eq!(
                self.author_fee_destination.owner,
                self.crate_token.author_fee_to,
                "author_fee_destination.owner"
            );
            assert_keys_eq!(
                self.protocol_fee_destination.mint,
                self.crate_underlying.mint,
                "protocol_fee_destination.mint"
            );
            assert_keys_eq!(
                self.protocol_fee_destination.owner,
                crate::FEE_TO_ADDRESS,
                "fee to mismatch"
            );
        }

        Ok(())
    }
}
