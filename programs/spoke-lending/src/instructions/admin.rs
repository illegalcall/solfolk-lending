use anchor_lang::prelude::*;

use crate::errors::SpokeError;
use crate::events::{ProtocolPaused, ProtocolUnpaused};
use crate::state::SpokeState;

#[derive(Accounts)]
pub struct AdminAction<'info> {
    #[account(
        mut,
        seeds = [b"spoke_state"],
        bump = spoke_state.bump,
        constraint = spoke_state.authority == authority.key() @ SpokeError::Unauthorized,
    )]
    pub spoke_state: Account<'info, SpokeState>,

    pub authority: Signer<'info>,
}

pub fn pause(ctx: Context<AdminAction>) -> Result<()> {
    let spoke = &mut ctx.accounts.spoke_state;
    spoke.paused = true;

    emit!(ProtocolPaused {
        spoke: spoke.key(),
        authority: ctx.accounts.authority.key(),
    });

    msg!("Protocol paused by {}", ctx.accounts.authority.key());
    Ok(())
}

pub fn unpause(ctx: Context<AdminAction>) -> Result<()> {
    let spoke = &mut ctx.accounts.spoke_state;
    spoke.paused = false;

    emit!(ProtocolUnpaused {
        spoke: spoke.key(),
        authority: ctx.accounts.authority.key(),
    });

    msg!("Protocol unpaused by {}", ctx.accounts.authority.key());
    Ok(())
}

pub fn update_keeper(ctx: Context<AdminAction>, new_keeper: Pubkey) -> Result<()> {
    let spoke = &mut ctx.accounts.spoke_state;
    spoke.keeper = new_keeper;
    msg!("Keeper updated to {}", new_keeper);
    Ok(())
}
