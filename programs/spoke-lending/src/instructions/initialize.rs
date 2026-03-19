use anchor_lang::prelude::*;

use crate::events::SpokeInitialized;
use crate::state::SpokeState;

#[derive(Accounts)]
pub struct InitializeSpoke<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + SpokeState::INIT_SPACE,
        seeds = [b"spoke_state"],
        bump,
    )]
    pub spoke_state: Account<'info, SpokeState>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitializeSpoke>,
    hub_chain_id: u16,
    hub_emitter: [u8; 32],
) -> Result<()> {
    let spoke = &mut ctx.accounts.spoke_state;
    spoke.authority = ctx.accounts.authority.key();
    spoke.keeper = ctx.accounts.authority.key(); // Initially same as authority
    spoke.hub_chain_id = hub_chain_id;
    spoke.hub_emitter = hub_emitter;
    spoke.paused = false;
    spoke.supported_assets = 0;
    spoke.message_sequence = 0;
    spoke.last_hub_sequence = 0;
    spoke.last_hub_sync = Clock::get()?.unix_timestamp;
    spoke.bump = ctx.bumps.spoke_state;

    emit!(SpokeInitialized {
        spoke: spoke.key(),
        authority: spoke.authority,
        hub_chain_id,
    });

    msg!("Spoke initialized: hub_chain_id={}", hub_chain_id);
    Ok(())
}
