use std::collections::HashMap;
use gc::{Trace, Finalize, Gc, GcCell};
use bytecode::{program::{ProgramBundle, Constant}, code};

#[derive(Debug)]
pub enum VMError {
    FunctionIndexOutOfBound,
    PCIndexOutOfBound,
    UnknownInstruction(u8),
    ConstantIndexOutOfBound,
    ConstantNotString,
    BadStack,
    InvalidType { expected: &'static str, got: &'static str },
    ArrayIndexOutOfBound,
    SuperDoesNotExist
}

#[derive(Debug, Trace, Finalize)]
struct Variables {
    parent: Option<Gc<Variables>>,
    this: Value
}

impl Variables {
    fn new(parent: Option<&Gc<Variables>>) -> Self {
        Self {
            parent: parent.map(|p| p.clone()),
            this: Value::new_obj()
        }
    }

    fn this_obj(&self) -> &Gc<GcCell<HashMap<String, Value>>> {
        self.this.as_obj().expect("`Variables.this` is not object")
    }
}

#[derive(Debug, Clone, Trace, Finalize)]
struct Closure {
    parent: Gc<Variables>,
    func_idx: usize
}

#[derive(Debug, Clone, Trace, Finalize)]
enum Value {
    Null,
    Int(i64),
    Float(f64),
    Bool(bool),
    String(Gc<String>),
    Object(Gc<GcCell<HashMap<String, Value>>>),
    Array(Gc<GcCell<Vec<Value>>>),
    Closure(Closure)
}

impl Value {
    fn type_to_str(&self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::Int(_) => "int",
            Self::Float(_) => "float",
            Self::Bool(_) => "bool",
            Self::String(_) => "string",
            Self::Object(_) => "object",
            Self::Array(_) => "array",
            Self::Closure(_) => "closure"
        }
    }

    fn new_str(s: impl Into<String>) -> Self {
        Self::String(Gc::new(s.into()))
    }

    fn new_obj() -> Self {
        Self::Object(Gc::new(GcCell::new(HashMap::new())))
    }

    fn new_arr(a: Vec<Value>) -> Self {
        Self::Array(Gc::new(GcCell::new(a)))
    }

    fn as_int(&self) -> Result<i64, VMError> {
        match self {
            Value::Int(i) => Ok(*i),
            _ => Err(VMError::InvalidType { expected: "int", got: self.type_to_str() })
        }
    }

    fn as_float(&self) -> Result<f64, VMError> {
        match self {
            Value::Float(f) => Ok(*f),
            _ => Err(VMError::InvalidType { expected: "float", got: self.type_to_str() })
        }
    }

    fn as_bool(&self) -> Result<bool, VMError> {
        match self {
            Value::Bool(b) => Ok(*b),
            _ => Err(VMError::InvalidType { expected: "bool", got: self.type_to_str() })
        }
    }

    fn as_str(&self) -> Result<&str, VMError> {
        match self {
            Value::String(s) => Ok(s),
            _ => Err(VMError::InvalidType { expected: "string", got: self.type_to_str() })
        }
    }

    fn as_obj(&self) -> Result<&Gc<GcCell<HashMap<String, Value>>>, VMError> {
        match self {
            Value::Object(o) => Ok(o),
            _ => Err(VMError::InvalidType { expected: "object", got: self.type_to_str() })
        }
    }

    fn as_closure(&self) -> Result<&Closure, VMError> {
        match self {
            Value::Closure(c) => Ok(c),
            _ => Err(VMError::InvalidType { expected: "closure", got: self.type_to_str() })
        }
    }

    fn cmp_gt(&self, other: &Value) -> Result<bool, VMError> {
        Ok(match self {
            Value::Int(i) => *i > other.as_int()?,
            Value::Float(f) => *f > other.as_float()?,
            Value::String(s) => s.as_str() > other.as_str()?,
            _ => return Err(VMError::InvalidType {
                expected: "int/float/string",
                got: self.type_to_str()
            })
        })
    }

    fn cmp_lt(&self, other: &Value) -> Result<bool, VMError> {
        Ok(match self {
            Value::Int(i) => *i < other.as_int()?,
            Value::Float(f) => *f < other.as_float()?,
            Value::String(s) => s.as_str() < other.as_str()?,
            _ => return Err(VMError::InvalidType {
                expected: "int/float/string",
                got: self.type_to_str()
            })
        })
    }
}

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

fn this(info: &Vec<StackInfo>) -> &Gc<GcCell<HashMap<String, Value>>> {
    cur_info(info).variables.this_obj()
}

fn parent(info: &Vec<StackInfo>) -> Result<&Gc<GcCell<HashMap<String, Value>>>, VMError> {
    Ok(cur_info(info).variables.parent.as_ref()
        .ok_or_else(|| VMError::SuperDoesNotExist)?
        .this_obj())
}

pub fn run_program(program: &ProgramBundle) -> Result<i32, VMError> {
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
                match this(&info).borrow().get(str) {
                    Some(v) => stack.push(v.clone()),
                    None => stack.push(Value::Null)
                }
            }
            code::LOAD_SUPER => {
                let str = next_str(&cur_func, &mut pc, program)?;
                match parent(&info)?.borrow().get(str) {
                    Some(v) => stack.push(v.clone()),
                    None => stack.push(Value::Null)
                }
            }
            code::LOAD_FIELD => {
                let str = next_str(&cur_func, &mut pc, program)?;
                let obj = stack_pop(&mut stack, ptr)?;
                match obj.as_obj()?.borrow().get(str) {
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
                        match o.borrow().get(idx) {
                            Some(v) => stack.push(v.clone()),
                            None => stack.push(Value::Null)
                        }
                    }
                    Value::Array(a) => {
                        let idx: usize = idx.as_int()?.try_into()
                            .map_err(|_| VMError::ArrayIndexOutOfBound)?;
                        stack.push(a.borrow().get(idx)
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
                match stack_top(&stack, ptr)? {
                    Value::Null => this(&info).borrow_mut().remove(str),
                    _ => this(&info).borrow_mut()
                        .insert(str.to_owned(), stack_top(&stack, ptr)?.clone())
                };
            }
            code::STORE_SUPER => {
                let str = next_str(&cur_func, &mut pc, program)?;
                match stack_top(&stack, ptr)? {
                    Value::Null => parent(&info)?.borrow_mut().remove(str),
                    _ => parent(&info)?.borrow_mut()
                        .insert(str.to_owned(), stack_top(&stack, ptr)?.clone())
                };
            }
            code::STORE_FIELD => {
                let str = next_str(&cur_func, &mut pc, program)?;
                let value = stack_pop(&mut stack, ptr)?;
                let obj = stack_pop(&mut stack, ptr)?;
                match &value {
                    Value::Null => obj.as_obj()?.borrow_mut().remove(str),
                    _ => obj.as_obj()?.borrow_mut().insert(str.to_owned(), value.clone())
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
                            Value::Null => o.borrow_mut().remove(idx),
                            _ => o.borrow_mut().insert(idx.to_owned(), value.clone())
                        };
                    }
                    Value::Array(a) => {
                        let idx: usize = idx.as_int()?.try_into()
                            .map_err(|_| VMError::ArrayIndexOutOfBound)?;
                        *a.borrow_mut().get_mut(idx)
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
                if let Value::Null = stack_top(&mut stack, ptr)? {
                    stack_pop(&mut stack, ptr)?;
                    pc = (pc as i64 - 1 + offset as i64) as usize;
                }
            }
            code::JT => {
                let offset = next(&cur_func, &mut pc)? as i8;
                if stack_top(&mut stack, ptr)?.as_bool()? {
                    stack_pop(&mut stack, ptr)?;
                    pc = (pc as i64 - 1 + offset as i64) as usize;
                }
            }
            code::JF => {
                let offset = next(&cur_func, &mut pc)? as i8;
                if !stack_top(&mut stack, ptr)?.as_bool()? {
                    stack_pop(&mut stack, ptr)?;
                    pc = (pc as i64 - 1 + offset as i64) as usize;
                }
            }
            code::CALL => {
                let arg_cnt: usize = next(&cur_func, &mut pc)?.into();
                if stack.len() < ptr + 1 + arg_cnt {
                    return Err(VMError::BadStack);
                }
                let closure = stack[stack.len() - arg_cnt - 1].as_closure()?;
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
            code::RETURN => {
                let value = stack_pop(&mut stack, ptr)?;
                let cur_info = cur_info(&info);
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
            code::IN => {
                let mut str = String::new();
                std::io::stdin().read_line(&mut str).expect("`stdin.read_line` failed");
                stack.push(Value::new_str(str));
            }
            code::OUT => {
                let value = stack_pop(&mut stack, ptr)?;
                match &value {
                    Value::Null => println!("null"),
                    Value::Int(i) => println!("{i}"),
                    Value::Float(f) => println!("{f}"),
                    Value::Bool(b) => println!("{b}"),
                    Value::String(s) => println!("{s}"),
                    _ => println!("{:?}", value)
                }
            },
            _ => return Err(VMError::UnknownInstruction(code))
        }
    }

    assert!(stack.len() == 1);

    match stack[0] {
        Value::Int(code) => Ok(code as i32),
        _ => Ok(0)
    }
}
