use anchor_lang::{error::Error, error_code};

#[error_code]
pub enum StakeErrorCode {
    #[msg("Minimum staking period has not elapsed")]
    UnstakeFreezePeriodNotElapsed,
}
