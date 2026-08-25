use anchor_lang::prelude::*;

#[error_code]
pub enum VestingError {
    #[msg("start_ts must be <= cliff_ts and cliff_ts must be < end_ts")]
    InvalidSchedule,
    #[msg("total_amount must be greater than zero")]
    InvalidAmount,
    #[msg("Creator token account does not hold enough to fund this grant")]
    InsufficientFunds,
    #[msg("Nothing is claimable at this time")]
    NothingToClaim,
    #[msg("This grant is not revocable")]
    NotRevocable,
    #[msg("This grant has already been revoked")]
    AlreadyRevoked,
    #[msg("Only the grant creator can perform this action")]
    UnauthorizedCreator,
    #[msg("Only the beneficiary can perform this action")]
    UnauthorizedBeneficiary,
    #[msg("Vault still holds tokens; claim or revoke before close")]
    VaultNotEmpty,
    #[msg("Grant is not fully settled; cannot close")]
    GrantNotSettled,
    #[msg("Arithmetic overflow")]
    MathOverflow,
    #[msg(
        "Transfer-fee Token-2022 mints are not supported; vault accounting assumes 1:1 transfers"
    )]
    TransferFeeNotSupported,
}
