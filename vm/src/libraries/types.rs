use std::{collections::HashMap, str::FromStr};
use crate::types::{Value, VMError};

pub fn load_libs(libs: &mut HashMap<String, Value>) {
    libs.insert("types".to_owned(), Value::new_lib_obj(|obj| {
        obj.insert("type_name".to_owned(), Value::NativeFunction(|_, _, args| {
            if args.len() != 1 {
                return Err(VMError::IllegalFunctionArguments);
            }
            Ok(Value::new_str(args[0].type_to_str()))
        }));
        obj.insert("to_string".to_owned(), Value::NativeFunction(|_, _, args| {
            if args.len() != 1 {
                return Err(VMError::IllegalFunctionArguments);
            }
            Ok(Value::new_str(args[0].to_string()))
        }));
        obj.insert("int_to_float".to_owned(), Value::NativeFunction(|_, _, args| {
            if args.len() != 1 {
                return Err(VMError::IllegalFunctionArguments);
            }
            let i = args[0].as_int()?;
            Ok(Value::Float(i as f64))
        }));
        obj.insert("float_to_int".to_owned(), Value::NativeFunction(|_, _, args| {
            if args.len() != 1 {
                return Err(VMError::IllegalFunctionArguments);
            }
            let f = args[0].as_float()?;
            Ok(Value::Int(f as i64))
        }));
        obj.insert("string_to_int".to_owned(), Value::NativeFunction(|_, _, args| {
            if args.len() != 1 {
                return Err(VMError::IllegalFunctionArguments);
            }
            let s = args[0].as_str()?;
            Ok(Value::Int(i64::from_str(s)
                .map_err(|_| VMError::IllegalFunctionArguments)?))
        }));
        obj.insert("string_to_float".to_owned(), Value::NativeFunction(|_, _, args| {
            if args.len() != 1 {
                return Err(VMError::IllegalFunctionArguments);
            }
            let s = args[0].as_str()?;
            Ok(Value::Float(f64::from_str(s)
                .map_err(|_| VMError::IllegalFunctionArguments)?))
        }));
    }));
}
