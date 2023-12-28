use std::collections::HashMap;
use crate::types::{Value, Context};

pub fn load_libs(ctx: &mut Context) {
    let mut lib = HashMap::new();

    lib.insert("entries".into(), Value::NativeFunction(|_, _, args| {
        let [obj] = Value::extract_args(args)?;
        let entries = obj.as_obj()?.get().iter().map(|(k, v)|
            Value::new_arr(vec![Value::String(k.clone()), v.clone()])
        ).collect();
        Ok(Value::new_arr(entries))
    }));

    lib.insert("keys".into(), Value::NativeFunction(|_, _, args| {
        let [obj] = Value::extract_args(args)?;
        let entries = obj.as_obj()?.get().iter().map(|(k, _)|
            Value::String(k.clone())
        ).collect();
        Ok(Value::new_arr(entries))
    }));

    lib.insert("values".into(), Value::NativeFunction(|_, _, args| {
        let [obj] = Value::extract_args(args)?;
        let entries = obj.as_obj()?.get().iter().map(|(_, v)|
            v.clone()
        ).collect();
        Ok(Value::new_arr(entries))
    }));

    ctx.add_lib("objects".into(), Value::new_locked_obj(lib));
}
