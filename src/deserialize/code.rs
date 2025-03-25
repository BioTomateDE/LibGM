use crate::chunk_reading::UTChunk;
use crate::deserialize::variables::UTVariable;

use colored::Colorize;
use std::cmp::PartialEq;
use std::collections::HashSet;
use num_enum::TryFromPrimitive;
use crate::deserialize::functions::{get_function, UTFunction};
use crate::deserialize::strings::UTStrings;
use crate::printing::hexdump;

// Taken from UndertaleModTool/UndertaleModLib/UndertaleCode.cs/UndertaleInstruction/
#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
enum UTOpcode {
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
enum UTInstructionType {
    SingleTypeInstruction,
    DoubleTypeInstruction,
    ComparisonInstruction,
    GotoInstruction,
    PushInstruction,
    PopInstruction,
    CallInstruction,
    BreakInstruction,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
enum UTDataType {
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
#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive)]
#[repr(i8)]
enum UTInstanceType {
    Undefined = 0, // actually, this is just object 0, but also occurs in places where no instance type was set
    Self_ = -1,
    Other = -2,
    All = -3,
    Noone = -4,
    Global = -5,
    Builtin = -6, // Note: Used only in UndertaleVariable.VarID (which is not really even InstanceType)
    Local = -7,
    Stacktop = -9,
    Arg = -15,
    Static = -16,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
enum UTVariableType {
    Array = 0x00,
    StackTop = 0x80,
    Normal = 0xA0,
    Instance = 0xE0,    // the InstanceType is an instance ID inside the room -100000
    ArrayPushAF = 0x10, // GMS2.3+, multidimensional array with pushaf
    ArrayPopAF = 0x90,  // GMS2.3+, multidimensional array with pushaf or popaf
}
#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
enum UTComparisonType {
    DUP = 0,    // custom
    LT = 1,
    LTE = 2,
    EQ = 3,
    NEQ = 4,
    GTE = 5,
    GT = 6,
}

fn get_instruction_type(opcode: UTOpcode) -> UTInstructionType {
    match opcode {
        UTOpcode::Neg
        | UTOpcode::Not
        | UTOpcode::Dup
        | UTOpcode::Ret
        | UTOpcode::Exit
        | UTOpcode::Popz
        | UTOpcode::CallV => UTInstructionType::SingleTypeInstruction,
        UTOpcode::Conv
        | UTOpcode::Mul
        | UTOpcode::Div
        | UTOpcode::Rem
        | UTOpcode::Mod
        | UTOpcode::Add
        | UTOpcode::Sub
        | UTOpcode::And
        | UTOpcode::Or
        | UTOpcode::Xor
        | UTOpcode::Shl
        | UTOpcode::Shr => UTInstructionType::DoubleTypeInstruction,
        UTOpcode::Cmp => UTInstructionType::ComparisonInstruction,
        UTOpcode::B | UTOpcode::Bt | UTOpcode::Bf | UTOpcode::PushEnv | UTOpcode::PopEnv => {
            UTInstructionType::GotoInstruction
        }

        UTOpcode::Pop => UTInstructionType::PopInstruction,
        UTOpcode::Push
        | UTOpcode::PushLoc
        | UTOpcode::PushGlb
        | UTOpcode::PushBltn
        | UTOpcode::PushI => UTInstructionType::PushInstruction,

        UTOpcode::Call => UTInstructionType::CallInstruction,
        UTOpcode::Break => UTInstructionType::BreakInstruction,
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct UTComparisonInstruction {
    // extra: u8,                              // extra byte that should be zero
    comparison_type: UTComparisonType, // comparison kind
    type1: UTDataType,                 // datatype of element to compare
    type2: UTDataType,                 // datatype of element to compare
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct UTGotoInstruction {
    jump_offset: i32,
    popenv_exit_magic: bool,
}
#[derive(Debug, Clone)]
struct UTPopInstruction {
    instance_type: UTInstanceType,
    type1: UTDataType,
    type2: UTDataType,
    destination: UTCodeVariable,
}
#[derive(Debug, Clone)]
struct UTPushInstruction {
    data_type: UTDataType,
    value: UTValue,
}
#[derive(Debug, Clone)]
struct UTCallInstruction {
    arguments_count: usize,
    data_type: UTDataType,
    function: UTFunction,
}
#[derive(Debug, Clone)]
struct UTBreakInstruction {
    value: i16,
    data_type: UTDataType,
    int_argument: Option<i32>,
}
#[derive(Debug, Clone)]
enum UTInstruction {
    Cmp(UTComparisonInstruction),
    Goto(UTGotoInstruction),
    Pop(UTPopInstruction),
    Push(UTPushInstruction),
    Call(UTCallInstruction),
    Break(UTBreakInstruction),
}

#[derive(Debug, Clone)]
enum UTCodeVariable {
    Var(UTVariable, UTVariableType),
    Unknown(usize, UTVariableType)
}



#[derive(Debug, Clone)]
enum UTValue {
    Double(f64),
    Float(f32),
    Int32(i32),
    Int64(i64),
    Boolean(bool),
    Variable(UTCodeVariable),
    String(String),
    Int16(i16),
}

#[derive(Debug)]
struct UTCodeMeta {
    name: String,
    start_position: usize, // start position of code in chunk CODE
    length: usize,
    locals_count: u32,
    arguments_count: u32,
}

#[derive(Debug)]
pub struct UTCode {
    pub name: String,
    pub instructions: Vec<UTInstruction>,
    pub locals_count: u32,
    pub arguments_count: u32,
}


// wrapper for raw data of a code "script" / instance
struct UTCodeBlob {
    raw_data: Vec<u8>,
    len: usize,
    file_index: usize,
}

impl UTCodeBlob {
    fn read_byte(&mut self) -> Result<u8, String> {
        if self.file_index + 1 > self.len {
            return Err(format!(
                "Trying to read u8 out of bounds while parsing code at position {}: {} > {}.",
                self.file_index,
                self.file_index + 1,
                self.len,
            ));
        }
        let byte: u8 = self.raw_data[self.file_index];
        self.file_index += 1;
        Ok(byte)
    }

    fn read_value(
        &mut self,
        data_type: UTDataType,
        strings: &UTStrings,
        variables: &[UTVariable],
    ) -> Result<UTValue, String> {
        match data_type {
            UTDataType::Double => {
                let raw: [u8; 8] = match self.raw_data[self.file_index..self.file_index+8].try_into() {
                    Ok(ok) => ok,
                    Err(_) => return Err("Trying to read f64 out of bounds while reading values in code.".to_string()),
                };
                self.file_index += 8;
                Ok(UTValue::Double(f64::from_le_bytes(raw)))
            },

            UTDataType::Float => {
                let raw: [u8; 4] = match self.raw_data[self.file_index..self.file_index+4].try_into() {
                    Ok(ok) => ok,
                    Err(_) => return Err("Trying to read f32 out of bounds while reading values in code.".to_string()),
                };
                self.file_index += 4;
                Ok(UTValue::Float(f32::from_le_bytes(raw)))
            },

            UTDataType::Int32 => {
                let raw: [u8; 4] = match self.raw_data[self.file_index..self.file_index+4].try_into() {
                    Ok(ok) => ok,
                    Err(_) => return Err("Trying to read i32 out of bounds while reading values in code.".to_string()),
                };
                self.file_index += 4;
                Ok(UTValue::Int32(i32::from_le_bytes(raw)))
            },

            UTDataType::Int64 => {
                let raw: [u8; 8] = match self.raw_data[self.file_index..self.file_index+8].try_into() {
                    Ok(ok) => ok,
                    Err(_) => return Err("Trying to read i64 out of bounds while reading values in code.".to_string()),
                };
                self.file_index += 8;
                Ok(UTValue::Int64(i64::from_le_bytes(raw)))
            },

            UTDataType::Boolean => {
                if self.raw_data.len() < 1 {
                    return Err("Trying to read boolean out of bounds while reading values in code.".to_string());
                }
                self.file_index += 1;
                Ok(UTValue::Boolean(self.raw_data[0] != 0))
            },

            UTDataType::Variable => {
                let raw: [u8; 4] = match self.raw_data[self.file_index..self.file_index+4].try_into() {
                    Ok(ok) => ok,
                    Err(_) => return Err("Trying to read UTVariable out of bounds while reading values in code.".to_string()),
                };
                self.file_index += 4;
                let raw_index: [u8; 2] = raw[0..2].try_into().unwrap();
                let raw_variable_type: u8 = raw[3];
                let index: usize = u16::from_le_bytes(raw_index) as usize;
                let variable_type: UTVariableType = match raw_variable_type.try_into() {
                    Ok(ok) => ok,
                    Err(_) => return Err(format!(
                        "Invalid Variable Type {:02X} while reading values in code.",
                        raw_variable_type
                    ))
                };

                // TODO deal with variable ids and scopes asfbjhiafshasf (var index is wrong)

                return Ok(UTValue::Variable(UTCodeVariable::Unknown{ 0: index, 1: variable_type }));

                // let variable: UTVariable = match variables.get(index) {
                //     Some(var) => var.clone(),
                //     // None => return Err(format!(
                //     //     "UTVariable index is out of bounds while reading values in code: {} >= {}.",
                //     //     index,
                //     //     variables.len()
                //     // ))
                //     None => {
                //         eprintln!("WARNING: Could not find variable with index {} (length: {}).", index, variables.len());
                //         return Ok(UTValue::Variable(UTCodeVariable::Unknown{ 0: index, 1: variable_type }));
                //     }
                // };
                // let code_variable: UTCodeVariable = UTCodeVariable::Var{ 0: variable, 1: variable_type };
                // Ok(UTValue::Variable(code_variable))
            },

            UTDataType::String => {
                // idk if it's position or string id
                let raw: [u8; 4] = match self.raw_data[self.file_index..self.file_index+4].try_into() {
                    Ok(ok) => ok,
                    Err(_) => return Err("Trying to read UTString out of bounds while reading values in code.".to_string()),
                };
                let string_index: usize = u32::from_le_bytes(raw) as usize;
                self.file_index += 4;
                match strings.get_string_by_index(string_index) {
                    Some(string) => Ok(UTValue::String(string.clone())),
                    None => Err(format!("Could not find UTString with String Index {} while reading values in code.", string_index))
                }
            },

            UTDataType::Int16 => {
                // i think it's within the instruction itself so backtrack
                let raw: [u8; 2] = match self.raw_data[self.file_index-4 .. self.file_index-2].try_into() {
                    Ok(ok) => ok,
                    Err(_) => return Err("Trying to read i16 out of bounds while reading values in code.".to_string()),
                };
                Ok(UTValue::Int16(i16::from_le_bytes(raw)))
            },

            _ => Err(format!("Trying to read unsupported data type {0:?} while reading values in code.", data_type)),
        }
    }
}


pub fn parse_chunk_CODE(
    mut chunk: UTChunk,
    bytecode14: bool,
    strings: &UTStrings,
    variables: &[UTVariable],
    functions: &[UTFunction],
) -> Result<Vec<UTCode>, String> {
    let codes_count: usize = chunk.read_usize()?;
    let mut code_meta_indexes: Vec<usize> = Vec::with_capacity(codes_count);
    for _ in 0..codes_count {
        let meta_index: usize = chunk.read_usize()? - chunk.abs_pos;
        code_meta_indexes.push(meta_index);
        // if i > 0 {
        //     println!("{} {}", code_id, code_id - code_ids[i-1]);
        // }
    }

    // let old_pos: usize = chunk.file_index;

    let mut code_metas: Vec<UTCodeMeta> = Vec::with_capacity(codes_count);

    for ts in code_meta_indexes {
        chunk.file_index = ts;
        let code_name: String = chunk.read_ut_string(strings)?;
        let code_length: usize = chunk.read_usize()?;
        let locals_count: u32 = chunk.read_u32()?;

        let start_offset: i32 = chunk.read_i32()?;
        let start_position: i32 = chunk.file_index as i32 + start_offset - 4;
        if start_position < 0 || start_position >= chunk.data_len as i32 {
            return Err(format!(
                "Code starting offset out of bounds \
                at position {} while parsing chunk 'CODE': \
                Offset {} corresponds to chunk position {} \
                which is not 0 <= {} < {}.",
                chunk.file_index, start_offset, start_position, start_position, chunk.data_len
            ));
        }
        let start_position: usize = start_position as usize;

        let arguments_count: u32 = chunk.read_u32()?;
        // println!("{:<16} {:<54} | {:<8} {:<6} {:<14} {:<3} {}", ts, code_name, code_length, locals_count, start_offset, arguments_count, start_position);
        code_metas.push(UTCodeMeta {
            name: code_name,
            start_position,
            length: code_length,
            locals_count,
            arguments_count,
        })
    }

    let mut codes: Vec<UTCode> = Vec::with_capacity(codes_count);
    for code_meta in code_metas {
        let raw_data: Vec<u8> = chunk.data[code_meta.start_position..code_meta.start_position + code_meta.length].to_owned();
        let mut code_blob: UTCodeBlob = UTCodeBlob {
            raw_data: raw_data.clone(),
            len: raw_data.len(),
            file_index: 0,
        };
        let mut instructions: Vec<UTInstruction> = vec![];

        while code_blob.file_index < code_blob.len {
            let instruction: UTInstruction = parse_code(&mut code_blob, bytecode14, &strings, variables, functions, code_meta.start_position-8)?;
            let dump: String = match hexdump(&*code_blob.raw_data, code_blob.file_index-4, Some(code_blob.file_index)) {
                Ok(ok) => ok,
                Err(_) => "()".to_string(),
            };
            // println!("{} | {}/{} | {} | {:?}",
            //     code_meta.name,
            //     code_blob.len,
            //     code_blob.file_index,
            //     dump,
            //     instruction,
            // );
            instructions.push(instruction);
        }

        codes.push(UTCode {
            name: code_meta.name,
            instructions,
            locals_count: code_meta.locals_count,
            arguments_count: code_meta.arguments_count,
        });
    }

    Ok(codes)
}

fn parse_code(
    blob: &mut UTCodeBlob,
    bytecode14: bool,
    strings: &UTStrings,
    variables: &[UTVariable],
    functions: &[UTFunction],
    code_start_pos: usize
) -> Result<UTInstruction, String> {
    let b0: u8 = blob.read_byte()?;
    let b1: u8 = blob.read_byte()?;
    let b2: u8 = blob.read_byte()?;
    let opcode_raw: u8 = blob.read_byte()?;

    let mut opcode: UTOpcode = match opcode_raw.try_into() {
        Ok(opcode) => opcode,
        Err(_) => return Err(format!("Invalid Opcode {opcode_raw:02X} while parsing code.")),
    };
    // if Bytecode14OrLower {
    //     let kind: u8 = convert_instruction_kind(kind);
    // }

    let instruction_type: UTInstructionType = get_instruction_type(opcode);
    match instruction_type {
        UTInstructionType::SingleTypeInstruction |
        UTInstructionType::DoubleTypeInstruction |
        UTInstructionType::ComparisonInstruction => {
            // Parse instruction components from bytes
            let mut comparison_type: UTComparisonType = match b1.try_into() {
                Ok(comparison_type) => comparison_type,
                Err(_) => {
                    return Err(format!(
                        "Invalid Comparison Type {b1:02X} while parsing Comparison Instruction."
                    ));
                }
            };
            let type1: u8 = b2 & 0xf;
            let type1: UTDataType = match type1.try_into() {
                Ok(data_type) => data_type,
                Err(_) => return Err(format!(
                    "Invalid Data Type {type1:02X} while parsing Comparison Instruction."
                )),

            };
            let type2: u8 = b2 >> 4;
            let type2: UTDataType = match type2.try_into() {
                Ok(data_type) => data_type,
                Err(_) => return Err(format!(
                    "Invalid Data Type {type2:02X} while parsing Comparison Instruction."
                )),
            };
            // Ensure basic conditions hold
            if b0 != 0 && opcode != UTOpcode::Dup && opcode != UTOpcode::CallV {
                return Err(format!("Invalid padding {:02X} while parsing Comparison Instruction.", b0));
            }

            if instruction_type == UTInstructionType::SingleTypeInstruction && (type2 as u8) != 0 {
                return Err(format!(
                    "Second type should be 0 but is {:02X} in \
                    SingleTypeInstruction while parsing Comparison Instruction",
                    type2 as u8
                ));
            }

            if bytecode14 && opcode == UTOpcode::Cmp {
                let comparison_type_raw: u8 = opcode_raw - 0x10;
                comparison_type = match comparison_type_raw.try_into() {
                    Ok(comparison_type) => comparison_type,
                    Err(_) => return Err(format!("Invalid Bytecode14 Comparison Type \
                    {comparison_type_raw:02X} (Opcode: {opcode_raw:02X}) while parsing Comparison Instruction.")),
                };
            }
            // short circuit stuff {}

            Ok(UTInstruction::Cmp(UTComparisonInstruction {
                comparison_type,
                type1,
                type2,
            }))
        }

        UTInstructionType::GotoInstruction => {
            if bytecode14 {
                let jump_offset: i32 = b0 as i32;
                let popenv_exit_magic: bool = jump_offset == -1048576;
                return Ok(UTInstruction::Goto(UTGotoInstruction {
                    jump_offset,
                    popenv_exit_magic,
                }));
            }

            let v: u32 = b0 as u32;   // redundancy in UndertaleModTool again?? or i don't understand c#
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

            Ok(UTInstruction::Goto(UTGotoInstruction {
                jump_offset,
                popenv_exit_magic,
            }))
        }

        UTInstructionType::PopInstruction => {
            // bug/redundancy in UndertaleModTool i think (bitshifting by 8 in a u8 will always be 0)
            let type1: u8 = b2 & 0xf;
            let type1: UTDataType = match type1.try_into() {
                Ok(ok) => ok,
                Err(_) => return Err(format!("Invalid Data Type {type1:02X} while parsing Pop Instruction.")),
            };

            let type2: u8 = b2 >> 4;
            let type2: UTDataType = match type2.try_into() {
                Ok(ok) => ok,
                Err(_) => return Err(format!("Invalid Data Type {type2:02X} while parsing Pop Instruction.")),
            };

            let instance_type: i8 = b0 as i8;
            let mut instance_type: UTInstanceType = match instance_type.try_into() {
                Ok(ok) => ok,
                // Err(_) => return Err(format!("Invalid Instance Type {instance_type:02X} while parsing Pop Instruction.")),
                Err(_) => {
                    println!("{}", format!("[WARNING] Invalid Instance Type {instance_type:02X} while parsing Pop Instruction. Value: {b0:02X} {b1:02X}").yellow());
                    UTInstanceType::Undefined
                }
            };

            // if type1 == UTDataType::Variable && type2 == UTDataType::Int32 {
            //     panic!("{} {b0:02X} {b1:02X} {b2:02X} {opcode_raw:02X} | {:02X} {:02X} {:02X} {:02X}", blob.file_index, blob.raw_data[blob.file_index+1],blob.raw_data[blob.file_index+2],blob.raw_data[blob.file_index+3],blob.raw_data[blob.file_index+4])
            // }

            // if type1 == UTDataType::Int16 {
            //     // Special scenario - the swap instruction (see UndertaleModTool/Issues/#129)
            //     let swap_extra: u16 = instance_type;
            //     instance_type = UTInstanceType::Undefined;
            // }
            // else
            // {
            //     // Destination is an actual variable
            //     let destination = readvaruiable();
            // }

            let destination: UTCodeVariable = match blob.read_value(UTDataType::Variable, strings, variables)? {
                UTValue::Variable(var) => var,
                _ => return Err("[INTERNAL ERROR] UTCodeBlob.read_value(UTDataType::Variable, ...)\
                 did not return a UTVariable while parsing Pop Instruction".to_string()),
            };

            Ok(UTInstruction::Pop(UTPopInstruction {
                instance_type,
                type1,
                type2,
                destination,
            }))
        }

        UTInstructionType::PushInstruction => {
            let data_type: u8 = b2;
            let data_type: UTDataType = match data_type.try_into() {
                Ok(ok) => ok,
                Err(_) => return Err(format!("Invalid Data Type {data_type:02X} while parsing Push Instruction.")),
            };
            // let value: i16 = b0 as i16;  // typecasting might be wrong
            let val: i16 = (b0 as i8) as i16;

            if bytecode14 {
                // println!("############# {:?} {:?}", data_type, val);
                match data_type {
                    UTDataType::Int16 => opcode = UTOpcode::PushI,
                    UTDataType::Variable => {
                        match val {
                            -5 => opcode = UTOpcode::PushGlb,
                            -6 => opcode = UTOpcode::PushBltn,
                            -7 => opcode = UTOpcode::PushLoc,
                            _ => ()
                        }
                    },
                    _ => ()
                }
            }

            // todo fix bullshit variable id

            let mut value: UTValue = blob.read_value(data_type, strings, variables)?;
            // println!("$$$$$ {:?}", value);

            Ok(UTInstruction::Push(UTPushInstruction {
                data_type,
                value,
            }))
        }

        UTInstructionType::CallInstruction => {
            let arguments_count: usize = b0 as usize;
            let data_type: u8 = b2;
            let data_type: UTDataType = match data_type.try_into() {
                Ok(ok) => ok,
                Err(_) => return Err(format!("Invalid Data Type {data_type:02X} while parsing Call Instruction.")),
            };

            blob.file_index += 4;
            let function: UTFunction = get_function(functions, code_start_pos + blob.file_index)?;

            Ok(UTInstruction::Call(UTCallInstruction {
                arguments_count,
                data_type,
                function,
            }))
        }

        UTInstructionType::BreakInstruction => {
            let value: i16 = b0 as i16;
            let data_type: u8 = b2;
            let data_type: UTDataType = match data_type.try_into() {
                Ok(ok) => ok,
                Err(_) => return Err(format!("Invalid Data Type {data_type:02X} while parsing Break Instruction.")),
            };
            let mut int_argument: Option<i32> = None;

            if data_type == UTDataType::Int32 {
                int_argument = Some(match blob.read_value(UTDataType::Int32, &strings, &variables)? {
                    UTValue::Int32(val) => val,
                    _ => return Err("[INTERNAL ERROR] UTCodeBlob.read_value(UTDataType::Int32, ...)\
                    did not return an i32 while parsing Break Instruction".to_string()),
                });
                // gms version stuff {}
            }

            // other gms version stuff {}

            Ok(UTInstruction::Break(UTBreakInstruction {
                value,
                data_type,
                int_argument,
            }))
        }

        _ => {
            // DO PROPER ERROR HANDLING
            panic!("[INTERNAL ERROR] Unhandled opcode {opcode_raw:02X}. (This should NOT happen in Release)");
        },
    }
}


// TODO:
// variables bullshit (scopes, ids)
// functions bullshit (look in npp; nothing makes fucking sense)
// weird error (didnt investigate yet) "Invalid Instance Type A5 while parsing Pop Instruction."
