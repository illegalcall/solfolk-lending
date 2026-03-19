# SolFolk Lending — Cross-Chain Spoke Lending Protocol

A Solana-side spoke program for a hub-and-spoke cross-chain lending protocol, built with Anchor. Designed to integrate with a hub chain via Wormhole message passing for unified cross-chain lending state management.

## Why This Architecture?

Traditional lending protocols deploy isolated instances on each chain. The hub-and-spoke model centralizes risk management and state on a hub chain while spokes handle token custody and user-facing operations on their respective chains. This enables:

- **Cross-chain collateralization** — Deposit on one chain, borrow on another (hub manages unified positions)
- **Unified risk engine** — Single source of truth for health factors and liquidation
- **Chain-specific optimization** — Each spoke leverages native features (Solana's speed, EVM's composability)

This mirrors the architecture used by [Folks Finance xChain](https://docs.xapp.folks.finance/xlending/architecture).

## Architecture

```
                        ┌───────────────────────┐
                        │      Hub Chain        │
                        │  (Unified Lending     │
                        │   State & Risk Engine)│
                        └───────┬───────────────┘
                                │ Wormhole Messages
                    ┌───────────┼───────────────┐
                    │           │               │
            ┌───────┴──┐   ┌────┴─────┐  ┌──────┴───┐
            │ Solana   │   │ Avalanche│  │   Base   │
            │ Spoke    │   │  Spoke   │  │  Spoke   │
            │ (this)   │   │          │  │          │
            └──────────┘   └──────────┘  └──────────┘
```

### Spoke Responsibilities
- Token custody (PDA vaults)
- User deposit/withdrawal
- Oracle price caching and validation
- Liquidation execution
- Message relay to/from hub

### Hub Responsibilities
- Borrow limit calculation
- Interest rate management
- Cross-chain health factor
- Withdrawal approval
- Liquidation ordering

## Program Instructions

| Instruction | Description | Access |
|---|---|---|
| `initialize` | Set up spoke with hub chain config | Authority |
| `register_asset` | Add supported collateral asset | Authority |
| `deposit` | Deposit SPL tokens as collateral | Any user |
| `withdraw` | Withdraw collateral (health check) | Depositor |
| `process_hub_message` | Parse inbound hub message | Keeper/Authority |
| `apply_borrow_approval` | Set user's borrow limit from hub | Keeper/Authority |
| `liquidate` | Seize collateral from unhealthy position | Any (keeper incentivized) |
| `update_oracle` | Refresh cached oracle price data | Any |
| `pause` / `unpause` | Emergency circuit breaker | Authority only |
| `update_keeper` | Change keeper address | Authority only |

## Wormhole Message Protocol

The spoke defines message types for cross-chain communication. Currently, `process_hub_message` deserializes and logs inbound messages; `apply_borrow_approval` is the first hub action fully wired up. Remaining hub message handlers (interest rate updates, withdrawal approvals, collateral parameter changes) are defined as types and ready to be connected as the hub implementation progresses.

### Spoke → Hub
```rust
DepositNotification { user, asset_mint, amount, oracle_price, timestamp }
WithdrawalRequest { user, asset_mint, amount }
OraclePriceUpdate { asset_mint, price, confidence, timestamp }
Heartbeat { slot, total_deposits, num_users }
```

### Hub → Spoke
```rust
BorrowApproval { user, asset_mint, approved_amount }          // ← applied via apply_borrow_approval
LiquidationOrder { user, asset_mint, amount_to_liquidate, liquidator_reward_bps }
InterestRateUpdate { asset_mint, rate_bps }
WithdrawalApproval { user, asset_mint, approved_amount }
CollateralParameterUpdate { asset_mint, collateral_factor_bps, ... }
```

## Security

- **Oracle staleness**: Rejects prices older than 30 seconds
- **Oracle confidence**: Rejects if confidence interval > 1% of price
- **Checked arithmetic**: All math uses `checked_*` methods
- **PDA validation**: All accounts derived from deterministic seeds
- **Access control**: Authority for admin ops, keeper for operational tasks
- **Health checks**: Withdrawal blocked if it would make position liquidatable
- **Liquidation bonus cap**: Max 20% enforced at asset registration
- **Emergency pause**: Authority can halt all operations instantly

## Building

```bash
# Build
anchor build  # or: cargo build --lib

# Test
cargo test --lib

# Deploy to devnet
solana config set --url devnet
anchor deploy
```

## Project Structure

```
programs/spoke-lending/src/
├── lib.rs                     # Program entrypoint (11 instructions)
├── state/
│   ├── spoke_state.rs         # Global protocol config
│   ├── asset_config.rs        # Per-asset risk parameters
│   └── user_deposit.rs        # Per-user deposit with health tracking
├── instructions/
│   ├── initialize.rs          # Spoke setup
│   ├── register_asset.rs      # Asset onboarding
│   ├── deposit.rs             # Collateral deposit
│   ├── withdraw.rs            # Collateral withdrawal
│   ├── process_hub_message.rs # Inbound message handler
│   ├── liquidate.rs           # Liquidation engine
│   ├── update_oracle.rs       # Oracle price refresh
│   └── admin.rs               # Pause/unpause/keeper mgmt
├── wormhole/
│   └── messages.rs            # Cross-chain message types
├── errors.rs                  # 27 custom error codes
└── events.rs                  # Event emission for indexing
```

## Key Design Decisions

1. **Keeper role separate from authority**: Keepers can relay hub messages and trigger `apply_borrow_approval`, but cannot modify protocol parameters or withdraw funds. Reduces blast radius of keeper key compromise.

2. **Oracle price is caller-provided, not read from Pyth on-chain**: The `update_oracle` instruction accepts price/confidence/timestamp as arguments and validates staleness and confidence bounds. In production, this would deserialize a Pyth `PriceUpdateV2` account directly. The current design keeps the oracle interface simple while the hub integration is developed.

3. **Simplified Wormhole integration**: In production, `process_hub_message` would verify a Wormhole VAA (posted_vaa account, emitter chain/address validation, sequence-based replay protection). This implementation accepts pre-verified messages from authorized relayers for testing. The message types and serialization format are production-ready.

4. **Per-asset liquidation parameters**: Each asset has independent collateral factor, liquidation threshold, and liquidation bonus. Enables granular risk management matching the hub's risk engine.

5. **Spoke handles collateral, hub handles borrowing**: This spoke does not have a `borrow` instruction — borrowing is managed by the hub which sets `hub_approved_borrow` limits on each spoke position. The spoke enforces these limits during withdrawal health checks.

## References

- [Folks Finance xChain Architecture](https://docs.xapp.folks.finance/xlending/architecture)
- [Wormhole Solana SDK](https://github.com/wormhole-foundation/wormhole/tree/main/solana)
- [Pyth Price Feeds on Solana](https://docs.pyth.network/price-feeds/use-real-time-data/solana)
- [Sealevel Attacks](https://github.com/coral-xyz/sealevel-attacks)

## License

MIT
