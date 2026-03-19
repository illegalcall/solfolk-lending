use anchor_lang::prelude::*;

#[event]
pub struct SpokeInitialized {
    pub spoke: Pubkey,
    pub authority: Pubkey,
    pub hub_chain_id: u16,
}

#[event]
pub struct AssetRegistered {
    pub spoke: Pubkey,
    pub asset_config: Pubkey,
    pub mint: Pubkey,
    pub oracle_feed: Pubkey,
    pub max_deposit: u64,
}

#[event]
pub struct DepositMade {
    pub user: Pubkey,
    pub asset: Pubkey,
    pub amount: u64,
    pub oracle_price: u64,
    pub total_deposited: u64,
    pub timestamp: i64,
}

#[event]
pub struct WithdrawalMade {
    pub user: Pubkey,
    pub asset: Pubkey,
    pub amount: u64,
    pub remaining_deposit: u64,
    pub timestamp: i64,
}

#[event]
pub struct WithdrawalRequested {
    pub user: Pubkey,
    pub asset: Pubkey,
    pub amount: u64,
    pub message_sequence: u64,
}

#[event]
pub struct HubMessageSent {
    pub spoke: Pubkey,
    pub message_type: String,
    pub sequence: u64,
    pub payload_hash: [u8; 32],
}

#[event]
pub struct HubMessageProcessed {
    pub spoke: Pubkey,
    pub message_type: String,
    pub sequence: u64,
}

#[event]
pub struct LiquidationExecuted {
    pub user: Pubkey,
    pub asset: Pubkey,
    pub amount_liquidated: u64,
    pub liquidator: Pubkey,
    pub liquidator_reward: u64,
}

#[event]
pub struct OracleUpdated {
    pub asset: Pubkey,
    pub price: u64,
    pub confidence: u64,
    pub timestamp: i64,
}

#[event]
pub struct ProtocolPaused {
    pub spoke: Pubkey,
    pub authority: Pubkey,
}

#[event]
pub struct ProtocolUnpaused {
    pub spoke: Pubkey,
    pub authority: Pubkey,
}

#[event]
pub struct BorrowLimitUpdated {
    pub user: Pubkey,
    pub asset: Pubkey,
    pub new_limit: u64,
}
