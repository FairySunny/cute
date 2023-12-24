use std::collections::HashMap;
use gc::{Gc, GcCell};
use bytecode::{program::{ProgramBundle, Constant}, code};
use crate::{types::{VMError, Lockable, Variables, Closure, Value}, libraries};

struct StackInfo {
    variables: Gc<Variables>,
    arg_cnt: usize,
    func_idx_return: usize,
    ptr_return: usize,
    pc_return: usize
}

fn next(func: &Vec<u8>, pc: &mut usize) -> Result<u8, VMError> {
    let code = *func.get(*pc)
        .ok_or_else(|| VMError::PCIndexOutOfBound)?;
    *pc = pc.checked_add(1)
        .ok_or_else(|| VMError::PCIndexOutOfBound)?;
    Ok(code)
}

fn next_str<'a>(func: &Vec<u8>, pc: &mut usize, program: &'a ProgramBundle) -> Result<&'a str, VMError> {
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

fn stack_top(stack: &Vec<Value>, ptr: usize) -> Result<&Value, VMError> {
    if stack.len() <= ptr {
        Err(VMError::BadStack)
    } else {
        Ok(stack.last().unwrap())
    }
}

fn stack_top_mut(stack: &mut Vec<Value>, ptr: usize) -> Result<&mut Value, VMError> {
    if stack.len() <= ptr {
        Err(VMError::BadStack)
    } else {
        Ok(stack.last_mut().unwrap())
    }
}

fn stack_pop(stack: &mut Vec<Value>, ptr: usize) -> Result<Value, VMError> {
    if stack.len() <= ptr {
        Err(VMError::BadStack)
    } else {
        Ok(stack.pop().unwrap())
    }
}

fn cur_info(info: &Vec<StackInfo>) -> &StackInfo {
    info.last().expect("`info` is empty")
}

fn this(info: &Vec<StackInfo>) -> &Gc<GcCell<Lockable<HashMap<String, Value>>>> {
    cur_info(info).variables.this_obj()
}

fn parent(info: &Vec<StackInfo>) -> Result<&Gc<GcCell<Lockable<HashMap<String, Value>>>>, VMError> {
    Ok(cur_info(info).variables.parent.as_ref()
        .ok_or_else(|| VMError::SuperDoesNotExist)?
        .this_obj())
}

pub fn run_program(program: &ProgramBundle) -> Result<(), VMError> {
    let mut stack = vec![Value::Null];
    let mut info = vec![
        StackInfo {
            variables: Gc::new(Variables::new(None)),
            arg_cnt: 0,
            func_idx_return: 0,
            ptr_return: 0,
            pc_return: 0
        }
    ];

    let mut libs = HashMap::new();
    libraries::misc::load_libs(&mut libs);
    libraries::types::load_libs(&mut libs);
    libraries::arrays::load_libs(&mut libs);

    let mut func_idx = 0usize;
    let mut cur_func = Box::new(program.func_list.get(func_idx)
        .ok_or_else(|| VMError::FunctionIndexOutOfBound)?);

    let mut pc = 0usize;
    let mut ptr = 1usize;

    loop {
        let code = next(&cur_func, &mut pc)?;

        match code {
            code::LOAD => {
                let str = next_str(&cur_func, &mut pc, program)?;
                match this(&info).borrow().get().get(str) {
                    Some(v) => stack.push(v.clone()),
                    None => stack.push(Value::Null)
                }
            }
            code::LOAD_SUPER => {
                let str = next_str(&cur_func, &mut pc, program)?;
                match parent(&info)?.borrow().get().get(str) {
                    Some(v) => stack.push(v.clone()),
                    None => stack.push(Value::Null)
                }
            }
            code::LOAD_FIELD => {
                let str = next_str(&cur_func, &mut pc, program)?;
                let obj = stack_pop(&mut stack, ptr)?;
                match obj.as_obj()?.borrow().get().get(str) {
                    Some(v) => stack.push(v.clone()),
                    None => stack.push(Value::Null)
                };
            }
            code::LOAD_ITEM => {
                let idx = stack_pop(&mut stack, ptr)?;
                let obj = stack_pop(&mut stack, ptr)?;
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
                    _ => return Err(VMError::InvalidType {
                        expected: "object/array",
                        got: obj.type_to_str()
                    })
                }
            }
            code::STORE => {
                let str = next_str(&cur_func, &mut pc, program)?;
                let value = stack_pop(&mut stack, ptr)?;
                match &value {
                    Value::Null => this(&info).borrow_mut().get_mut()?.remove(str),
                    _ => this(&info).borrow_mut().get_mut()?
                        .insert(str.to_owned(), value)
                };
            }
            code::STORE_SUPER => {
                let str = next_str(&cur_func, &mut pc, program)?;
                let value = stack_pop(&mut stack, ptr)?;
                match &value {
                    Value::Null => parent(&info)?.borrow_mut().get_mut()?.remove(str),
                    _ => parent(&info)?.borrow_mut().get_mut()?
                        .insert(str.to_owned(), value)
                };
            }
            code::STORE_FIELD => {
                let str = next_str(&cur_func, &mut pc, program)?;
                let value = stack_pop(&mut stack, ptr)?;
                let obj = stack_pop(&mut stack, ptr)?;
                match &value {
                    Value::Null => obj.as_obj()?.borrow_mut().get_mut()?.remove(str),
                    _ => obj.as_obj()?.borrow_mut().get_mut()?
                        .insert(str.to_owned(), value.clone())
                };
            }
            code::STORE_ITEM => {
                let value = stack_pop(&mut stack, ptr)?;
                let idx = stack_pop(&mut stack, ptr)?;
                let obj = stack_pop(&mut stack, ptr)?;
                match &obj {
                    Value::Object(o) => {
                        let idx = idx.as_str()?;
                        match &value {
                            Value::Null => o.borrow_mut().get_mut()?.remove(idx),
                            _ => o.borrow_mut().get_mut()?
                                .insert(idx.to_owned(), value.clone())
                        };
                    }
                    Value::Array(a) => {
                        let idx: usize = idx.as_idx()?;
                        *a.borrow_mut().get_mut()?.get_mut(idx)
                            .ok_or_else(|| VMError::ArrayIndexOutOfBound)? = value.clone();
                    }
                    _ => return Err(VMError::InvalidType {
                        expected: "object/array",
                        got: obj.type_to_str()
                    })
                }
            }
            code::DUP => stack.push(stack_top(&stack, ptr)?.clone()),
            code::DUP_PRE2 => {
                if stack.len() < ptr + 2 {
                    return Err(VMError::BadStack);
                }
                stack.insert(stack.len() - 2, stack.last().unwrap().clone());
            }
            code::DUP_PRE3 => {
                if stack.len() < ptr + 3 {
                    return Err(VMError::BadStack);
                }
                stack.insert(stack.len() - 3, stack.last().unwrap().clone());
            }
            code::POP => {
                stack_pop(&mut stack, ptr)?;
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
                    Constant::String(v) => Value::new_str(v)
                });
            }
            code::NEW_ARRAY => {
                let cnt: usize = next(&cur_func, &mut pc)?.into();
                if stack.len() < ptr + cnt {
                    return Err(VMError::BadStack);
                }
                let v = Value::new_arr(stack.drain(stack.len() - cnt ..).collect());
                stack.push(v);
            }
            code::PUSH_ARG => {
                let arg_idx: usize = next(&cur_func, &mut pc)?.into();
                let arg_cnt = cur_info(&info).arg_cnt;
                if arg_idx < arg_cnt {
                    stack.push(stack[ptr - arg_cnt + arg_idx].clone());
                } else {
                    stack.push(Value::Null);
                }
            }
            code::PUSH_SELF => stack.push(cur_info(&info).variables.this.clone()),
            code::PUSH_SUPER => {
                let lvl: u32 = next(&cur_func, &mut pc)?.into();
                let mut vars = Box::new(&cur_info(&info).variables);
                for _ in 0 .. lvl + 1 {
                    vars = Box::new(vars.parent.as_ref()
                        .ok_or_else(|| VMError::SuperDoesNotExist)?);
                }
                stack.push(vars.this.clone());
            }
            code::PUSH_CLOSURE => {
                let idx: usize = next(&cur_func, &mut pc)?.into();
                stack.push(Value::Closure(Closure {
                    parent: cur_info(&info).variables.clone(),
                    func_idx: idx
                }));
            }
            code::JMP => {
                let offset = next(&cur_func, &mut pc)? as i8;
                pc = (pc as i64 - 1 + offset as i64) as usize;
            }
            code::JN => {
                let offset = next(&cur_func, &mut pc)? as i8;
                if let Value::Null = stack_pop(&mut stack, ptr)? {
                    pc = (pc as i64 - 1 + offset as i64) as usize;
                }
            }
            code::JT => {
                let offset = next(&cur_func, &mut pc)? as i8;
                if stack_pop(&mut stack, ptr)?.as_bool()? {
                    pc = (pc as i64 - 1 + offset as i64) as usize;
                }
            }
            code::JF => {
                let offset = next(&cur_func, &mut pc)? as i8;
                if !stack_pop(&mut stack, ptr)?.as_bool()? {
                    pc = (pc as i64 - 1 + offset as i64) as usize;
                }
            }
            code::CALL => {
                let arg_cnt: usize = next(&cur_func, &mut pc)?.into();
                if stack.len() < ptr + 1 + arg_cnt {
                    return Err(VMError::BadStack);
                }
                match &stack[stack.len() - arg_cnt - 1] {
                    Value::Closure(closure) => {
                        info.push(StackInfo {
                            variables: Gc::new(Variables::new(Some(&closure.parent))),
                            arg_cnt,
                            func_idx_return: func_idx,
                            ptr_return: ptr,
                            pc_return: pc
                        });
                        func_idx = closure.func_idx;
                        cur_func = Box::new(program.func_list.get(func_idx)
                            .ok_or_else(|| VMError::FunctionIndexOutOfBound)?);
                        pc = 0;
                        ptr = stack.len();
                    }
                    &Value::NativeFunction(func) => {
                        let args = stack.drain(stack.len() - arg_cnt ..).collect();
                        stack.pop().unwrap();
                        stack.push(func(args)?);
                    }
                    v => return Err(VMError::InvalidType {
                        expected: "closure/native function",
                        got: v.type_to_str()
                    })
                }
            }
            code::RETURN => {
                let value = stack_pop(&mut stack, ptr)?;
                let cur_info = cur_info(&info);
                if stack.len() != ptr {
                    return Err(VMError::BadStack);
                }
                stack.resize(ptr - cur_info.arg_cnt - 1, Value::Null);
                stack.push(value);
                if info.len() <= 1 {
                    break;
                }
                func_idx = cur_info.func_idx_return;
                cur_func = Box::new(program.func_list.get(func_idx)
                    .ok_or_else(|| VMError::FunctionIndexOutOfBound)?);
                pc = cur_info.pc_return;
                ptr = cur_info.ptr_return;
                info.pop();
            }
            code::ADD => {
                let v2 = stack_pop(&mut stack, ptr)?;
                let v1 = stack_top_mut(&mut stack, ptr)?;
                match v1 {
                    Value::Int(v1) => *v1 += v2.as_int()?,
                    Value::Float(v1) => *v1 += v2.as_float()?,
                    Value::String(s) => *v1 = Value::new_str(s.to_string() + v2.as_str()?),
                    _ => return Err(VMError::InvalidType {
                        expected: "int/float/string",
                        got: v1.type_to_str()
                    })
                }
            }
            code::SUB => {
                let v2 = stack_pop(&mut stack, ptr)?;
                let v1 = stack_top_mut(&mut stack, ptr)?;
                match v1 {
                    Value::Int(v1) => *v1 -= v2.as_int()?,
                    Value::Float(v1) => *v1 -= v2.as_float()?,
                    _ => return Err(VMError::InvalidType {
                        expected: "int/float",
                        got: v1.type_to_str()
                    })
                }
            }
            code::MUL => {
                let v2 = stack_pop(&mut stack, ptr)?;
                let v1 = stack_top_mut(&mut stack, ptr)?;
                match v1 {
                    Value::Int(v1) => *v1 *= v2.as_int()?,
                    Value::Float(v1) => *v1 *= v2.as_float()?,
                    _ => return Err(VMError::InvalidType {
                        expected: "int/float",
                        got: v1.type_to_str()
                    })
                }
            }
            code::DIV => {
                let v2 = stack_pop(&mut stack, ptr)?;
                let v1 = stack_top_mut(&mut stack, ptr)?;
                match v1 {
                    Value::Int(v1) => *v1 /= v2.as_int()?,
                    Value::Float(v1) => *v1 /= v2.as_float()?,
                    _ => return Err(VMError::InvalidType {
                        expected: "int/float",
                        got: v1.type_to_str()
                    })
                }
            }
            code::MOD => {
                let v2 = stack_pop(&mut stack, ptr)?;
                let v1 = stack_top_mut(&mut stack, ptr)?;
                *v1.as_int_mut()? %= v2.as_int()?;
            }
            code::NEG => {
                let v = stack_top_mut(&mut stack, ptr)?;
                match v {
                    Value::Int(v) => *v = -*v,
                    Value::Float(v) => *v = -*v,
                    _ => return Err(VMError::InvalidType {
                        expected: "int/float",
                        got: v.type_to_str()
                    })
                }
            }
            code::CMP_EQ => {
                let v2 = stack_pop(&mut stack, ptr)?;
                let v1 = stack_pop(&mut stack, ptr)?;
                stack.push(Value::Bool(v1.cmp_eq(&v2)));
            }
            code::CMP_NE => {
                let v2 = stack_pop(&mut stack, ptr)?;
                let v1 = stack_pop(&mut stack, ptr)?;
                stack.push(Value::Bool(!v1.cmp_eq(&v2)));
            }
            code::CMP_GT => {
                let v2 = stack_pop(&mut stack, ptr)?;
                let v1 = stack_pop(&mut stack, ptr)?;
                stack.push(Value::Bool(v1.cmp_gt(&v2)?));
            }
            code::CMP_LT => {
                let v2 = stack_pop(&mut stack, ptr)?;
                let v1 = stack_pop(&mut stack, ptr)?;
                stack.push(Value::Bool(v1.cmp_lt(&v2)?));
            }
            code::CMP_GE => {
                let v2 = stack_pop(&mut stack, ptr)?;
                let v1 = stack_pop(&mut stack, ptr)?;
                stack.push(Value::Bool(!v1.cmp_lt(&v2)?));
            }
            code::CMP_LE => {
                let v2 = stack_pop(&mut stack, ptr)?;
                let v1 = stack_pop(&mut stack, ptr)?;
                stack.push(Value::Bool(!v1.cmp_gt(&v2)?));
            }
            code::NOT => {
                let v = stack_top_mut(&mut stack, ptr)?;
                match v {
                    Value::Bool(b) => *b = !*b,
                    _ => return Err(VMError::InvalidType {
                        expected: "bool",
                        got: v.type_to_str()
                    })
                }
            }
            code::BAND => {
                let v2 = stack_pop(&mut stack, ptr)?;
                let v1 = stack_top_mut(&mut stack, ptr)?;
                *v1.as_int_mut()? &= v2.as_int()?;
            }
            code::BOR => {
                let v2 = stack_pop(&mut stack, ptr)?;
                let v1 = stack_top_mut(&mut stack, ptr)?;
                *v1.as_int_mut()? |= v2.as_int()?;
            }
            code::BXOR => {
                let v2 = stack_pop(&mut stack, ptr)?;
                let v1 = stack_top_mut(&mut stack, ptr)?;
                *v1.as_int_mut()? ^= v2.as_int()?;
            }
            code::BINV => {
                let v = stack_top_mut(&mut stack, ptr)?;
                let i = v.as_int_mut()?;
                *i = !*i;
            }
            code::SHL => {
                let v2 = stack_pop(&mut stack, ptr)?;
                let v1 = stack_top_mut(&mut stack, ptr)?;
                *v1.as_int_mut()? <<= v2.as_int()?;
            }
            code::SHR => {
                let v2 = stack_pop(&mut stack, ptr)?;
                let v1 = stack_top_mut(&mut stack, ptr)?;
                *v1.as_int_mut()? >>= v2.as_int()?;
            }
            code::LEN => {
                let v = stack_pop(&mut stack, ptr)?;
                let len = match &v {
                    Value::String(s) => s.len(),
                    Value::Object(o) => o.borrow().get().len(),
                    Value::Array(a) => a.borrow().get().len(),
                    _ => return Err(VMError::InvalidType {
                        expected: "string/object/array",
                        got: v.type_to_str()
                    })
                };
                stack.push(Value::Int(len as i64));
            }
            code::IN => {
                let mut str = String::new();
                std::io::stdin().read_line(&mut str).expect("`stdin.read_line` failed");
                if str.ends_with("\n") {
                    str.truncate(str.len() - 1);
                }
                stack.push(Value::new_str(str));
            }
            code::OUT => println!("{}", stack_pop(&mut stack, ptr)?.to_string()),
            code::LOAD_LIB => {
                let str = next_str(&cur_func, &mut pc, program)?;
                stack.push(libs.get(str)
                    .ok_or_else(|| VMError::UnknownLibrary(str.to_owned()))?
                    .clone());
            }
            _ => return Err(VMError::UnknownInstruction(code))
        }
    }

    assert!(stack.len() == 1);

    Ok(())
}
