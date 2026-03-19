use anchor_lang::prelude::*;

use crate::errors::SpokeError;
use crate::events::OracleUpdated;
use crate::state::{AssetConfig, SpokeState};

/// Maximum staleness for oracle data: 30 seconds
pub const MAX_ORACLE_STALENESS: i64 = 30;
/// Maximum confidence interval: 1% of price
pub const MAX_CONFIDENCE_RATIO_BPS: u64 = 100;

#[derive(Accounts)]
pub struct UpdateOracle<'info> {
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

    /// CHECK: In production, this would be a Pyth PriceUpdateV2 account.
    /// We validate the key matches the registered oracle_feed.
    #[account(
        constraint = oracle_feed.key() == asset_config.oracle_feed @ SpokeError::OracleInvalidPrice,
    )]
    pub oracle_feed: UncheckedAccount<'info>,

    /// Keeper or authority updating the oracle
    pub updater: Signer<'info>,
}

pub fn handler(
    ctx: Context<UpdateOracle>,
    price: u64,
    confidence: u64,
    publish_timestamp: i64,
) -> Result<()> {
    // Validate oracle data
    require!(price > 0, SpokeError::OracleInvalidPrice);

    let current_time = Clock::get()?.unix_timestamp;
    let staleness = current_time.saturating_sub(publish_timestamp);
    require!(staleness <= MAX_ORACLE_STALENESS, SpokeError::OracleStale);

    // Check confidence interval
    if price > 0 {
        let confidence_ratio_bps = (confidence as u128)
            .checked_mul(10_000)
            .unwrap_or(u128::MAX)
            / (price as u128);
        require!(
            confidence_ratio_bps <= MAX_CONFIDENCE_RATIO_BPS as u128,
            SpokeError::OracleConfidenceTooWide
        );
    }

    // Update cached oracle data
    let asset = &mut ctx.accounts.asset_config;
    asset.last_oracle_price = price;
    asset.last_oracle_timestamp = publish_timestamp;

    emit!(OracleUpdated {
        asset: asset.mint,
        price,
        confidence,
        timestamp: publish_timestamp,
    });

    Ok(())
}
