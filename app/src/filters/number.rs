use minijinja::{Value, Error, ErrorKind};

/// Formata número como moeda
pub fn currency(value: Value) -> Result<Value, Error> {
    // Tenta extrair o valor numérico
    let num = if value.is_number() {
        if let Some(i) = value.as_i64() {
            i as f64  // Converte i64 para f64
        } else {
            return Err(Error::new(
                ErrorKind::InvalidOperation,
                "valor não é um número válido",
            ));
        }
    } else {
        return Err(Error::new(
            ErrorKind::InvalidOperation,
            "valor não é um número",
        ));
    };
    
    Ok(Value::from(format!("R$ {:.2}", num)))
}

/// Formata número com separadores de milhar
pub fn format_number(value: Value) -> Result<Value, Error> {
    // Tenta extrair como inteiro
    let num = if let Some(i) = value.as_i64() {
        i
    } else {
        return Err(Error::new(
            ErrorKind::InvalidOperation,
            "valor não é um número inteiro",
        ));
    };

    let s = num.to_string();
    let bytes: Vec<_> = s.bytes().rev().collect();
    let chunks: Vec<_> = bytes
        .chunks(3)
        .map(|chunk| std::str::from_utf8(chunk).unwrap())
        .collect();
    let formatted = chunks.join(".").chars().rev().collect::<String>();
    
    Ok(Value::from(formatted))
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