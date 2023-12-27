use std::{collections::HashMap, path::Path};

use crate::{types::{VMError, Value, Context}, executor};

pub fn load_libs(ctx: &mut Context) {
    let mut lib = HashMap::new();

    lib.insert("exit".into(), Value::NativeFunction(|_, _, args| {
        Value::check_arg_cnt(&args, 1)?;
        let code = args[0].as_int()?;
        Err(VMError::Exit(code))
    }));

    lib.insert("import".into(), Value::NativeFunction(|ctx, state, args| {
        Value::check_arg_cnt(&args, 1)?;
        let lib_path = Path::new(args[0].as_str()?.as_ref());
        let lib_path = if lib_path.is_absolute() {
            lib_path.canonicalize()?
        } else {
            ctx.get_program_dir(state.program_idx)
                .ok_or_else(|| VMError::IllegalState)?
                .join(lib_path)
                .canonicalize()?
        };
        Ok(match ctx.get_file_lib(&lib_path) {
            Some(lib) => lib.clone(),
            None => {
                let lib = executor::execute_file(ctx, &lib_path)?;
                ctx.add_file_lib(lib_path, lib.clone());
                lib
            }
        })
    }));

    lib.insert("as_lib".into(), Value::NativeFunction(|ctx, _, args| {
        Value::check_arg_cnt(&args, 2)?;
        let name = args[1].as_str()?;
        if let Some(_) = ctx.get_lib(name) {
            return Err(VMError::IllegalState);
        }
        ctx.add_lib(name.clone(), args.into_iter().next().unwrap());
        Ok(Value::Null)
    }));

    ctx.add_lib("sys".into(), Value::new_locked_obj(lib));
}
