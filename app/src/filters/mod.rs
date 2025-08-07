mod datetime;
mod string;
mod number;
mod array;

pub use datetime::*;
pub use string::*;
pub use number::*;
pub use array::*;

use minijinja::Environment;

/// Registra todos os filtros no ambiente MiniJinja
pub fn register_filters(env: &mut Environment) {
    // Datetime filters
    env.add_filter("format_datetime", datetime::format_datetime_filter);

    // String filters
    env.add_filter("uppercase", uppercase);
    env.add_filter("lowercase", lowercase);
    env.add_filter("truncate", truncate);
    env.add_filter("capitalize_first", capitalize_first);
    
    // Number filters
    env.add_filter("currency", number::currency);
    env.add_filter("format_number", number::format_number);
    env.add_filter("currency_float", number::currency_float);
    env.add_filter("format_number_int", number::format_number_int);
    
    // Array filters
    env.add_filter("join", join);
    env.add_filter("unique", unique);
}