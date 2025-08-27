
/* 
Filtra o status do chamado com base no seu valor numérico.
let mut env = minijinja::Environment::new();
env.add_filter("status_label", status_filter);
 */
pub fn status_filter(value: i32) -> String {
    match value {
        0 => "Aberto".to_string(),
        1 => "Em atendimento".to_string(),
        2 => "Pausado".to_string(),
        3 => "Resolvido".to_string(),
        4 => "Fechado".to_string(),
        _ => "Desconhecido".to_string(),
    }
}