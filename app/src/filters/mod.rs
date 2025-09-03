mod array;
mod datetime;
mod number;
mod string;

pub use array::*;
pub use datetime::format_datetime_filter;
pub use number::{currency_float, format_number, format_number_int};
pub use string::*;

use minijinja::Environment;

use crate::{
    chamado::status_filter,
    filters::number::{currency_brl, format_decimal},
};

/// Registra todos os filtros no ambiente MiniJinja
pub fn register_filters(env: &mut Environment) {
    // Datetime filters
    env.add_filter("format_datetime", format_datetime_filter);

    // String filters
    env.add_filter("uppercase", uppercase);
    env.add_filter("lowercase", lowercase);
    env.add_filter("truncate", truncate);
    env.add_filter("capitalize_first", capitalize_first);

    // Number filters
    env.add_filter("format_decimal", format_decimal);
    env.add_filter("currency", currency_brl);
    env.add_filter("format_number", format_number);
    env.add_filter("currency_float", currency_float);
    env.add_filter("format_number_int", format_number_int);

    // Array filters
    env.add_filter("join", join);
    env.add_filter("unique", unique);

    // Chamado filters
    env.add_filter("status_label", status_filter);
}

#[cfg(test)]
mod tests {
    use minijinja::Value;
    #[test]
    fn test_currency() {
        assert_eq!(
            super::currency_brl(Value::from("250000.01"))
                .unwrap()
                .to_string(),
            "\"R$ 250.000,01\""
        );
        assert_eq!(
            super::currency_brl(Value::from("250000.00"))
                .unwrap()
                .to_string(),
            "\"R$ 250.000,00\""
        );
        assert_eq!(
            super::currency_brl(Value::from(1234.5))
                .unwrap()
                .to_string(),
            "\"R$ 1.234,50\""
        );
        assert_eq!(
            super::currency_brl(Value::from(0)).unwrap().to_string(),
            "\"R$ 0,00\""
        );
    }
}
