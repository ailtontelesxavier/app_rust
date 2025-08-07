/// Junta array com separador
pub fn join(values: Vec<String>, sep: &str) -> String {
    values.join(sep)
}

/// Filtra valores únicos
pub fn unique(values: Vec<String>) -> Vec<String> {
    let mut unique = values.clone();
    unique.sort();
    unique.dedup();
    unique
}