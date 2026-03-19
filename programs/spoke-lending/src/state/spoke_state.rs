use anchor_lang::prelude::*;

pub const MAX_SUPPORTED_ASSETS: u8 = 16;

#[account]
#[derive(InitSpace)]
pub struct SpokeState {
    /// Admin authority (ideally a multisig)
    pub authority: Pubkey,
    /// Secondary keeper for operational tasks (oracle updates, message relay)
    pub keeper: Pubkey,
    /// Wormhole chain ID of the hub chain
    pub hub_chain_id: u16,
    /// Expected Wormhole emitter address on the hub chain
    pub hub_emitter: [u8; 32],
    /// Whether the protocol is paused
    pub paused: bool,
    /// Total number of registered assets
    pub supported_assets: u8,
    /// Global message sequence counter
    pub message_sequence: u64,
    /// Last processed hub message sequence (prevents replay)
    pub last_hub_sequence: u64,
    /// Timestamp of last hub sync
    pub last_hub_sync: i64,
    /// PDA bump
    pub bump: u8,
}

impl SpokeState {
    pub fn next_sequence(&mut self) -> u64 {
        let seq = self.message_sequence;
        self.message_sequence = self.message_sequence.saturating_add(1);
        seq
    }
}
