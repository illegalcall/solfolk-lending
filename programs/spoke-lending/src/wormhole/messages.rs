use anchor_lang::prelude::*;

/// Messages sent from this Solana spoke to the hub chain.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum SpokeToHubMessage {
    /// Notify hub of a new deposit on this spoke.
    DepositNotification {
        user: Pubkey,
        asset_mint: Pubkey,
        amount: u64,
        oracle_price: u64,
        timestamp: i64,
    },
    /// Request hub approval for a withdrawal.
    WithdrawalRequest {
        user: Pubkey,
        asset_mint: Pubkey,
        amount: u64,
    },
    /// Send updated oracle price to hub for risk calculations.
    OraclePriceUpdate {
        asset_mint: Pubkey,
        price: u64,
        confidence: u64,
        timestamp: i64,
    },
    /// Heartbeat to confirm spoke is operational.
    Heartbeat {
        slot: u64,
        total_deposits: u64,
        num_users: u32,
    },
}

/// Messages received from the hub chain to this Solana spoke.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum HubToSpokeMessage {
    /// Hub approves a borrow limit for a user.
    BorrowApproval {
        user: Pubkey,
        asset_mint: Pubkey,
        approved_amount: u64,
    },
    /// Hub orders liquidation of a user's position.
    LiquidationOrder {
        user: Pubkey,
        asset_mint: Pubkey,
        amount_to_liquidate: u64,
        liquidator_reward_bps: u16,
    },
    /// Hub updates interest rate for an asset.
    InterestRateUpdate {
        asset_mint: Pubkey,
        rate_bps: u64,
    },
    /// Hub approves a pending withdrawal.
    WithdrawalApproval {
        user: Pubkey,
        asset_mint: Pubkey,
        approved_amount: u64,
    },
    /// Hub updates collateral parameters for an asset.
    CollateralParameterUpdate {
        asset_mint: Pubkey,
        collateral_factor_bps: u16,
        liquidation_threshold_bps: u16,
        liquidation_bonus_bps: u16,
    },
}

impl SpokeToHubMessage {
    /// Serialize the message to bytes for Wormhole payload.
    pub fn to_payload(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        self.serialize(&mut buf).expect("serialization failed");
        buf
    }

    /// Get a human-readable message type string.
    pub fn message_type(&self) -> &str {
        match self {
            Self::DepositNotification { .. } => "DepositNotification",
            Self::WithdrawalRequest { .. } => "WithdrawalRequest",
            Self::OraclePriceUpdate { .. } => "OraclePriceUpdate",
            Self::Heartbeat { .. } => "Heartbeat",
        }
    }
}

impl HubToSpokeMessage {
    /// Deserialize a message from Wormhole payload bytes.
    pub fn from_payload(data: &[u8]) -> Result<Self> {
        let mut cursor = data;
        Self::deserialize(&mut cursor).map_err(|_| error!(crate::errors::SpokeError::InvalidHubMessage))
    }

    /// Get a human-readable message type string.
    pub fn message_type(&self) -> &str {
        match self {
            Self::BorrowApproval { .. } => "BorrowApproval",
            Self::LiquidationOrder { .. } => "LiquidationOrder",
            Self::InterestRateUpdate { .. } => "InterestRateUpdate",
            Self::WithdrawalApproval { .. } => "WithdrawalApproval",
            Self::CollateralParameterUpdate { .. } => "CollateralParameterUpdate",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spoke_message_roundtrip() {
        let msg = SpokeToHubMessage::DepositNotification {
            user: Pubkey::default(),
            asset_mint: Pubkey::default(),
            amount: 1_000_000,
            oracle_price: 50_000_000,
            timestamp: 12345,
        };
        let payload = msg.to_payload();
        assert!(!payload.is_empty());
    }

    #[test]
    fn test_hub_message_roundtrip() {
        let msg = HubToSpokeMessage::BorrowApproval {
            user: Pubkey::default(),
            asset_mint: Pubkey::default(),
            approved_amount: 500_000,
        };
        let mut buf = Vec::new();
        msg.serialize(&mut buf).unwrap();
        let decoded = HubToSpokeMessage::from_payload(&buf).unwrap();
        assert_eq!(decoded.message_type(), "BorrowApproval");
    }
}
