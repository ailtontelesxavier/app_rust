use std::fmt::Display;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct FlashData {
    pub flash: String,
    pub flash_status: String,
}

#[derive(Clone)]
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

/// Cria dados de flash a partir de query parameters
pub fn create_flash_data(message: impl Into<String>, status: FlashStatus) -> FlashData {
    FlashData {
        flash: message.into(),
        flash_status: status.to_string(),
    }
}

/// Cria uma URL com parâmetros de mensagem flash
pub fn create_flash_url(base_url: &str, message: &str, status: FlashStatus) -> String {
    let encoded_message = urlencoding::encode(message);
    let status_str = match status {
        FlashStatus::Success => "success",
        FlashStatus::Error => "error",
        FlashStatus::Info => "info",
    };
    
    format!("{}?msg={}&status={}", base_url, encoded_message, status_str)
}
