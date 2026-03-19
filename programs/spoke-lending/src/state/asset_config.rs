use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct AssetConfig {
    /// SPL token mint for this asset
    pub mint: Pubkey,
    /// PDA vault holding deposited tokens
    pub vault: Pubkey,
    /// Pyth price feed account
    pub oracle_feed: Pubkey,
    /// Maximum total deposit allowed for this asset
    pub max_deposit: u64,
    /// Whether deposits are currently enabled
    pub deposit_enabled: bool,
    /// Total amount deposited across all users
    pub total_deposited: u64,
    /// Token decimals (cached from mint)
    pub decimals: u8,
    /// Interest rate in basis points (set by hub)
    pub interest_rate_bps: u64,
    /// Last oracle price (cached for quick reads)
    pub last_oracle_price: u64,
    /// Last oracle update timestamp
    pub last_oracle_timestamp: i64,
    /// Collateral factor in basis points (e.g., 8000 = 80%)
    pub collateral_factor_bps: u16,
    /// Liquidation threshold in basis points (e.g., 8500 = 85%)
    pub liquidation_threshold_bps: u16,
    /// Liquidation bonus in basis points (e.g., 500 = 5%)
    pub liquidation_bonus_bps: u16,
    /// PDA bump
    pub bump: u8,
}

impl AssetConfig {
    /// Calculate the collateral value of a deposit amount in USD terms.
    /// Returns value scaled by 10^6 (6 decimal places).
    pub fn calculate_collateral_value(&self, amount: u64) -> Option<u64> {
        let value = (amount as u128)
            .checked_mul(self.last_oracle_price as u128)?
            .checked_div(10u128.pow(self.decimals as u32))?;

        let collateral_value = value
            .checked_mul(self.collateral_factor_bps as u128)?
            .checked_div(10_000)?;

        Some(collateral_value as u64)
    }

    /// Check if a deposit of `amount` would exceed the cap.
    pub fn can_deposit(&self, amount: u64) -> bool {
        self.deposit_enabled
            && self.total_deposited.checked_add(amount).map_or(false, |total| total <= self.max_deposit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_config() -> AssetConfig {
        AssetConfig {
            mint: Pubkey::default(),
            vault: Pubkey::default(),
            oracle_feed: Pubkey::default(),
            max_deposit: 1_000_000_000_000, // 1M tokens (6 decimals)
            deposit_enabled: true,
            total_deposited: 0,
            decimals: 6,
            interest_rate_bps: 500, // 5%
            last_oracle_price: 1_000_000, // $1.00 (6 decimals)
            last_oracle_timestamp: 0,
            collateral_factor_bps: 8000, // 80%
            liquidation_threshold_bps: 8500,
            liquidation_bonus_bps: 500,
            bump: 0,
        }
    }

    #[test]
    fn test_collateral_value() {
        let config = mock_config();
        // 1000 tokens at $1.00, 80% collateral factor = $800
        let value = config.calculate_collateral_value(1_000_000_000).unwrap();
        assert_eq!(value, 800_000_000); // $800 in 6 decimals
    }

    #[test]
    fn test_can_deposit() {
        let mut config = mock_config();
        assert!(config.can_deposit(100_000_000));

        config.deposit_enabled = false;
        assert!(!config.can_deposit(100_000_000));

        config.deposit_enabled = true;
        config.total_deposited = config.max_deposit;
        assert!(!config.can_deposit(1));
    }
}
