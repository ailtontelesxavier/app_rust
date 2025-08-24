use serde::{Deserialize, Deserializer};

/*
utilizado nos shemas para converter checkbox em booleano
*/
pub fn bool_from_str<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    Ok(matches!(
        opt.as_deref(),
        Some("true") | Some("on") | Some("1") | Some("yes")
    ))
}

// versão para Option<bool>
pub fn option_bool_from_str<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    Ok(opt.map(|s| matches!(s.as_str(), "true" | "on" | "1" | "yes")))
}
