use std::collections::HashMap;
use gc::{Trace, Finalize, Gc, GcCell};
use bytecode::{program::ProgramBundle, code};

#[derive(Trace, Finalize)]
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
}

#[derive(Clone, Trace, Finalize)]
struct Closure {
    parent: Gc<Variables>,
    func_idx: usize
}

#[derive(Clone, Trace, Finalize)]
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
    fn new_str(s: String) -> Self {
        Self::String(Gc::new(s))
    }

    fn new_obj() -> Self {
        Self::Object(Gc::new(GcCell::new(HashMap::new())))
    }

    fn as_int(&self) -> i64 {
        if let Value::Int(i) = self {
            *i
        } else {
            panic!("Invalid type: Int expected")
        }
    }

    fn as_float(&self) -> f64 {
        if let Value::Float(f) = self {
            *f
        } else {
            panic!("Invalid type: Float expected")
        }
    }

    fn as_str(&self) -> &str {
        if let Value::String(s) = self {
            s
        } else {
            panic!("Invalid type: String expected")
        }
    }

    fn as_obj(&self) -> &Gc<GcCell<HashMap<String, Value>>> {
        if let Value::Object(o) = self {
            o
        } else {
            panic!("Invalid type: Object expected")
        }
    }

    fn as_closure(&self) -> &Closure {
        if let Value::Closure(c) = self {
            c
        } else {
            panic!("Invalid type: Closure expected")
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

fn next(func: &Vec<u8>, pc: &mut usize) -> u8 {
    let code = func[*pc];
    *pc += 1;
    code
}

fn cur_info(info: & Vec<StackInfo>) -> &StackInfo {
    info.last().unwrap()
}

fn this(info: &Vec<StackInfo>) -> &Gc<GcCell<HashMap<String, Value>>> {
    cur_info(info).variables.this.as_obj()
}

pub fn run_program(program: &ProgramBundle) {
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
    let mut cur_func = Box::new(&program.func_list[func_idx]);

    let mut pc = 0usize;
    let mut ptr = 1usize;

    loop {
        let code = next(&cur_func, &mut pc);

        match code {
            code::LOAD => {
                let str = program.get_string(next(&cur_func, &mut pc)).unwrap();
                if let Some(v) = this(&info).borrow().get(str) {
                    stack.push(v.clone());
                } else {
                    stack.push(Value::Null);
                }
            }
            code::STORE => {
                let str = program.get_string(next(&cur_func, &mut pc)).unwrap();
                if let Value::Null = stack.last().unwrap() {
                    this(&info).borrow_mut().remove(str);
                } else {
                    this(&info).borrow_mut().insert(str.clone(), stack.last().unwrap().clone());
                }
            }
            code::DUP => stack.push(stack.last().unwrap().clone()),
            code::POP => {
                stack.pop().unwrap();
            }
            code::PUSH_ARG => {
                let arg_idx: usize = next(&cur_func, &mut pc).into();
                let arg_cnt = cur_info(&info).arg_cnt;
                if arg_idx < arg_cnt {
                    stack.push(stack[ptr - arg_cnt + arg_idx].clone());
                } else {
                    stack.push(Value::Null);
                }
            }
            code::PUSH_SELF => stack.push(cur_info(&info).variables.this.clone()),
            code::PUSH_CLOSURE => {
                let idx: usize = next(&cur_func, &mut pc).into();
                stack.push(Value::Closure(Closure {
                    parent: cur_info(&info).variables.clone(),
                    func_idx: idx
                }));
            }
            code::CALL => {
                let arg_cnt: usize = next(&cur_func, &mut pc).into();
                let closure = stack[stack.len() - 1 - arg_cnt].as_closure();
                info.push(StackInfo {
                    variables: Gc::new(Variables::new(Some(&closure.parent))),
                    arg_cnt,
                    func_idx_return: func_idx,
                    ptr_return: ptr,
                    pc_return: pc
                });
                func_idx = closure.func_idx;
                cur_func = Box::new(&program.func_list[func_idx]);
                pc = 0;
                ptr = stack.len();
            }
            code::RETURN => {
                let value = stack.pop().unwrap();
                let cur_info = cur_info(&info);
                stack.resize(stack.len() - cur_info.arg_cnt - 2, Value::Null);
                stack.push(value);
                if info.len() <= 1 {
                    break;
                }
                func_idx = cur_info.func_idx_return;
                cur_func = Box::new(&program.func_list[func_idx]);
                pc = cur_info.pc_return;
                ptr = cur_info.ptr_return;
                info.pop();
            }
            code::ADD => {
                let v2 = stack.pop().unwrap();
                let v1 = stack.last_mut().unwrap();
                match v1 {
                    Value::Int(v1) => *v1 += v2.as_int(),
                    Value::Float(v1) => *v1 += v2.as_float(),
                    Value::String(_) => *v1 = Value::new_str(String::new() + v1.as_str() + v2.as_str()),
                    _ => panic!("Invalid type: (+) Int/Float/String expected")
                }
            }
            code::IN => {
                let mut str = String::new();
                std::io::stdin().read_line(&mut str).unwrap();
                stack.push(Value::new_str(str));
            }
            code::OUT => println!("{}", stack.pop().unwrap().as_str()),
            _ => panic!("Unknown instruction {:#x}", code)
        }
    }
}
