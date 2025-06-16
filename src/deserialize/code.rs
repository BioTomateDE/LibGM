use crate::deserialize::chunk_reading::GMRef;
use crate::deserialize::chunk_reading::GMChunk;
use crate::deserialize::variables::{GMVariable, GMVariables};
use std::cmp::PartialEq;
use std::fmt::{Display, Formatter};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use crate::deserialize::functions::{GMFunction, GMFunctions};
use crate::deserialize::game_objects::GMGameObject;
use crate::deserialize::general_info::GMGeneralInfo;
use crate::deserialize::strings::GMStrings;

// Taken from UndertaleModTool/UndertaleModLib/UndertaleCode.cs/UndertaleInstruction/
#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive, IntoPrimitive, Serialize, Deserialize)]
#[repr(u8)]
pub enum GMOpcode {
    Conv = 0x07,     // Push((Types.Second)Pop) // DoubleTypeInstruction
    Mul = 0x08,      // Push(Pop() * Pop()) // DoubleTypeInstruction
    Div = 0x09,      // Push(Pop() / Pop()) // DoubleTypeInstruction
    Rem = 0x0A,      // Push(Remainder(Pop(), Pop())) // DoubleTypeInstruction
    Mod = 0x0B,      // Push(Pop() % Pop()) // DoubleTypeInstruction
    Add = 0x0C,      // Push(Pop() + Pop()) // DoubleTypeInstruction
    Sub = 0x0D,      // Push(Pop() - Pop()) // DoubleTypeInstruction
    And = 0x0E,      // Push(Pop() & Pop()) // DoubleTypeInstruction
    Or = 0x0F,       // Push(Pop() | Pop()) // DoubleTypeInstruction
    Xor = 0x10,      // Push(Pop() ^ Pop()) // DoubleTypeInstruction
    Neg = 0x11,      // Push(-Pop()) // SingleTypeInstruction
    Not = 0x12,      // Push(~Pop()) // SingleTypeInstruction
    Shl = 0x13,      // Push(Pop() << Pop()) // DoubleTypeInstruction
    Shr = 0x14,      // Push(Pop() >>= Pop()) // DoubleTypeInstruction
    Cmp = 0x15,      // Push(Pop() `cmp` Pop())// ComparisonInstruction
    Pop = 0x45,      // Instance.Destination = Pop();
    Dup = 0x86,      // Push(Peek()) // SingleTypeInstruction
    Ret = 0x9C,      // return Pop() // SingleTypeInstruction
    Exit = 0x9D,     // return; // SingleTypeInstruction
    Popz = 0x9E,     // Pop(); // SingleTypeInstruction
    B = 0xB6,        // goto Index + Offset*4; // GotoInstruction
    Bt = 0xB7,       // if (Pop()) goto Index + Offset*4; // GotoInstruction
    Bf = 0xB8,       // if (!Pop()) goto Index + Offset*4; // GotoInstruction
    PushEnv = 0xBA,  // GotoInstruction
    PopEnv = 0xBB,   // GotoInstruction
    Push = 0xC0,     // Push(Value) // push constant
    PushLoc = 0xC1,  // Push(Value) // push local
    PushGlb = 0xC2,  // Push(Value) // push global
    PushBltn = 0xC3, // Push(Value) // push builtin variable
    PushI = 0x84,    // Push(Value) // push int16
    Call = 0xD9,     // Function(arg0, arg1, ..., argn) where arg = Pop() and n = ArgumentsCount
    CallV = 0x99, // TODO: Unknown, maybe to do with calling using the stack? Generates with "show_message((function(){return 5;})());"
    Break = 0xFF, // TODO: Several sub-opcodes in GMS 2.3
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum GMInstructionType {
    SingleTypeInstruction,
    DoubleTypeInstruction,
    ComparisonInstruction,
    GotoInstruction,
    PushInstruction,
    PopInstruction,
    CallInstruction,
    BreakInstruction,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum GMDataType {
    Double,
    Float,
    Int32,
    Int64,
    Boolean,
    Variable,
    String,
    Instance, // obsolete??
    Delete,   // these 3 types apparently exist
    Undefined,
    UnsignedInt,
    Int16 = 0x0f,
}
#[derive(Debug, Clone, PartialEq)]
pub enum GMInstanceType {
    Undefined,  // Idk
    Instance(Option<GMRef<GMGameObject>>),      // Represents the current chunk instance.
    Other,      // Represents the other context, which has multiple definitions based on the location used.
    All,        // Represents all active object instances. Assignment operations can perform a loop.
    None,       // Represents no object/instance.
    Global,     // Used for global variables.
    Builtin,    // Used for GML built-in variables.
    Local,      // Used for local variables; local to their code script.
    StackTop,   // Instance is stored in a Variable data type on the top of the stack.
    Argument,   // Used for function argument variables in GMLv2 (GMS 2.3).
    Static,     // Used for static variables.
}
impl Display for GMInstanceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            GMInstanceType::Undefined => write!(f, "Undefined"),
            GMInstanceType::Instance(None) => write!(f, "Self"),
            GMInstanceType::Instance(Some(reference)) => write!(f, "Self<{}>", reference.index),
            GMInstanceType::Other => write!(f, "Other"),
            GMInstanceType::All => write!(f, "All"),
            GMInstanceType::None => write!(f, "None"),
            GMInstanceType::Global => write!(f, "Global"),
            GMInstanceType::Builtin => write!(f, "Builtin"),
            GMInstanceType::Local => write!(f, "Local"),
            GMInstanceType::StackTop => write!(f, "StackTop"),
            GMInstanceType::Argument => write!(f, "Argument"),
            GMInstanceType::Static => write!(f, "Static"),
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum GMVariableType {
    Array = 0x00,           // Used for normal single-dimension array variables
    StackTop = 0x80,        // Used when referencing a variable on another variable, e.g. a chain referenc
    Normal = 0xA0,          // normal
    Instance = 0xE0,        // used when referencing variables on room instance IDs, e.g. something like "inst_01ABCDEF.x" in GML
    MultiPush = 0x10,       // GMS2.3+, multidimensional array with pushaf
    MultiPushPop = 0x90,    // GMS2.3+, multidimensional array with pushaf or popaf
}
#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum GMComparisonType {
    LT = 1,
    LTE = 2,
    EQ = 3,
    NEQ = 4,
    GTE = 5,
    GT = 6,
}

fn get_instruction_type(opcode: GMOpcode) -> GMInstructionType {
    match opcode {
        GMOpcode::Neg
        | GMOpcode::Not
        | GMOpcode::Dup
        | GMOpcode::Ret
        | GMOpcode::Exit
        | GMOpcode::Popz
        | GMOpcode::CallV => GMInstructionType::SingleTypeInstruction,
        GMOpcode::Conv
        | GMOpcode::Mul
        | GMOpcode::Div
        | GMOpcode::Rem
        | GMOpcode::Mod
        | GMOpcode::Add
        | GMOpcode::Sub
        | GMOpcode::And
        | GMOpcode::Or
        | GMOpcode::Xor
        | GMOpcode::Shl
        | GMOpcode::Shr => GMInstructionType::DoubleTypeInstruction,
        GMOpcode::Cmp => GMInstructionType::ComparisonInstruction,
        GMOpcode::B | GMOpcode::Bt | GMOpcode::Bf | GMOpcode::PushEnv | GMOpcode::PopEnv => {
            GMInstructionType::GotoInstruction
        }

        GMOpcode::Pop => GMInstructionType::PopInstruction,
        GMOpcode::Push
        | GMOpcode::PushLoc
        | GMOpcode::PushGlb
        | GMOpcode::PushBltn
        | GMOpcode::PushI => GMInstructionType::PushInstruction,

        GMOpcode::Call => GMInstructionType::CallInstruction,
        GMOpcode::Break => GMInstructionType::BreakInstruction,
    }
}

fn convert_instruction_kind(kind: u8) -> u8 {
    // Convert from bytecode 14 instruction opcodes to modern opcodes
    match kind {
        0x03 => 0x07,
        0x04 => 0x08,
        0x05 => 0x09,
        0x06 => 0x0A,
        0x07 => 0x0B,
        0x08 => 0x0C,
        0x09 => 0x0D,
        0x0A => 0x0E,
        0x0B => 0x0F,
        0x0C => 0x10,
        0x0D => 0x11,
        0x0E => 0x12,
        0x0F => 0x13,
        0x10 => 0x14,
        0x11 | 0x12 | 0x13 | 0x14 | 0x16 => 0x15,
        0xDA => 0xD9,
        0x41 => 0x45,
        0x82 => 0x86,
        0xB7 => 0xB6,
        0xB8 => 0xB7,
        0xB9 => 0xB8,
        0x9D => 0x9C,
        0x9E => 0x9D,
        0x9F => 0x9E,
        0xBB => 0xBA,
        0xBC => 0xBB,
        _ => kind,
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GMSingleTypeInstruction {
    pub opcode: GMOpcode,
    pub extra: u8,
    pub data_type: GMDataType,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GMDoubleTypeInstruction {
    pub opcode: GMOpcode,
    pub type1: GMDataType,
    pub type2: GMDataType,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GMComparisonInstruction {
    pub opcode: GMOpcode,
    pub comparison_type: GMComparisonType,  // comparison kind
    pub type1: GMDataType,                  // datatype of element to compare
    pub type2: GMDataType,                  // datatype of element to compare
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GMGotoInstruction {
    pub opcode: GMOpcode,
    pub jump_offset: i32,
    pub popenv_exit_magic: bool,
}
#[derive(Debug, Clone, PartialEq)]
pub struct GMPopInstruction {
    pub opcode: GMOpcode,
    pub instance_type: GMInstanceType,
    pub type1: GMDataType,
    pub type2: GMDataType,
    pub destination: GMCodeVariable,
}
#[derive(Debug, Clone, PartialEq)]
pub struct GMPushInstruction {
    pub opcode: GMOpcode,
    pub data_type: GMDataType,
    pub value: GMValue,
}
#[derive(Debug, Clone, PartialEq)]
pub struct GMCallInstruction {
    pub opcode: GMOpcode,
    pub arguments_count: u8,
    pub data_type: GMDataType,
    pub function: GMRef<GMFunction>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct GMBreakInstruction {
    pub opcode: GMOpcode,
    pub value: i16,
    pub data_type: GMDataType,
    pub int_argument: Option<i32>,
}
#[derive(Debug, Clone, PartialEq)]
pub enum GMInstruction {
    SingleType(GMSingleTypeInstruction),
    DoubleType(GMDoubleTypeInstruction),
    Comparison(GMComparisonInstruction),
    Goto(GMGotoInstruction),
    Pop(GMPopInstruction),
    Push(GMPushInstruction),
    Call(GMCallInstruction),
    Break(GMBreakInstruction),
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMCodeVariable {
    pub variable: GMRef<GMVariable>,
    pub variable_type: GMVariableType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GMValue {
    Double(f64),
    Float(f32),
    Int32(i32),
    Int64(i64),
    Boolean(bool),
    String(GMRef<String>),
    Variable(GMCodeVariable),
    Int16(i16),
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMCode {
    pub name: GMRef<String>,
    pub instructions: Vec<GMInstruction>,
    pub bytecode15_info: Option<GMCodeBytecode15>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMCodeBytecode15 {
    pub locals_count: u16,
    pub arguments_count: u16,
    pub weird_local_flag: bool,
    pub offset: usize,
}

#[derive(Debug, Clone)]
pub struct GMCodes {
    pub codes_by_index: Vec<GMCode>,
}

pub fn parse_chunk_code(chunk: &mut GMChunk, bytecode14: bool, strings: &GMStrings, variables: &GMVariables, functions: &GMFunctions) -> Result<GMCodes, String> {
    chunk.cur_pos = 0;
    let codes_count: usize = chunk.read_usize_count()?;
    let mut code_meta_start_positions: Vec<usize> = Vec::with_capacity(codes_count);
    for _ in 0..codes_count {
        let meta_index: usize = chunk.read_usize_pos()? - chunk.abs_pos;
        code_meta_start_positions.push(meta_index);
    }

    let mut codes_by_index: Vec<GMCode> = Vec::with_capacity(codes_count);

    for (i, code_meta_start_position) in code_meta_start_positions.iter().enumerate() {
        chunk.cur_pos = *code_meta_start_position;
        let name: GMRef<String> = chunk.read_gm_string(strings)?;
        let code_length: usize = chunk.read_usize_pos()?;

        let end: usize;
        let bytecode15_info: Option<GMCodeBytecode15> = if bytecode14 {
            end = chunk.cur_pos + code_length;
            None
        } else {
            let locals_count: u16 = chunk.read_u16()?;
            let arguments_count_raw: u16 = chunk.read_u16()?;
            let arguments_count: u16 = arguments_count_raw & 0x7FFF;
            let weird_local_flag: bool = arguments_count_raw & 0x8000 != 0;

            let bytecode_relative_address: i32 = chunk.read_i32()?;
            let bytecode_start_address: usize = (bytecode_relative_address + chunk.cur_pos as i32 - 4) as usize;

            let offset: usize = chunk.read_usize_pos()?;

            // child check {~~}

            chunk.cur_pos = bytecode_start_address;
            end = bytecode_start_address + code_length;

            Some(GMCodeBytecode15 {
                locals_count,
                arguments_count,
                weird_local_flag,
                offset,
            })
        };

        let mut instructions: Vec<GMInstruction> = Vec::new();
        let start: usize = chunk.cur_pos;
        while chunk.cur_pos < end {
            let instruction: GMInstruction = parse_instruction(chunk, bytecode14, variables, functions).map_err(|e| format!(
                "{e} for Instruction #{} (at absolute position {}) of Code #{i}/{codes_count} with name \"{}\" and absolute start position {}",
                instructions.len(), chunk.cur_pos+chunk.abs_pos, name.display(strings), start + chunk.abs_pos,
            ))?;
            instructions.push(instruction);
        }

        codes_by_index.push(GMCode {
            name,
            instructions,
            bytecode15_info,
        })
    }

    Ok(GMCodes { codes_by_index })
}


fn read_code_value(chunk: &mut GMChunk, data_type: GMDataType) -> Result<GMValue, String> {
    match data_type {
        GMDataType::Double => chunk.read_f64().map(|i| GMValue::Double(i)),
        GMDataType::Float => chunk.read_f32().map(|i| GMValue::Float(i)),
        GMDataType::Int32 => chunk.read_i32().map(|i| GMValue::Int32(i)),
        GMDataType::Int64 => chunk.read_i64().map(|i| GMValue::Int64(i)),
        GMDataType::Boolean => chunk.read_u8().map(|i| match i {
            0 => Ok(GMValue::Boolean(false)),
            1 => Ok(GMValue::Boolean(true)),
            other => Err(format!("Invalid boolean value {other} (0x{other:02X}) while reading value in code at absolute position {}", chunk.abs_pos+chunk.cur_pos-1))
        })?,
        GMDataType::String => chunk.read_usize_count().map(|i| GMValue::String(GMRef::new(i))),
        GMDataType::Int16 => {
            chunk.cur_pos -= 4;
            let number: i16 = chunk.read_i16()?;
            chunk.cur_pos += 2;
            Ok(GMValue::Int16(number))
        }
        other => Err(format!("Trying to read unsupported data type {other:?} while reading value in code at absolute position {}", chunk.abs_pos+chunk.cur_pos)),
    }
}

fn read_variable(chunk: &mut GMChunk, variables: &GMVariables, instance_type: &GMInstanceType) -> Result<GMCodeVariable, String> {
    let occurrence_position: usize = chunk.cur_pos;
    let raw_value: i32 = chunk.read_i32()?;

    let variable_type: i32 = (raw_value >> 24) & 0xF8;
    let variable_type: u8 = variable_type as u8;
    let variable_type: GMVariableType = variable_type.try_into()
        .map_err(|_| format!("Invalid Variable Type {variable_type} (0x{variable_type:02X}) while parsing variable reference chain"))?;

    let variable: GMRef<GMVariable> = variables.occurrences_to_refs.get(&occurrence_position)
        .ok_or_else(|| format!(
            "Could not find {} Variable with occurrence position {} in hashmap with length {} while parsing code value",
            instance_type, occurrence_position, variables.occurrences_to_refs.len(),
        ))?.clone();

    Ok(GMCodeVariable { variable, variable_type })
}


pub fn parse_instruction(
    chunk: &mut GMChunk,
    bytecode14: bool,
    variables: &GMVariables,
    functions: &GMFunctions,
) -> Result<GMInstruction, String> {
    let b0: u8 = chunk.read_u8()?;
    let b1: u8 = chunk.read_u8()?;
    let b2: u8 = chunk.read_u8()?;
    let mut opcode_raw: u8 = chunk.read_u8()?;

    if bytecode14 {
        opcode_raw = convert_instruction_kind(opcode_raw);
    }
    let mut opcode: GMOpcode = opcode_raw.try_into()
        .map_err(|_| format!("Invalid Opcode 0x{opcode_raw:02X}"))?;

    let instruction_type: GMInstructionType = get_instruction_type(opcode);
    match instruction_type {
        GMInstructionType::SingleTypeInstruction => {
            let data_type: u8 = b2 & 0xf;
            let data_type: GMDataType = data_type.try_into().map_err(|_| format!(
                "Invalid Data Type {data_type:02X} while parsing Single Type Instruction"
            ))?;

            // Ensure basic conditions hold
            if b0 != 0 && opcode != GMOpcode::Dup && opcode != GMOpcode::CallV {
                return Err(format!("Invalid padding {:02X} while parsing Single Type Instruction", b0));
            }
            if b2 >> 4 != 0 {
                return Err(format!("Second type should be zero but is {0} (0x{0:02X}) for Single Type Instruction", b2 >> 4))
            }

            if b1 != 0 {
                return Err(format!("b1 should be zero but is {b1} (0x{b1:02X}) for Single Type Instruction"))
            }

            Ok(GMInstruction::SingleType(GMSingleTypeInstruction {
                opcode,
                extra: b0,
                data_type,
            }))
        }

        GMInstructionType::DoubleTypeInstruction => {
            let type1: u8 = b2 & 0xf;
            let type1: GMDataType = type1.try_into().map_err(|_| format!(
                "Invalid Data Type {type1:02X} while parsing Double Type Instruction"
            ))?;

            let type2: u8 = b2 >> 4;
            let type2: GMDataType = type2.try_into().map_err(|_| format!(
                "Invalid Data Type {type2:02X} while parsing Double Type Instruction"
            ))?;

            if b1 != 0 {
                return Err(format!("b1 should be zero but is {b1} (0x{b1:02X}) for Double Type Instruction"))
            }

            Ok(GMInstruction::DoubleType(GMDoubleTypeInstruction {
                opcode,
                type1,
                type2,
            }))
        }

        GMInstructionType::ComparisonInstruction => {
            // Parse instruction components from bytes
            let mut comparison_type: GMComparisonType = b1.try_into().map_err(|_| format!(
                "Invalid Comparison Type {b1:02X} while parsing Comparison Instruction"
            ))?;    // TODO probably doesn't work for bytecode14; needs to be checked before

            let type1: u8 = b2 & 0xf;
            let type1: GMDataType = type1.try_into().map_err(|_| format!(
                "Invalid Data Type {type1:02X} while parsing Comparison Instruction"
            ))?;

            let type2: u8 = b2 >> 4;
            let type2: GMDataType = type2.try_into().map_err(|_| format!(
                "Invalid Data Type {type2:02X} while parsing Comparison Instruction"
            ))?;

            if bytecode14 {
                // in bytecode14, the comparison kind is encoded in the opcode
                let comparison_type_raw: u8 = opcode_raw - 0x10;
                comparison_type = comparison_type_raw.try_into()
                    .map_err(|_| format!("Invalid Bytecode14 Comparison Type {comparison_type_raw:02X} \
                        (Opcode: 0x{opcode_raw:02X}) while parsing Comparison Instruction"))?;
            }

            // short circuit stuff {~~}

            Ok(GMInstruction::Comparison(GMComparisonInstruction {
                opcode,
                comparison_type,
                type1,
                type2,
            }))
        }

        GMInstructionType::GotoInstruction => {
            if bytecode14 {
                let jump_offset: i32 = b0 as i32 | ((b1 as u32) << 8) as i32 | ((b2 as i32) << 16);     // yeah idk
                let popenv_exit_magic: bool = jump_offset == -1048576;      // little endian [00 00 F0]
                return Ok(GMInstruction::Goto(GMGotoInstruction {
                    opcode,
                    jump_offset,
                    popenv_exit_magic,
                }));
            }

            let v: u32 = b0 as u32 | ((b1 as u32) << 8) | ((b2 as u32) << 16);      // i hate bitshifting
            let popenv_exit_magic: bool = (v & 0x800000) != 0;
            if popenv_exit_magic && v != 0xF00000 {
                return Err("\"Popenv magic doesn't work\" while parsing Goto Instruction".to_string());
            }
            // The rest is int23 signed value, so make sure (<-- idk what this is supposed to mean)
            let mut jump_offset: u32 = v & 0x003FFFFF;
            if (v & 0x00C00000) != 0 {
                jump_offset |= 0xFFC00000;
            }
            let jump_offset: i32 = jump_offset as i32;

            Ok(GMInstruction::Goto(GMGotoInstruction {
                opcode,
                jump_offset,
                popenv_exit_magic,
            }))
        }

        GMInstructionType::PopInstruction => {
            let type1: u8 = b2 & 0xf;
            let type2: u8 = b2 >> 4;
            let instance_type: i16 = b0 as i16 | ((b1 as i16) << 8);

            let type1: GMDataType = type1.try_into().map_err(|_| format!("Invalid Data Type {type1:02X} while parsing Pop Instruction"))?;
            let type2: GMDataType = type2.try_into().map_err(|_| format!("Invalid Data Type {type2:02X} while parsing Pop Instruction"))?;
            let instance_type: GMInstanceType = parse_instance_type(instance_type)?;

            if type1 == GMDataType::Int16 {
                return Err(format!(
                    "[Internal Error] Unhandled \"Special swap instruction\" (UndertaleModTool/Issues/#129) \
                    occurred at absolute position {} while parsing Pop Instruction.\
                    Please report this error to https://github.com/BioTomateDE/LibGM/issues",
                    chunk.cur_pos + chunk.abs_pos,
                ));
            }

            let destination: GMCodeVariable = read_variable(chunk, variables, &instance_type)?;
            Ok(GMInstruction::Pop(GMPopInstruction {
                opcode,
                instance_type,
                type1,
                type2,
                destination,
            }))
        }

        GMInstructionType::PushInstruction => {
            let data_type: u8 = b2;
            let data_type: GMDataType = data_type.try_into()
                .map_err(|_| format!("Invalid Data Type {data_type:02X} while parsing Push Instruction"))?;

            let val: i16 = (b0 as i16) | ((b1 as i16) << 8);

            if bytecode14 {
                match data_type {
                    GMDataType::Int16 => opcode = GMOpcode::PushI,
                    GMDataType::Variable => {
                        match val {
                            -5 => opcode = GMOpcode::PushGlb,
                            -6 => opcode = GMOpcode::PushBltn,
                            -7 => opcode = GMOpcode::PushLoc,
                            _ => ()
                        }
                    },
                    _ => ()
                }
            }

            let value: GMValue = if data_type == GMDataType::Variable {
                let instance_type: GMInstanceType = parse_instance_type(val)?;
                let variable: GMCodeVariable = read_variable(chunk, variables, &instance_type)?;
                GMValue::Variable(variable)
            } else {
                read_code_value(chunk, data_type)?
            };

            Ok(GMInstruction::Push(GMPushInstruction {
                opcode,
                data_type,
                value,
            }))
        }

        GMInstructionType::CallInstruction => {
            let data_type: u8 = b2;
            let data_type: GMDataType = data_type.try_into().map_err(|_| format!("Invalid Data Type {data_type:02X} while parsing Call Instruction"))?;

            let function: &GMRef<GMFunction> = functions.occurrences_to_refs.get(&chunk.cur_pos).ok_or_else(|| format!(
                "Could not find any function with absolute occurrence position {} in map with length {} (functions len: {}) while parsing Call Instruction",
                chunk.cur_pos, functions.occurrences_to_refs.len(), functions.functions_by_index.len(),
            ))?;
            chunk.cur_pos += 4;

            Ok(GMInstruction::Call(GMCallInstruction {
                opcode,
                arguments_count: b0,
                data_type,
                function: function.clone(),
            }))
        }

        GMInstructionType::BreakInstruction => {
            let value: i16 = b0 as i16 | ((b1 as i16) << 8);
            let data_type: u8 = b2;
            let data_type: GMDataType = data_type.try_into().map_err(|_| format!("Invalid Data Type {data_type:02X} while parsing Break Instruction"))?;

            let int_argument: Option<i32> = if data_type == GMDataType::Int32 {
                Some(chunk.read_i32()?)
                // set gms version to at least ... {~~}
            } else {
                None
            };

            // other set gms version stuff {~~}

            Ok(GMInstruction::Break(GMBreakInstruction {
                opcode,
                value,
                data_type,
                int_argument,
            }))
        }
    }
}


pub fn parse_instance_type(raw_value: i16) -> Result<GMInstanceType, String> {
    // If > 0; then game object id. If < 0, then variable instance type.
    if raw_value > 0 {
        return Ok(GMInstanceType::Instance(Some(GMRef::new(raw_value as usize))))
    }

    let instance_type = match raw_value {
        0 => GMInstanceType::Undefined,         // this doesn't exit in UTMT anymore but enums in C# can hold any value so idk
        -1 => GMInstanceType::Instance(None),
        -2 => GMInstanceType::Other,
        -3 => GMInstanceType::All,
        -4 => GMInstanceType::None,
        -5 => GMInstanceType::Global,
        -6 => GMInstanceType::Builtin,
        -7 => GMInstanceType::Local,
        -9 => GMInstanceType::StackTop,
        -15 => GMInstanceType::Argument,
        -16 => GMInstanceType::Static,
        _ => return Err(format!("Invalid instance type {raw_value} (0x{raw_value:04X})"))
    };

    Ok(instance_type)
}


pub fn parse_occurrence_chain(
    chunk_code: &mut GMChunk,
    general_info: &GMGeneralInfo,
    gm_name: &str,
    first_occurrence_abs_pos: i32,
    occurrence_count: usize,
    is_function: bool,
) -> Result<(Vec<usize>, i32), String> {
    if occurrence_count < 1 {
        return Ok((vec![], first_occurrence_abs_pos));
    }
    // FIXME: in UTMT, this is checked using gmversion 2.3 (not 2.0!); but deltarune 1.00 (?) has gmversion 2.0.0.0 and needs the zero offset.
    // if are any errors regarding function parsing, this is probably the cause
    let initial_offset: i32 = if is_function && general_info.is_version_at_least(2, 0, 0, 0) {
        0
    } else {
        4
    };
    let occurrence_pos: i32 = first_occurrence_abs_pos - chunk_code.abs_pos as i32 + initial_offset;
    let mut occurrence_pos: usize = occurrence_pos.try_into()
        .map_err(|_| format!(
            "First occurrence of {} \"{}\" is out of bounds; should be: {} <= {} < {}",
            if is_function {"function"} else {"variable"}, gm_name, chunk_code.abs_pos, first_occurrence_abs_pos, chunk_code.abs_pos + chunk_code.data.len(),
        ))?;

    let mut occurrences: Vec<usize> = Vec::with_capacity(occurrence_count);
    let mut offset: i32 = first_occurrence_abs_pos;

    for _ in 0..occurrence_count {
        occurrences.push(occurrence_pos);
        chunk_code.cur_pos = occurrence_pos;
        offset = read_occurrence(chunk_code, is_function)?;
        occurrence_pos += offset as usize;
    }

    let name_string_id: i32 = offset & 0xFFFFFF;

    Ok((occurrences, name_string_id))
}


fn read_occurrence(chunk_code: &mut GMChunk, is_function: bool) -> Result<i32, String> {
    let raw_value: i32 = chunk_code.read_i32()?;
    let next_occurrence_offset: i32 = raw_value & 0x07FFFFFF;
    if next_occurrence_offset < 1 {
        return Err(format!(
            "Next occurrence offset is {0} (0x{0:08X}) which is not a positive number while parsing \
            {3} occurrences at absolute position {1} (raw value is 0x{2:08X})",
            next_occurrence_offset, chunk_code.cur_pos-4, raw_value, if is_function {"function"} else {"variable"},
        ))
    }
    Ok(next_occurrence_offset)
}

