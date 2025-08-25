use chrono::{DateTime, Utc};
use minijinja::{Error, ErrorKind, value::Value};
//use tracing::debug;

// Converte para string formatada: 07/08/2025 00:37
fn format_datetime_utc(dt: DateTime<Utc>) -> String {
    dt.format("%d/%m/%Y %H:%M").to_string()
}

// Filtro customizado para MiniJinja
pub fn format_datetime_filter(value: Value) -> Result<Value, Error> {
    // Se for timestamp inteiro
    if let Some(timestamp) = value.as_i64() {
        return DateTime::<Utc>::from_timestamp(timestamp, 0)
            .map(|dt| Value::from(format_datetime_utc(dt)))
            .ok_or_else(|| Error::new(ErrorKind::InvalidOperation, "Timestamp inválido"));
    }

    // Se for string no formato ISO 8601
    if let Some(s) = value.as_str() {
        //debug!("valor recebido como string: {}", s);

        return DateTime::parse_from_rfc3339(s)
            .map(|dt| dt.with_timezone(&Utc)) // converte para Utc
            .map(|dt| Value::from(format_datetime_utc(dt)))
            .map_err(|e| {
                Error::new(
                    ErrorKind::InvalidOperation,
                    format!("Formato de data inválido: {}", e),
                )
            });
    }

    Err(Error::new(
        ErrorKind::InvalidOperation,
        "Valor deve ser timestamp (número) ou string de data",
    ))
}
