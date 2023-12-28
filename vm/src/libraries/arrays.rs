use std::collections::HashMap;
use crate::{types::{VMError, Value, Context}, executor};

pub fn load_libs(ctx: &mut Context) {
    let mut lib = HashMap::new();

    lib.insert("push".into(), Value::NativeFunction(|_, _, mut args| {
        Value::check_arg_range(&args, 2..)?;
        let mut elements = args.drain(1..).collect();
        args[0].as_arr()?.get_mut()?.append(&mut elements);
        Ok(Value::Null)
    }));

    lib.insert("pop".into(), Value::NativeFunction(|_, _, args| {
        Value::check_arg_cnt(&args, 1)?;
        args[0].as_arr()?.get_mut()?.pop()
            .ok_or_else(|| VMError::ArrayIndexOutOfBound)
    }));

    lib.insert("splice".into(), Value::NativeFunction(|_, _, mut args| {
        Value::check_arg_range(&args, 3..)?;
        let start = args[1].as_idx()?;
        let del_cnt = args[2].as_idx()?;
        let end = start.checked_add(del_cnt)
            .ok_or_else(|| VMError::ArrayIndexOutOfBound)?;
        let elements: Vec<_> = args.drain(3..).collect();
        let mut arr = args[0].as_arr()?.get_mut()?;
        if end > arr.len() {
            return Err(VMError::ArrayIndexOutOfBound);
        }
        Ok(Value::new_arr(arr.splice(start .. end, elements).collect()))
    }));

    lib.insert("slice".into(), Value::NativeFunction(|_, _, args| {
        Value::check_arg_range(&args, 2..4)?;
        let arr = args[0].as_arr()?.get();
        let start = args[1].as_idx()?;
        let end = match args.get(2) {
            Some(v) => v.as_idx()?,
            None => arr.len()
        };
        if start > end || end > arr.len() {
            return Err(VMError::ArrayIndexOutOfBound);
        }
        Ok(Value::new_arr(arr[start .. end].to_owned()))
    }));

    lib.insert("find_first_index".into(), Value::NativeFunction(|ctx, _, args| {
        Value::check_arg_cnt(&args, 2)?;
        let arr = args[0].as_arr()?.get().clone();
        let closure = args[1].as_closure()?;
        for (idx, elem) in arr.into_iter().enumerate() {
            let res = executor::call(
                ctx,
                closure,
                vec![elem, Value::Int(idx as i64)]
            )?;
            if res.as_bool()? {
                return Ok(Value::Int(idx as i64));
            }
        }
        return Ok(Value::Int(-1));
    }));

    lib.insert("find_last_index".into(), Value::NativeFunction(|ctx, _, args| {
        Value::check_arg_cnt(&args, 2)?;
        let arr = args[0].as_arr()?.get().clone();
        let closure = args[1].as_closure()?;
        for (idx, elem) in arr.into_iter().enumerate().rev() {
            let res = executor::call(
                ctx,
                closure,
                vec![elem, Value::Int(idx as i64)]
            )?;
            if res.as_bool()? {
                return Ok(Value::Int(idx as i64));
            }
        }
        return Ok(Value::Int(-1));
    }));

    lib.insert("for_each".into(), Value::NativeFunction(|ctx, _, args| {
        Value::check_arg_cnt(&args, 2)?;
        let arr = args[0].as_arr()?.get().clone();
        let closure = args[1].as_closure()?;
        for (idx, elem) in arr.into_iter().enumerate() {
            executor::call(
                ctx,
                closure,
                vec![elem, Value::Int(idx as i64)]
            )?;
        }
        Ok(Value::Null)
    }));

    lib.insert("filter".into(), Value::NativeFunction(|ctx, _, args| {
        Value::check_arg_cnt(&args, 2)?;
        let arr = args[0].as_arr()?.get().clone();
        let closure = args[1].as_closure()?;
        let mut filtered = Vec::with_capacity(arr.len());
        for (idx, elem) in arr.into_iter().enumerate() {
            let res = executor::call(
                ctx,
                closure,
                vec![elem.clone(), Value::Int(idx as i64)]
            )?;
            if res.as_bool()? {
                filtered.push(elem);
            }
        }
        filtered.shrink_to_fit();
        Ok(Value::new_arr(filtered))
    }));

    lib.insert("map".into(), Value::NativeFunction(|ctx, _, args| {
        Value::check_arg_cnt(&args, 2)?;
        let arr = args[0].as_arr()?.get().clone();
        let closure = args[1].as_closure()?;
        let mut mapped = Vec::with_capacity(arr.len());
        for (idx, elem) in arr.into_iter().enumerate() {
            let res = executor::call(
                ctx,
                closure,
                vec![elem, Value::Int(idx as i64)]
            )?;
            mapped.push(res);
        }
        Ok(Value::new_arr(mapped))
    }));

    ctx.add_lib("arrays".into(), Value::new_locked_obj(lib));
}
