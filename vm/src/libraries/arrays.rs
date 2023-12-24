use std::collections::HashMap;
use crate::types::{Value, VMError};

pub fn load_libs(libs: &mut HashMap<String, Value>) {
    libs.insert("arrays".to_owned(), Value::new_lib_obj(|obj| {
        obj.insert("push".to_owned(), Value::NativeFunction(|mut args| {
            if args.len() < 2 {
                return Err(VMError::IllegalFunctionArguments);
            }
            let mut elements = args.drain(1..).collect();
            args[0].as_arr()?.borrow_mut().get_mut()?.append(&mut elements);
            Ok(Value::Null)
        }));
        obj.insert("pop".to_owned(), Value::NativeFunction(|args| {
            if args.len() != 1 {
                return Err(VMError::IllegalFunctionArguments);
            }
            args[0].as_arr()?.borrow_mut().get_mut()?.pop()
                .ok_or_else(|| VMError::ArrayIndexOutOfBound)
        }));
        obj.insert("splice".to_owned(), Value::NativeFunction(|mut args| {
            if args.len() < 3 {
                return Err(VMError::IllegalFunctionArguments);
            }
            let start = args[1].as_idx()?;
            let del_cnt = args[2].as_idx()?;
            let end = start.checked_add(del_cnt)
                .ok_or_else(|| VMError::ArrayIndexOutOfBound)?;
            let elements: Vec<_> = args.drain(3..).collect();
            let mut arr = args[0].as_arr()?.borrow_mut();
            if end > arr.get().len() {
                return Err(VMError::ArrayIndexOutOfBound);
            }
            Ok(Value::new_arr(arr.get_mut()?.splice(start .. end, elements).collect()))
        }));
        obj.insert("slice".to_owned(), Value::NativeFunction(|args| {
            if args.len() < 2 || args.len() > 3 {
                return Err(VMError::IllegalFunctionArguments);
            }
            let arr = args[0].as_arr()?.borrow_mut();
            let start = args[1].as_idx()?;
            let end = match args.get(2) {
                Some(v) => v.as_idx()?,
                None => arr.get().len()
            };
            if start > end || end > arr.get().len() {
                return Err(VMError::ArrayIndexOutOfBound);
            }
            Ok(Value::new_arr(arr.get()[start .. end].to_owned()))
        }));
    }));
}
