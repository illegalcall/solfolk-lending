use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::errors::SpokeError;
use crate::events::AssetRegistered;
use crate::state::{AssetConfig, SpokeState, MAX_SUPPORTED_ASSETS};

#[derive(Accounts)]
pub struct RegisterAsset<'info> {
    #[account(
        mut,
        seeds = [b"spoke_state"],
        bump = spoke_state.bump,
        constraint = spoke_state.authority == authority.key() @ SpokeError::Unauthorized,
    )]
    pub spoke_state: Account<'info, SpokeState>,

    #[account(
        init,
        payer = authority,
        space = 8 + AssetConfig::INIT_SPACE,
        seeds = [b"asset_config", mint.key().as_ref()],
        bump,
    )]
    pub asset_config: Account<'info, AssetConfig>,

    pub mint: Account<'info, Mint>,

    #[account(
        init,
        payer = authority,
        token::mint = mint,
        token::authority = spoke_state,
        seeds = [b"asset_vault", mint.key().as_ref()],
        bump,
    )]
    pub vault: Account<'info, TokenAccount>,

    /// CHECK: Pyth oracle feed account — validated by the oracle update instruction
    pub oracle_feed: UncheckedAccount<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(
    ctx: Context<RegisterAsset>,
    max_deposit: u64,
    collateral_factor_bps: u16,
    liquidation_threshold_bps: u16,
    liquidation_bonus_bps: u16,
) -> Result<()> {
    let spoke = &mut ctx.accounts.spoke_state;
    require!(
        spoke.supported_assets < MAX_SUPPORTED_ASSETS,
        SpokeError::MaxAssetsReached
    );
    require!(!spoke.paused, SpokeError::ProtocolPaused);

    // Validate parameters
    require!(collateral_factor_bps <= 9500, SpokeError::MathOverflow); // Max 95%
    require!(
        liquidation_threshold_bps > collateral_factor_bps,
        SpokeError::MathOverflow
    );
    require!(liquidation_bonus_bps <= 2000, SpokeError::MathOverflow); // Max 20%

    let asset = &mut ctx.accounts.asset_config;
    asset.mint = ctx.accounts.mint.key();
    asset.vault = ctx.accounts.vault.key();
    asset.oracle_feed = ctx.accounts.oracle_feed.key();
    asset.max_deposit = max_deposit;
    asset.deposit_enabled = true;
    asset.total_deposited = 0;
    asset.decimals = ctx.accounts.mint.decimals;
    asset.interest_rate_bps = 0; // Set by hub
    asset.last_oracle_price = 0;
    asset.last_oracle_timestamp = 0;
    asset.collateral_factor_bps = collateral_factor_bps;
    asset.liquidation_threshold_bps = liquidation_threshold_bps;
    asset.liquidation_bonus_bps = liquidation_bonus_bps;
    asset.bump = ctx.bumps.asset_config;

    spoke.supported_assets = spoke
        .supported_assets
        .checked_add(1)
        .ok_or(SpokeError::MathOverflow)?;

    emit!(AssetRegistered {
        spoke: spoke.key(),
        asset_config: asset.key(),
        mint: asset.mint,
        oracle_feed: asset.oracle_feed,
        max_deposit,
    });

    msg!("Asset registered: mint={}", asset.mint);
    Ok(())
}
