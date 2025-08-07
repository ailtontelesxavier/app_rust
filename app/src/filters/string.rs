use minijinja::{Value, Error, ErrorKind};

pub fn uppercase(value: Value) -> Result<Value, Error> {
    let s = value.as_str()
        .ok_or_else(|| Error::new(ErrorKind::InvalidOperation, "valor não é uma string"))?;
    Ok(Value::from(s.to_uppercase()))
}

pub fn lowercase(value: Value) -> Result<Value, Error> {
    let s = value.as_str()
        .ok_or_else(|| Error::new(ErrorKind::InvalidOperation, "valor não é uma string"))?;
    Ok(Value::from(s.to_lowercase()))
}

pub fn truncate(value: Value, len: usize) -> Result<Value, Error> {
    let s = value.as_str()
        .ok_or_else(|| Error::new(ErrorKind::InvalidOperation, "valor não é uma string"))?;
    Ok(Value::from(if s.len() > len {
        format!("{}...", &s[..len])
    } else {
        s.to_string()
    }))
}

pub fn capitalize_first(value: Value) -> Result<Value, Error> {
    let s = value.as_str()
        .ok_or_else(|| Error::new(ErrorKind::InvalidOperation, "valor não é uma string"))?;
    let mut chars = s.chars();
    match chars.next() {
        None => Ok(Value::from("")),
        Some(f) => {
            let capitalized = f.to_uppercase().chain(chars).collect::<String>();
            Ok(Value::from(capitalized))
        }
    }
}