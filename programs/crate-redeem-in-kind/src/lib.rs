//! In-kind distributions for redeeming Crate assets.
#![deny(rustdoc::all)]
#![allow(rustdoc::missing_doc_code_examples)]

mod account_validators;

pub mod events;

use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_lang::solana_program::account_info::next_account_infos;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use num_traits::cast::ToPrimitive;
use static_pubkey::static_pubkey;
use vipers::validate::Validate;
use vipers::{invariant, unwrap_int};

use events::*;

declare_id!("1NKyU3qShZC3oJgvCCftAHDi5TFxcJwfyUz2FeZsiwE");

/// Address of the withdraw authority to use for this Crate.
pub static WITHDRAW_AUTHORITY_ADDRESS: Pubkey =
    static_pubkey!("2amCDqmgpQ2qkryLArCcYeX8DzyNqvjuy7yKq6hsonqF");

/// Bump seed of the above address.
pub const WITHDRAW_AUTHORITY_ADDRESS_BUMP: u8 = 255;

/// Signer seeds of the [WITHDRAW_AUTHORITY_ADDRESS].
pub static WITHDRAW_AUTHORITY_SIGNER_SEEDS: &[&[&[u8]]] =
    &[&[b"CrateRedeemInKind", &[WITHDRAW_AUTHORITY_ADDRESS_BUMP]]];

/// [crate_redeem_in_kind] program.
#[program]
pub mod crate_redeem_in_kind {
    use std::collections::BTreeMap;

    use super::*;

    /// Redeems Crate tokens for their underlying assets, in-kind.
    /// This redemption limits the number of assets that can be redeemed,
    /// but it ensures that all assets are redeemed equally.
    #[access_control(ctx.accounts.validate())]
    pub fn redeem<'info>(
        ctx: Context<'_, '_, '_, 'info, Redeem<'info>>,
        amount: u64,
    ) -> Result<()> {
        let burn = token::Burn {
            mint: ctx.accounts.crate_mint.to_account_info(),
            from: ctx.accounts.crate_source.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };

        token::burn(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), burn),
            amount,
        )?;

        // calculate the fractional slice of each account
        let num_remaining_accounts = ctx.remaining_accounts.len();
        if num_remaining_accounts == 0 {
            return Ok(());
        }
        invariant!(
            num_remaining_accounts % 4 == 0,
            "must have even number of tokens"
        );
        let num_tokens = unwrap_int!(num_remaining_accounts.checked_div(4));
        // TODO: add check to make sure every single token in the crate was redeemed

        let remaining_accounts_iter = &mut ctx.remaining_accounts.iter();

        for _i in 0..num_tokens {
            // none of these accounts need to be validated further, since
            // [crate_token::cpi::withdraw] already handles it.
            let bumps = &mut BTreeMap::new();
            let asset: RedeemAsset = Accounts::try_accounts(
                &crate::ID,
                &mut next_account_infos(remaining_accounts_iter, 4)?,
                &[],
                bumps,
            )?;

            let share: u64 = unwrap_int!((asset.crate_underlying.amount as u128)
                .checked_mul(amount.into())
                .and_then(|num| num.checked_div(ctx.accounts.crate_mint.supply.into()))
                .and_then(|num| num.to_u64()));

            crate_token::cpi::withdraw(
                CpiContext::new_with_signer(
                    ctx.accounts.crate_token_program.to_account_info(),
                    crate_token::cpi::accounts::Withdraw {
                        crate_token: ctx.accounts.crate_token.to_account_info(),
                        crate_underlying: asset.crate_underlying.to_account_info(),
                        withdraw_authority: ctx.accounts.withdraw_authority.to_account_info(),
                        withdraw_destination: asset.withdraw_destination.to_account_info(),
                        author_fee_destination: asset.author_fee_destination.to_account_info(),
                        protocol_fee_destination: asset.protocol_fee_destination.to_account_info(),
                        token_program: ctx.accounts.token_program.to_account_info(),
                    },
                    WITHDRAW_AUTHORITY_SIGNER_SEEDS,
                ),
                share,
            )?;
        }

        emit!(RedeemEvent {
            crate_key: ctx.accounts.crate_token.key(),
            source: ctx.accounts.crate_source.key(),
            amount
        });

        Ok(())
    }
}

// --------------------------------
// Context Structs
// --------------------------------

/// Accounts for [crate_redeem_in_kind::redeem].
#[derive(Accounts)]
pub struct Redeem<'info> {
    /// The withdraw authority PDA.
    /// CHECK: Arbitrary.
    pub withdraw_authority: UncheckedAccount<'info>,

    /// Information about the crate.
    #[account(has_one = withdraw_authority)]
    pub crate_token: Account<'info, crate_token::CrateToken>,

    /// [Mint] of the [crate_token::CrateToken].
    #[account(mut)]
    pub crate_mint: Account<'info, Mint>,

    /// Source of the crate tokens.
    #[account(mut)]
    pub crate_source: Account<'info, TokenAccount>,

    /// Owner of the crate source.
    pub owner: Signer<'info>,

    /// [Token] program.
    pub token_program: Program<'info, Token>,

    /// [crate_token] program.
    pub crate_token_program: Program<'info, crate_token::program::CrateToken>,
}

/// Asset redeemed in [crate_redeem_in_kind::redeem].
#[derive(Accounts)]
pub struct RedeemAsset<'info> {
    /// Crate account of the tokens
    #[account(mut)]
    pub crate_underlying: Account<'info, TokenAccount>,

    /// Destination of the tokens to redeem
    #[account(mut)]
    pub withdraw_destination: Account<'info, TokenAccount>,

    /// Author fee token destination
    #[account(mut)]
    pub author_fee_destination: Account<'info, TokenAccount>,

    /// Protocol fee token destination
    #[account(mut)]
    pub protocol_fee_destination: Account<'info, TokenAccount>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_withdraw_authority_address() {
        let (key, bump) = Pubkey::find_program_address(&[b"CrateRedeemInKind"], &crate::ID);
        assert_eq!(key, WITHDRAW_AUTHORITY_ADDRESS);
        assert_eq!(bump, WITHDRAW_AUTHORITY_ADDRESS_BUMP);
    }
}
