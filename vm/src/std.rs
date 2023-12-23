pub mod misc {
    use std::collections::HashMap;
    use crate::vm::Value;

    pub fn load_libs(libs: &mut HashMap<String, Value>) {
        // the 'global' object
        libs.insert("G".to_owned(), Value::new_obj());

        // the null constant
        libs.insert("null".to_owned(), Value::Null);

        // boolean constants
        libs.insert("true".to_owned(), Value::Bool(true));
        libs.insert("false".to_owned(), Value::Bool(false));

        // float constants
        libs.insert("nan".to_owned(), Value::Float(f64::NAN));
        libs.insert("inf".to_owned(), Value::Float(f64::INFINITY));
    }
}
