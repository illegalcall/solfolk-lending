use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::errors::SpokeError;
use crate::events::DepositMade;
use crate::state::{AssetConfig, SpokeState, UserDeposit};

#[derive(Accounts)]
pub struct Deposit<'info> {
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
        init_if_needed,
        payer = depositor,
        space = 8 + UserDeposit::INIT_SPACE,
        seeds = [
            b"user_deposit",
            depositor.key().as_ref(),
            asset_config.mint.as_ref(),
        ],
        bump,
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
    pub depositor: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    require!(amount > 0, SpokeError::ZeroDeposit);

    let spoke = &ctx.accounts.spoke_state;
    require!(!spoke.paused, SpokeError::ProtocolPaused);

    let asset = &ctx.accounts.asset_config;
    require!(asset.can_deposit(amount), SpokeError::DepositCapExceeded);

    // Transfer tokens from user to vault
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_account.to_account_info(),
                to: ctx.accounts.asset_vault.to_account_info(),
                authority: ctx.accounts.depositor.to_account_info(),
            },
        ),
        amount,
    )?;

    // Update user deposit
    let user_deposit = &mut ctx.accounts.user_deposit;
    if user_deposit.deposited_amount == 0 {
        // First deposit — initialize
        user_deposit.owner = ctx.accounts.depositor.key();
        user_deposit.asset_config = ctx.accounts.asset_config.key();
        user_deposit.deposit_timestamp = Clock::get()?.unix_timestamp;
        user_deposit.hub_approved_borrow = 0;
        user_deposit.borrowed_amount = 0;
        user_deposit.pending_withdrawal = 0;
        user_deposit.withdrawal_pending = false;
        user_deposit.bump = ctx.bumps.user_deposit;
    }
    user_deposit.deposited_amount = user_deposit
        .deposited_amount
        .checked_add(amount)
        .ok_or(SpokeError::MathOverflow)?;

    // Update asset totals
    let asset = &mut ctx.accounts.asset_config;
    asset.total_deposited = asset
        .total_deposited
        .checked_add(amount)
        .ok_or(SpokeError::MathOverflow)?;

    let timestamp = Clock::get()?.unix_timestamp;

    emit!(DepositMade {
        user: ctx.accounts.depositor.key(),
        asset: asset.mint,
        amount,
        oracle_price: asset.last_oracle_price,
        total_deposited: user_deposit.deposited_amount,
        timestamp,
    });

    // In production, this would also send a Wormhole message to the hub:
    // SpokeToHubMessage::DepositNotification { ... }

    msg!(
        "Deposit: {} tokens of {} by {}",
        amount,
        asset.mint,
        ctx.accounts.depositor.key()
    );

    Ok(())
}
