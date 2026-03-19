use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::errors::SpokeError;
use crate::events::LiquidationExecuted;
use crate::state::{AssetConfig, SpokeState, UserDeposit};

#[derive(Accounts)]
pub struct Liquidate<'info> {
    #[account(
        seeds = [b"spoke_state"],
        bump = spoke_state.bump,
    )]
    pub spoke_state: Account<'info, SpokeState>,

    #[account(
        seeds = [b"asset_config", asset_config.mint.as_ref()],
        bump = asset_config.bump,
    )]
    pub asset_config: Account<'info, AssetConfig>,

    #[account(
        mut,
        constraint = user_deposit.asset_config == asset_config.key() @ SpokeError::AssetNotRegistered,
    )]
    pub user_deposit: Account<'info, UserDeposit>,

    #[account(
        mut,
        constraint = asset_vault.key() == asset_config.vault @ SpokeError::Unauthorized,
    )]
    pub asset_vault: Account<'info, TokenAccount>,

    /// Liquidator receives the liquidation bonus
    #[account(mut)]
    pub liquidator_token_account: Account<'info, TokenAccount>,

    /// Anyone can call liquidate (keeper incentivized)
    pub liquidator: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<Liquidate>, amount_to_liquidate: u64) -> Result<()> {
    let spoke = &ctx.accounts.spoke_state;
    require!(!spoke.paused, SpokeError::ProtocolPaused);

    let asset = &ctx.accounts.asset_config;
    let user_deposit = &ctx.accounts.user_deposit;

    // Verify position is actually liquidatable
    let collateral_value = asset
        .calculate_collateral_value(user_deposit.deposited_amount)
        .ok_or(SpokeError::MathOverflow)?;

    require!(
        user_deposit.is_liquidatable(collateral_value, asset.liquidation_threshold_bps),
        SpokeError::PositionHealthy
    );

    // Cap liquidation amount
    require!(
        amount_to_liquidate <= user_deposit.deposited_amount,
        SpokeError::LiquidationTooLarge
    );

    // Calculate liquidator reward
    let liquidator_reward = (amount_to_liquidate as u128)
        .checked_mul(asset.liquidation_bonus_bps as u128)
        .ok_or(SpokeError::MathOverflow)?
        / 10_000;
    let liquidator_reward = liquidator_reward as u64;

    let total_seized = amount_to_liquidate
        .checked_add(liquidator_reward)
        .ok_or(SpokeError::LiquidationRewardOverflow)?;

    // Cap total seized to available deposit
    let actual_seized = total_seized.min(user_deposit.deposited_amount);

    // Transfer seized collateral to liquidator
    let seeds = &[b"spoke_state".as_ref(), &[spoke.bump]];
    let signer_seeds = &[&seeds[..]];

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.asset_vault.to_account_info(),
                to: ctx.accounts.liquidator_token_account.to_account_info(),
                authority: ctx.accounts.spoke_state.to_account_info(),
            },
            signer_seeds,
        ),
        actual_seized,
    )?;

    // Update user deposit
    let user_deposit = &mut ctx.accounts.user_deposit;
    user_deposit.deposited_amount = user_deposit
        .deposited_amount
        .checked_sub(actual_seized)
        .ok_or(SpokeError::MathOverflow)?;

    // Reduce borrowed amount proportionally
    if user_deposit.borrowed_amount > 0 {
        let repay_ratio = (amount_to_liquidate as u128)
            .checked_mul(10_000)
            .ok_or(SpokeError::MathOverflow)?
            / (user_deposit.deposited_amount.checked_add(actual_seized).ok_or(SpokeError::MathOverflow)? as u128);

        let debt_repaid = (user_deposit.borrowed_amount as u128)
            .checked_mul(repay_ratio)
            .ok_or(SpokeError::MathOverflow)?
            / 10_000;

        user_deposit.borrowed_amount = user_deposit
            .borrowed_amount
            .saturating_sub(debt_repaid as u64);
    }

    emit!(LiquidationExecuted {
        user: user_deposit.owner,
        asset: asset.mint,
        amount_liquidated: actual_seized,
        liquidator: ctx.accounts.liquidator.key(),
        liquidator_reward,
    });

    msg!(
        "Liquidation: {} tokens seized, {} reward to liquidator",
        actual_seized,
        liquidator_reward
    );

    Ok(())
}
