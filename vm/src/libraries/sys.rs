use std::collections::HashMap;

use crate::types::{VMError, Value, Context};

pub fn load_libs(ctx: &mut Context) {
    let mut lib = HashMap::new();

    lib.insert("exit".into(), Value::NativeFunction(|_, _, args| {
        let [code] = Value::extract_args(args)?;
        Err(VMError::Exit(code.as_int()?))
    }));

    lib.insert("locked_copy".into(), Value::NativeFunction(|_, _, args| {
        let [obj] = Value::extract_args(args)?;
        let locked = match &obj {
            Value::Object(o) => Value::new_locked_obj(o.get().clone()),
            Value::Array(a) => Value::new_locked_arr(a.get().clone()),
            v => return Err(VMError::invalid_type("object/array", v))
        };
        Ok(locked)
    }));

    lib.insert("as_lib".into(), Value::NativeFunction(|ctx, _, args| {
        let [value, name] = Value::extract_args(args)?;
        let name = name.as_str()?;
        if let Some(_) = ctx.get_lib(name) {
            return Err(VMError::IllegalState);
        }
        ctx.add_lib(name.clone(), value);
        Ok(Value::Null)
    }));

    ctx.add_lib("sys".into(), Value::new_locked_obj(lib));
}
