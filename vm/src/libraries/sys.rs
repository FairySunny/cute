use std::collections::HashMap;
use crate::{types::{VMError, Value, Context}, executor};

pub fn load_libs(ctx: &mut Context) {
    let mut lib = HashMap::new();

    lib.insert("call".into(), Value::NativeFunction(|ctx, state, args| {
        let [func, arg_arr] = Value::extract_args(args)?;
        let arg_arr = arg_arr.as_arr()?.get().clone();
        match &func {
            Value::Closure(c) => executor::call(ctx, c, arg_arr),
            Value::NativeFunction(f) => f(ctx, state, arg_arr),
            v => Err(VMError::invalid_type("closure/native function", v))
        }
    }));

    lib.insert("arguments".into(), Value::NativeFunction(|_, state, args| {
        let [] = Value::extract_args(args)?;
        Ok(Value::new_arr(state.args.clone()))
    }));

    lib.insert("this".into(), Value::NativeFunction(|_, state, args| {
        let [] = Value::extract_args(args)?;
        Ok(state.variables.this().clone())
    }));

    lib.insert("super".into(), Value::NativeFunction(|_, state, args| {
        let ([], [level]) = Value::extract_args_and_optional(args)?;
        let level: u64 = match &level {
            Some(lvl) => lvl.as_int()?.try_into()
                .map_err(|_| VMError::IllegalFunctionArguments)?,
            None => 0
        };
        Ok(state.variables.ancestor(level)?.clone())
    }));

    lib.insert("locked_copy".into(), Value::NativeFunction(|_, _, args| {
        let [obj] = Value::extract_args(args)?;
        let locked = match &obj {
            Value::Object(o) => Value::new_locked_obj(o.get().clone()),
            Value::Array(a) => Value::new_locked_arr(a.get().clone()),
            v => return Err(VMError::invalid_type("object/array", v))
        };
        Ok(locked)
    }));

    lib.insert("add_lib".into(), Value::NativeFunction(|ctx, _, args| {
        let [name, lib] = Value::extract_args(args)?;
        let name = name.as_str()?;
        if let Some(_) = ctx.get_lib(name) {
            return Err(VMError::IllegalState);
        }
        ctx.add_lib(name.clone(), lib);
        Ok(Value::Null)
    }));

    lib.insert("get_libs".into(), Value::NativeFunction(|ctx, _, args| {
        let [] = Value::extract_args(args)?;
        let libs_iter = ctx.get_libs().iter().map(|(k, v)|
            Value::new_arr(vec![
                Value::String(k.clone()),
                v.clone()
            ])
        );
        let file_libs_iter = ctx.get_file_libs().iter().map(|(k, v)|
            Value::new_arr(vec![
                k.to_str().map_or(Value::Null, |s| Value::String(s.into())),
                v.clone()
            ])
        );
        Ok(Value::new_arr(libs_iter.chain(file_libs_iter).collect()))
    }));

    lib.insert("script_path".into(), Value::NativeFunction(|ctx, state, args| {
        let [] = Value::extract_args(args)?;
        let path = ctx.get_program_path(state.program_idx)
            .and_then(|p| p.to_str())
            .map_or(Value::Null, |s| Value::String(s.into()));
        Ok(path)
    }));

    lib.insert("script_directory".into(), Value::NativeFunction(|ctx, state, args| {
        let [] = Value::extract_args(args)?;
        let dir = ctx.get_program_dir(state.program_idx)
            .and_then(|p| p.to_str())
            .map_or(Value::Null, |s| Value::String(s.into()));
        Ok(dir)
    }));

    ctx.add_lib("sys".into(), Value::new_locked_obj(lib));
}
