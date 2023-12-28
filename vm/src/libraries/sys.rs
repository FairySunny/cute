use std::collections::HashMap;

use crate::types::{VMError, Value, Context};

pub fn load_libs(ctx: &mut Context) {
    let mut lib = HashMap::new();

    lib.insert("exit".into(), Value::NativeFunction(|_, _, args| {
        Value::check_arg_cnt(&args, 1)?;
        let code = args[0].as_int()?;
        Err(VMError::Exit(code))
    }));

    lib.insert("locked_copy".into(), Value::NativeFunction(|_, _, args| {
        Value::check_arg_cnt(&args, 1)?;
        let locked = match &args[0] {
            Value::Object(o) => Value::new_locked_obj(o.get().clone()),
            Value::Array(a) => Value::new_locked_arr(a.get().clone()),
            v => return Err(VMError::invalid_type("object/array", v.type_to_str()))
        };
        Ok(locked)
    }));

    lib.insert("as_lib".into(), Value::NativeFunction(|ctx, _, args| {
        Value::check_arg_cnt(&args, 2)?;
        let name = args[1].as_str()?;
        if let Some(_) = ctx.get_lib(name) {
            return Err(VMError::IllegalState);
        }
        ctx.add_lib(name.clone(), args.into_iter().next().unwrap());
        Ok(Value::Null)
    }));

    ctx.add_lib("sys".into(), Value::new_locked_obj(lib));
}
