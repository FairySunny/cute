use std::{collections::HashMap, str::FromStr};
use crate::types::{VMError, Value, Context};

pub fn load_libs(ctx: &mut Context) {
    let mut lib = HashMap::new();

    lib.insert("type_name".into(), Value::NativeFunction(|_, _, args| {
        let [value] = Value::extract_args(args)?;
        Ok(Value::String(value.type_to_str().into()))
    }));

    lib.insert("require".into(), Value::NativeFunction(|_, _, args| {
        let ([value], types) = Value::extract_args_and_array(args)?;
        let type_str = value.type_to_str();
        let mut required = Vec::with_capacity(types.len());
        for t in &types {
            let arg_str = t.as_str()?.as_ref();
            if type_str == arg_str {
                return Ok(Value::Null);
            }
            required.push(arg_str);
        }
        Err(VMError::InvalidType {
            expected: required.join("/"),
            got: type_str.to_owned()
        })
    }));

    lib.insert("to_string".into(), Value::NativeFunction(|_, _, args| {
        let [value] = Value::extract_args(args)?;
        let str = match &value {
            Value::String(_) => value,
            _ => Value::String(value.to_string().into())
        };
        Ok(str)
    }));

    lib.insert("int_to_float".into(), Value::NativeFunction(|_, _, args| {
        let [int] = Value::extract_args(args)?;
        Ok(Value::Float(int.as_int()? as f64))
    }));

    lib.insert("float_to_int".into(), Value::NativeFunction(|_, _, args| {
        let [float] = Value::extract_args(args)?;
        Ok(Value::Int(float.as_float()? as i64))
    }));

    lib.insert("string_to_int".into(), Value::NativeFunction(|_, _, args| {
        let [str] = Value::extract_args(args)?;
        let int = Value::Int(i64::from_str(str.as_str()?)
            .map_err(|_| VMError::IllegalFunctionArguments)?);
        Ok(int)
    }));

    lib.insert("string_to_float".into(), Value::NativeFunction(|_, _, args| {
        let [str] = Value::extract_args(args)?;
        let float = Value::Float(f64::from_str(str.as_str()?)
            .map_err(|_| VMError::IllegalFunctionArguments)?);
        Ok(float)
    }));

    ctx.add_lib("types".into(), Value::new_locked_obj(lib));
}
