use base64::{Engine as _, engine::general_purpose};
use image::Luma;
use qrcode::QrCode;
use std::io::Cursor;

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

pub fn get_qr_code_base64(otp_url: &str) -> anyhow::Result<String> {
    // Gera o QR code a partir da URL
    let code = QrCode::new(otp_url.as_bytes())?;

    // Converte para uma imagem em tons de cinza
    let image = code.render::<Luma<u8>>().build();

    // Salva a imagem em memória (PNG)
    let mut buffer = Cursor::new(Vec::new());
    image::DynamicImage::ImageLuma8(image).write_to(&mut buffer, image::ImageFormat::Png)?;

    // Codifica em Base64
    let img_base64 = general_purpose::STANDARD.encode(&buffer.into_inner());

    // Retorna no formato data URI
    Ok(format!("data:image/png;base64,{}", img_base64))
}
