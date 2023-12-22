use std::{rc::Rc, collections::HashMap};
use crate::code;

#[derive(Debug)]
pub enum GeneratingError {
    ConstantPoolExceeding,
    ClosureListExceeding,
    ArgumentListExceeding,
    JumpingTooFar
}

enum Constant {
    Int(i64),
    Float(f64),
    String(Rc<String>)
}

struct ConstantPool {
    constant_list: Vec<Constant>,
    str_map: HashMap<String, usize>
}

impl ConstantPool {
    fn new() -> Self {
        Self {
            constant_list: vec![],
            str_map: HashMap::new()
        }
    }

    fn int(&mut self, i: i64) -> usize {
        let idx = self.constant_list.len();
        self.constant_list.push(Constant::Int(i));
        idx
    }

    fn float(&mut self, f: f64) -> usize {
        let idx = self.constant_list.len();
        self.constant_list.push(Constant::Float(f));
        idx
    }

    fn str(&mut self, s: &str) -> usize {
        // string exists
        if let Some(&v) = self.str_map.get(s) {
            return v
        }
        // string does not exist, create
        let idx = self.constant_list.len();
        self.constant_list.push(Constant::String(Rc::new(s.to_owned())));
        self.str_map.insert(s.to_owned(), idx);
        idx
    }
}

struct Func {
    code: Vec<u8>,
    arg_idx: u32
}

impl Func {
    fn new() -> Self {
        Self {
            code: vec![],
            arg_idx: 0
        }
    }

    fn next_arg_idx(&mut self) -> Result<u8, GeneratingError> {
        let idx = self.arg_idx.try_into()
            .map_err(|_| GeneratingError::ArgumentListExceeding)?;
        self.arg_idx += 1;
        Ok(idx)
    }
}

pub struct Program {
    constant_pool: ConstantPool,
    func_list: Vec<Func>,
    idx: Vec<usize>
}

pub struct JumpWhere {
    pos: usize
}

impl Program {
    fn current_func(&self) -> &Func {
        &self.func_list[*self.idx.last().unwrap()]
    }

    fn current_func_mut(&mut self) -> &mut Func {
        &mut self.func_list[*self.idx.last().unwrap()]
    }

    pub fn new() -> Self {
        Self {
            constant_pool: ConstantPool::new(),
            func_list: vec![Func::new()],
            idx: vec![0]
        }
    }

    pub fn byte(&mut self, byte: u8) {
        self.current_func_mut().code.push(byte);
    }

    pub fn str(&mut self, s: &str) -> Result<(), GeneratingError> {
        let idx = self.constant_pool.str(s).try_into()
            .map_err(|_| GeneratingError::ConstantPoolExceeding)?;
        self.byte(idx);
        Ok(())
    }

    pub fn push_int(&mut self, i: i64) -> Result<(), GeneratingError> {
        match <i64 as TryInto<i8>>::try_into(i) {
            Ok(i) => {
                self.byte(code::PUSH_INT);
                self.byte(i as u8);
            }
            Err(..) => {
                let idx = self.constant_pool.int(i).try_into()
                    .map_err(|_| GeneratingError::ConstantPoolExceeding)?;
                self.byte(code::PUSH_CONST);
                self.byte(idx);
            }
        }
        Ok(())
    }

    pub fn push_float(&mut self, f: f64) -> Result<(), GeneratingError> {
        let idx = self.constant_pool.float(f).try_into()
            .map_err(|_| GeneratingError::ConstantPoolExceeding)?;
        self.byte(code::PUSH_CONST);
        self.byte(idx);
        Ok(())
    }

    pub fn push_str(&mut self, s: &str) -> Result<(), GeneratingError> {
        self.byte(code::PUSH_CONST);
        self.str(s)?;
        Ok(())
    }

    pub fn push_arg(&mut self) -> Result<(), GeneratingError> {
        let idx = self.current_func_mut().next_arg_idx()?;
        self.byte(code::PUSH_ARG);
        self.byte(idx);
        Ok(())
    }

    pub fn push_closure_and_switch(&mut self) -> Result<(), GeneratingError> {
        let idx = self.func_list.len().try_into()
            .map_err(|_| GeneratingError::ClosureListExceeding)?;
        self.byte(code::PUSH_CLOSURE);
        self.byte(idx);
        self.func_list.push(Func::new());
        self.idx.push(idx.into());
        Ok(())
    }

    pub fn switch_back(&mut self) {
        if self.idx.len() <= 1 {
            panic!("Illegal state: this is the last function");
        }
        self.idx.pop();
    }

    pub fn get_pos(&self) -> usize {
        self.current_func().code.len()
    }

    pub fn jump_back(&mut self, pos: usize) -> Result<(), GeneratingError> {
        let delta: i32 = (self.get_pos() - pos).try_into()
            .map_err(|_| GeneratingError::JumpingTooFar)?;
        let delta: i8 = (-delta).try_into()
            .map_err(|_| GeneratingError::JumpingTooFar)?;
        self.byte(delta as u8);
        Ok(())
    }

    pub fn jump_where(&mut self) -> JumpWhere {
        let pos = self.get_pos();
        self.byte(0);
        JumpWhere { pos }
    }

    pub fn jump_here(&mut self, jump: JumpWhere) -> Result<(), GeneratingError> {
        let delta: i8 = (self.get_pos() - jump.pos).try_into()
            .map_err(|_| GeneratingError::JumpingTooFar)?;
        self.current_func_mut().code[jump.pos] = delta as u8;
        Ok(())
    }

    pub fn print(&self) {
        println!("Constant Pool:");
        for constant in &self.constant_pool.constant_list {
            match constant {
                Constant::Int(v) => println!("  int: {v}"),
                Constant::Float(v) => println!("  float: {v}"),
                Constant::String(v) => println!("  string: {v}")
            }
        }
        println!();

        println!("Closures:");
        for func in &self.func_list {
            let mut idx = 0;
            while idx < func.code.len() {
                let info = &code::CODE_INFO[func.code[idx] as usize];
                idx += 1;
                print!("  {}", info.name);
                for _ in 0..info.params {
                    print!(" {:#x}", func.code[idx]);
                    idx += 1;
                }
                println!();
            }
            println!("  ----");
        }
    }
}
