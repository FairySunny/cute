use std::{rc::Rc, path::{Path, PathBuf}, fs};
use bytecode::{program::{ProgramBundle, Constant}, code};
use crate::types::{VMError, Variables, VMString, Closure, Value, Context, ProgramState};

fn next(func: &Vec<u8>, pc: &mut usize) -> Result<u8, VMError> {
    let code = *func.get(*pc)
        .ok_or(VMError::PCIndexOutOfBound)?;
    *pc = pc.checked_add(1)
        .ok_or(VMError::PCIndexOutOfBound)?;
    Ok(code)
}

fn next_str<'a>(func: &Vec<u8>, pc: &mut usize, program: &'a ProgramBundle) -> Result<&'a [u16], VMError> {
    let str_idx: usize = next(func, pc)?.into();
    let constant = get_constant(program, str_idx)?;
    match constant {
        Constant::String(s) => Ok(s),
        _ => Err(VMError::ConstantNotString)
    }
}

fn jump(pc: &mut usize, offset: u8) -> Result<(), VMError> {
    *pc -= 1;
    *pc = pc.checked_add_signed((offset as i8).into())
        .ok_or(VMError::PCIndexOutOfBound)?;
    Ok(())
}

fn get_constant(program: &ProgramBundle, idx: usize) -> Result<&Constant, VMError> {
    program.constant_pool.get(idx)
        .ok_or(VMError::ConstantIndexOutOfBound)
}

fn stack_top(stack: &Vec<Value>) -> Result<&Value, VMError> {
    stack.last().ok_or(VMError::BadStack)
}

fn stack_top_mut(stack: &mut Vec<Value>) -> Result<&mut Value, VMError> {
    stack.last_mut().ok_or(VMError::BadStack)
}

fn stack_pop(stack: &mut Vec<Value>) -> Result<Value, VMError> {
    stack.pop().ok_or(VMError::BadStack)
}

fn execute_closure(ctx: &mut Context, state: ProgramState) -> Result<Value, VMError> {
    if state.func_idx >= ctx.get_program(state.program_idx).func_list.len() {
        return Err(VMError::FunctionIndexOutOfBound);
    }

    let this = state.variables.this_obj();

    let mut stack = vec![];

    let mut pc = 0usize;

    loop {
        let program = ctx.get_program(state.program_idx);
        let cur_func = &program.func_list[state.func_idx];

        let code = next(&cur_func, &mut pc)?;

        match code {
            code::LOAD => {
                let str = next_str(&cur_func, &mut pc, program)?;
                match this.get().get(str) {
                    Some(v) => stack.push(v.clone()),
                    None => stack.push(Value::Null)
                }
            }
            code::LOAD_SUPER => {
                let str = next_str(&cur_func, &mut pc, program)?;
                match state.variables.parent_obj()?.get().get(str) {
                    Some(v) => stack.push(v.clone()),
                    None => stack.push(Value::Null)
                }
            }
            code::LOAD_FIELD => {
                let str = next_str(&cur_func, &mut pc, program)?;
                let obj = stack_pop(&mut stack)?;
                match obj.as_obj()?.get().get(str) {
                    Some(v) => stack.push(v.clone()),
                    None => stack.push(Value::Null)
                };
            }
            code::LOAD_ITEM => {
                let idx = stack_pop(&mut stack)?;
                let obj = stack_pop(&mut stack)?;
                match &obj {
                    Value::String(s) => {
                        let idx = idx.as_idx()?;
                        let char = *s.data().get(idx)
                            .ok_or(VMError::ArrayIndexOutOfBound)?;
                        stack.push(Value::String([char][..].into()));
                    }
                    Value::Object(o) => {
                        let idx = idx.as_str()?;
                        match o.get().get(idx) {
                            Some(v) => stack.push(v.clone()),
                            None => stack.push(Value::Null)
                        }
                    }
                    Value::Array(a) => {
                        let idx = idx.as_idx()?;
                        let elem = a.get().get(idx)
                            .ok_or(VMError::ArrayIndexOutOfBound)?
                            .clone();
                        stack.push(elem);
                    }
                    _ => return Err(VMError::invalid_type("object/array", &obj))
                }
            }
            code::LOAD_SLICE => {
                let end = stack_pop(&mut stack)?.as_slice_idx()?;
                let start = stack_pop(&mut stack)?.as_slice_idx()?.unwrap_or(0);
                let obj = stack_pop(&mut stack)?;
                let slice = match &obj {
                    Value::String(s) => {
                        let str = s.data();
                        let end = end.unwrap_or(str.len());
                        let slice = str.get(start .. end)
                            .ok_or(VMError::ArrayIndexOutOfBound)?;
                        Value::String(slice.into())
                    }
                    Value::Array(a) => {
                        let arr = a.get();
                        let end = end.unwrap_or(arr.len());
                        let slice = arr.get(start .. end)
                            .ok_or(VMError::ArrayIndexOutOfBound)?;
                        Value::new_arr(slice.to_vec())
                    }
                    _ => return Err(VMError::invalid_type("string/array", &obj))
                };
                stack.push(slice);
            }
            code::STORE => {
                let str = next_str(&cur_func, &mut pc, program)?;
                let value = stack_pop(&mut stack)?;
                match &value {
                    Value::Null => this.get_mut()?.remove(str),
                    _ => this.get_mut()?.insert(str.into(), value)
                };
            }
            code::STORE_SUPER => {
                let str = next_str(&cur_func, &mut pc, program)?;
                let value = stack_pop(&mut stack)?;
                match &value {
                    Value::Null => state.variables.parent_obj()?.get_mut()?.remove(str),
                    _ => state.variables.parent_obj()?.get_mut()?.insert(str.into(), value)
                };
            }
            code::STORE_FIELD => {
                let str = next_str(&cur_func, &mut pc, program)?;
                let value = stack_pop(&mut stack)?;
                let obj = stack_pop(&mut stack)?;
                match &value {
                    Value::Null => obj.as_obj()?.get_mut()?.remove(str),
                    _ => obj.as_obj()?.get_mut()?.insert(str.into(), value.clone())
                };
            }
            code::STORE_ITEM => {
                let value = stack_pop(&mut stack)?;
                let idx = stack_pop(&mut stack)?;
                let obj = stack_pop(&mut stack)?;
                match &obj {
                    Value::Object(o) => {
                        let idx = idx.as_str()?;
                        match &value {
                            Value::Null => o.get_mut()?.remove(idx),
                            _ => o.get_mut()?.insert(idx.clone(), value.clone())
                        };
                    }
                    Value::Array(a) => {
                        let idx = idx.as_idx()?;
                        *a.get_mut()?.get_mut(idx)
                            .ok_or(VMError::ArrayIndexOutOfBound)? = value.clone();
                    }
                    _ => return Err(VMError::invalid_type("object/array", &obj))
                }
            }
            code::STORE_SLICE => {
                let value = stack_pop(&mut stack)?;
                let end = stack_pop(&mut stack)?.as_slice_idx()?;
                let start = stack_pop(&mut stack)?.as_slice_idx()?.unwrap_or(0);
                let obj = stack_pop(&mut stack)?;
                match &obj {
                    Value::Array(a) => {
                        let mut arr = a.get_mut()?;
                        let end = end.unwrap_or(arr.len());
                        arr.get(start .. end)
                            .ok_or(VMError::ArrayIndexOutOfBound)?;
                        arr.splice(start .. end, value.as_arr()?.get().clone());
                    }
                    _ => return Err(VMError::invalid_type("array", &obj))
                };
            }
            code::DUP => stack.push(stack_top(&stack)?.clone()),
            code::DUP_PRE2 => {
                if stack.len() < 2 {
                    return Err(VMError::BadStack);
                }
                stack.insert(stack.len() - 2, stack.last().unwrap().clone());
            }
            code::DUP_PRE3 => {
                if stack.len() < 3 {
                    return Err(VMError::BadStack);
                }
                stack.insert(stack.len() - 3, stack.last().unwrap().clone());
            }
            code::DUP_PRE4 => {
                if stack.len() < 4 {
                    return Err(VMError::BadStack);
                }
                stack.insert(stack.len() - 4, stack.last().unwrap().clone());
            }
            code::POP => {
                stack_pop(&mut stack)?;
            }
            code::PUSH_NULL => stack.push(Value::Null),
            code::PUSH_INT => {
                let i = next(&cur_func, &mut pc)? as i8;
                stack.push(Value::Int(i.into()));
            }
            code::PUSH_CONST => {
                let const_idx: usize = next(&cur_func, &mut pc)?.into();
                let value = match get_constant(program, const_idx)? {
                    Constant::Int(v) => Value::Int(*v),
                    Constant::Float(v) => Value::Float(*v),
                    Constant::String(v) => Value::String(v[..].into())
                };
                stack.push(value);
            }
            code::NEW_ARRAY => {
                let cnt: usize = next(&cur_func, &mut pc)?.into();
                if stack.len() < cnt {
                    return Err(VMError::BadStack);
                }
                let arr = stack.drain(stack.len() - cnt ..).collect();
                stack.push(Value::new_arr(arr));
            }
            code::PUSH_ARG => {
                let arg_idx: usize = next(&cur_func, &mut pc)?.into();
                stack.push(state.args.get(arg_idx).unwrap_or(&Value::Null).clone());
            }
            code::PUSH_SELF => stack.push(state.variables.this().clone()),
            code::PUSH_SUPER => {
                let lvl: u64 = next(&cur_func, &mut pc)?.into();
                stack.push(state.variables.ancestor(lvl)?.clone());
            }
            code::PUSH_CLOSURE => {
                let idx: usize = next(&cur_func, &mut pc)?.into();
                let closure = Closure {
                    parent: state.variables.clone(),
                    program_idx: state.program_idx,
                    func_idx: idx
                };
                stack.push(Value::Closure(closure));
            }
            code::JMP => {
                let offset = next(&cur_func, &mut pc)?;
                jump(&mut pc, offset)?;
            }
            code::JN => {
                let offset = next(&cur_func, &mut pc)?;
                if let Value::Null = stack_pop(&mut stack)? {
                    jump(&mut pc, offset)?;
                }
            }
            code::JT => {
                let offset = next(&cur_func, &mut pc)?;
                if stack_pop(&mut stack)?.as_bool()? {
                    jump(&mut pc, offset)?;
                }
            }
            code::JF => {
                let offset = next(&cur_func, &mut pc)?;
                if !stack_pop(&mut stack)?.as_bool()? {
                    jump(&mut pc, offset)?;
                }
            }
            code::CALL => {
                let arg_cnt: usize = next(&cur_func, &mut pc)?.into();
                if stack.len() < 1 + arg_cnt {
                    return Err(VMError::BadStack);
                }
                let args = stack.drain(stack.len() - arg_cnt ..).collect();
                let func = stack.pop().unwrap();
                match &func {
                    Value::Closure(closure) =>
                        stack.push(call(ctx, closure, args)?),
                    Value::NativeFunction(func) =>
                        stack.push(func(ctx, &state, args)?),
                    v =>
                        return Err(VMError::invalid_type("closure/native function", v))
                }
            }
            code::RETURN => break,
            code::ADD => {
                let v2 = stack_pop(&mut stack)?;
                let v1 = stack_top_mut(&mut stack)?;
                match v1 {
                    Value::Int(v1) => *v1 = v1.wrapping_add(v2.as_int()?),
                    Value::Float(v1) => *v1 += v2.as_float()?,
                    Value::String(s) => {
                        let str = [s.data(), v2.as_str()?.data()].concat();
                        *v1 = Value::String(str[..].into());
                    }
                    Value::Array(a) => {
                        let arr = [&a.get()[..], &v2.as_arr()?.get()[..]].concat();
                        *v1 = Value::new_arr(arr);
                    }
                    _ => return Err(VMError::invalid_type("int/float/string/array", v1))
                }
            }
            code::SUB => {
                let v2 = stack_pop(&mut stack)?;
                let v1 = stack_top_mut(&mut stack)?;
                match v1 {
                    Value::Int(v1) => *v1 = v1.wrapping_sub(v2.as_int()?),
                    Value::Float(v1) => *v1 -= v2.as_float()?,
                    _ => return Err(VMError::invalid_type("int/float", v1))
                }
            }
            code::MUL => {
                let v2 = stack_pop(&mut stack)?;
                let v1 = stack_top_mut(&mut stack)?;
                match v1 {
                    Value::Int(v1) => *v1 = v1.wrapping_mul(v2.as_int()?),
                    Value::Float(v1) => *v1 *= v2.as_float()?,
                    _ => return Err(VMError::invalid_type("int/float", v1))
                }
            }
            code::DIV => {
                let v2 = stack_pop(&mut stack)?;
                let v1 = stack_top_mut(&mut stack)?;
                match v1 {
                    Value::Int(v1) => {
                        let v2 = v2.as_int()?;
                        if v2 == 0 {
                            return Err(VMError::DivideByZeroError);
                        }
                        *v1 = v1.wrapping_div(v2)
                    }
                    Value::Float(v1) => *v1 /= v2.as_float()?,
                    _ => return Err(VMError::invalid_type("int/float", v1))
                }
            }
            code::MOD => {
                let v2 = stack_pop(&mut stack)?;
                let v1 = stack_top_mut(&mut stack)?;
                let v1 = v1.as_int_mut()?;
                let v2 = v2.as_int()?;
                if v2 == 0 {
                    return Err(VMError::DivideByZeroError);
                }
                *v1 = v1.wrapping_rem(v2);
            }
            code::NEG => {
                let v = stack_top_mut(&mut stack)?;
                match v {
                    Value::Int(v) => *v = v.wrapping_neg(),
                    Value::Float(v) => *v = -*v,
                    _ => return Err(VMError::invalid_type("int/float", v))
                }
            }
            code::CMP_EQ => {
                let v2 = stack_pop(&mut stack)?;
                let v1 = stack_pop(&mut stack)?;
                stack.push(Value::Bool(v1.cmp_eq(&v2)));
            }
            code::CMP_NE => {
                let v2 = stack_pop(&mut stack)?;
                let v1 = stack_pop(&mut stack)?;
                stack.push(Value::Bool(!v1.cmp_eq(&v2)));
            }
            code::CMP_GT => {
                let v2 = stack_pop(&mut stack)?;
                let v1 = stack_pop(&mut stack)?;
                stack.push(Value::Bool(v1.cmp_gt(&v2)?));
            }
            code::CMP_LT => {
                let v2 = stack_pop(&mut stack)?;
                let v1 = stack_pop(&mut stack)?;
                stack.push(Value::Bool(v1.cmp_lt(&v2)?));
            }
            code::CMP_GE => {
                let v2 = stack_pop(&mut stack)?;
                let v1 = stack_pop(&mut stack)?;
                stack.push(Value::Bool(!v1.cmp_lt(&v2)?));
            }
            code::CMP_LE => {
                let v2 = stack_pop(&mut stack)?;
                let v1 = stack_pop(&mut stack)?;
                stack.push(Value::Bool(!v1.cmp_gt(&v2)?));
            }
            code::NOT => {
                let v = stack_top_mut(&mut stack)?;
                match v {
                    Value::Bool(b) => *b = !*b,
                    _ => return Err(VMError::invalid_type("bool", v))
                }
            }
            code::BAND => {
                let v2 = stack_pop(&mut stack)?;
                let v1 = stack_top_mut(&mut stack)?;
                *v1.as_int_mut()? &= v2.as_int()?;
            }
            code::BOR => {
                let v2 = stack_pop(&mut stack)?;
                let v1 = stack_top_mut(&mut stack)?;
                *v1.as_int_mut()? |= v2.as_int()?;
            }
            code::BXOR => {
                let v2 = stack_pop(&mut stack)?;
                let v1 = stack_top_mut(&mut stack)?;
                *v1.as_int_mut()? ^= v2.as_int()?;
            }
            code::BINV => {
                let v = stack_top_mut(&mut stack)?;
                let i = v.as_int_mut()?;
                *i = !*i;
            }
            code::SHL => {
                let v2 = stack_pop(&mut stack)?;
                let v1 = stack_top_mut(&mut stack)?;
                let v1 = v1.as_int_mut()?;
                let v2 = v2.as_int()?;
                *v1 = v1.wrapping_shl(v2 as u32);
            }
            code::SHR => {
                let v2 = stack_pop(&mut stack)?;
                let v1 = stack_top_mut(&mut stack)?;
                match v1 {
                    Value::Int(v1) => {
                        let v2 = v2.as_int()?;
                        *v1 = v1.wrapping_shr(v2 as u32);
                    }
                    Value::Array(arr) => {
                        let arr = arr.get().clone();
                        let func = v2.as_closure()?;
                        let mut res_arr = vec![];
                        for elem in arr.into_iter() {
                            let res = call(ctx, func, vec![elem.clone()])?;
                            match res {
                                Value::Null => {}
                                _ => res_arr.push(res)
                            }
                        }
                        *v1 = Value::new_arr(res_arr);
                    }
                    _ => return Err(VMError::invalid_type("int/array", v1))
                }
            }
            code::LEN => {
                let v = stack_pop(&mut stack)?;
                let len = match &v {
                    Value::String(s) => s.data().len(),
                    Value::Object(o) => o.get().len(),
                    Value::Array(a) => a.get().len(),
                    _ => return Err(VMError::invalid_type("string/object/array", &v))
                };
                stack.push(Value::Int(len as i64));
            }
            code::IN => {
                let mut str = String::new();
                std::io::stdin().read_line(&mut str).expect("`stdin.read_line` failed");
                if str.ends_with("\n") {
                    str.truncate(str.len() - 1);
                }
                stack.push(Value::String(str[..].into()));
            }
            code::OUT => println!("{}", stack_pop(&mut stack)?.to_string()),
            code::LOAD_LIB => {
                let str = next_str(&cur_func, &mut pc, program)?.into();
                let value = load_library(ctx, &state, &str)?;
                stack.push(value);
            }
            _ => return Err(VMError::UnknownInstruction(code))
        }
    }

    assert!(stack.len() == 1);

    Ok(stack.pop().unwrap())
}

pub fn call(ctx: &mut Context, closure: &Closure, args: Vec<Value>) -> Result<Value, VMError> {
    execute_closure(ctx, ProgramState {
        program_idx: closure.program_idx,
        func_idx: closure.func_idx,
        variables: Variables::new_gc(Some(&closure.parent)),
        args
    })
}

pub fn execute_file(ctx: &mut Context, path: Rc<Path>) -> Result<Value, VMError> {
    let program = compiler::compile_chars(fs::read_to_string(&path)?.chars())?;
    let program_idx = ctx.add_program(program, Some(path));
    execute_closure(ctx, ProgramState {
        program_idx,
        func_idx: 0,
        variables: Variables::new_gc(None),
        args: vec![]
    })
}

pub fn load_library(ctx: &mut Context, state: &ProgramState, name: &VMString) -> Result<Value, VMError> {
    let value = match ctx.get_lib(name) {
        Some(lib) => lib.clone(),
        None => {
            let lib_path = PathBuf::from(name.to_string() + ".cute");
            let lib_path: Rc<Path> = if lib_path.is_absolute() {
                lib_path
            } else {
                ctx.get_program_dir(state.program_idx)
                    .ok_or(VMError::IllegalState)?
                    .join(lib_path)
            }.canonicalize()?.into();
            match ctx.get_file_lib(&lib_path) {
                Some(lib) => lib.clone(),
                None => {
                    let lib = execute_file(ctx, lib_path.clone())?;
                    ctx.add_file_lib(lib_path, lib.clone());
                    lib
                }
            }
        }
    };
    Ok(value)
}

pub fn execute_program(program: ProgramBundle, path: Option<Rc<Path>>) -> Result<(), VMError> {
    let mut ctx = Context::new(program, path);
    execute_closure(&mut ctx, ProgramState {
        program_idx: 0,
        func_idx: 0,
        variables: Variables::new_gc(None),
        args: vec![]
    })?;
    Ok(())
}
