use crate::externo::StatusTramitacaoEnum;

pub fn contato_color_filter(value: i32) -> String {
    let status = StatusTramitacaoEnum::from_i32(value);
    status.color().to_string()
}

// filtro que converte o valor i32 em descrição do status
pub fn contato_status_filter(value: i32) -> String {
    let status = StatusTramitacaoEnum::from_i32(value);
    status.to_string()
}