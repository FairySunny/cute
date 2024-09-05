use std::{collections::HashMap, str::FromStr};
use crate::types::{VMError, Value, Context};

pub fn load_libs(ctx: &mut Context) {
    let mut lib = HashMap::new();

    lib.insert("type_name".into(), Value::NativeFunction(|_, _, args| {
        let [value] = Value::extract_args(args)?;
        Ok(Value::String(value.type_to_str().into()))
    }));

    lib.insert("expect_type".into(), Value::NativeFunction(|_, _, args| {
        let ([value], types) = Value::extract_args_and_array(args)?;
        let type_str = value.type_to_str();
        let mut expected = Vec::with_capacity(types.len());
        for t in &types {
            let arg_str = t.as_str()?.to_string();
            if type_str == arg_str {
                return Ok(Value::Null);
            }
            expected.push(arg_str);
        }
        Err(VMError::InvalidType {
            expected: expected.join("/"),
            got: type_str.to_owned()
        })
    }));

    lib.insert("to_string".into(), Value::NativeFunction(|_, _, args| {
        let [value] = Value::extract_args(args)?;
        let str = match &value {
            Value::String(_) => value,
            _ => Value::String(value.to_string()[..].into())
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
        let int = Value::Int(i64::from_str(&str.as_str()?.to_string())
            .map_err(|_| VMError::IllegalFunctionArguments)?);
        Ok(int)
    }));

    lib.insert("string_to_float".into(), Value::NativeFunction(|_, _, args| {
        let [str] = Value::extract_args(args)?;
        let float = Value::Float(f64::from_str(&str.as_str()?.to_string())
            .map_err(|_| VMError::IllegalFunctionArguments)?);
        Ok(float)
    }));

    lib.insert("char_code_to_string".into(), Value::NativeFunction(|_, _, args| {
        let [arr] = Value::extract_args(args)?;
        let arr = arr.as_arr()?.get();
        let mut chars = Vec::with_capacity(arr.len());
        for item in arr.iter() {
            chars.push(item.as_int()? as u16);
        }
        Ok(Value::String(chars[..].into()))
    }));

    lib.insert("string_to_char_code".into(), Value::NativeFunction(|_, _, args| {
        let [str] = Value::extract_args(args)?;
        let arr = str.as_str()?.data().iter()
            .map(|&c| Value::Int(c.into()))
            .collect();
        Ok(Value::new_arr(arr))
    }));

    ctx.add_lib("types".into(), Value::new_locked_obj(lib));
}
