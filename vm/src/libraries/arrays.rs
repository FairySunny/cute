use std::collections::HashMap;
use crate::{types::{VMError, Value, Context}, executor};

pub fn load_libs(ctx: &mut Context) {
    let mut lib = HashMap::new();

    lib.insert("splice".into(), Value::NativeFunction(|_, _, args| {
        let ([arr, start, del_cnt], elements) = Value::extract_args_and_array(args)?;
        let start = start.as_idx()?;
        let del_cnt = del_cnt.as_idx()?;
        let end = start.checked_add(del_cnt)
            .ok_or(VMError::ArrayIndexOutOfBound)?;
        let mut arr = arr.as_arr()?.get_mut()?;
        if end > arr.len() {
            return Err(VMError::ArrayIndexOutOfBound);
        }
        Ok(Value::new_arr(arr.splice(start .. end, elements).collect()))
    }));

    lib.insert("find_first_index".into(), Value::NativeFunction(|ctx, _, args| {
        let [arr, pred] = Value::extract_args(args)?;
        let arr = arr.as_arr()?.get().clone();
        let pred = pred.as_closure()?;
        for (idx, elem) in arr.into_iter().enumerate() {
            let res = executor::call(
                ctx,
                pred,
                vec![elem, Value::Int(idx as i64)]
            )?;
            if res.as_bool()? {
                return Ok(Value::Int(idx as i64));
            }
        }
        Ok(Value::Int(-1))
    }));

    lib.insert("find_last_index".into(), Value::NativeFunction(|ctx, _, args| {
        let [arr, pred] = Value::extract_args(args)?;
        let arr = arr.as_arr()?.get().clone();
        let pred = pred.as_closure()?;
        for (idx, elem) in arr.into_iter().enumerate().rev() {
            let res = executor::call(
                ctx,
                pred,
                vec![elem, Value::Int(idx as i64)]
            )?;
            if res.as_bool()? {
                return Ok(Value::Int(idx as i64));
            }
        }
        Ok(Value::Int(-1))
    }));

    lib.insert("for_each".into(), Value::NativeFunction(|ctx, _, args| {
        let [arr, action] = Value::extract_args(args)?;
        let arr = arr.as_arr()?.get().clone();
        let action = action.as_closure()?;
        for (idx, elem) in arr.into_iter().enumerate() {
            executor::call(
                ctx,
                action,
                vec![elem, Value::Int(idx as i64)]
            )?;
        }
        Ok(Value::Null)
    }));

    lib.insert("filter".into(), Value::NativeFunction(|ctx, _, args| {
        let [arr, filter] = Value::extract_args(args)?;
        let arr = arr.as_arr()?.get().clone();
        let filter = filter.as_closure()?;
        let mut filtered = Vec::with_capacity(arr.len());
        for (idx, elem) in arr.into_iter().enumerate() {
            let res = executor::call(
                ctx,
                filter,
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
        let [arr, mapping] = Value::extract_args(args)?;
        let arr = arr.as_arr()?.get().clone();
        let mapping = mapping.as_closure()?;
        let mut mapped = Vec::with_capacity(arr.len());
        for (idx, elem) in arr.into_iter().enumerate() {
            let res = executor::call(
                ctx,
                mapping,
                vec![elem, Value::Int(idx as i64)]
            )?;
            mapped.push(res);
        }
        Ok(Value::new_arr(mapped))
    }));

    ctx.add_lib("arrays".into(), Value::new_locked_obj(lib));
}
