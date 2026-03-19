use anchor_lang::prelude::*;

use crate::errors::SpokeError;
use crate::events::{BorrowLimitUpdated, HubMessageProcessed};
use crate::state::{AssetConfig, SpokeState, UserDeposit};
use crate::wormhole::HubToSpokeMessage;

#[derive(Accounts)]
pub struct ProcessHubMessage<'info> {
    #[account(
        mut,
        seeds = [b"spoke_state"],
        bump = spoke_state.bump,
    )]
    pub spoke_state: Account<'info, SpokeState>,

    /// The keeper or authority relaying the hub message
    pub relayer: Signer<'info>,
}

/// Process an inbound message from the hub chain.
///
/// In production, this would verify a Wormhole VAA:
/// 1. Parse the VAA from posted_vaa account
/// 2. Verify emitter chain matches hub_chain_id
/// 3. Verify emitter address matches hub_emitter
/// 4. Verify sequence > last_hub_sequence (replay protection)
/// 5. Deserialize and process the message
///
/// For this implementation, we accept a serialized message directly
/// from an authorized relayer (keeper or authority).
pub fn handler(ctx: Context<ProcessHubMessage>, message_data: Vec<u8>) -> Result<()> {
    let spoke = &ctx.accounts.spoke_state;

    // Verify relayer is authorized
    require!(
        ctx.accounts.relayer.key() == spoke.authority
            || ctx.accounts.relayer.key() == spoke.keeper,
        SpokeError::UnauthorizedKeeper
    );
    require!(!spoke.paused, SpokeError::ProtocolPaused);

    // Deserialize the hub message
    let message = HubToSpokeMessage::from_payload(&message_data)?;
    let message_type = message.message_type().to_string();

    // Update spoke state
    let spoke = &mut ctx.accounts.spoke_state;
    spoke.last_hub_sequence = spoke.last_hub_sequence.saturating_add(1);
    spoke.last_hub_sync = Clock::get()?.unix_timestamp;

    emit!(HubMessageProcessed {
        spoke: spoke.key(),
        message_type: message_type.clone(),
        sequence: spoke.last_hub_sequence,
    });

    msg!(
        "Hub message processed: type={}, seq={}",
        message_type,
        spoke.last_hub_sequence
    );

    Ok(())
}

/// Separate instruction to apply borrow approval from hub.
/// This modifies the UserDeposit account directly.
#[derive(Accounts)]
pub struct ApplyBorrowApproval<'info> {
    #[account(
        seeds = [b"spoke_state"],
        bump = spoke_state.bump,
        constraint = spoke_state.authority == authority.key()
            || spoke_state.keeper == authority.key()
            @ SpokeError::UnauthorizedKeeper,
    )]
    pub spoke_state: Account<'info, SpokeState>,

    #[account(mut)]
    pub user_deposit: Account<'info, UserDeposit>,

    pub authority: Signer<'info>,
}

pub fn apply_borrow_approval(
    ctx: Context<ApplyBorrowApproval>,
    approved_amount: u64,
) -> Result<()> {
    let user_deposit = &mut ctx.accounts.user_deposit;
    user_deposit.hub_approved_borrow = approved_amount;
    user_deposit.last_hub_sync_slot = Clock::get()?.slot;

    emit!(BorrowLimitUpdated {
        user: user_deposit.owner,
        asset: user_deposit.asset_config,
        new_limit: approved_amount,
    });

    Ok(())
}
