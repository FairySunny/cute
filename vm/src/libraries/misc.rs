use std::{collections::HashMap, rc::Rc};
use crate::types::Value;

pub fn load_libs(libs: &mut HashMap<Rc<str>, Value>) {
    // the 'global' object
    libs.insert("G".into(), Value::new_obj());

    // the null constant
    libs.insert("null".into(), Value::Null);

    // boolean constants
    libs.insert("true".into(), Value::Bool(true));
    libs.insert("false".into(), Value::Bool(false));

    // float constants
    libs.insert("nan".into(), Value::Float(f64::NAN));
    libs.insert("inf".into(), Value::Float(f64::INFINITY));
}
