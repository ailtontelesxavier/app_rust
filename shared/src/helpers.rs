#[derive(Clone)]
pub enum FlashStatus {
    Error,
    Info,
    Success,
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
