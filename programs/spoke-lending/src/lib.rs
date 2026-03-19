use anchor_lang::prelude::*;

pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;
pub mod wormhole;

use instructions::*;

declare_id!("SPoKELendvz7Qh3rXjSJQnAECH5fYCFoML4gnc4L6SY");

#[program]
pub mod spoke_lending {
    use super::*;

    /// Initialize the spoke protocol with hub chain configuration.
    pub fn initialize(
        ctx: Context<InitializeSpoke>,
        hub_chain_id: u16,
        hub_emitter: [u8; 32],
    ) -> Result<()> {
        instructions::initialize::handler(ctx, hub_chain_id, hub_emitter)
    }

    /// Register a new supported asset with its oracle and risk parameters.
    pub fn register_asset(
        ctx: Context<RegisterAsset>,
        max_deposit: u64,
        collateral_factor_bps: u16,
        liquidation_threshold_bps: u16,
        liquidation_bonus_bps: u16,
    ) -> Result<()> {
        instructions::register_asset::handler(
            ctx,
            max_deposit,
            collateral_factor_bps,
            liquidation_threshold_bps,
            liquidation_bonus_bps,
        )
    }

    /// Deposit tokens as collateral.
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        instructions::deposit::handler(ctx, amount)
    }

    /// Withdraw deposited collateral (subject to health checks).
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        instructions::withdraw::handler(ctx, amount)
    }

    /// Process an inbound message from the hub chain.
    pub fn process_hub_message(
        ctx: Context<ProcessHubMessage>,
        message_data: Vec<u8>,
    ) -> Result<()> {
        instructions::process_hub_message::handler(ctx, message_data)
    }

    /// Apply a borrow approval from the hub to a user's deposit.
    pub fn apply_borrow_approval(
        ctx: Context<ApplyBorrowApproval>,
        approved_amount: u64,
    ) -> Result<()> {
        instructions::process_hub_message::apply_borrow_approval(ctx, approved_amount)
    }

    /// Liquidate an unhealthy position.
    pub fn liquidate(ctx: Context<Liquidate>, amount: u64) -> Result<()> {
        instructions::liquidate::handler(ctx, amount)
    }

    /// Update cached oracle price for an asset.
    pub fn update_oracle(
        ctx: Context<UpdateOracle>,
        price: u64,
        confidence: u64,
        publish_timestamp: i64,
    ) -> Result<()> {
        instructions::update_oracle::handler(ctx, price, confidence, publish_timestamp)
    }

    /// Pause the protocol (authority only).
    pub fn pause(ctx: Context<AdminAction>) -> Result<()> {
        instructions::admin::pause(ctx)
    }

    /// Unpause the protocol (authority only).
    pub fn unpause(ctx: Context<AdminAction>) -> Result<()> {
        instructions::admin::unpause(ctx)
    }

    /// Update the keeper address (authority only).
    pub fn update_keeper(ctx: Context<AdminAction>, new_keeper: Pubkey) -> Result<()> {
        instructions::admin::update_keeper(ctx, new_keeper)
    }
}
