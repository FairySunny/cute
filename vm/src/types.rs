use std::{collections::HashMap, rc::Rc, io, path::{Path, PathBuf}};
use gc::{Trace, Finalize, Gc, GcCell};
use bytecode::program::ProgramBundle;
use crate::libraries;

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
    SuperDoesNotExist,
    UnknownLibrary(Rc<str>),
    ObjectLocked,
    IllegalFunctionArguments,
    IOError(io::Error)
}

impl From<io::Error> for VMError {
    fn from(e: io::Error) -> Self {
        Self::IOError(e)
    }
}

#[derive(Trace, Finalize)]
pub struct Lockable<T> {
    data: T,
    locked: bool
}

impl<T> Lockable<T> {
    pub fn new(data: T, locked: bool) -> Self {
        Self { data, locked }
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }

    pub fn lock(&mut self) {
        self.locked = true;
    }

    pub fn unlock(&mut self) {
        self.locked = false;
    }

    pub fn get(&self) -> &T {
        &self.data
    }

    pub fn get_mut(&mut self) -> Result<&mut T, VMError> {
        if self.locked {
            Err(VMError::ObjectLocked)
        } else {
            Ok(&mut self.data)
        }
    }
}

#[derive(Trace, Finalize)]
pub struct Variables {
    pub parent: Option<Gc<Variables>>,
    pub this: Value
}

impl Variables {
    pub fn new(parent: Option<&Gc<Variables>>) -> Self {
        Self {
            parent: parent.map(|p| p.clone()),
            this: Value::new_obj()
        }
    }

    pub fn this_obj(&self) -> &Gc<GcCell<Lockable<HashMap<Rc<str>, Value>>>> {
        self.this.as_obj().expect("`Variables.this` is not object")
    }
}

#[derive(Clone, Trace, Finalize)]
pub struct Closure {
    pub parent: Gc<Variables>,
    pub program_idx: usize,
    pub func_idx: usize
}

#[derive(Clone, Trace, Finalize)]
pub enum Value {
    Null,
    Int(i64),
    Float(f64),
    Bool(bool),
    String(Rc<str>),
    Object(Gc<GcCell<Lockable<HashMap<Rc<str>, Value>>>>),
    Array(Gc<GcCell<Lockable<Vec<Value>>>>),
    Closure(Closure),
    NativeFunction(
        #[unsafe_ignore_trace]
        fn(&mut Context, Vec<Value>) -> Result<Value, VMError>
    )
}

impl Value {
    pub fn type_to_str(&self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::Int(_) => "int",
            Self::Float(_) => "float",
            Self::Bool(_) => "bool",
            Self::String(_) => "string",
            Self::Object(_) => "object",
            Self::Array(_) => "array",
            Self::Closure(_) => "closure",
            Self::NativeFunction(_) => "native function"
        }
    }

    pub fn new_obj() -> Self {
        Self::Object(Gc::new(GcCell::new(Lockable::new(HashMap::new(), false))))
    }

    pub fn new_lib_obj(create: impl FnOnce(&mut HashMap<Rc<str>, Value>)) -> Self {
        let lib = Value::new_obj();
        let mut lib_obj = lib.as_obj().unwrap().borrow_mut();
        create(lib_obj.get_mut().unwrap());
        lib_obj.lock();
        drop(lib_obj);
        lib
    }

    pub fn new_arr(a: Vec<Value>) -> Self {
        Self::Array(Gc::new(GcCell::new(Lockable::new(a, false))))
    }

    pub fn as_int(&self) -> Result<i64, VMError> {
        match self {
            Value::Int(i) => Ok(*i),
            _ => Err(VMError::InvalidType { expected: "int", got: self.type_to_str() })
        }
    }

    pub fn as_int_mut(&mut self) -> Result<&mut i64, VMError> {
        match self {
            Value::Int(i) => Ok(i),
            _ => Err(VMError::InvalidType { expected: "int", got: self.type_to_str() })
        }
    }

    pub fn as_idx(&self) -> Result<usize, VMError> {
        self.as_int()?.try_into()
            .map_err(|_| VMError::ArrayIndexOutOfBound)
    }

    pub fn as_float(&self) -> Result<f64, VMError> {
        match self {
            Value::Float(f) => Ok(*f),
            _ => Err(VMError::InvalidType { expected: "float", got: self.type_to_str() })
        }
    }

    pub fn as_bool(&self) -> Result<bool, VMError> {
        match self {
            Value::Bool(b) => Ok(*b),
            _ => Err(VMError::InvalidType { expected: "bool", got: self.type_to_str() })
        }
    }

    pub fn as_str(&self) -> Result<&Rc<str>, VMError> {
        match self {
            Value::String(s) => Ok(s),
            _ => Err(VMError::InvalidType { expected: "string", got: self.type_to_str() })
        }
    }

    pub fn as_obj(&self) -> Result<&Gc<GcCell<Lockable<HashMap<Rc<str>, Value>>>>, VMError> {
        match self {
            Value::Object(o) => Ok(o),
            _ => Err(VMError::InvalidType { expected: "object", got: self.type_to_str() })
        }
    }

    pub fn as_arr(&self) -> Result<&Gc<GcCell<Lockable<Vec<Value>>>>, VMError> {
        match self {
            Value::Array(a) => Ok(a),
            _ => Err(VMError::InvalidType { expected: "array", got: self.type_to_str() })
        }
    }

    pub fn as_closure(&self) -> Result<&Closure, VMError> {
        match self {
            Value::Closure(c) => Ok(c),
            _ => Err(VMError::InvalidType { expected: "closure", got: self.type_to_str() })
        }
    }

    pub fn cmp_eq(&self, other: &Value) -> bool {
        match self {
            Value::Null => match other {
                Value::Null => true,
                _ => false
            }
            Value::Int(v) => match other {
                Value::Int(v2) => v == v2,
                _ => false
            }
            Value::Float(v) => match other {
                Value::Float(v2) => v == v2,
                _ => false
            }
            Value::Bool(v) => match other {
                Value::Bool(v2) => v == v2,
                _ => false
            }
            Value::String(v) => match other {
                Value::String(v2) => v == v2,
                _ => false
            }
            Value::Object(v) => match other {
                Value::Object(v2) => Gc::ptr_eq(v, v2),
                _ => false
            }
            Value::Array(v) => match other {
                Value::Array(v2) => Gc::ptr_eq(v, v2),
                _ => false
            }
            Value::Closure(v) => match other {
                Value::Closure(v2) =>
                    v.func_idx == v2.func_idx && Gc::ptr_eq(&v.parent, &v2.parent),
                _ => false
            }
            Value::NativeFunction(v) => match other {
                Value::NativeFunction(v2) => v == v2,
                _ => false
            }
        }
    }

    pub fn cmp_gt(&self, other: &Value) -> Result<bool, VMError> {
        Ok(match self {
            Value::Int(i) => *i > other.as_int()?,
            Value::Float(f) => *f > other.as_float()?,
            Value::String(s) => s > other.as_str()?,
            _ => return Err(VMError::InvalidType {
                expected: "int/float/string",
                got: self.type_to_str()
            })
        })
    }

    pub fn cmp_lt(&self, other: &Value) -> Result<bool, VMError> {
        Ok(match self {
            Value::Int(i) => *i < other.as_int()?,
            Value::Float(f) => *f < other.as_float()?,
            Value::String(s) => s < other.as_str()?,
            _ => return Err(VMError::InvalidType {
                expected: "int/float/string",
                got: self.type_to_str()
            })
        })
    }

    pub fn to_string(&self) -> String {
        match self {
            Value::Null => "null".to_owned(),
            Value::Int(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::String(s) => s.to_string(),
            Value::Object(_) => "[object]".to_owned(),
            Value::Array(_) => "[array]".to_owned(),
            Value::Closure(_) => "[closure]".to_owned(),
            Value::NativeFunction(_) => "[native function]".to_owned()
        }
    }
}

pub struct Context {
    programs: Vec<ProgramBundle>,
    libs: HashMap<Rc<str>, Value>,
    paths: Vec<String>
}

impl Context {
    pub fn new(program: ProgramBundle, paths: Vec<String>) -> Self {
        let mut ctx = Self {
            programs: vec![program],
            libs: HashMap::new(),
            paths
        };

        libraries::misc::load_libs(&mut ctx);
        libraries::types::load_libs(&mut ctx);
        libraries::arrays::load_libs(&mut ctx);

        ctx
    }

    pub fn add_program(&mut self, program: ProgramBundle) -> usize {
        let idx = self.programs.len();
        self.programs.push(program);
        idx
    }

    pub fn get_program(&self, idx: usize) -> &ProgramBundle {
        &self.programs[idx]
    }

    pub fn add_lib(&mut self, name: Rc<str>, lib: Value) {
        self.libs.insert(name, lib);
    }

    pub fn get_lib(&self, name: &str) -> Option<&Value> {
        self.libs.get(name)
    }

    pub fn find_path(&self, name: &str) -> Option<PathBuf> {
        self.paths.iter().find_map(|p| {
            let file_path = Path::new(p).join(name.to_owned() + ".cute");
            if file_path.is_file() {
                Some(file_path)
            } else {
                None
            }
        })
    }
}
