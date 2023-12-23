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

    fn as_int(&self) -> Result<i64, VMError> {
        if let Value::Int(i) = self {
            Ok(*i)
        } else {
            Err(VMError::InvalidType { expected: "int", got: self.type_to_str() })
        }
    }

    fn as_float(&self) -> Result<f64, VMError> {
        if let Value::Float(f) = self {
            Ok(*f)
        } else {
            Err(VMError::InvalidType { expected: "float", got: self.type_to_str() })
        }
    }

    fn as_bool(&self) -> Result<bool, VMError> {
        if let Value::Bool(b) = self {
            Ok(*b)
        } else {
            Err(VMError::InvalidType { expected: "bool", got: self.type_to_str() })
        }
    }

    fn as_str(&self) -> Result<&str, VMError> {
        if let Value::String(s) = self {
            Ok(s)
        } else {
            Err(VMError::InvalidType { expected: "string", got: self.type_to_str() })
        }
    }

    fn as_obj(&self) -> Result<&Gc<GcCell<HashMap<String, Value>>>, VMError> {
        if let Value::Object(o) = self {
            Ok(o)
        } else {
            Err(VMError::InvalidType { expected: "object", got: self.type_to_str() })
        }
    }

    fn as_closure(&self) -> Result<&Closure, VMError> {
        if let Value::Closure(c) = self {
            Ok(c)
        } else {
            Err(VMError::InvalidType { expected: "closure", got: self.type_to_str() })
        }
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
    if let Constant::String(s) = constant {
        Ok(s)
    } else {
        Err(VMError::ConstantNotString)
    }
}

fn get_constant(program: &ProgramBundle, idx: usize) -> Result<&Constant, VMError> {
    program.constant_pool.get(idx)
        .ok_or_else(|| VMError::ConstantIndexOutOfBound)
}

fn stack_top(stack: &Vec<Value>) -> Result<&Value, VMError> {
    stack.last()
        .ok_or_else(|| VMError::BadStack)
}

fn stack_top_mut(stack: &mut Vec<Value>) -> Result<&mut Value, VMError> {
    stack.last_mut()
        .ok_or_else(|| VMError::BadStack)
}

fn stack_pop(stack: &mut Vec<Value>) -> Result<Value, VMError> {
    stack.pop()
        .ok_or_else(|| VMError::BadStack)
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
                if let Some(v) = this(&info).borrow().get(str) {
                    stack.push(v.clone());
                } else {
                    stack.push(Value::Null);
                }
            }
            code::LOAD_SUPER => {
                let str = next_str(&cur_func, &mut pc, program)?;
                if let Some(v) = parent(&info)?.borrow().get(str) {
                    stack.push(v.clone())
                } else {
                    stack.push(Value::Null)
                }
            }
            code::STORE => {
                let str = next_str(&cur_func, &mut pc, program)?;
                if let Value::Null = stack_top(&stack)? {
                    this(&info).borrow_mut().remove(str);
                } else {
                    this(&info).borrow_mut().insert(str.to_owned(), stack_top(&stack)?.clone());
                }
            }
            code::STORE_SUPER => {
                let str = next_str(&cur_func, &mut pc, program)?;
                if let Value::Null = stack_top(&stack)? {
                    parent(&info)?.borrow_mut().remove(str);
                } else {
                    parent(&info)?.borrow_mut().insert(str.to_owned(), stack_top(&stack)?.clone());
                }
            }
            code::DUP => stack.push(stack_top(&stack)?.clone()),
            code::POP => {
                stack_pop(&mut stack)?;
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
            code::PUSH_ARG => {
                let arg_idx: usize = next(&cur_func, &mut pc)?.into();
                let arg_cnt = cur_info(&info).arg_cnt;
                if arg_idx < arg_cnt {
                    stack.push(stack.get(ptr - arg_cnt + arg_idx)
                        .ok_or_else(|| VMError::BadStack)?
                        .clone());
                } else {
                    stack.push(Value::Null);
                }
            }
            code::PUSH_SELF => stack.push(cur_info(&info).variables.this.clone()),
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
                let closure = stack.get(stack.len() - 1 - arg_cnt)
                    .ok_or_else(|| VMError::BadStack)?
                    .as_closure()?;
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
                let value = stack_pop(&mut stack)?;
                let cur_info = cur_info(&info);
                stack.resize(stack.len() - cur_info.arg_cnt - 1, Value::Null);
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
                let v2 = stack_pop(&mut stack)?;
                let v1 = stack_top_mut(&mut stack)?;
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
                let v2 = stack_pop(&mut stack)?;
                let v1 = stack_top_mut(&mut stack)?;
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
                let v2 = stack_pop(&mut stack)?;
                let v1 = stack_top_mut(&mut stack)?;
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
                let v2 = stack_pop(&mut stack)?;
                let v1 = stack_top_mut(&mut stack)?;
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
                let v = stack_top_mut(&mut stack)?;
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
                let v2 = stack_pop(&mut stack)?;
                let v1 = stack_pop(&mut stack)?;
                stack.push(Value::Bool(match &v1 {
                    Value::Int(v1) => *v1 > v2.as_int()?,
                    Value::Float(v1) => *v1 > v2.as_float()?,
                    Value::String(s) => s.as_str() > v2.as_str()?,
                    _ => return Err(VMError::InvalidType {
                        expected: "int/float/string",
                        got: v1.type_to_str()
                    })
                }));
            }
            code::CMP_LT => {
                let v2 = stack_pop(&mut stack)?;
                let v1 = stack_pop(&mut stack)?;
                stack.push(Value::Bool(match &v1 {
                    Value::Int(v1) => *v1 < v2.as_int()?,
                    Value::Float(v1) => *v1 < v2.as_float()?,
                    Value::String(s) => s.as_str() < v2.as_str()?,
                    _ => return Err(VMError::InvalidType {
                        expected: "int/float/string",
                        got: v1.type_to_str()
                    })
                }));
            }
            code::IN => {
                let mut str = String::new();
                std::io::stdin().read_line(&mut str).expect("`stdin.read_line` failed");
                stack.push(Value::new_str(str));
            }
            code::OUT => println!("{:?}", stack_pop(&mut stack)?),
            _ => return Err(VMError::UnknownInstruction(code))
        }
    }

    Ok(())
}
