use std::rc::Rc;
use gc::Gc;
use bytecode::{program::{ProgramBundle, Constant}, code};
use crate::types::{VMError, Variables, Closure, Value, Context};

fn next(func: &Vec<u8>, pc: &mut usize) -> Result<u8, VMError> {
    let code = *func.get(*pc)
        .ok_or_else(|| VMError::PCIndexOutOfBound)?;
    *pc = pc.checked_add(1)
        .ok_or_else(|| VMError::PCIndexOutOfBound)?;
    Ok(code)
}

fn next_str<'a>(func: &Vec<u8>, pc: &mut usize, program: &'a ProgramBundle) -> Result<&'a Rc<str>, VMError> {
    let str_idx: usize = next(func, pc)?.into();
    let constant = get_constant(program, str_idx)?;
    match constant {
        Constant::String(s) => Ok(s),
        _ => Err(VMError::ConstantNotString)
    }
}

fn get_constant(program: &ProgramBundle, idx: usize) -> Result<&Constant, VMError> {
    program.constant_pool.get(idx)
        .ok_or_else(|| VMError::ConstantIndexOutOfBound)
}

fn stack_top(stack: &Vec<Value>) -> Result<&Value, VMError> {
    stack.last().ok_or_else(|| VMError::BadStack)
}

fn stack_top_mut(stack: &mut Vec<Value>) -> Result<&mut Value, VMError> {
    stack.last_mut().ok_or_else(|| VMError::BadStack)
}

fn stack_pop(stack: &mut Vec<Value>) -> Result<Value, VMError> {
    stack.pop().ok_or_else(|| VMError::BadStack)
}

pub fn execute_closure(
    ctx: &mut Context,
    program_idx: usize,
    func_idx: usize,
    variables: Gc<Variables>,
    args: Vec<Value>
) -> Result<Value, VMError> {
    if func_idx >= ctx.get_program(program_idx).func_list.len() {
        return Err(VMError::FunctionIndexOutOfBound);
    }

    let this = variables.this_obj();

    let mut stack = vec![];

    let mut pc = 0usize;

    loop {
        let program = ctx.get_program(program_idx);
        let cur_func = &program.func_list[func_idx];

        let code = next(&cur_func, &mut pc)?;

        match code {
            code::LOAD => {
                let str = next_str(&cur_func, &mut pc, program)?;
                match this.borrow().get().get(str) {
                    Some(v) => stack.push(v.clone()),
                    None => stack.push(Value::Null)
                }
            }
            code::LOAD_SUPER => {
                let str = next_str(&cur_func, &mut pc, program)?;
                match variables.parent_obj()?.borrow().get().get(str) {
                    Some(v) => stack.push(v.clone()),
                    None => stack.push(Value::Null)
                }
            }
            code::LOAD_FIELD => {
                let str = next_str(&cur_func, &mut pc, program)?;
                let obj = stack_pop(&mut stack)?;
                match obj.as_obj()?.borrow().get().get(str) {
                    Some(v) => stack.push(v.clone()),
                    None => stack.push(Value::Null)
                };
            }
            code::LOAD_ITEM => {
                let idx = stack_pop(&mut stack)?;
                let obj = stack_pop(&mut stack)?;
                match &obj {
                    Value::Object(o) => {
                        let idx = idx.as_str()?;
                        match o.borrow().get().get(idx) {
                            Some(v) => stack.push(v.clone()),
                            None => stack.push(Value::Null)
                        }
                    }
                    Value::Array(a) => {
                        let idx: usize = idx.as_idx()?;
                        stack.push(a.borrow().get().get(idx)
                            .ok_or_else(|| VMError::ArrayIndexOutOfBound)?
                            .clone());
                    }
                    _ => return Err(VMError::invalid_type("object/array", obj.type_to_str()))
                }
            }
            code::STORE => {
                let str = next_str(&cur_func, &mut pc, program)?;
                let value = stack_pop(&mut stack)?;
                match &value {
                    Value::Null => this.borrow_mut().get_mut()?.remove(str),
                    _ => this.borrow_mut().get_mut()?
                        .insert(str.clone(), value)
                };
            }
            code::STORE_SUPER => {
                let str = next_str(&cur_func, &mut pc, program)?;
                let value = stack_pop(&mut stack)?;
                match &value {
                    Value::Null => variables.parent_obj()?.borrow_mut().get_mut()?.remove(str),
                    _ => variables.parent_obj()?.borrow_mut().get_mut()?
                        .insert(str.clone(), value)
                };
            }
            code::STORE_FIELD => {
                let str = next_str(&cur_func, &mut pc, program)?;
                let value = stack_pop(&mut stack)?;
                let obj = stack_pop(&mut stack)?;
                match &value {
                    Value::Null => obj.as_obj()?.borrow_mut().get_mut()?.remove(str),
                    _ => obj.as_obj()?.borrow_mut().get_mut()?
                        .insert(str.clone(), value.clone())
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
                            Value::Null => o.borrow_mut().get_mut()?.remove(idx),
                            _ => o.borrow_mut().get_mut()?
                                .insert(idx.clone(), value.clone())
                        };
                    }
                    Value::Array(a) => {
                        let idx: usize = idx.as_idx()?;
                        *a.borrow_mut().get_mut()?.get_mut(idx)
                            .ok_or_else(|| VMError::ArrayIndexOutOfBound)? = value.clone();
                    }
                    _ => return Err(VMError::invalid_type("object/array", obj.type_to_str()))
                }
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
                stack.push(match get_constant(program, const_idx)? {
                    Constant::Int(v) => Value::Int(*v),
                    Constant::Float(v) => Value::Float(*v),
                    Constant::String(v) => Value::String(v.clone())
                });
            }
            code::NEW_ARRAY => {
                let cnt: usize = next(&cur_func, &mut pc)?.into();
                if stack.len() < cnt {
                    return Err(VMError::BadStack);
                }
                let v = Value::new_arr(stack.drain(stack.len() - cnt ..).collect());
                stack.push(v);
            }
            code::PUSH_ARG => {
                let arg_idx: usize = next(&cur_func, &mut pc)?.into();
                stack.push(args.get(arg_idx).unwrap_or(&Value::Null).clone());
            }
            code::PUSH_SELF => stack.push(variables.this().clone()),
            code::PUSH_SUPER => {
                let lvl: u32 = next(&cur_func, &mut pc)?.into();
                let mut vars = Box::new(&variables);
                for _ in 0 .. lvl + 1 {
                    vars = Box::new(vars.parent()?);
                }
                stack.push(vars.this().clone());
            }
            code::PUSH_CLOSURE => {
                let idx: usize = next(&cur_func, &mut pc)?.into();
                stack.push(Value::Closure(Closure {
                    parent: variables.clone(),
                    program_idx,
                    func_idx: idx
                }));
            }
            code::JMP => {
                let offset = next(&cur_func, &mut pc)? as i8;
                pc = (pc as i64 - 1 + offset as i64) as usize;
            }
            code::JN => {
                let offset = next(&cur_func, &mut pc)? as i8;
                if let Value::Null = stack_pop(&mut stack)? {
                    pc = (pc as i64 - 1 + offset as i64) as usize;
                }
            }
            code::JT => {
                let offset = next(&cur_func, &mut pc)? as i8;
                if stack_pop(&mut stack)?.as_bool()? {
                    pc = (pc as i64 - 1 + offset as i64) as usize;
                }
            }
            code::JF => {
                let offset = next(&cur_func, &mut pc)? as i8;
                if !stack_pop(&mut stack)?.as_bool()? {
                    pc = (pc as i64 - 1 + offset as i64) as usize;
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
                    Value::Closure(closure) => stack.push(
                        call(ctx, closure, args)?
                    ),
                    Value::NativeFunction(func) => stack.push(
                        func(ctx, args)?
                    ),
                    v => return Err(
                        VMError::invalid_type("closure/native function", v.type_to_str())
                    )
                }
            }
            code::RETURN => break,
            code::ADD => {
                let v2 = stack_pop(&mut stack)?;
                let v1 = stack_top_mut(&mut stack)?;
                match v1 {
                    Value::Int(v1) => *v1 += v2.as_int()?,
                    Value::Float(v1) => *v1 += v2.as_float()?,
                    Value::String(s) => *v1 = Value::String((s.to_string() + v2.as_str()?).into()),
                    _ => return Err(VMError::invalid_type("int/float/string", v1.type_to_str()))
                }
            }
            code::SUB => {
                let v2 = stack_pop(&mut stack)?;
                let v1 = stack_top_mut(&mut stack)?;
                match v1 {
                    Value::Int(v1) => *v1 -= v2.as_int()?,
                    Value::Float(v1) => *v1 -= v2.as_float()?,
                    _ => return Err(VMError::invalid_type("int/float", v1.type_to_str()))
                }
            }
            code::MUL => {
                let v2 = stack_pop(&mut stack)?;
                let v1 = stack_top_mut(&mut stack)?;
                match v1 {
                    Value::Int(v1) => *v1 *= v2.as_int()?,
                    Value::Float(v1) => *v1 *= v2.as_float()?,
                    _ => return Err(VMError::invalid_type("int/float", v1.type_to_str()))
                }
            }
            code::DIV => {
                let v2 = stack_pop(&mut stack)?;
                let v1 = stack_top_mut(&mut stack)?;
                match v1 {
                    Value::Int(v1) => *v1 /= v2.as_int()?,
                    Value::Float(v1) => *v1 /= v2.as_float()?,
                    _ => return Err(VMError::invalid_type("int/float", v1.type_to_str()))
                }
            }
            code::MOD => {
                let v2 = stack_pop(&mut stack)?;
                let v1 = stack_top_mut(&mut stack)?;
                *v1.as_int_mut()? %= v2.as_int()?;
            }
            code::NEG => {
                let v = stack_top_mut(&mut stack)?;
                match v {
                    Value::Int(v) => *v = -*v,
                    Value::Float(v) => *v = -*v,
                    _ => return Err(VMError::invalid_type("int/float", v.type_to_str()))
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
                    _ => return Err(VMError::invalid_type("bool", v.type_to_str()))
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
                *v1.as_int_mut()? <<= v2.as_int()?;
            }
            code::SHR => {
                let v2 = stack_pop(&mut stack)?;
                let v1 = stack_top_mut(&mut stack)?;
                *v1.as_int_mut()? >>= v2.as_int()?;
            }
            code::LEN => {
                let v = stack_pop(&mut stack)?;
                let len = match &v {
                    Value::String(s) => s.len(),
                    Value::Object(o) => o.borrow().get().len(),
                    Value::Array(a) => a.borrow().get().len(),
                    _ => return Err(VMError::invalid_type("string/object/array", v.type_to_str()))
                };
                stack.push(Value::Int(len as i64));
            }
            code::IN => {
                let mut str = String::new();
                std::io::stdin().read_line(&mut str).expect("`stdin.read_line` failed");
                if str.ends_with("\n") {
                    str.truncate(str.len() - 1);
                }
                stack.push(Value::String(str.into()));
            }
            code::OUT => println!("{}", stack_pop(&mut stack)?.to_string()),
            code::LOAD_LIB => {
                let str = next_str(&cur_func, &mut pc, program)?;
                let value = match ctx.get_lib(str) {
                    Some(v) => v.clone(),
                    None => {
                        let lib_path = ctx.find_path(str)
                            .ok_or_else(|| VMError::UnknownLibrary(str.to_string()))?;
                        let lib_prog = compiler::compile_file(lib_path)?;
                        let lib_name = str.clone();
                        let lib_prog_idx = ctx.add_program(lib_prog);
                        let lib = execute_closure(
                            ctx, lib_prog_idx, 0, Variables::new_gc(None), vec![]
                        )?;
                        ctx.add_lib(lib_name, lib.clone());
                        lib
                    }
                };
                stack.push(value);
            }
            _ => return Err(VMError::UnknownInstruction(code))
        }
    }

    assert!(stack.len() == 1);

    Ok(stack.pop().unwrap())
}

pub fn call(
    ctx: &mut Context,
    closure: & Closure,
    args: Vec<Value>) -> Result<Value, VMError> {
    execute_closure(
        ctx,
        closure.program_idx,
        closure.func_idx,
        Variables::new_gc(Some(&closure.parent)),
        args
    )
}

pub fn execute_program(program: ProgramBundle, paths: Vec<String>) -> Result<(), VMError> {
    let mut ctx = Context::new(program, paths);

    execute_closure(&mut ctx, 0, 0, Variables::new_gc(None), vec![])?;

    Ok(())
}
