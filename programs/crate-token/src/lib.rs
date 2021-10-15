//! Crate Token.
#![deny(rustdoc::all)]
#![allow(rustdoc::missing_doc_code_examples)]

mod account_validators;

pub mod events;
pub mod state;

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use num_traits::cast::ToPrimitive;
use vipers::validate::Validate;
use vipers::{assert_keys, invariant, unwrap_int};

use events::*;
use state::*;

declare_id!("CRAToQ6Ycrxp6ei3VBcdi8oRxETkwDJZxRu6ziCFGD5a");

/// Address where fees are sent to.
pub mod fee_to_address {
    use anchor_lang::declare_id;
    declare_id!("2ddvVUH6fmZoifqEAwQ31EBUXy5CHgpk9wWbvkBgsfdT");
}

/// Address where fees are sent to.
pub static FEE_TO_ADDRESS: Pubkey = fee_to_address::ID;

/// Denominator for computing the withdraw fee. Currently 1bp.
pub static WITHDRAW_FEE_DENOM: u64 = 10_000;

/// [crate_token] program.
#[program]
pub mod crate_token {
    use super::*;

    /// Provisions a new Crate.
    #[access_control(ctx.accounts.validate())]
    pub fn new_crate(ctx: Context<NewCrate>, bump: u8) -> ProgramResult {
        let info = &mut ctx.accounts.crate_info;
        info.mint = ctx.accounts.crate_mint.key();
        info.bump = bump;
        info.issue_authority = ctx.accounts.issue_authority.key();

        emit!(NewCrateEvent {
            issue_authority: ctx.accounts.issue_authority.key(),
            crate_key: ctx.accounts.crate_info.key(),
        });

        Ok(())
    }

    /// Issues Crate tokens.
    #[access_control(ctx.accounts.validate())]
    pub fn issue(ctx: Context<Issue>, amount: u64) -> ProgramResult {
        let seeds: &[&[u8]] = &[
            b"CrateInfo".as_ref(),
            &ctx.accounts.crate_info.mint.to_bytes(),
            &[ctx.accounts.crate_info.bump],
        ];
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::MintTo {
                    mint: ctx.accounts.crate_mint.to_account_info(),
                    to: ctx.accounts.mint_destination.to_account_info(),
                    authority: ctx.accounts.crate_info.to_account_info(),
                },
                &[seeds],
            ),
            amount,
        )?;

        emit!(IssueEvent {
            crate_key: ctx.accounts.crate_info.key(),
            destination: ctx.accounts.mint_destination.key(),
            amount,
        });

        Ok(())
    }

    /// Redeems Crate tokens for their underlying assets.
    #[access_control(ctx.accounts.validate())]
    pub fn redeem<'info>(
        ctx: Context<'_, '_, '_, 'info, Redeem<'info>>,
        amount: u64,
    ) -> ProgramResult {
        let burn = token::Burn {
            mint: ctx.accounts.crate_mint.to_account_info(),
            to: ctx.accounts.crate_source.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };

        let token_program = ctx.accounts.token_program.to_account_info();
        token::burn(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), burn),
            amount,
        )?;

        let seeds: &[&[u8]] = &[
            b"CrateInfo".as_ref(),
            &ctx.accounts.crate_info.mint.to_bytes(),
            &[ctx.accounts.crate_info.bump],
        ];
        let signer_seeds: &[&[&[u8]]] = &[seeds];

        // calculate the fractional slice of each account
        let num_remaining_accounts = ctx.remaining_accounts.len();
        if num_remaining_accounts == 0 {
            return Ok(());
        }
        invariant!(
            num_remaining_accounts % 3 == 0,
            "must have even number of tokens"
        );
        let num_tokens = unwrap_int!(num_remaining_accounts.checked_div(3));
        // TODO: add check to make sure every single token in the crate was redeemed

        let remaining_accounts_iter = &mut ctx.remaining_accounts.iter();

        for _i in 0..num_tokens {
            let pair = RedeemPair {
                crate_underlying_token_account: Account::try_from(next_account_info(
                    remaining_accounts_iter,
                )?)?,
                dest_token_account: Account::try_from(next_account_info(remaining_accounts_iter)?)?,
                fee_token_account: Account::try_from(next_account_info(remaining_accounts_iter)?)?,
            };

            assert_keys!(
                pair.crate_underlying_token_account.owner,
                ctx.accounts.crate_info,
                "pair.crate_underlying_token_account.owner"
            );
            assert_keys!(
                pair.crate_underlying_token_account.mint,
                pair.dest_token_account.mint,
                "pair mint"
            );
            assert_keys!(
                pair.crate_underlying_token_account.mint,
                pair.fee_token_account.mint,
                "fee mint"
            );
            assert_keys!(
                pair.fee_token_account.owner,
                FEE_TO_ADDRESS,
                "fee to mismatch"
            );

            let share_opt = (pair.crate_underlying_token_account.amount as u128)
                .checked_mul(amount.into())
                .and_then(|num| num.checked_div(ctx.accounts.crate_mint.supply.into()))
                .and_then(|num| num.to_u64());
            let share: u64 = unwrap_int!(share_opt);
            let fee = unwrap_int!(share.checked_div_euclid(WITHDRAW_FEE_DENOM));

            // share
            token::transfer(
                CpiContext::new_with_signer(
                    token_program.clone(),
                    token::Transfer {
                        from: pair.crate_underlying_token_account.to_account_info(),
                        to: pair.dest_token_account.to_account_info(),
                        authority: ctx.accounts.crate_info.to_account_info(),
                    },
                    signer_seeds,
                ),
                unwrap_int!(share.checked_sub(fee)),
            )?;

            // fees
            token::transfer(
                CpiContext::new_with_signer(
                    token_program.clone(),
                    token::Transfer {
                        from: pair.crate_underlying_token_account.to_account_info(),
                        to: pair.fee_token_account.to_account_info(),
                        authority: ctx.accounts.crate_info.to_account_info(),
                    },
                    signer_seeds,
                ),
                fee,
            )?;
        }

        emit!(RedeemEvent {
            crate_key: ctx.accounts.crate_info.key(),
            source: ctx.accounts.crate_source.key(),
            amount
        });

        Ok(())
    }
}

// --------------------------------
// Context Structs
// --------------------------------

/// Accounts for [crate_token::new_crate].
#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct NewCrate<'info> {
    /// Information about the crate.
    #[account(
        init,
        seeds = [
            b"CrateInfo".as_ref(),
            crate_mint.key().to_bytes().as_ref()
        ],
        bump = bump,
        payer = payer
    )]
    pub crate_info: Account<'info, CrateInfo>,

    /// [Mint] of the [CrateInfo].
    pub crate_mint: Account<'info, Mint>,

    /// The authority that can issue new [CrateInfo] tokens.
    pub issue_authority: UncheckedAccount<'info>,

    /// Payer of the crate initialization.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// System program.
    pub system_program: Program<'info, System>,
}

/// Accounts for [crate_token::issue].
#[derive(Accounts)]
pub struct Issue<'info> {
    /// Information about the crate.
    pub crate_info: Account<'info, CrateInfo>,

    /// [Mint] of the [CrateInfo].
    #[account(mut)]
    pub crate_mint: Account<'info, Mint>,

    /// Authority of the account issuing Crate tokens.
    pub issue_authority: Signer<'info>,

    /// Destination of the minted tokens.
    #[account(mut)]
    pub mint_destination: Account<'info, TokenAccount>,

    /// [Token] program.
    pub token_program: Program<'info, Token>,
}

/// Accounts for [crate_token::redeem].
#[derive(Accounts)]
pub struct Redeem<'info> {
    /// Information about the crate.
    pub crate_info: Account<'info, CrateInfo>,

    /// [Mint] of the [CrateInfo].
    #[account(mut)]
    pub crate_mint: Account<'info, Mint>,

    /// Source of the crate tokens.
    #[account(mut)]
    pub crate_source: Account<'info, TokenAccount>,

    /// Owner of the crate source.
    pub owner: Signer<'info>,

    /// [Token] program.
    pub token_program: Program<'info, Token>,
}

/// Accounts for [crate_token::new_crate].
#[derive(Accounts)]
pub struct RedeemPair<'info> {
    /// Crate account of the tokens
    pub crate_underlying_token_account: Account<'info, TokenAccount>,
    /// Destination of the tokens to redeem
    pub dest_token_account: Account<'info, TokenAccount>,
    /// Fee token destination
    pub fee_token_account: Account<'info, TokenAccount>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_fee_to_address() {
        let (key, bump) = Pubkey::find_program_address(&[b"CrateFees"], &crate::ID);
        assert_eq!(key, FEE_TO_ADDRESS);
        assert_eq!(bump, 253);
    }
}
