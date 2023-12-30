use std::collections::HashMap;
use crate::types::{VMError, Value, Context};

pub fn load_libs(ctx: &mut Context) {
    let mut lib = HashMap::new();

    lib.insert("chars".into(), Value::NativeFunction(|_, _, args| {
        let [str] = Value::extract_args(args)?;
        let arr = str.as_str()?.chars()
            .map(|c| Value::String(c.to_string().into()))
            .collect();
        Ok(Value::new_arr(arr))
    }));

    lib.insert("code_point".into(), Value::NativeFunction(|_, _, args| {
        let [str] = Value::extract_args(args)?;
        let mut chars = str.as_str()?.chars();
        let char = chars.next()
            .ok_or(VMError::IllegalFunctionArguments)?;
        if !chars.next().is_none() {
            return Err(VMError::IllegalFunctionArguments);
        }
        Ok(Value::Int(char as i64))
    }));

    lib.insert("from_code_point".into(), Value::NativeFunction(|_, _, args| {
        let mut chars: Vec<char> = Vec::with_capacity(args.len());
        for arg in &args {
            let char = char::from_u32(arg.as_int()? as u32)
                .ok_or(VMError::IllegalFunctionArguments)?;
            chars.push(char);
        }
        Ok(Value::String(String::from_iter(chars).into()))
    }));

    ctx.add_lib("strings".into(), Value::new_locked_obj(lib));
}
