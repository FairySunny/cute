use std::collections::HashMap;
use crate::types::{Value, Context};

pub fn load_libs(ctx: &mut Context) {
    // the 'global' object
    ctx.add_lib("G".into(), Value::new_obj(HashMap::new()));

    // the null constant
    ctx.add_lib("null".into(), Value::Null);

    // boolean constants
    ctx.add_lib("true".into(), Value::Bool(true));
    ctx.add_lib("false".into(), Value::Bool(false));

    // float constants
    ctx.add_lib("nan".into(), Value::Float(f64::NAN));
    ctx.add_lib("inf".into(), Value::Float(f64::INFINITY));
}
