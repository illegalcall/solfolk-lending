use anchor_lang::prelude::*;

#[error_code]
pub enum SpokeError {
    // Authority & Access
    #[msg("Unauthorized: caller is not the spoke authority")]
    Unauthorized,
    #[msg("Unauthorized: caller is not the designated keeper")]
    UnauthorizedKeeper,

    // Pause
    #[msg("Spoke protocol is currently paused")]
    ProtocolPaused,

    // Asset Config
    #[msg("Asset deposits are currently disabled")]
    DepositsDisabled,
    #[msg("Deposit exceeds maximum allowed for this asset")]
    DepositCapExceeded,
    #[msg("Asset is not registered on this spoke")]
    AssetNotRegistered,
    #[msg("Asset is already registered")]
    AssetAlreadyRegistered,
    #[msg("Maximum number of supported assets reached")]
    MaxAssetsReached,

    // Deposits & Withdrawals
    #[msg("Deposit amount must be greater than zero")]
    ZeroDeposit,
    #[msg("Withdrawal amount must be greater than zero")]
    ZeroWithdrawal,
    #[msg("Insufficient deposited balance for withdrawal")]
    InsufficientDeposit,
    #[msg("Withdrawal not approved by hub — submit request first")]
    WithdrawalNotApproved,
    #[msg("Withdrawal amount exceeds hub-approved limit")]
    WithdrawalExceedsApproval,

    // Oracle
    #[msg("Oracle price data is stale — exceeds max staleness")]
    OracleStale,
    #[msg("Oracle price confidence interval too wide")]
    OracleConfidenceTooWide,
    #[msg("Oracle returned invalid or zero price")]
    OracleInvalidPrice,

    // Hub Communication
    #[msg("Invalid hub message: could not deserialize payload")]
    InvalidHubMessage,
    #[msg("Hub message from unauthorized emitter")]
    UnauthorizedEmitter,
    #[msg("Hub message chain ID does not match configuration")]
    InvalidHubChainId,
    #[msg("Message sequence already processed")]
    MessageAlreadyProcessed,
    #[msg("Message has already been sent")]
    MessageAlreadySent,

    // Liquidation
    #[msg("Position is healthy — liquidation not allowed")]
    PositionHealthy,
    #[msg("Liquidation amount exceeds position size")]
    LiquidationTooLarge,
    #[msg("Liquidation reward calculation overflow")]
    LiquidationRewardOverflow,

    // Math
    #[msg("Arithmetic overflow in calculation")]
    MathOverflow,
    #[msg("Division by zero")]
    DivisionByZero,

    // Timelock
    #[msg("Timelock period has not elapsed")]
    TimelockNotElapsed,
}
