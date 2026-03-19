use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::errors::SpokeError;
use crate::events::WithdrawalMade;
use crate::state::{AssetConfig, SpokeState, UserDeposit};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        seeds = [b"spoke_state"],
        bump = spoke_state.bump,
    )]
    pub spoke_state: Account<'info, SpokeState>,

    #[account(
        mut,
        seeds = [b"asset_config", asset_config.mint.as_ref()],
        bump = asset_config.bump,
    )]
    pub asset_config: Account<'info, AssetConfig>,

    #[account(
        mut,
        seeds = [
            b"user_deposit",
            withdrawer.key().as_ref(),
            asset_config.mint.as_ref(),
        ],
        bump = user_deposit.bump,
        constraint = user_deposit.owner == withdrawer.key() @ SpokeError::Unauthorized,
    )]
    pub user_deposit: Account<'info, UserDeposit>,

    #[account(
        mut,
        constraint = asset_vault.key() == asset_config.vault @ SpokeError::Unauthorized,
    )]
    pub asset_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_token_account.mint == asset_config.mint @ SpokeError::AssetNotRegistered,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub withdrawer: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    require!(amount > 0, SpokeError::ZeroWithdrawal);

    let user_deposit = &ctx.accounts.user_deposit;
    require!(
        user_deposit.deposited_amount >= amount,
        SpokeError::InsufficientDeposit
    );

    // If user has borrows, check that withdrawal doesn't make position unhealthy
    if user_deposit.borrowed_amount > 0 {
        let asset = &ctx.accounts.asset_config;
        let remaining = user_deposit
            .deposited_amount
            .checked_sub(amount)
            .ok_or(SpokeError::MathOverflow)?;
        let remaining_value = asset
            .calculate_collateral_value(remaining)
            .ok_or(SpokeError::MathOverflow)?;

        // After withdrawal, health must stay above liquidation threshold
        let min_collateral = (user_deposit.borrowed_amount as u128)
            .checked_mul(asset.liquidation_threshold_bps as u128)
            .ok_or(SpokeError::MathOverflow)?
            / 10_000;

        require!(
            remaining_value as u128 >= min_collateral,
            SpokeError::WithdrawalExceedsApproval
        );
    }

    // Transfer tokens from vault to user (PDA signer)
    let seeds = &[b"spoke_state".as_ref(), &[ctx.accounts.spoke_state.bump]];
    let signer_seeds = &[&seeds[..]];

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.asset_vault.to_account_info(),
                to: ctx.accounts.user_token_account.to_account_info(),
                authority: ctx.accounts.spoke_state.to_account_info(),
            },
            signer_seeds,
        ),
        amount,
    )?;

    // Update state
    let user_deposit = &mut ctx.accounts.user_deposit;
    user_deposit.deposited_amount = user_deposit
        .deposited_amount
        .checked_sub(amount)
        .ok_or(SpokeError::MathOverflow)?;

    let asset = &mut ctx.accounts.asset_config;
    asset.total_deposited = asset
        .total_deposited
        .checked_sub(amount)
        .ok_or(SpokeError::MathOverflow)?;

    let timestamp = Clock::get()?.unix_timestamp;

    emit!(WithdrawalMade {
        user: ctx.accounts.withdrawer.key(),
        asset: asset.mint,
        amount,
        remaining_deposit: user_deposit.deposited_amount,
        timestamp,
    });

    Ok(())
}
