use std::str::FromStr;

use bigdecimal::BigDecimal;
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

/*
utilizado no formulario html com formato que trata.
20.000,00 → 20000.00.
*/
pub fn brl_to_bigdecimal<'de, D>(deserializer: D) -> Result<BigDecimal, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    // remove pontos de milhar e troca vírgula por ponto
    let normalized = s.replace(".", "").replace(",", ".");
    BigDecimal::from_str(&normalized).map_err(serde::de::Error::custom)
}
