use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("sqlx error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("Argon2 password hash error: {0}")]
    Argon2Error(#[from] argon2::password_hash::Error),
}
