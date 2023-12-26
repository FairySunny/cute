use std::str::FromStr;
use crate::types::{VMError, Value, Context};

pub fn load_libs(ctx: &mut Context) {
    ctx.add_lib("types".into(), Value::new_lib_obj(|obj| {
        obj.insert("type_name".into(), Value::NativeFunction(|_, args| {
            if args.len() != 1 {
                return Err(VMError::IllegalFunctionArguments);
            }
            Ok(Value::String(args[0].type_to_str().into()))
        }));
        obj.insert("to_string".into(), Value::NativeFunction(|_, args| {
            if args.len() != 1 {
                return Err(VMError::IllegalFunctionArguments);
            }
            Ok(match &args[0] {
                Value::String(_) => args.into_iter().next().unwrap(),
                v => Value::String(v.to_string().into())
            })
        }));
        obj.insert("int_to_float".into(), Value::NativeFunction(|_, args| {
            if args.len() != 1 {
                return Err(VMError::IllegalFunctionArguments);
            }
            let i = args[0].as_int()?;
            Ok(Value::Float(i as f64))
        }));
        obj.insert("float_to_int".into(), Value::NativeFunction(|_, args| {
            if args.len() != 1 {
                return Err(VMError::IllegalFunctionArguments);
            }
            let f = args[0].as_float()?;
            Ok(Value::Int(f as i64))
        }));
        obj.insert("string_to_int".into(), Value::NativeFunction(|_, args| {
            if args.len() != 1 {
                return Err(VMError::IllegalFunctionArguments);
            }
            let s = args[0].as_str()?;
            Ok(Value::Int(i64::from_str(s)
                .map_err(|_| VMError::IllegalFunctionArguments)?))
        }));
        obj.insert("string_to_float".into(), Value::NativeFunction(|_, args| {
            if args.len() != 1 {
                return Err(VMError::IllegalFunctionArguments);
            }
            let s = args[0].as_str()?;
            Ok(Value::Float(f64::from_str(s)
                .map_err(|_| VMError::IllegalFunctionArguments)?))
        }));
    }));
}
