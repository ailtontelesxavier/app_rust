use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Default)]
pub struct FilterOptions {
    pub page: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Deserialize, Debug)]
pub struct ParamOptions {
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModuleCreateShema {
    pub title: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModuleUpdateShema {
    pub title: Option<String>,
}
