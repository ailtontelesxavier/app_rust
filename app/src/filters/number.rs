use bigdecimal::ToPrimitive;
use std::str::FromStr;

use bigdecimal::BigDecimal;
use minijinja::{Error, ErrorKind, Value};

/// Serializa BigDecimal para string BRL ("20000,00")
pub fn format_decimal(value: Value) -> Result<Value, Error> {
    if let Some(s) = value.as_str() {
        // caso venha string
        let bd = BigDecimal::from_str(s)
            .map_err(|_| Error::new(ErrorKind::InvalidOperation, "valor inválido para decimal"))?;
        let f = bd.to_f64().ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidOperation,
                "não foi possível converter para f64",
            )
        })?;
        return Ok(Value::from(format!("{:.2}", f)));
    }

    if let Some(f) = value.as_i64() {
        return Ok(Value::from(format!("{:.2}", f)));
    }

    Err(Error::new(
        ErrorKind::InvalidOperation,
        "valor não é um decimal",
    ))
}

/// Formata número como moeda
pub fn currency_brl(value: Value) -> Result<Value, Error> {
    // tenta extrair como BigDecimal
    let bd = if let Some(s) = value.as_str() {
        BigDecimal::from_str(s)
            .map_err(|_| Error::new(ErrorKind::InvalidOperation, "valor não é número válido"))?
    } else if let Some(i) = value.as_i64() {
        BigDecimal::from(i)
    } else if let Some(f) = value.as_i64() {
        BigDecimal::from_str(&format!("{:.2}", f)).map_err(|_| {
            Error::new(
                ErrorKind::InvalidOperation,
                "erro ao converter float para BigDecimal",
            )
        })?
    } else {
        return Err(Error::new(
            ErrorKind::InvalidOperation,
            "valor não é número",
        ));
    };

    // garante 2 casas decimais (igual toFixed(2) no JS)
    let scaled = bd.with_scale(2);

    // separa parte inteira e fracionária
    let inteiro = scaled.with_scale(0).to_i64().unwrap_or(0);
    let frac = (scaled.clone() - BigDecimal::from(inteiro))
        .with_scale(2)
        .to_string()
        .trim_start_matches('-')
        .trim_start_matches('0')
        .replace(".", "");

    // formata parte inteira com separador de milhar
    let mut s = inteiro.abs().to_string();
    let mut result = String::new();
    while s.len() > 3 {
        let (rest, chunk) = s.split_at(s.len() - 3);
        result = format!(".{}{}", chunk, result);
        s = rest.to_string();
    }
    result = format!("{}{}", s, result);

    // monta número BRL
    let formatted = format!(
        "{}{},{:02}",
        if inteiro < 0 { "-" } else { "" },
        result,
        frac
    );

    Ok(Value::from(format!("R$ {}", formatted)))
}

/// Formata número com separadores de milhar
pub fn format_number(value: Value) -> Result<Value, Error> {
    // Primeiro tenta como i64
    if let Some(i) = value.as_i64() {
        return Ok(Value::from(format_number_int(i)));
    }

    // Depois tenta como string -> BigDecimal
    if let Some(s) = value.as_str() {
        if let Ok(bd) = BigDecimal::from_str(s) {
            if let Some(i) = bd.to_i64() {
                return Ok(Value::from(format_number_int(i)));
            }
        }
    }

    Err(Error::new(
        ErrorKind::InvalidOperation,
        "valor não é um número inteiro",
    ))
}

// Versões auxiliares para uso interno (opcional)
pub fn currency_float(value: f64) -> String {
    format!("R$ {:.2}", value)
}

pub fn format_number_int(value: i64) -> String {
    let s = value.to_string();
    let bytes: Vec<_> = s.bytes().rev().collect();
    let chunks: Vec<_> = bytes
        .chunks(3)
        .map(|chunk| std::str::from_utf8(chunk).unwrap())
        .collect();
    chunks.join(".").chars().rev().collect()
}
