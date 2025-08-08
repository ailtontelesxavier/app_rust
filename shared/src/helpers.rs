use super::error::AppError;
use tower_sessions::Session;
use std::fmt::Display;
use serde::Serialize;

#[derive(Serialize)]
pub struct FlashData {
    pub flash: String,
    pub flash_status: String,
}

pub enum FlashStatus {
    Error,
    Info,
    Success,
}

impl Display for FlashStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let flash_string = match self {
            FlashStatus::Error => "error-flash",
            FlashStatus::Info => "info-flash",
            FlashStatus::Success => "success-flash",
        };
        write!(f, "{}", flash_string)
    }
}

/// Recupera e remove a mensagem flash da sessão.
/// Retorna `Ok(None)` se não houver mensagem.
pub async fn get_flash(session: &Session) -> Result<Option<FlashData>, AppError> {
    let flash = session.remove::<String>("flash").await?;
    let flash_status = session.remove::<String>("flash_status").await?;

    match (flash, flash_status) {
        (Some(flash), Some(flash_status)) => Ok(Some(FlashData { flash, flash_status })),
        _ => Ok(None),
    }
}

/// Define uma nova mensagem flash na sessão.
pub async fn set_flash(
    session: &Session,
    message: impl Into<String>,
    status: FlashStatus,
) -> Result<(), AppError> {
    session.insert("flash", message.into()).await?;
    session.insert("flash_status", status.to_string()).await?;
    Ok(())
}
