use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct UserDeposit {
    /// Depositor's wallet
    pub owner: Pubkey,
    /// Links to the AssetConfig for this deposit
    pub asset_config: Pubkey,
    /// Amount of tokens deposited
    pub deposited_amount: u64,
    /// Timestamp of first deposit
    pub deposit_timestamp: i64,
    /// Last time this deposit was synced with the hub
    pub last_hub_sync_slot: u64,
    /// Borrow limit approved by the hub (in USD, 6 decimals)
    pub hub_approved_borrow: u64,
    /// Amount already borrowed against this deposit
    pub borrowed_amount: u64,
    /// Pending withdrawal amount (awaiting hub approval)
    pub pending_withdrawal: u64,
    /// Whether a withdrawal is pending hub approval
    pub withdrawal_pending: bool,
    /// PDA bump
    pub bump: u8,
}

impl UserDeposit {
    /// Calculate health factor: (collateral_value / borrowed_amount) * 100
    /// Returns percentage (100 = healthy at 1:1, higher is safer)
    pub fn calculate_health_factor(&self, collateral_value_usd: u64) -> u64 {
        if self.borrowed_amount == 0 {
            return u64::MAX; // No borrows = infinite health
        }
        (collateral_value_usd as u128)
            .checked_mul(100)
            .and_then(|v| v.checked_div(self.borrowed_amount as u128))
            .unwrap_or(0) as u64
    }

    /// Check if position is liquidatable based on liquidation threshold.
    pub fn is_liquidatable(
        &self,
        collateral_value_usd: u64,
        liquidation_threshold_bps: u16,
    ) -> bool {
        if self.borrowed_amount == 0 {
            return false;
        }
        // Liquidatable when: collateral_value / borrowed < threshold
        let threshold_value = (self.borrowed_amount as u128)
            .checked_mul(liquidation_threshold_bps as u128)
            .map(|v| v / 10_000)
            .unwrap_or(u128::MAX);

        (collateral_value_usd as u128) < threshold_value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_deposit() -> UserDeposit {
        UserDeposit {
            owner: Pubkey::default(),
            asset_config: Pubkey::default(),
            deposited_amount: 1_000_000_000, // 1000 tokens
            deposit_timestamp: 0,
            last_hub_sync_slot: 0,
            hub_approved_borrow: 800_000_000, // $800
            borrowed_amount: 0,
            pending_withdrawal: 0,
            withdrawal_pending: false,
            bump: 0,
        }
    }

    #[test]
    fn test_health_factor_no_borrows() {
        let deposit = mock_deposit();
        assert_eq!(deposit.calculate_health_factor(800_000_000), u64::MAX);
    }

    #[test]
    fn test_health_factor_with_borrows() {
        let mut deposit = mock_deposit();
        deposit.borrowed_amount = 400_000_000; // $400
        // Collateral $800, borrowed $400 → health = 200%
        assert_eq!(deposit.calculate_health_factor(800_000_000), 200);
    }

    #[test]
    fn test_not_liquidatable_healthy() {
        let mut deposit = mock_deposit();
        deposit.borrowed_amount = 400_000_000;
        // Collateral $800, threshold 85% → threshold_value = $340
        // $800 > $340 → not liquidatable
        assert!(!deposit.is_liquidatable(800_000_000, 8500));
    }

    #[test]
    fn test_liquidatable_underwater() {
        let mut deposit = mock_deposit();
        deposit.borrowed_amount = 1_000_000_000; // $1000
        // Collateral $800, threshold 85% → threshold_value = $850
        // $800 < $850 → liquidatable
        assert!(deposit.is_liquidatable(800_000_000, 8500));
    }
}
