use std::{collections::HashMap, str::FromStr};
use crate::types::{VMError, Value, Context};

pub fn load_libs(ctx: &mut Context) {
    let mut lib = HashMap::new();

    lib.insert("type_name".into(), Value::NativeFunction(|_, _, args| {
        Value::check_arg_cnt(&args, 1)?;
        Ok(Value::String(args[0].type_to_str().into()))
    }));

    lib.insert("require".into(), Value::NativeFunction(|_, _, args| {
        Value::check_arg_range(&args, 2..)?;
        let type_str = args[0].type_to_str();
        let mut required = Vec::with_capacity(args.len() - 1);
        for arg in &args[1..] {
            let arg_str = arg.as_str()?.as_ref();
            if type_str == arg_str {
                return Ok(Value::Null);
            }
            required.push(arg_str);
        }
        Err(VMError::invalid_type(&required.join("/"), type_str))
    }));

    lib.insert("to_string".into(), Value::NativeFunction(|_, _, args| {
        Value::check_arg_cnt(&args, 1)?;
        Ok(match &args[0] {
            Value::String(_) => args.into_iter().next().unwrap(),
            v => Value::String(v.to_string().into())
        })
    }));

    lib.insert("int_to_float".into(), Value::NativeFunction(|_, _, args| {
        Value::check_arg_cnt(&args, 1)?;
        let i = args[0].as_int()?;
        Ok(Value::Float(i as f64))
    }));

    lib.insert("float_to_int".into(), Value::NativeFunction(|_, _, args| {
        Value::check_arg_cnt(&args, 1)?;
        let f = args[0].as_float()?;
        Ok(Value::Int(f as i64))
    }));

    lib.insert("string_to_int".into(), Value::NativeFunction(|_, _, args| {
        Value::check_arg_cnt(&args, 1)?;
        let s = args[0].as_str()?;
        Ok(Value::Int(i64::from_str(s)
            .map_err(|_| VMError::IllegalFunctionArguments)?))
    }));

    lib.insert("string_to_float".into(), Value::NativeFunction(|_, _, args| {
        Value::check_arg_cnt(&args, 1)?;
        let s = args[0].as_str()?;
        Ok(Value::Float(f64::from_str(s)
            .map_err(|_| VMError::IllegalFunctionArguments)?))
    }));

    ctx.add_lib("types".into(), Value::new_locked_obj(lib));
}
