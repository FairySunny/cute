use std::{collections::HashMap, rc::Rc, ops::RangeBounds, io, path::Path};
use gc::{Trace, Finalize, Gc, GcCell, GcCellRef, GcCellRefMut};
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
    InvalidType { expected: String, got: String },
    ArrayIndexOutOfBound,
    SuperDoesNotExist,
    ObjectLocked,
    IllegalFunctionArguments,
    IllegalState,
    IOError(io::Error),
    Exit(i64)
}

impl VMError {
    pub fn invalid_type(expected: &str, got: &str) -> Self {
        Self::InvalidType {
            expected: expected.to_owned(),
            got: got.to_owned()
        }
    }
}

impl From<io::Error> for VMError {
    fn from(e: io::Error) -> Self {
        Self::IOError(e)
    }
}

#[derive(Trace, Finalize)]
pub struct Lockable<T: Trace + Finalize + 'static> {
    data: GcCell<T>,
    locked: bool
}

impl<T: Trace + Finalize + 'static> Lockable<T> {
    pub fn new(data: T, locked: bool) -> Self {
        Self { data: GcCell::new(data), locked }
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }

    pub fn get(&self) -> GcCellRef<T> {
        self.data.borrow()
    }

    pub fn get_mut(&self) -> Result<GcCellRefMut<T>, VMError> {
        if self.locked {
            Err(VMError::ObjectLocked)
        } else {
            Ok(self.data.borrow_mut())
        }
    }
}

#[derive(Trace, Finalize)]
pub struct Variables {
    parent: Option<Gc<Variables>>,
    this: Value
}

impl Variables {
    pub fn new(parent: Option<&Gc<Variables>>) -> Self {
        Self {
            parent: parent.map(|p| p.clone()),
            this: Value::new_obj(HashMap::new())
        }
    }

    pub fn new_gc(parent: Option<&Gc<Variables>>) -> Gc<Self> {
        Gc::new(Self::new(parent))
    }

    pub fn this(&self) -> &Value {
        &self.this
    }

    pub fn this_obj(&self) -> &Gc<Lockable<HashMap<Rc<str>, Value>>> {
        self.this.as_obj().expect("`Variables.this` is not object")
    }

    pub fn parent(&self) -> Result<&Gc<Variables>, VMError> {
        self.parent.as_ref().ok_or_else(|| VMError::SuperDoesNotExist)
    }

    pub fn parent_obj(&self) -> Result<&Gc<Lockable<HashMap<Rc<str>, Value>>>, VMError> {
        Ok(self.parent()?.this_obj())
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
    Object(Gc<Lockable<HashMap<Rc<str>, Value>>>),
    Array(Gc<Lockable<Vec<Value>>>),
    Closure(Closure),
    NativeFunction(
        #[unsafe_ignore_trace]
        fn(&mut Context, &ProgramState, Vec<Value>) -> Result<Value, VMError>
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

    pub fn new_obj(o: HashMap<Rc<str>, Value>) -> Self {
        Self::Object(Gc::new(Lockable::new(o, false)))
    }

    pub fn new_locked_obj(o: HashMap<Rc<str>, Value>) -> Self {
        Self::Object(Gc::new(Lockable::new(o, true)))
    }

    pub fn new_arr(a: Vec<Value>) -> Self {
        Self::Array(Gc::new(Lockable::new(a, false)))
    }

    pub fn new_locked_arr(a: Vec<Value>) -> Self {
        Self::Array(Gc::new(Lockable::new(a, true)))
    }

    pub fn as_int(&self) -> Result<i64, VMError> {
        match self {
            Value::Int(i) => Ok(*i),
            _ => Err(VMError::invalid_type("int", self.type_to_str()))
        }
    }

    pub fn as_int_mut(&mut self) -> Result<&mut i64, VMError> {
        match self {
            Value::Int(i) => Ok(i),
            _ => Err(VMError::invalid_type("int", self.type_to_str()))
        }
    }

    pub fn as_idx(&self) -> Result<usize, VMError> {
        self.as_int()?.try_into()
            .map_err(|_| VMError::ArrayIndexOutOfBound)
    }

    pub fn as_float(&self) -> Result<f64, VMError> {
        match self {
            Value::Float(f) => Ok(*f),
            _ => Err(VMError::invalid_type("float", self.type_to_str()))
        }
    }

    pub fn as_bool(&self) -> Result<bool, VMError> {
        match self {
            Value::Bool(b) => Ok(*b),
            _ => Err(VMError::invalid_type("bool", self.type_to_str()))
        }
    }

    pub fn as_str(&self) -> Result<&Rc<str>, VMError> {
        match self {
            Value::String(s) => Ok(s),
            _ => Err(VMError::invalid_type("string", self.type_to_str()))
        }
    }

    pub fn as_obj(&self) -> Result<&Gc<Lockable<HashMap<Rc<str>, Value>>>, VMError> {
        match self {
            Value::Object(o) => Ok(o),
            _ => Err(VMError::invalid_type("object", self.type_to_str()))
        }
    }

    pub fn as_arr(&self) -> Result<&Gc<Lockable<Vec<Value>>>, VMError> {
        match self {
            Value::Array(a) => Ok(a),
            _ => Err(VMError::invalid_type("array", self.type_to_str()))
        }
    }

    pub fn as_closure(&self) -> Result<&Closure, VMError> {
        match self {
            Value::Closure(c) => Ok(c),
            _ => Err(VMError::invalid_type("closure", self.type_to_str()))
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
            _ => return Err(VMError::invalid_type("int/float/string", self.type_to_str()))
        })
    }

    pub fn cmp_lt(&self, other: &Value) -> Result<bool, VMError> {
        Ok(match self {
            Value::Int(i) => *i < other.as_int()?,
            Value::Float(f) => *f < other.as_float()?,
            Value::String(s) => s < other.as_str()?,
            _ => return Err(VMError::invalid_type("int/float/string", self.type_to_str()))
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

    pub fn check_arg_cnt(args: &Vec<Value>, cnt: usize) -> Result<(), VMError> {
        if cnt == args.len() {
            Ok(())
        } else {
            Err(VMError::IllegalFunctionArguments)
        }
    }

    pub fn check_arg_range(args: &Vec<Value>, range: impl RangeBounds<usize>) -> Result<(), VMError> {
        if range.contains(&args.len()) {
            Ok(())
        } else {
            Err(VMError::IllegalFunctionArguments)
        }
    }
}

pub struct Context {
    programs: Vec<(ProgramBundle, Option<Rc<Path>>)>,
    libs: HashMap<Rc<str>, Value>,
    file_libs: HashMap<Rc<Path>, Value>
}

impl Context {
    pub fn new(program: ProgramBundle, path: Option<Rc<Path>>) -> Self {
        let mut ctx = Self {
            programs: vec![(program, path)],
            libs: HashMap::new(),
            file_libs: HashMap::new()
        };

        libraries::misc::load_libs(&mut ctx);
        libraries::types::load_libs(&mut ctx);
        libraries::arrays::load_libs(&mut ctx);
        libraries::sys::load_libs(&mut ctx);

        ctx
    }

    pub fn add_program(&mut self, program: ProgramBundle, path: Option<Rc<Path>>) -> usize {
        eprintln!("Program: {:?}", path);
        let idx = self.programs.len();
        self.programs.push((program, path));
        idx
    }

    pub fn get_program(&self, idx: usize) -> &ProgramBundle {
        &self.programs[idx].0
    }

    pub fn get_program_path(&self, idx: usize) -> Option<&Path> {
        self.programs[idx].1.as_ref().map(|p| p.as_ref())
    }

    pub fn get_program_dir(&self, idx: usize) -> Option<&Path> {
        self.get_program_path(idx).and_then(|p| p.parent())
    }

    pub fn add_lib(&mut self, name: Rc<str>, lib: Value) {
        self.libs.insert(name, lib);
    }

    pub fn get_lib(&self, name: &str) -> Option<&Value> {
        self.libs.get(name)
    }

    pub fn add_file_lib(&mut self, path: Rc<Path>, lib: Value) {
        eprintln!("Import: {:?}", path);
        self.file_libs.insert(path, lib);
    }

    pub fn get_file_lib(&self, path: &Path) -> Option<&Value> {
        self.file_libs.get(path)
    }
}

pub struct ProgramState {
    pub program_idx: usize,
    pub func_idx: usize,
    pub variables: Gc<Variables>
}
