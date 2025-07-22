use serde::{Deserialize, Serialize};
use crate::gamemaker::elements::code::{GMCodeBytecode15, GMCodeValue, GMCodeVariable, GMComparisonType, GMDataType, GMInstanceType, GMInstruction, GMInstructionData, GMOpcode, GMVariableType};
use crate::modding::export::{convert_additions, edit_field, edit_field_convert, ModExporter, ModRef};
use crate::modding::ordered_list::{export_changes_ordered_list, DataChange};
use crate::modding::unordered_list::{export_changes_unordered_list, EditUnorderedList};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddCode {
    pub name: ModRef,  // String
    pub instructions: Vec<ModInstruction>,
    pub bytecode15_info: Option<AddCodeBytecode15>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddCodeBytecode15 {
    pub locals_count: u16,
    pub arguments_count: u16,
    pub local_flag: bool,
    pub offset: usize,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditCode {
    pub name: Option<ModRef>,  // String
    pub instructions: Vec<DataChange<ModInstruction>>,
    pub bytecode15_info: Option<AddCodeBytecode15>,
}
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditCodeBytecode15 {
    pub locals_count: u16,
    pub arguments_count: u16,
    pub weird_local_flag: bool,
    /// TODO: vulnerable; check overflow
    pub offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModInstruction {
    pub opcode: ModOpcode,
    pub kind: ModInstructionKind,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModInstructionKind {
    SingleType(ModDataType),
    DoubleType(ModDataType, ModDataType),
    Comparison(ModDataType, ModDataType, ModComparisonType),
    Goto(ModGotoTarget),
    Push(ModValue),
    Pop(ModDataType, ModDataType, ModCodeVariable),
    Call(ModDataType, ModRef, u8),   // function ref, args count
    Break(ModDataType, i16, Option<i32>),   // TODO this will probably also be really incompatible
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModDataType {
    Double,
    Float,
    Int16,
    Int32,
    Int64,
    Boolean,
    Variable,
    String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModComparisonType {
    LT,
    LTE,
    EQ,
    NEQ,
    GTE,
    GT,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModGotoTarget {
    PopenvMagic,
    /// Offset in instruction counts; not bytes. TODO implement
    Offset(i32),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModValue {
    Double(f64),
    Float(f32),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Boolean(bool),
    String(ModRef),
    Variable(ModCodeVariable),
    Function(ModRef),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModCodeVariable {
    pub variable: ModRef,
    pub variable_type: ModVariableType,
    pub instance_type: ModInstanceType,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ModInstanceType {
    Undefined,
    Instance(Option<ModRef>),   // GMGameObject. Is optional depending on context
    Global,
    Local,
    Argument,
    Other,
    // TODO: this will cause issues lol; i removed like half the instance types
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModVariableType {
    Normal,
    StackTop,
    Instance,
    Array,
    ArrayPushAF,
    ArrayPopAF,
}
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
pub enum ModOpcode {
    Conv = 0x07,
    Mul = 0x08,
    Div = 0x09,
    Rem = 0x0A,
    Mod = 0x0B,
    Add = 0x0C,
    Sub = 0x0D,
    And = 0x0E,
    Or = 0x0F,
    Xor = 0x10,
    Neg = 0x11,
    Not = 0x12,
    Shl = 0x13,
    Shr = 0x14,
    Cmp = 0x15,
    Pop = 0x45,
    Dup = 0x86,
    Ret = 0x9C,
    Exit = 0x9D,
    PopZ = 0x9E,
    Branch = 0xB6,
    BranchTrue = 0xB7,
    BranchFalse = 0xB8,
    PushEnv = 0xBA,
    PopEnv = 0xBB,
    Push = 0xC0,
    PushLoc = 0xC1,
    PushGlb = 0xC2,
    PushBltn = 0xC3,
    PushI = 0x84,
    Call = 0xD9,
    CallV = 0x99,
    Extended = 0xFF,
}


impl ModExporter<'_, '_> {
    pub fn export_codes(&self) -> Result<EditUnorderedList<AddCode, EditCode>, String> {
        export_changes_unordered_list(
            &self.original_data.codes.codes,
            &self.modified_data.codes.codes,
            |i| Ok(AddCode {
                name: self.convert_string_ref(&i.name)?,
                instructions: convert_additions(&i.instructions, |i| self.convert_instruction(i))?,
                bytecode15_info: i.bytecode15_info.as_ref().map(convert_bytecode15_info),
            }),
            |o, m| Ok(EditCode {
                name: edit_field_convert(&o.name, &o.name, |r| self.convert_string_ref(r))?,
                instructions: export_changes_ordered_list(&o.instructions, &m.instructions, |r| self.convert_instruction(r))?,
                bytecode15_info: edit_field(&o.bytecode15_info, &m.bytecode15_info).unwrap_or(None).as_ref().map(convert_bytecode15_info),
            }),
            false,
        )
    }
    
    fn convert_instruction(&self, instruction: &GMInstruction) -> Result<ModInstruction, String> {
        let kind: ModInstructionKind = match &instruction.kind {
            GMInstructionData::SingleType(i) => ModInstructionKind::SingleType(
                convert_data_type(i.data_type)?,
            ),
            GMInstructionData::DoubleType(i) => ModInstructionKind::DoubleType(
                convert_data_type(i.type1)?,
                convert_data_type(i.type2)?,
            ),
            GMInstructionData::Comparison(i) => ModInstructionKind::Comparison(
                convert_data_type(i.type1)?,
                convert_data_type(i.type2)?,
                convert_comparison_type(i.comparison_type),
            ),
            GMInstructionData::Goto(i) => ModInstructionKind::Goto(
                if i.popenv_exit_magic {
                    ModGotoTarget::PopenvMagic
                } else {
                    ModGotoTarget::Offset(i.jump_offset)
                }
            ),
            GMInstructionData::Pop(i) => ModInstructionKind::Pop(
                convert_data_type(i.type1)?,
                convert_data_type(i.type2)?,
                self.convert_code_variable(&i.destination)?,
                
            ),
            GMInstructionData::Push(i) => ModInstructionKind::Push(
                self.convert_value(&i.value)?,
            ),
            GMInstructionData::Call(i) => ModInstructionKind::Call(
                convert_data_type(i.data_type)?,
                self.convert_function_ref(&i.function)?,
                i.arguments_count,
            ),
            GMInstructionData::Break(i) => ModInstructionKind::Break(
                convert_data_type(i.data_type)?,
                i.extended_kind,
                i.int_argument,
            ),
        };
        
        let opcode: ModOpcode = match instruction.opcode {
            GMOpcode::Convert => ModOpcode::Conv,
            GMOpcode::Multiply => ModOpcode::Mul,
            GMOpcode::Divide => ModOpcode::Div,
            GMOpcode::Remainder => ModOpcode::Rem,
            GMOpcode::Modulus => ModOpcode::Mod,
            GMOpcode::Add => ModOpcode::Add,
            GMOpcode::Subtract => ModOpcode::Sub,
            GMOpcode::And => ModOpcode::And,
            GMOpcode::Or => ModOpcode::Or,
            GMOpcode::Xor => ModOpcode::Xor,
            GMOpcode::Negate => ModOpcode::Neg,
            GMOpcode::Not => ModOpcode::Not,
            GMOpcode::ShiftLeft => ModOpcode::Shl,
            GMOpcode::ShiftRight => ModOpcode::Shr,
            GMOpcode::Compare => ModOpcode::Cmp,
            GMOpcode::Pop => ModOpcode::Pop,
            GMOpcode::Duplicate => ModOpcode::Dup,
            GMOpcode::Return => ModOpcode::Ret,
            GMOpcode::Exit => ModOpcode::Exit,
            GMOpcode::PopDiscard => ModOpcode::PopZ,
            GMOpcode::Branch => ModOpcode::Branch,
            GMOpcode::BranchIf => ModOpcode::BranchTrue,
            GMOpcode::BranchUnless => ModOpcode::BranchFalse,
            GMOpcode::PushWithContext => ModOpcode::PushEnv,
            GMOpcode::PopWithContext => ModOpcode::PopEnv,
            GMOpcode::Push => ModOpcode::Push,
            GMOpcode::PushLocal => ModOpcode::PushLoc,
            GMOpcode::PushGlobal => ModOpcode::PushGlb,
            GMOpcode::PushBuiltin => ModOpcode::PushBltn,
            GMOpcode::PushImmediate => ModOpcode::PushI,
            GMOpcode::Call => ModOpcode::Call,
            GMOpcode::CallVariable => ModOpcode::CallV,
            GMOpcode::Extended => ModOpcode::Extended,
        };

        Ok(ModInstruction {
            opcode,
            kind,
        })
    }
    
    pub fn convert_instance_type(&self, i: &GMInstanceType) -> Result<ModInstanceType, String> {
        match i {
            GMInstanceType::Self_(obj_ref) => Ok(ModInstanceType::Instance(self.convert_game_object_ref_opt(obj_ref)?)),
            GMInstanceType::Global => Ok(ModInstanceType::Global),
            GMInstanceType::Local => Ok(ModInstanceType::Local),
            GMInstanceType::Argument => Ok(ModInstanceType::Argument),
            GMInstanceType::Undefined => Ok(ModInstanceType::Undefined),
            GMInstanceType::Other => Ok(ModInstanceType::Other),
            GMInstanceType::All |
            GMInstanceType::None |
            GMInstanceType::Builtin |
            GMInstanceType::StackTop |
            GMInstanceType::RoomInstance(_) |
            GMInstanceType::Static => Err(format!("Instance Type {i:?} not (yet) supported for modding")),
        }
    }

    fn convert_code_variable(&self, i: &GMCodeVariable) -> Result<ModCodeVariable, String> {
        Ok(ModCodeVariable {
            variable: self.convert_variable_ref(&i.variable)?,
            variable_type: match i.variable_type {
                GMVariableType::Array => ModVariableType::Array,
                GMVariableType::StackTop => ModVariableType::StackTop,
                GMVariableType::Normal => ModVariableType::Normal,
                GMVariableType::Instance => ModVariableType::Instance,
                GMVariableType::MultiPush => ModVariableType::ArrayPushAF,
                GMVariableType::MultiPushPop => ModVariableType::ArrayPopAF,
            },
            instance_type: self.convert_instance_type(&i.instance_type)?,
        })
    }

    fn convert_value(&self, val: &GMCodeValue) -> Result<ModValue, String> {
        Ok(match val {
            GMCodeValue::Double(i) => ModValue::Double(*i),
            GMCodeValue::Float(i) => ModValue::Float(*i),
            GMCodeValue::Int16(i) => ModValue::Int16(*i),
            GMCodeValue::Int32(i) => ModValue::Int32(*i),
            GMCodeValue::Int64(i) => ModValue::Int64(*i),
            GMCodeValue::Boolean(i) => ModValue::Boolean(*i),
            GMCodeValue::String(i) => ModValue::String(self.convert_string_ref(i)?),
            GMCodeValue::Variable(i) => ModValue::Variable(self.convert_code_variable(i)?),
            GMCodeValue::Function(i) => ModValue::Function(self.convert_function_ref(i)?),
        })
    }
}

fn convert_bytecode15_info(i: &GMCodeBytecode15) -> AddCodeBytecode15 {
    AddCodeBytecode15 {
        locals_count: i.locals_count,
        arguments_count: i.arguments_count,
        local_flag: i.weird_local_flag,
        offset: i.offset,
    }
}


fn convert_data_type(i: GMDataType) -> Result<ModDataType, String> {
    match i {
        GMDataType::Double => Ok(ModDataType::Double),
        GMDataType::Float => Ok(ModDataType::Float),
        GMDataType::Int32 => Ok(ModDataType::Int32),
        GMDataType::Int64 => Ok(ModDataType::Int64),
        GMDataType::Boolean => Ok(ModDataType::Boolean),
        GMDataType::Variable => Ok(ModDataType::Variable),
        GMDataType::String => Ok(ModDataType::String),
        GMDataType::Int16 => Ok(ModDataType::Int16),
    }
}

/// truly revolutionary
fn convert_comparison_type(i: GMComparisonType) -> ModComparisonType {
    match i {
        GMComparisonType::LT => ModComparisonType::LT,
        GMComparisonType::LTE => ModComparisonType::LTE,
        GMComparisonType::EQ => ModComparisonType::EQ,
        GMComparisonType::NEQ => ModComparisonType::NEQ,
        GMComparisonType::GTE => ModComparisonType::GTE,
        GMComparisonType::GT => ModComparisonType::GT,
    }
}

