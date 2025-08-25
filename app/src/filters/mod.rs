mod array;
mod datetime;
mod number;
mod string;

pub use array::*;
pub use datetime::{format_datetime_filter};
pub use number::{currency, format_number, currency_float, format_number_int};
pub use string::*;

use minijinja::Environment;

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
    env.add_filter("currency", currency);
    env.add_filter("format_number", format_number);
    env.add_filter("currency_float", currency_float);
    env.add_filter("format_number_int", format_number_int);

    // Array filters
    env.add_filter("join", join);
    env.add_filter("unique", unique);
}
