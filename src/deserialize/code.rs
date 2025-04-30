use crate::deserialize::chunk_reading::GMRef;
use crate::deserialize::chunk_reading::GMChunk;
use crate::deserialize::variables::{GMVariable, GMVariables};
use std::cmp::PartialEq;
use std::env::var;
use num_enum::TryFromPrimitive;
use crate::deserialize::functions::{GMFunction, GMFunctions};
use crate::deserialize::game_objects::GMGameObject;
use crate::deserialize::strings::GMStrings;

// Taken from UndertaleModTool/UndertaleModLib/UndertaleCode.cs/UndertaleInstruction/
#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
enum GMOpcode {
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

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
enum GMDataType {
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
#[derive(Debug, Clone)]
pub enum GMInstanceType {
    Undefined,  // actually, this is just object 0, but also occurs in places where no instance type was set
    Self_(Option<GMRef<GMGameObject>>),
    Other,
    All,
    Noone,
    Global,
    Local,
    Stacktop,
    Arg,
    Static,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
enum GMVariableType {
    Array = 0x00,
    StackTop = 0x80,
    Normal = 0xA0,
    Instance = 0xE0,    // the InstanceType is an instance ID inside the room -100000
    ArrayPushAF = 0x10, // GMS2.3+, multidimensional array with pushaf
    ArrayPopAF = 0x90,  // GMS2.3+, multidimensional array with pushaf or popaf
}
#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
enum GMComparisonType {
    DUP = 0,    // custom
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct GMComparisonInstruction {
    // extra: u8,                           // extra byte that should be zero
    pub comparison_type: GMComparisonType,  // comparison kind
    pub type1: GMDataType,                  // datatype of element to compare
    pub type2: GMDataType,                  // datatype of element to compare
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct GMGotoInstruction {
    pub opcode: GMOpcode,
    pub jump_offset: i32,
    pub popenv_exit_magic: bool,
}
#[derive(Debug, Clone)]
pub struct GMPopInstruction {
    pub opcode: GMOpcode,
    pub instance_type: GMInstanceType,
    pub type1: GMDataType,
    pub type2: GMDataType,
    pub destination: GMCodeVariable,
}
#[derive(Debug, Clone)]
pub struct GMPushInstruction {
    pub opcode: GMOpcode,
    pub data_type: GMDataType,
    pub value: GMValue,
}
#[derive(Debug, Clone)]
pub struct GMCallInstruction {
    pub opcode: GMOpcode,
    pub arguments_count: usize,
    pub data_type: GMDataType,
    pub function: GMRef<GMFunction>,
}
#[derive(Debug, Clone)]
pub struct GMBreakInstruction {
    pub opcode: GMOpcode,
    pub value: i16,
    pub data_type: GMDataType,
    pub int_argument: Option<i32>,
}
#[derive(Debug, Clone)]
pub enum GMInstruction {
    Cmp(GMComparisonInstruction),
    Goto(GMGotoInstruction),
    Pop(GMPopInstruction),
    Push(GMPushInstruction),
    Call(GMCallInstruction),
    Break(GMBreakInstruction),
}

#[derive(Debug, Clone)]
pub struct GMCodeVariable {
    variable: GMRef<GMVariable>,
    variable_type: GMVariableType,
}


#[derive(Debug, Clone)]
enum GMValue {
    Double(f64),
    Float(f32),
    Int32(i32),
    Int64(i64),
    Boolean(bool),
    String(GMRef<String>),
    Int16(i16),
}

#[derive(Debug)]
struct GMCodeMeta {
    name: GMRef<String>,
    start_position: usize, // start position of code in chunk CODE
    length: usize,
    locals_count: u32,
    arguments_count: u32,
}

#[derive(Debug, Clone)]
pub struct GMCode {
    pub name: GMRef<String>,
    pub instructions: Vec<GMInstruction>,
    pub locals_count: u32,
    pub arguments_count: u32,
}


// wrapper for raw data of a code "script" / instance
pub struct GMCodeBlob {
    pub raw_data: Vec<u8>,
    pub len: usize,
    pub file_index: usize,
}

impl GMCodeBlob {
    fn read_byte(&mut self) -> Result<u8, String> {
        if self.file_index + 1 > self.len {
            return Err(format!(
                "Trying to read u8 out of bounds while parsing code at position {}: {} > {}.",
                self.file_index, self.file_index + 1, self.len,
            ));
        }
        let byte: u8 = self.raw_data[self.file_index];
        self.file_index += 1;
        Ok(byte)
    }

    fn read_value(&mut self, data_type: GMDataType) -> Result<GMValue, String> {
        match data_type {
            GMDataType::Double => {
                let raw: [u8; 8] = match self.raw_data[self.file_index..self.file_index+8].try_into() {
                    Ok(ok) => ok,
                    Err(_) => return Err("Trying to read f64 out of bounds while reading values in code.".to_string()),
                };
                self.file_index += 8;
                Ok(GMValue::Double(f64::from_le_bytes(raw)))
            },

            GMDataType::Float => {
                let raw: [u8; 4] = match self.raw_data[self.file_index..self.file_index+4].try_into() {
                    Ok(ok) => ok,
                    Err(_) => return Err("Trying to read f32 out of bounds while reading values in code.".to_string()),
                };
                self.file_index += 4;
                Ok(GMValue::Float(f32::from_le_bytes(raw)))
            },

            GMDataType::Int32 => {
                let raw: [u8; 4] = match self.raw_data[self.file_index..self.file_index+4].try_into() {
                    Ok(ok) => ok,
                    Err(_) => return Err("Trying to read i32 out of bounds while reading values in code.".to_string()),
                };
                self.file_index += 4;
                Ok(GMValue::Int32(i32::from_le_bytes(raw)))
            },

            GMDataType::Int64 => {
                let raw: [u8; 8] = match self.raw_data[self.file_index..self.file_index+8].try_into() {
                    Ok(ok) => ok,
                    Err(_) => return Err("Trying to read i64 out of bounds while reading values in code.".to_string()),
                };
                self.file_index += 8;
                Ok(GMValue::Int64(i64::from_le_bytes(raw)))
            },

            GMDataType::Boolean => {
                if self.raw_data.len() < 1 {
                    return Err("Trying to read boolean out of bounds while reading values in code.".to_string());
                }
                self.file_index += 1;
                Ok(GMValue::Boolean(self.raw_data[0] != 0))
            },

            GMDataType::String => {
                // idk if it's position or string id
                let raw: [u8; 4] = match self.raw_data[self.file_index..self.file_index+4].try_into() {
                    Ok(ok) => ok,
                    Err(_) => return Err("Trying to read GMString out of bounds while reading values in code.".to_string()),
                };
                let string_index: usize = u32::from_le_bytes(raw) as usize;
                self.file_index += 4;
                Ok(GMValue::String(GMRef::new(string_index)))
            },

            GMDataType::Int16 => {
                // i think it's within the instruction itself so backtrack
                let raw: [u8; 2] = match self.raw_data[self.file_index-4 .. self.file_index-2].try_into() {
                    Ok(ok) => ok,
                    Err(_) => return Err("Trying to read i16 out of bounds while reading values in code.".to_string()),
                };
                Ok(GMValue::Int16(i16::from_le_bytes(raw)))
            },

            _ => Err(format!("Trying to read unsupported data type {0:?} while reading values in code.", data_type)),
        }
    }

    fn read_variable(&mut self, variables: &GMVariables) -> Result<GMCodeVariable, String> {
        let raw: [u8; 4] = match self.raw_data[self.file_index..self.file_index+4].try_into() {
            Ok(ok) => ok,
            Err(_) => return Err("Trying to read GMVariable out of bounds while reading values in code.".to_string()),
        };
        self.file_index += 4;
        let raw_index: [u8; 2] = raw[0..2].try_into().unwrap();
        let raw_variable_type: u8 = raw[3];
        let index: usize = u16::from_le_bytes(raw_index) as usize;
        let variable_type: GMVariableType = match raw_variable_type.try_into() {
            Ok(ok) => ok,
            Err(_) => return Err(format!(
                "Invalid Variable Type {:02X} while reading values in code.",
                raw_variable_type
            ))
        };

        // TODO deal with variable ids and scopes asfbjhiafshasf (var index is wrong)

        Ok(GMCodeVariable{ variable: GMRef::new(99999999963299999), variable_type })

        // let variable: GMVariable = match variables.get(index) {
        //     Some(var) => var.clone(),
        //     // None => return Err(format!(
        //     //     "GMVariable index is out of bounds while reading values in code: {} >= {}.",
        //     //     index,
        //     //     variables.len()
        //     // ))
        //     None => {
        //         eprintln!("WARNING: Could not find variable with index {} (length: {}).", index, variables.len());
        //         return Ok(GMValue::Variable(GMCodeVariable::Unknown{ 0: index, 1: variable_type }));
        //     }
        // };
        // let code_variable: GMCodeVariable = GMCodeVariable::Var{ 0: variable, 1: variable_type };
        // Ok(GMValue::Variable(code_variable))
    }
}


pub fn parse_chunk_code(
    chunk: &mut GMChunk,
    bytecode14: bool,
    strings: &GMStrings,
    variables: &GMVariables,
    functions: &GMFunctions,
) -> Result<Vec<GMCode>, String> {
    chunk.cur_pos = 0;
    let codes_count: usize = chunk.read_usize()?;
    let mut code_meta_indexes: Vec<usize> = Vec::with_capacity(codes_count);
    for _ in 0..codes_count {
        let meta_index: usize = chunk.read_usize()? - chunk.abs_pos;
        code_meta_indexes.push(meta_index);
    }

    let mut code_metas: Vec<GMCodeMeta> = Vec::with_capacity(codes_count);

    for ts in code_meta_indexes {
        chunk.cur_pos = ts;
        let code_name: GMRef<String> = chunk.read_gm_string(strings)?;
        let code_length: usize = chunk.read_usize()?;
        let locals_count: u32 = chunk.read_u32()?;

        let start_offset: i32 = chunk.read_i32()?;
        let start_position: i32 = chunk.cur_pos as i32 + start_offset - 4;
        if start_position < 0 || start_position >= chunk.data.len() as i32 {
            return Err(format!(
                "Code starting offset out of bounds \
                at position {} while parsing chunk 'CODE': \
                Offset {} corresponds to chunk position {} \
                which is not 0 <= {} < {}.",
                chunk.cur_pos, start_offset, start_position, start_position, chunk.data.len()
            ));
        }
        let start_position: usize = start_position as usize;

        let arguments_count: u32 = chunk.read_u32()?;
        // println!("{:<16} {:<54} | {:<8} {:<6} {:<14} {:<3} {}", ts, code_name, code_length, locals_count, start_offset, arguments_count, start_position);
        code_metas.push(GMCodeMeta {
            name: code_name,
            start_position,
            length: code_length,
            locals_count,
            arguments_count,
        })
    }

    let mut codes: Vec<GMCode> = Vec::with_capacity(codes_count);
    for code_meta in code_metas {
        let raw_data: Vec<u8> = chunk.data[code_meta.start_position..code_meta.start_position + code_meta.length].to_owned();
        let mut code_blob: GMCodeBlob = GMCodeBlob {
            raw_data: raw_data.clone(),
            len: raw_data.len(),
            file_index: 0,
        };
        let mut instructions: Vec<GMInstruction> = vec![];

        while code_blob.file_index < code_blob.len {
            let instruction: GMInstruction = parse_instruction(&mut code_blob, bytecode14, variables, functions, code_meta.start_position-8)?;
            // let dump: String = match hexdump(&*code_blob.raw_data, code_blob.file_index-4, Some(code_blob.file_index)) {
            //     Ok(ok) => ok,
            //     Err(_) => "()".to_string(),
            // };
            // println!("{} | {}/{} | {} | {:?}",
            //     code_meta.name,
            //     code_blob.len,
            //     code_blob.file_index,
            //     dump,
            //     instruction,
            // );
            instructions.push(instruction);
        }

        codes.push(GMCode {
            name: code_meta.name,
            instructions,
            locals_count: code_meta.locals_count,
            arguments_count: code_meta.arguments_count,
        });
    }

    Ok(codes)
}

pub fn parse_instruction(
    blob: &mut GMCodeBlob,
    bytecode14: bool,
    variables: &GMVariables,
    functions: &GMFunctions,
    code_start_pos: usize
) -> Result<GMInstruction, String> {
    let b0: u8 = blob.read_byte()?;
    let b1: u8 = blob.read_byte()?;
    let b2: u8 = blob.read_byte()?;
    let mut opcode_raw: u8 = blob.read_byte()?;

    if bytecode14 {
        opcode_raw = convert_instruction_kind(opcode_raw);
    }
    let mut opcode: GMOpcode = opcode_raw.try_into()
        .map_err(|_| format!("Invalid Opcode 0x{opcode_raw:02X} while parsing code instruction."))?;

    let instruction_type: GMInstructionType = get_instruction_type(opcode);
    match instruction_type {
        GMInstructionType::SingleTypeInstruction |
        GMInstructionType::DoubleTypeInstruction |
        GMInstructionType::ComparisonInstruction => {
            // Parse instruction components from bytes
            let mut comparison_type: GMComparisonType = match b1.try_into() {
                Ok(comparison_type) => comparison_type,
                Err(_) => {
                    return Err(format!(
                        "Invalid Comparison Type {b1:02X} while parsing Comparison Instruction."
                    ));
                }
            };
            let type1: u8 = b2 & 0xf;
            let type1: GMDataType = match type1.try_into() {
                Ok(data_type) => data_type,
                Err(_) => return Err(format!(
                    "Invalid Data Type {type1:02X} while parsing Comparison Instruction."
                )),

            };
            let type2: u8 = b2 >> 4;
            let type2: GMDataType = match type2.try_into() {
                Ok(data_type) => data_type,
                Err(_) => return Err(format!(
                    "Invalid Data Type {type2:02X} while parsing Comparison Instruction."
                )),
            };
            // Ensure basic conditions hold
            if b0 != 0 && opcode != GMOpcode::Dup && opcode != GMOpcode::CallV {
                return Err(format!("Invalid padding {:02X} while parsing Comparison Instruction.", b0));
            }

            if instruction_type == GMInstructionType::SingleTypeInstruction && (type2 as u8) != 0 {
                return Err(format!(
                    "Second type should be 0 but is {:02X} in \
                    SingleTypeInstruction while parsing Comparison Instruction",
                    type2 as u8
                ));
            }

            if bytecode14 && opcode == GMOpcode::Cmp {
                let comparison_type_raw: u8 = opcode_raw - 0x10;
                comparison_type = match comparison_type_raw.try_into() {
                    Ok(comparison_type) => comparison_type,
                    Err(_) => return Err(format!("Invalid Bytecode14 Comparison Type \
                    {comparison_type_raw:02X} (Opcode: {opcode_raw:02X}) while parsing Comparison Instruction.")),
                };
            }
            // short circuit stuff {}

            Ok(GMInstruction::Cmp(GMComparisonInstruction {
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
            let type1: GMDataType = match type1.try_into() {
                Ok(ok) => ok,
                Err(_) => return Err(format!("Invalid Data Type {type1:02X} while parsing Pop Instruction.")),
            };

            let type2: u8 = b2 >> 4;
            let type2: GMDataType = match type2.try_into() {
                Ok(ok) => ok,
                Err(_) => return Err(format!("Invalid Data Type {type2:02X} while parsing Pop Instruction.")),
            };

            let instance_type: i16 = b0 as i16 | ((b1 as i16) << 8);
            let instance_type: GMInstanceType = parse_instance_type(instance_type)?;

            if type1 == GMDataType::Int16 {
                return Err(format!(
                    "[Internal Error] Unhandled \"Special swap instruction\" (UndertaleModTool/Issues/#129) \
                    occurred at position {} while parsing Pop Instruction.\
                    Please report this error to github.com/BioTomateDE/LibGM/Issues \
                    along with your data.win file.",
                    blob.file_index + code_start_pos
                ));
            }

            let destination: GMCodeVariable = blob.read_variable(variables)?;
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
                .map_err(|_| format!("Invalid Data Type {data_type:02X} while parsing Push Instruction."))?;

            let val: i16 = (b0 as i16) | ((b1 as i16) << 8);

            if bytecode14 {
                // println!("############# {:?} {:?}", data_type, val);
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

            // todo fix bullshit variable id

            let value: GMValue = blob.read_value(data_type)?;
            // println!("$$$$$ {:?}", value);

            Ok(GMInstruction::Push(GMPushInstruction {
                opcode,
                data_type,
                value,
            }))
        }

        GMInstructionType::CallInstruction => {
            let arguments_count: usize = b0 as usize;       // idgaf it's always one anyways
            let data_type: u8 = b2;
            let data_type: GMDataType = match data_type.try_into() {
                Ok(ok) => ok,
                Err(_) => return Err(format!("Invalid Data Type {data_type:02X} while parsing Call Instruction.")),
            };

            blob.file_index += 4;
            let function: &GMRef<GMFunction> = functions.occurrences_to_refs.get(&(code_start_pos + blob.file_index))
                .ok_or(format!("Could not find any function with absolute occurrence position {} in map with length {} (functions len: {}).", 
                    code_start_pos + blob.file_index, functions.occurrences_to_refs.len(), functions.functions_by_index.len()))?;

            Ok(GMInstruction::Call(GMCallInstruction {
                opcode,
                arguments_count,
                data_type,
                function: function.clone(),
            }))
        }

        GMInstructionType::BreakInstruction => {
            let value: i16 = b0 as i16 | ((b1 as i16) << 8);
            let data_type: u8 = b2;
            let data_type: GMDataType = match data_type.try_into() {
                Ok(ok) => ok,
                Err(_) => return Err(format!("Invalid Data Type {data_type:02X} while parsing Break Instruction.")),
            };
            let mut int_argument: Option<i32> = None;

            if data_type == GMDataType::Int32 {
                int_argument = Some(match blob.read_value(GMDataType::Int32)? {
                    GMValue::Int32(val) => val,
                    _ => return Err("[INTERNAL ERROR] GMCodeBlob.read_value(GMDataType::Int32, ...)\
                    did not return an i32 while parsing Break Instruction".to_string()),
                });
                // gms version stuff {}
            }

            // other gms version stuff {}

            Ok(GMInstruction::Break(GMBreakInstruction {
                opcode,
                value,
                data_type,
                int_argument,
            }))
        }

        // _ => Err(format!(
        //     "Unhandled opcode {:02X} at position {}/{} (abs: {}) while parsing code. \
        //     Please report this error to github.com/BioTomateDE/UndertaleModManager/issues.",
        //     opcode_raw, blob.file_index, blob.len, code_start_pos + blob.len,
        // )),
    }
}


pub fn parse_instance_type(raw_value: i16) -> Result<GMInstanceType, String> {
    // If >= 0; then game object id. If < 0, then variable instance type.
    if raw_value >= 0 {
        return Ok(GMInstanceType::Self_(Some(GMRef::new(raw_value as usize))))
    }

    let instance_type = match raw_value {
        0 => GMInstanceType::Undefined,
        -2 => GMInstanceType::Other,
        -3 => GMInstanceType::All,
        -4 => GMInstanceType::Noone,
        -5 => GMInstanceType::Global,
        -7 => GMInstanceType::Local,
        -9 => GMInstanceType::Stacktop,
        -15 => GMInstanceType::Arg,
        -16 => GMInstanceType::Static,
        _ => return Err(format!("Invalid instance type {raw_value} (0x{raw_value:04X})."))
    };

    Ok(instance_type)
}


pub fn read_variable_reference(chunk: &mut GMChunk, variable: GMRef<GMVariable>) -> Result<(GMPopInstruction, usize), String> {
    // chunk.cur_pos -= 4;
    let b0: u8 = chunk.read_u8()?;
    let b1: u8 = chunk.read_u8()?;
    let b2: u8 = chunk.read_u8()?;
    let raw_opcode: u8 = chunk.read_u8()?;
    let raw_value: i32 = chunk.read_i32()?;

    // if bytecode14 {
    //     raw_opcode = convert_instruction_kind(raw_opcode);
    // }
    let opcode: GMOpcode = raw_opcode.try_into()
        .map_err(|_| format!("Invalid Opcode 0x{raw_opcode:02X} while parsing code instruction."))?;

    // TODO type1 and type1 only make sense for a pop instruction; push needs to be parsed differently
    let type1: u8 = b2 & 0xf;
    let type1: GMDataType = type1.try_into()
        .map_err(|_| format!("Invalid Data Type 1 {type1:02X} while parsing Pop Instruction (variable reference)."))?;

    let type2: u8 = b2 >> 4;
    let type2: GMDataType = type2.try_into()
        .map_err(|_| format!("Invalid Data Type 1 {type2:02X} while parsing Pop Instruction (variable reference)."))?;

    let variable_type: i32 = (raw_value >> 24) & 0xF8;
    let variable_type: u8 = variable_type as u8;
    let variable_type: GMVariableType = variable_type.try_into()
        .map_err(|_| format!("Invalid Variable Type 0x{variable_type:02X} while parsing variable reference chain."))?;

    let next_occurrence_offset: i32 = raw_value & 0x07FFFFFF;
    let next_occurrence_offset: usize = next_occurrence_offset as usize;

    log::info!("VarRef | {opcode:?} | {b0} {b1} {b2} {raw_value} | @{} +{} {:?} | {type1:?} {type2:?}", chunk.cur_pos, next_occurrence_offset, variable_type);

    let instance_type: i16 = b0 as i16 | ((b1 as i16) << 8);
    let instance_type: GMInstanceType = parse_instance_type(instance_type)?;

    let destination = GMCodeVariable {
        variable,
        variable_type
    };

    // if type1 != GMDataType::Variable || type2 != GMDataType::Double {
    // if opcode == GMOpcode::Pop {
    // if true {
    //     log::debug!("VarRef | {opcode:?} | {b0} {_b1} {b2} {raw_value} | @{} +{} {:?} | {type1:?} {type2:?}", chunk.cur_pos, next_occurrence_offset, variable_type);
    // }

    let instruction = GMPopInstruction {
        opcode,
        instance_type,
        type1,
        type2,
        destination,
    };

    Ok((instruction, next_occurrence_offset))
}

