use std::collections::HashMap;
use gc::Gc;
use crate::{types::{VMError, Variables, Value, Context}, executor};

pub fn load_libs(ctx: &mut Context) {
    let mut lib = HashMap::new();

    lib.insert("push".into(), Value::NativeFunction(|_, mut args| {
        Value::check_arg_range(&args, 2..)?;
        let mut elements = args.drain(1..).collect();
        args[0].as_arr()?.borrow_mut().get_mut()?.append(&mut elements);
        Ok(Value::Null)
    }));

    lib.insert("pop".into(), Value::NativeFunction(|_, args| {
        Value::check_arg_cnt(&args, 1)?;
        args[0].as_arr()?.borrow_mut().get_mut()?.pop()
            .ok_or_else(|| VMError::ArrayIndexOutOfBound)
    }));

    lib.insert("splice".into(), Value::NativeFunction(|_, mut args| {
        Value::check_arg_range(&args, 3..)?;
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

    lib.insert("slice".into(), Value::NativeFunction(|_, args| {
        Value::check_arg_range(&args, 2..4)?;
        let arr = args[0].as_arr()?.borrow();
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

    lib.insert("for_each".into(), Value::NativeFunction(|ctx, args| {
        Value::check_arg_cnt(&args, 2)?;
        let arr = args[0].as_arr()?.borrow().get().clone();
        let closure = args[1].as_closure()?;
        for (idx, elem) in arr.into_iter().enumerate() {
            executor::execute_closure(
                ctx,
                closure.program_idx,
                closure.func_idx,
                Gc::new(Variables::new(Some(&closure.parent))),
                vec![elem, Value::Int(idx as i64)]
            )?;
        }
        Ok(Value::Null)
    }));

    lib.insert("filter".into(), Value::NativeFunction(|ctx, args| {
        Value::check_arg_cnt(&args, 2)?;
        let arr = args[0].as_arr()?.borrow().get().clone();
        let closure = args[1].as_closure()?;
        let mut filtered = Vec::with_capacity(arr.len());
        for (idx, elem) in arr.into_iter().enumerate() {
            let res = executor::execute_closure(
                ctx,
                closure.program_idx,
                closure.func_idx,
                Gc::new(Variables::new(Some(&closure.parent))),
                vec![elem.clone(), Value::Int(idx as i64)]
            )?;
            if res.as_bool()? {
                filtered.push(elem);
            }
        }
        filtered.shrink_to_fit();
        Ok(Value::new_arr(filtered))
    }));

    lib.insert("map".into(), Value::NativeFunction(|ctx, args| {
        Value::check_arg_cnt(&args, 2)?;
        let arr = args[0].as_arr()?.borrow().get().clone();
        let closure = args[1].as_closure()?;
        let mut mapped = Vec::with_capacity(arr.len());
        for (idx, elem) in arr.into_iter().enumerate() {
            let res = executor::execute_closure(
                ctx,
                closure.program_idx,
                closure.func_idx,
                Gc::new(Variables::new(Some(&closure.parent))),
                vec![elem, Value::Int(idx as i64)]
            )?;
            mapped.push(res);
        }
        Ok(Value::new_arr(mapped))
    }));

    ctx.add_lib("arrays".into(), Value::new_locked_obj(lib));
}
