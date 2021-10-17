//! Crate Token.
#![deny(rustdoc::all)]
#![allow(rustdoc::missing_doc_code_examples)]

mod account_validators;
mod macros;

pub mod events;
pub mod state;

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use vipers::validate::Validate;

use events::*;
pub use state::*;

declare_id!("CRATwLpu6YZEeiVq9ajjxs61wPQ9f29s1UoQR9siJCRs");

/// Address where fees are sent to.
pub mod fee_to_address {
    use anchor_lang::declare_id;
    declare_id!("AAqAKWdsUPepSgXf7Msbp1pQ7yCPgYkBvXmNfTFBGAqp");
}

/// Address where fees are sent to.
pub static FEE_TO_ADDRESS: Pubkey = fee_to_address::ID;

/// Issuance fee as a portion of the crate's fee, in bps.
pub static ISSUE_FEE_BPS: u16 = 2_000;

/// Withdraw fee as a portion of the crate's fee, in bps.
pub static WITHDRAW_FEE_BPS: u16 = 2_000;

/// Maximum fee for anything.
pub const MAX_FEE_BPS: u16 = 10_000;

/// [crate_token] program.
#[program]
pub mod crate_token {
    use super::*;

    /// Provisions a new Crate.
    #[access_control(ctx.accounts.validate())]
    pub fn new_crate(ctx: Context<NewCrate>, bump: u8) -> ProgramResult {
        let info = &mut ctx.accounts.crate_token;
        info.mint = ctx.accounts.crate_mint.key();
        info.bump = bump;

        info.fee_setter_authority = ctx.accounts.fee_setter_authority.key();
        info.issue_authority = ctx.accounts.issue_authority.key();
        info.withdraw_authority = ctx.accounts.withdraw_authority.key();
        info.author_fee_to = ctx.accounts.author_fee_to.key();

        info.issue_fee_bps = 0;
        info.withdraw_fee_bps = 0;

        emit!(NewCrateEvent {
            issue_authority: ctx.accounts.issue_authority.key(),
            withdraw_authority: ctx.accounts.withdraw_authority.key(),
            crate_key: ctx.accounts.crate_token.key(),
        });

        Ok(())
    }

    /// Set the issue fee.
    #[access_control(ctx.accounts.validate())]
    pub fn set_issue_fee(ctx: Context<SetFees>, issue_fee_bps: u16) -> ProgramResult {
        require!(issue_fee_bps <= MAX_FEE_BPS, MaxFeeExceeded);
        let crate_token = &mut ctx.accounts.crate_token;
        crate_token.issue_fee_bps = issue_fee_bps;
        Ok(())
    }

    /// Set the withdraw fee.
    #[access_control(ctx.accounts.validate())]
    pub fn set_withdraw_fee(ctx: Context<SetFees>, withdraw_fee_bps: u16) -> ProgramResult {
        require!(withdraw_fee_bps <= MAX_FEE_BPS, MaxFeeExceeded);
        let crate_token = &mut ctx.accounts.crate_token;
        crate_token.withdraw_fee_bps = withdraw_fee_bps;
        Ok(())
    }

    /// Issues Crate tokens.
    #[access_control(ctx.accounts.validate())]
    pub fn issue(ctx: Context<Issue>, amount: u64) -> ProgramResult {
        // Do nothing if there is a zero amount.
        if amount == 0 {
            return Ok(());
        }

        let seeds: &[&[u8]] = gen_crate_signer_seeds!(ctx.accounts.crate_token);
        let crate_token = &ctx.accounts.crate_token;
        let state::Fees {
            amount,
            author_fee,
            protocol_fee,
        } = crate_token.apply_issue_fee(amount)?;

        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::MintTo {
                    mint: ctx.accounts.crate_mint.to_account_info(),
                    to: ctx.accounts.mint_destination.to_account_info(),
                    authority: ctx.accounts.crate_token.to_account_info(),
                },
                &[seeds],
            ),
            amount,
        )?;

        if author_fee > 0 {
            token::mint_to(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    token::MintTo {
                        mint: ctx.accounts.crate_mint.to_account_info(),
                        to: ctx.accounts.author_fee_destination.to_account_info(),
                        authority: ctx.accounts.crate_token.to_account_info(),
                    },
                    &[seeds],
                ),
                author_fee,
            )?;
        }

        if protocol_fee > 0 {
            token::mint_to(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    token::MintTo {
                        mint: ctx.accounts.crate_mint.to_account_info(),
                        to: ctx.accounts.protocol_fee_destination.to_account_info(),
                        authority: ctx.accounts.crate_token.to_account_info(),
                    },
                    &[seeds],
                ),
                protocol_fee,
            )?;
        }

        emit!(IssueEvent {
            crate_key: ctx.accounts.crate_token.key(),
            destination: ctx.accounts.mint_destination.key(),
            amount,
            author_fee,
            protocol_fee
        });

        Ok(())
    }

    /// Withdraws Crate tokens.
    #[access_control(ctx.accounts.validate())]
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> ProgramResult {
        // Do nothing if there is a zero amount.
        if amount == 0 {
            return Ok(());
        }

        let token_program = ctx.accounts.token_program.to_account_info();
        let seeds = gen_crate_signer_seeds!(ctx.accounts.crate_token);
        let signer_seeds: &[&[&[u8]]] = &[seeds];
        let crate_token = &ctx.accounts.crate_token;
        let state::Fees {
            amount,
            author_fee,
            protocol_fee,
        } = crate_token.apply_withdraw_fee(amount)?;

        // share
        token::transfer(
            CpiContext::new_with_signer(
                token_program.clone(),
                token::Transfer {
                    from: ctx.accounts.crate_underlying.to_account_info(),
                    to: ctx.accounts.withdraw_destination.to_account_info(),
                    authority: ctx.accounts.crate_token.to_account_info(),
                },
                signer_seeds,
            ),
            amount,
        )?;

        if author_fee > 0 {
            token::transfer(
                CpiContext::new_with_signer(
                    token_program.clone(),
                    token::Transfer {
                        from: ctx.accounts.crate_underlying.to_account_info(),
                        to: ctx.accounts.author_fee_destination.to_account_info(),
                        authority: ctx.accounts.crate_token.to_account_info(),
                    },
                    signer_seeds,
                ),
                author_fee,
            )?;
        }

        if protocol_fee > 0 {
            token::transfer(
                CpiContext::new_with_signer(
                    token_program.clone(),
                    token::Transfer {
                        from: ctx.accounts.crate_underlying.to_account_info(),
                        to: ctx.accounts.protocol_fee_destination.to_account_info(),
                        authority: ctx.accounts.crate_token.to_account_info(),
                    },
                    signer_seeds,
                ),
                protocol_fee,
            )?;
        }

        emit!(WithdrawEvent {
            crate_key: ctx.accounts.crate_token.key(),
            token: ctx.accounts.crate_underlying.mint,
            destination: ctx.accounts.withdraw_destination.key(),
            amount,
            author_fee,
            protocol_fee,
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
            b"CrateToken".as_ref(),
            crate_mint.key().to_bytes().as_ref()
        ],
        bump = bump,
        payer = payer
    )]
    pub crate_token: Account<'info, CrateToken>,

    /// [Mint] of the [CrateToken].
    pub crate_mint: Account<'info, Mint>,

    /// The authority that can set fees.
    pub fee_setter_authority: UncheckedAccount<'info>,

    /// The authority that can issue new [CrateToken] tokens.
    pub issue_authority: UncheckedAccount<'info>,

    /// The authority that can redeem the [CrateToken] token underlying.
    pub withdraw_authority: UncheckedAccount<'info>,

    /// Owner of the author fee accounts.
    pub author_fee_to: UncheckedAccount<'info>,

    /// Payer of the crate initialization.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// System program.
    pub system_program: Program<'info, System>,
}

/// Accounts for [crate_token::set_issue_fee] and [crate_token::set_withdraw_fee].
#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct SetFees<'info> {
    /// Information about the crate.
    #[account(mut)]
    pub crate_token: Account<'info, CrateToken>,

    /// Account that can set the fees.
    pub fee_setter: Signer<'info>,
}

/// Accounts for [crate_token::issue].
#[derive(Accounts)]
pub struct Issue<'info> {
    /// Information about the crate.
    pub crate_token: Account<'info, CrateToken>,

    /// [Mint] of the [CrateToken].
    #[account(mut)]
    pub crate_mint: Account<'info, Mint>,

    /// Authority of the account issuing Crate tokens.
    pub issue_authority: Signer<'info>,

    /// Destination of the minted tokens.
    #[account(mut)]
    pub mint_destination: Account<'info, TokenAccount>,

    /// Destination of the author fee tokens.
    #[account(mut)]
    pub author_fee_destination: Account<'info, TokenAccount>,

    /// Destination of the protocol fee tokens.
    #[account(mut)]
    pub protocol_fee_destination: Account<'info, TokenAccount>,

    /// [Token] program.
    pub token_program: Program<'info, Token>,
}

/// Accounts for [crate_token::withdraw].
#[derive(Accounts)]
pub struct Withdraw<'info> {
    /// Information about the crate.
    pub crate_token: Account<'info, CrateToken>,

    /// Crate-owned account of the tokens
    #[account(mut)]
    pub crate_underlying: Account<'info, TokenAccount>,

    /// Authority that can withdraw.
    pub withdraw_authority: Signer<'info>,

    /// Destination of the withdrawn tokens.
    #[account(mut)]
    pub withdraw_destination: Account<'info, TokenAccount>,

    /// Destination of the author fee tokens.
    #[account(mut)]
    pub author_fee_destination: Account<'info, TokenAccount>,

    /// Destination of the protocol fee tokens.
    #[account(mut)]
    pub protocol_fee_destination: Account<'info, TokenAccount>,

    /// [Token] program.
    pub token_program: Program<'info, Token>,
}

#[error]
/// Error codes.
pub enum ErrorCode {
    #[msg("Maximum fee exceeded.")]
    MaxFeeExceeded,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_fee_to_address() {
        let (key, bump) = Pubkey::find_program_address(&[b"CrateFees"], &crate::ID);
        assert_eq!(key, FEE_TO_ADDRESS);
        assert_eq!(bump, 254);
    }
}
