use std::collections::HashMap;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io::{Read, Write};
use thiserror::Error;

// copying over bril_rs because we can't use it as a cargo dependency // has platform specific code
use bril_rs::conversion::{ConversionError, PositionalConversionError};
use bril_rs::{
    AbstractArgument, AbstractCode, AbstractFunction, AbstractInstruction, AbstractProgram,
    AbstractType,
};
use serde::{Deserialize, Serialize};

pub type Position = bril_rs::Position;
pub type Literal = bril_rs::Literal;
pub type ConstOps = bril_rs::ConstOps;

impl TryFrom<AbstractProgram> for Program {
    type Error = PositionalConversionError;
    fn try_from(AbstractProgram { functions, .. }: AbstractProgram) -> Result<Self, Self::Error> {
        Ok(Self {
            #[cfg(feature = "import")]
            imports,
            functions: functions
                .into_iter()
                .map(std::convert::TryInto::try_into)
                .collect::<Result<Vec<Function>, _>>()?,
        })
    }
}

impl TryFrom<AbstractFunction> for Function {
    type Error = PositionalConversionError;
    fn try_from(
        AbstractFunction {
            args,
            instrs,
            name,
            return_type,
            #[cfg(feature = "position")]
            pos,
        }: AbstractFunction,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            args: args
                .into_iter()
                .map(std::convert::TryInto::try_into)
                .collect::<Result<Vec<Argument>, _>>()
                .map_err(|e| e.add_pos(pos.clone()))?,
            instrs: instrs
                .into_iter()
                .map(std::convert::TryInto::try_into)
                .collect::<Result<Vec<Code>, _>>()?,
            name,
            return_type: match return_type {
                None => None,
                Some(t) => Some(
                    t.try_into()
                        .map_err(|e: ConversionError| e.add_pos(pos.clone()))?,
                ),
            },
            #[cfg(feature = "position")]
            pos,
        })
    }
}

impl TryFrom<AbstractArgument> for Argument {
    type Error = ConversionError;
    fn try_from(
        AbstractArgument { name, arg_type }: AbstractArgument,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            name,
            arg_type: arg_type.try_into()?,
        })
    }
}

impl TryFrom<AbstractCode> for Code {
    type Error = PositionalConversionError;
    fn try_from(c: AbstractCode) -> Result<Self, Self::Error> {
        Ok(match c {
            AbstractCode::Label {
                label,
                #[cfg(feature = "position")]
                pos,
            } => Self::Label {
                label,
                #[cfg(feature = "position")]
                pos,
            },
            AbstractCode::Instruction(i) => Self::Instruction(i.try_into()?),
        })
    }
}

impl TryFrom<AbstractInstruction> for Instruction {
    type Error = PositionalConversionError;
    fn try_from(i: AbstractInstruction) -> Result<Self, Self::Error> {
        Ok(match i {
            AbstractInstruction::Constant {
                dest,
                op,
                const_type,
                value,
                #[cfg(feature = "position")]
                pos,
            } => Self::Constant {
                dest,
                op,
                const_type: const_type
                    .try_into()
                    .map_err(|e: ConversionError| e.add_pos(pos.clone()))?,
                value,
                #[cfg(feature = "position")]
                pos,
            },
            AbstractInstruction::Value {
                args,
                dest,
                funcs,
                labels,
                op,
                op_type,
                #[cfg(feature = "position")]
                pos,
            } => Self::Value {
                args,
                dest,
                funcs,
                labels,
                op_type: op_type
                    .try_into()
                    .map_err(|e: ConversionError| e.add_pos(pos.clone()))?,
                #[cfg(feature = "position")]
                pos: pos.clone(),
                op: match op.as_ref() {
                    "add" => ValueOps::Add,
                    "mul" => ValueOps::Mul,
                    "div" => ValueOps::Div,
                    "eq" => ValueOps::Eq,
                    "lt" => ValueOps::Lt,
                    "gt" => ValueOps::Gt,
                    "le" => ValueOps::Le,
                    "ge" => ValueOps::Ge,
                    "not" => ValueOps::Not,
                    "and" => ValueOps::And,
                    "or" => ValueOps::Or,
                    "call" => ValueOps::Call,
                    "id" => ValueOps::Id,
                    "sub" => ValueOps::Sub,
                    #[cfg(feature = "ssa")]
                    "phi" => ValueOps::Phi,
                    #[cfg(feature = "float")]
                    "fadd" => ValueOps::Fadd,
                    #[cfg(feature = "float")]
                    "fsub" => ValueOps::Fsub,
                    #[cfg(feature = "float")]
                    "fmul" => ValueOps::Fmul,
                    #[cfg(feature = "float")]
                    "fdiv" => ValueOps::Fdiv,
                    #[cfg(feature = "float")]
                    "feq" => ValueOps::Feq,
                    #[cfg(feature = "float")]
                    "flt" => ValueOps::Flt,
                    #[cfg(feature = "float")]
                    "fgt" => ValueOps::Fgt,
                    #[cfg(feature = "float")]
                    "fle" => ValueOps::Fle,
                    #[cfg(feature = "float")]
                    "fge" => ValueOps::Fge,
                    #[cfg(feature = "char")]
                    "ceq" => ValueOps::Ceq,
                    #[cfg(feature = "char")]
                    "clt" => ValueOps::Clt,
                    #[cfg(feature = "char")]
                    "cgt" => ValueOps::Cgt,
                    #[cfg(feature = "char")]
                    "cle" => ValueOps::Cle,
                    #[cfg(feature = "char")]
                    "cge" => ValueOps::Cge,
                    #[cfg(feature = "char")]
                    "char2int" => ValueOps::Char2int,
                    #[cfg(feature = "char")]
                    "int2char" => ValueOps::Int2char,
                    #[cfg(feature = "memory")]
                    "alloc" => ValueOps::Alloc,
                    #[cfg(feature = "memory")]
                    "load" => ValueOps::Load,
                    #[cfg(feature = "memory")]
                    "ptradd" => ValueOps::PtrAdd,
                    v => {
                        return Err(ConversionError::InvalidValueOps(v.to_string()))
                            .map_err(|e| e.add_pos(pos))
                    }
                },
            },
            AbstractInstruction::Effect {
                args,
                funcs,
                labels,
                op,
                #[cfg(feature = "position")]
                pos,
            } => Self::Effect {
                args,
                funcs,
                labels,
                #[cfg(feature = "position")]
                pos: pos.clone(),
                op: match op.as_ref() {
                    "jmp" => EffectOps::Jump,
                    "br" => EffectOps::Branch,
                    "call" => EffectOps::Call,
                    "ret" => EffectOps::Return,
                    "print" => EffectOps::Print,
                    "nop" => EffectOps::Nop,
                    #[cfg(feature = "memory")]
                    "store" => EffectOps::Store,
                    #[cfg(feature = "memory")]
                    "free" => EffectOps::Free,
                    #[cfg(feature = "speculate")]
                    "speculate" => EffectOps::Speculate,
                    #[cfg(feature = "speculate")]
                    "commit" => EffectOps::Commit,
                    #[cfg(feature = "speculate")]
                    "guard" => EffectOps::Guard,
                    e => {
                        return Err(ConversionError::InvalidEffectOps(e.to_string()))
                            .map_err(|e| e.add_pos(pos))
                    }
                },
            },
        })
    }
}

impl TryFrom<Option<AbstractType>> for Type {
    type Error = ConversionError;

    fn try_from(value: Option<AbstractType>) -> Result<Self, Self::Error> {
        value.map_or(Err(ConversionError::MissingType), TryInto::try_into)
    }
}

impl TryFrom<AbstractType> for Type {
    type Error = ConversionError;
    fn try_from(value: AbstractType) -> Result<Self, Self::Error> {
        Ok(match value {
            AbstractType::Primitive(t) if t == "int" => Self::Int,
            AbstractType::Primitive(t) if t == "bool" => Self::Bool,
            #[cfg(feature = "float")]
            AbstractType::Primitive(t) if t == "float" => Self::Float,
            #[cfg(feature = "char")]
            AbstractType::Primitive(t) if t == "char" => Self::Char,
            AbstractType::Primitive(t) => return Err(ConversionError::InvalidPrimitive(t)),
            #[cfg(feature = "memory")]
            AbstractType::Parameterized(t, ty) if t == "ptr" => {
                Self::Pointer(Box::new((*ty).try_into()?))
            }
            AbstractType::Parameterized(t, ty) => {
                return Err(ConversionError::InvalidParameterized(t, ty.to_string()))
            }
        })
    }
}

/// <https://capra.cs.cornell.edu/bril/lang/syntax.html#effect-operation>
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
// Having the #[error(...)] for all variants derives the Display trait as well
#[derive(Error)]
pub enum InterpError {
    #[error("Attempt to divide by 0")]
    DivisionByZero,
    #[error("Some memory locations have not been freed by the end of execution")]
    MemLeak,
    #[error("Trying to load from uninitialized memory")]
    UsingUninitializedMemory,
    #[error("phi node executed with no last label")]
    NoLastLabel,
    #[error("Could not find label: {0}")]
    MissingLabel(String),
    #[error("no main function defined, doing nothing")]
    NoMainFunction,
    #[error("phi node has unequal numbers of labels and args")]
    UnequalPhiNode,
    #[error("char must have one character")]
    NotOneChar,
    #[error("multiple functions of the same name found")]
    DuplicateFunction,
    #[error("duplicate label `{0}` found")]
    DuplicateLabel(String),
    #[error("Expected empty return for `{0}`, found value")]
    NonEmptyRetForFunc(String),
    #[error("cannot allocate `{0}` entries")]
    CannotAllocSize(i64),
    #[error("Tried to free illegal memory location base: `{0}`, offset: `{1}`. Offset must be 0.")]
    IllegalFree(usize, i64), // (base, offset)
    #[error("Uninitialized heap location `{0}` and/or illegal offset `{1}`")]
    InvalidMemoryAccess(usize, i64), // (base, offset)
    #[error("Expected `{0}` function arguments, found `{1}`")]
    BadNumFuncArgs(usize, usize), // (expected, actual)
    #[error("Expected `{0}` instruction arguments, found `{1}`")]
    BadNumArgs(usize, usize), // (expected, actual)
    #[error("Expected `{0}` labels, found `{1}`")]
    BadNumLabels(usize, usize), // (expected, actual)
    #[error("Expected `{0}` functions, found `{1}`")]
    BadNumFuncs(usize, usize), // (expected, actual)
    #[error("no function of name `{0}` found")]
    FuncNotFound(String),
    #[error("undefined variable `{0}`")]
    VarUndefined(String),
    #[error("Label `{0}` for phi node not found")]
    PhiMissingLabel(String),
    #[error("unspecified pointer type `{0:?}`")]
    ExpectedPointerType(Type), // found type
    #[error("Expected type `{0:?}` for function argument, found `{1:?}`")]
    BadFuncArgType(Type, String), // (expected, actual)
    #[error("Expected type `{0:?}` for assignment, found `{1:?}`")]
    BadAsmtType(Type, Type), // (expected, actual). For when the LHS type of an instruction is bad
    #[error("value ${0} cannot be converted to char")]
    ToCharError(i64),
}

impl InterpError {
    #[must_use]
    pub fn add_pos(self, pos: Option<Position>) -> PositionalInterpError {
        PositionalInterpError {
            e: Box::new(self),
            pos,
        }
    }
}

#[derive(Error, Debug)]
pub struct PositionalInterpError {
    pub e: Box<dyn Error>,
    pub pos: Option<Position>,
}

impl Display for PositionalInterpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self {
                e,
                pos:
                    Some(Position {
                        pos,
                        pos_end: Some(end),
                        src: Some(s),
                    }),
            } => {
                write!(
                    f,
                    "{s}:{}:{} to {s}:{}:{} \n\t {e}",
                    pos.row, pos.col, end.row, end.col
                )
            }
            Self {
                e,
                pos:
                    Some(Position {
                        pos,
                        pos_end: None,
                        src: Some(s),
                    }),
            } => {
                write!(f, "{s}:{}:{} \n\t {e}", pos.row, pos.col)
            }
            Self {
                e,
                pos:
                    Some(Position {
                        pos,
                        pos_end: Some(end),
                        src: None,
                    }),
            } => {
                write!(
                    f,
                    "Line {}, Column {} to Line {}, Column {}: {e}",
                    pos.row, pos.col, end.row, end.col
                )
            }
            Self {
                e,
                pos:
                    Some(Position {
                        pos,
                        pos_end: None,
                        src: None,
                    }),
            } => {
                write!(f, "Line {}, Column {}: {e}", pos.row, pos.col)
            }
            Self { e, pos: None } => write!(f, "{e}"),
        }
    }
}

impl From<InterpError> for PositionalInterpError {
    fn from(e: InterpError) -> Self {
        Self {
            e: Box::new(e),
            pos: None,
        }
    }
}

/// <https://capra.cs.cornell.edu/bril/lang/syntax.html#effect-operation>
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum EffectOps {
    /// <https://capra.cs.cornell.edu/bril/lang/core.html#control>
    #[serde(rename = "jmp")]
    Jump,
    /// <https://capra.cs.cornell.edu/bril/lang/core.html#control>
    #[serde(rename = "br")]
    Branch,
    /// <https://capra.cs.cornell.edu/bril/lang/core.html#control>
    Call,
    /// <https://capra.cs.cornell.edu/bril/lang/core.html#control>
    #[serde(rename = "ret")]
    Return,
    /// <https://capra.cs.cornell.edu/bril/lang/core.html#miscellaneous>
    Print,
    /// <https://capra.cs.cornell.edu/bril/lang/core.html#miscellaneous>
    Nop,
    /// <https://capra.cs.cornell.edu/bril/lang/memory.html#operations>
    #[cfg(feature = "memory")]
    Store,
    /// <https://capra.cs.cornell.edu/bril/lang/memory.html#operations>
    #[cfg(feature = "memory")]
    Free,
    /// <https://capra.cs.cornell.edu/bril/lang/spec.html#operations>
    #[cfg(feature = "speculate")]
    Speculate,
    /// <https://capra.cs.cornell.edu/bril/lang/spec.html#operations>
    #[cfg(feature = "speculate")]
    Commit,
    /// <https://capra.cs.cornell.edu/bril/lang/spec.html#operations>
    #[cfg(feature = "speculate")]
    Guard,
}

impl Display for EffectOps {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Jump => write!(f, "jmp"),
            Self::Branch => write!(f, "br"),
            Self::Call => write!(f, "call"),
            Self::Return => write!(f, "ret"),
            Self::Print => write!(f, "print"),
            Self::Nop => write!(f, "nop"),
            #[cfg(feature = "memory")]
            Self::Store => write!(f, "store"),
            #[cfg(feature = "memory")]
            Self::Free => write!(f, "free"),
            #[cfg(feature = "speculate")]
            Self::Speculate => write!(f, "speculate"),
            #[cfg(feature = "speculate")]
            Self::Commit => write!(f, "commit"),
            #[cfg(feature = "speculate")]
            Self::Guard => write!(f, "guard"),
        }
    }
}

/// <https://capra.cs.cornell.edu/bril/lang/syntax.html#value-operation>
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ValueOps {
    /// <https://capra.cs.cornell.edu/bril/lang/core.html#arithmetic>
    Add,
    /// <https://capra.cs.cornell.edu/bril/lang/core.html#arithmetic>
    Sub,
    /// <https://capra.cs.cornell.edu/bril/lang/core.html#arithmetic>
    Mul,
    /// <https://capra.cs.cornell.edu/bril/lang/core.html#arithmetic>
    Div,
    /// <https://capra.cs.cornell.edu/bril/lang/core.html#comparison>
    Eq,
    /// <https://capra.cs.cornell.edu/bril/lang/core.html#comparison>
    Lt,
    /// <https://capra.cs.cornell.edu/bril/lang/core.html#comparison>
    Gt,
    /// <https://capra.cs.cornell.edu/bril/lang/core.html#comparison>
    Le,
    /// <https://capra.cs.cornell.edu/bril/lang/core.html#comparison>
    Ge,
    /// <https://capra.cs.cornell.edu/bril/lang/core.html#logic>
    Not,
    /// <https://capra.cs.cornell.edu/bril/lang/core.html#logic>
    And,
    /// <https://capra.cs.cornell.edu/bril/lang/core.html#logic>
    Or,
    /// <https://capra.cs.cornell.edu/bril/lang/core.html#control>
    Call,
    /// <https://capra.cs.cornell.edu/bril/lang/core.html#miscellaneous>
    Id,
    /// <https://capra.cs.cornell.edu/bril/lang/ssa.html#operations>
    #[cfg(feature = "ssa")]
    Phi,
    /// <https://capra.cs.cornell.edu/bril/lang/float.html#operations>
    #[cfg(feature = "float")]
    Fadd,
    /// <https://capra.cs.cornell.edu/bril/lang/float.html#operations>
    #[cfg(feature = "float")]
    Fsub,
    /// <https://capra.cs.cornell.edu/bril/lang/float.html#operations>
    #[cfg(feature = "float")]
    Fmul,
    /// <https://capra.cs.cornell.edu/bril/lang/float.html#operations>
    #[cfg(feature = "float")]
    Fdiv,
    /// <https://capra.cs.cornell.edu/bril/lang/float.html#operations>
    #[cfg(feature = "float")]
    Feq,
    /// <https://capra.cs.cornell.edu/bril/lang/float.html#operations>
    #[cfg(feature = "float")]
    Flt,
    /// <https://capra.cs.cornell.edu/bril/lang/float.html#operations>
    #[cfg(feature = "float")]
    Fgt,
    /// <https://capra.cs.cornell.edu/bril/lang/float.html#operations>
    #[cfg(feature = "float")]
    Fle,
    /// <https://capra.cs.cornell.edu/bril/lang/float.html#operations>
    #[cfg(feature = "float")]
    Fge,
    /// <https://capra.cs.cornell.edu/bril/lang/char.html#operations>
    #[cfg(feature = "char")]
    Ceq,
    /// <https://capra.cs.cornell.edu/bril/lang/char.html#operations>
    #[cfg(feature = "char")]
    Clt,
    /// <https://capra.cs.cornell.edu/bril/lang/char.html#operations>
    #[cfg(feature = "char")]
    Cgt,
    /// <https://capra.cs.cornell.edu/bril/lang/char.html#operations>
    #[cfg(feature = "char")]
    Cle,
    /// <https://capra.cs.cornell.edu/bril/lang/char.html#operations>
    #[cfg(feature = "char")]
    Cge,
    /// <https://capra.cs.cornell.edu/bril/lang/char.html#operations>
    #[cfg(feature = "char")]
    Char2int,
    /// <https://capra.cs.cornell.edu/bril/lang/char.html#operations>
    #[cfg(feature = "char")]
    Int2char,
    /// <https://capra.cs.cornell.edu/bril/lang/memory.html#operations>
    #[cfg(feature = "memory")]
    Alloc,
    /// <https://capra.cs.cornell.edu/bril/lang/memory.html#operations>
    #[cfg(feature = "memory")]
    Load,
    /// <https://capra.cs.cornell.edu/bril/lang/memory.html#operations>
    #[cfg(feature = "memory")]
    PtrAdd,
}

impl Display for ValueOps {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Add => write!(f, "add"),
            Self::Sub => write!(f, "sub"),
            Self::Mul => write!(f, "mul"),
            Self::Div => write!(f, "div"),
            Self::Eq => write!(f, "eq"),
            Self::Lt => write!(f, "lt"),
            Self::Gt => write!(f, "gt"),
            Self::Le => write!(f, "le"),
            Self::Ge => write!(f, "ge"),
            Self::Not => write!(f, "not"),
            Self::And => write!(f, "and"),
            Self::Or => write!(f, "or"),
            Self::Call => write!(f, "call"),
            Self::Id => write!(f, "id"),
            #[cfg(feature = "ssa")]
            Self::Phi => write!(f, "phi"),
            #[cfg(feature = "float")]
            Self::Fadd => write!(f, "fadd"),
            #[cfg(feature = "float")]
            Self::Fsub => write!(f, "fsub"),
            #[cfg(feature = "float")]
            Self::Fmul => write!(f, "fmul"),
            #[cfg(feature = "float")]
            Self::Fdiv => write!(f, "fdiv"),
            #[cfg(feature = "float")]
            Self::Feq => write!(f, "feq"),
            #[cfg(feature = "float")]
            Self::Flt => write!(f, "flt"),
            #[cfg(feature = "float")]
            Self::Fgt => write!(f, "fgt"),
            #[cfg(feature = "float")]
            Self::Fle => write!(f, "fle"),
            #[cfg(feature = "float")]
            Self::Fge => write!(f, "fge"),
            #[cfg(feature = "char")]
            Self::Ceq => write!(f, "ceq"),
            #[cfg(feature = "char")]
            Self::Clt => write!(f, "clt"),
            #[cfg(feature = "char")]
            Self::Cgt => write!(f, "cgt"),
            #[cfg(feature = "char")]
            Self::Cle => write!(f, "cle"),
            #[cfg(feature = "char")]
            Self::Cge => write!(f, "cge"),
            #[cfg(feature = "char")]
            Self::Char2int => write!(f, "char2int"),
            #[cfg(feature = "char")]
            Self::Int2char => write!(f, "int2char"),
            #[cfg(feature = "memory")]
            Self::Alloc => write!(f, "alloc"),
            #[cfg(feature = "memory")]
            Self::Load => write!(f, "load"),
            #[cfg(feature = "memory")]
            Self::PtrAdd => write!(f, "ptradd"),
        }
    }
}
/// <https://capra.cs.cornell.edu/bril/lang/syntax.html#source-positions>
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColRow {
    /// Column
    pub col: u64,
    /// Row
    pub row: u64,
}

pub fn load_program_from_read<R: std::io::Read>(mut input: R) -> Program {
    let mut buffer = String::new();
    input.read_to_string(&mut buffer).unwrap();
    serde_json::from_str(&buffer).unwrap()
}

/// <https://capra.cs.cornell.edu/bril/lang/syntax.html#instruction>
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum Instruction {
    /// <https://capra.cs.cornell.edu/bril/lang/syntax.html#constant>
    Constant {
        /// destination variable
        dest: String,
        /// "const"
        op: ConstOps,
        #[cfg(feature = "position")]
        /// The source position of the instruction if provided
        #[serde(flatten, skip_serializing_if = "Option::is_none")]
        pos: Option<Position>,
        /// Type of variable
        #[serde(rename = "type")]
        const_type: Type,
        /// The literal being stored in the variable
        value: Literal,
    },
    /// <https://capra.cs.cornell.edu/bril/lang/syntax.html#value-operation>
    Value {
        /// List of variables as arguments
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        args: Vec<String>,
        /// destination variable
        dest: String,
        /// List of strings as function names
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        funcs: Vec<String>,
        /// List of strings as labels
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        labels: Vec<String>,
        /// Operation being executed
        op: ValueOps,
        /// The source position of the instruction if provided
        #[cfg(feature = "position")]
        #[serde(flatten, skip_serializing_if = "Option::is_none")]
        pos: Option<Position>,
        /// Type of variable
        #[serde(rename = "type")]
        op_type: Type,
    },
    /// <https://capra.cs.cornell.edu/bril/lang/syntax.html#effect-operation>
    Effect {
        /// List of variables as arguments
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        args: Vec<String>,
        /// List of strings as function names
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        funcs: Vec<String>,
        /// List of strings as labels
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        labels: Vec<String>,
        /// Operation being executed
        op: EffectOps,
        /// The source position of the instruction if provided
        #[cfg(feature = "position")]
        #[serde(flatten, skip_serializing_if = "Option::is_none")]
        pos: Option<Position>,
    },
}

#[cfg(feature = "position")]
impl Instruction {
    /// A helper function to extract the position value if it exists from an instruction
    #[must_use]
    pub fn get_pos(&self) -> Option<Position> {
        match self {
            Self::Constant { pos, .. } | Self::Value { pos, .. } | Self::Effect { pos, .. } => {
                pos.clone()
            }
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Constant {
                op,
                dest,
                const_type,
                value,
                #[cfg(feature = "position")]
                    pos: _,
            } => {
                write!(f, "{dest}: {const_type} = {op} {value};")
            }
            Self::Value {
                op,
                dest,
                op_type,
                args,
                funcs,
                labels,
                #[cfg(feature = "position")]
                    pos: _,
            } => {
                write!(f, "{dest}: {op_type} = {op}")?;
                for func in funcs {
                    write!(f, " @{func}")?;
                }
                for arg in args {
                    write!(f, " {arg}")?;
                }
                for label in labels {
                    write!(f, " .{label}")?;
                }
                write!(f, ";")
            }
            Self::Effect {
                op,
                args,
                funcs,
                labels,
                #[cfg(feature = "position")]
                    pos: _,
            } => {
                write!(f, "{op}")?;
                for func in funcs {
                    write!(f, " @{func}")?;
                }
                for arg in args {
                    write!(f, " {arg}")?;
                }
                for label in labels {
                    write!(f, " .{label}")?;
                }
                write!(f, ";")
            }
        }
    }
}

/// <https://capra.cs.cornell.edu/bril/lang/syntax.html#type>
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Type {
    /// <https://capra.cs.cornell.edu/bril/lang/core.html#types>
    Int,
    /// <https://capra.cs.cornell.edu/bril/lang/core.html#types>
    Bool,
    /// <https://capra.cs.cornell.edu/bril/lang/float.html#types>
    #[cfg(feature = "float")]
    Float,
    /// <https://capra.cs.cornell.edu/bril/lang/char.html#types>
    #[cfg(feature = "char")]
    Char,
    /// <https://capra.cs.cornell.edu/bril/lang/memory.html#types>
    #[cfg(feature = "memory")]
    #[serde(rename = "ptr")]
    Pointer(Box<Self>),
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int => write!(f, "int"),
            Self::Bool => write!(f, "bool"),
            #[cfg(feature = "float")]
            Self::Float => write!(f, "float"),
            #[cfg(feature = "char")]
            Self::Char => write!(f, "char"),
            #[cfg(feature = "memory")]
            Self::Pointer(tpe) => write!(f, "ptr<{tpe}>"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Argument {
    /// a
    pub name: String,
    #[serde(rename = "type")]
    /// int
    pub arg_type: Type,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum Code {
    /// <https://capra.cs.cornell.edu/bril/lang/syntax.html#label>
    Label {
        /// The name of the label
        label: String,
        /// Where the label is located in source code
        #[cfg(feature = "position")]
        #[serde(flatten, skip_serializing_if = "Option::is_none")]
        pos: Option<Position>,
    },
    /// <https://capra.cs.cornell.edu/bril/lang/syntax.html#instruction>
    Instruction(Instruction),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Function {
    /// Any arguments the function accepts
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<Argument>,
    /// The instructions of this function
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub instrs: Vec<Code>,
    /// The name of the function
    pub name: String,
    /// The position of this function in the original source code
    #[cfg(feature = "position")]
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub pos: Option<Position>,
    /// The possible return type of this function
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_type: Option<Type>,
}

/// Equivalent to a file of bril code
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Program {
    /// A list of functions declared in the program
    pub functions: Vec<Function>,
    #[cfg(feature = "import")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// A list of imports for this program
    pub imports: Vec<Import>,
}

#[doc(hidden)]
#[derive(Debug)]
pub struct BBFunction {
    pub name: String,
    pub args: Vec<Argument>,
    pub return_type: Option<Type>,
    pub blocks: Vec<BasicBlock>,
    // the following is an optimization by replacing the string representation of variables with a number
    // Variable names are ordered from 0 to num_of_vars.
    // These replacements are found for function args and for code in the BasicBlocks
    pub num_of_vars: usize,
    pub args_as_nums: Vec<usize>,
    pub pos: Option<Position>,
}

impl BBFunction {
    fn new(f: Function, func_map: &HashMap<String, usize>) -> Result<Self, InterpError> {
        let (mut func, label_map) = Self::find_basic_blocks(f, func_map).unwrap();
        func.build_cfg(&label_map)?;
        Ok(func)
    }

    fn find_basic_blocks(
        func: Function,
        func_map: &HashMap<String, usize>,
    ) -> Result<(Self, HashMap<String, usize>), PositionalInterpError> {
        let mut blocks = Vec::new();
        let mut label_map = HashMap::default();

        let mut num_of_vars = 0;
        let mut num_var_map = HashMap::default();

        let args_as_nums = func
            .args
            .iter()
            .map(|a| get_num_from_map(&a.name, &mut num_of_vars, &mut num_var_map))
            .collect();

        let mut curr_block = BasicBlock::new();
        for instr in func.instrs {
            match instr {
                Code::Label { label, pos } => {
                    if !curr_block.instrs.is_empty() || curr_block.label.is_some() {
                        blocks.push(curr_block);
                        curr_block = BasicBlock::new();
                    }
                    if label_map.insert(label.to_string(), blocks.len()).is_some() {
                        return Err(InterpError::DuplicateLabel(label).add_pos(pos));
                    }
                    curr_block.label = Some(label);
                }
                Code::Instruction(i @ Instruction::Effect { op, .. })
                    if op == EffectOps::Jump
                        || op == EffectOps::Branch
                        || op == EffectOps::Return =>
                {
                    curr_block.numified_instrs.push(NumifiedInstruction::new(
                        &i,
                        &mut num_of_vars,
                        &mut num_var_map,
                        func_map,
                    )?);
                    curr_block.instrs.push(i);
                    blocks.push(curr_block);
                    curr_block = BasicBlock::new();
                }
                Code::Instruction(code) => {
                    curr_block.numified_instrs.push(NumifiedInstruction::new(
                        &code,
                        &mut num_of_vars,
                        &mut num_var_map,
                        func_map,
                    )?);
                    curr_block.instrs.push(code);
                }
            }
        }

        if !curr_block.instrs.is_empty() || curr_block.label.is_some() {
            blocks.push(curr_block);
        }

        Ok((
            Self {
                name: func.name,
                args: func.args,
                return_type: func.return_type,
                blocks,
                args_as_nums,
                num_of_vars,
                pos: func.pos,
            },
            label_map,
        ))
    }

    fn build_cfg(&mut self, label_map: &HashMap<String, usize>) -> Result<(), InterpError> {
        if self.blocks.is_empty() {
            return Ok(());
        }
        let last_idx = self.blocks.len() - 1;
        for (i, block) in self.blocks.iter_mut().enumerate() {
            // Get the last instruction
            let last_instr = block.instrs.last().cloned();
            if let Some(Instruction::Effect {
                op: EffectOps::Jump | EffectOps::Branch,
                labels,
                ..
            }) = last_instr
            {
                for l in labels {
                    block
                        .exit
                        .push(*label_map.get(&l).ok_or(InterpError::MissingLabel(l))?);
                }
            } else if let Some(Instruction::Effect {
                op: EffectOps::Return,
                ..
            }) = last_instr
            {
                // We are done, there is no exit from this block
            } else {
                // If we're before the last block
                if i < last_idx {
                    block.exit.push(i + 1);
                }
            }
        }
        Ok(())
    }
}

/// A program represented as basic blocks. This is the IR of brilirs
#[derive(Debug)]
pub struct BBProgram {
    #[doc(hidden)]
    pub index_of_main: Option<usize>,
    #[doc(hidden)]
    pub func_index: Vec<BBFunction>,
}

impl TryFrom<Program> for BBProgram {
    type Error = InterpError;

    fn try_from(prog: Program) -> Result<Self, Self::Error> {
        Self::new(prog)
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct NumifiedInstruction {
    pub dest: Option<usize>,
    pub args: Vec<usize>,
    pub funcs: Vec<usize>,
}

impl BBProgram {
    /// Converts a [`Program`] into a [`BBProgram`]
    /// # Errors
    /// Will return an error if the program is invalid in some way.
    /// Reasons include the `Program` have multiple functions with the same name, a function name is not found, or a label is expected by an instruction but missing.
    pub fn new(prog: Program) -> Result<Self, InterpError> {
        let num_funcs = prog.functions.len();

        let func_map: HashMap<String, usize> = prog
            .functions
            .iter()
            .enumerate()
            .map(|(idx, func)| (func.name.clone(), idx))
            .collect();

        let func_index = prog
            .functions
            .into_iter()
            .map(|func| BBFunction::new(func, &func_map))
            .collect::<Result<Vec<BBFunction>, InterpError>>()?;

        let bb = Self {
            index_of_main: func_map.get("main").copied(),
            func_index,
        };
        if func_map.len() == num_funcs {
            Ok(bb)
        } else {
            Err(InterpError::DuplicateFunction)
        }
    }

    #[doc(hidden)]
    #[must_use]
    pub fn get(&self, func_name: usize) -> Option<&BBFunction> {
        self.func_index.get(func_name)
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct BasicBlock {
    pub label: Option<String>,
    // These two vecs work in parallel
    // One is the normal instruction
    // The other contains the numified version of the destination and arguments
    pub instrs: Vec<Instruction>,
    pub numified_instrs: Vec<NumifiedInstruction>,
    pub exit: Vec<usize>,
}

impl BasicBlock {
    const fn new() -> Self {
        Self {
            label: None,
            instrs: Vec::new(),
            numified_instrs: Vec::new(),
            exit: Vec::new(),
        }
    }
}

impl NumifiedInstruction {
    fn new(
        instr: &Instruction,
        // The total number of variables so far. Only grows
        num_of_vars: &mut usize,
        // A map from variables to numbers
        num_var_map: &mut HashMap<String, usize>,
        // A map from function names to numbers
        func_map: &HashMap<String, usize>,
    ) -> Result<Self, PositionalInterpError> {
        Ok(match instr {
            Instruction::Constant { dest, .. } => Self {
                dest: Some(get_num_from_map(dest, num_of_vars, num_var_map)),
                args: Vec::new(),
                funcs: Vec::new(),
            },
            Instruction::Value {
                dest,
                args,
                funcs,
                pos,
                ..
            } => Self {
                dest: Some(get_num_from_map(dest, num_of_vars, num_var_map)),
                args: args
                    .iter()
                    .map(|v| get_num_from_map(v, num_of_vars, num_var_map))
                    .collect(),
                funcs: funcs
                    .iter()
                    .map(|f| {
                        func_map.get(f).copied().ok_or_else(|| {
                            InterpError::FuncNotFound(f.to_string()).add_pos(pos.clone())
                        })
                    })
                    .collect::<Result<Vec<usize>, PositionalInterpError>>()?,
            },
            Instruction::Effect {
                args, funcs, pos, ..
            } => Self {
                dest: None,
                args: args
                    .iter()
                    .map(|v| get_num_from_map(v, num_of_vars, num_var_map))
                    .collect(),
                funcs: funcs
                    .iter()
                    .map(|f| {
                        func_map.get(f).copied().ok_or_else(|| {
                            InterpError::FuncNotFound(f.to_string()).add_pos(pos.clone())
                        })
                    })
                    .collect::<Result<Vec<usize>, PositionalInterpError>>()?,
            },
        })
    }
}

fn get_num_from_map(
    variable_name: &str,
    // The total number of variables so far. Only grows
    num_of_vars: &mut usize,
    // A map from variables to numbers
    num_var_map: &mut HashMap<String, usize>,
) -> usize {
    // https://github.com/rust-lang/rust-clippy/issues/8346
    #[allow(clippy::option_if_let_else)]
    if let Some(i) = num_var_map.get(variable_name) {
        *i
    } else {
        let x = *num_of_vars;
        num_var_map.insert(variable_name.to_string(), x);
        *num_of_vars += 1;
        x
    }
}
