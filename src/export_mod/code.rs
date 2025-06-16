use serde::{Deserialize, Serialize};
use crate::deserialize::code::{GMCodeBytecode15, GMCodeVariable, GMComparisonType, GMDataType, GMInstanceType, GMInstruction, GMOpcode, GMValue, GMVariableType};
use crate::export_mod::export::{convert_additions, edit_field, edit_field_convert, ModExporter, ModRef};
use crate::export_mod::ordered_list::{export_changes_ordered_list, DataChange};
use crate::export_mod::unordered_list::{export_changes_unordered_list, EditUnorderedList};


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
    pub offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModInstruction {
    pub opcode: GMOpcode,  // {!!} GM 
    pub kind: ModInstructionKind,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModInstructionKind {
    SingleType(ModDataType),
    DoubleType(ModDataType, ModDataType),
    Comparison(ModDataType, ModDataType, ModComparisonType),
    Goto(ModGotoTarget),
    Push(ModDataType, ModValue),
    Pop(ModDataType, ModDataType, ModInstanceType, ModCodeVariable),
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
    Offset(i32),    // TODO this integer will immensely fuck up compatibility but idk how else to do it
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
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModCodeVariable {
    pub variable: ModRef,
    pub variable_type: ModVariableType,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ModInstanceType {
    Undefined,
    Instance(Option<ModRef>),   // GMGameObject. Is optional depending on context
    Other,
    All,
    None,
    Global,
    Builtin,
    Local,
    StackTop,
    Argument,
    Static,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModVariableType {
    Normal,
    StackTop,
    Instance,
    Array,
    MultiPush,
    MultiPushPop,
}

impl ModExporter<'_, '_> {
    pub fn export_codes(&self) -> Result<EditUnorderedList<AddCode, EditCode>, String> {
        export_changes_unordered_list(
            &self.original_data.codes.codes_by_index,
            &self.modified_data.codes.codes_by_index,
            |i| Ok(AddCode {
                name: self.convert_string_ref(&i.name)?,
                instructions: convert_additions(&i.instructions, |i| self.convert_instruction(i))?,
                bytecode15_info: i.bytecode15_info.as_ref().map(convert_bytecode15_info),
            }),
            |o, m| Ok(EditCode {
                name: edit_field_convert(&o.name, &o.name, |r| self.convert_string_ref(r))?,
                instructions: export_changes_ordered_list(&o.instructions, &m.instructions, |r| self.convert_instruction(r))?,
                bytecode15_info: edit_field(&o.bytecode15_info, &m.bytecode15_info).unwrap_or(None).as_ref().map(convert_bytecode15_info),
            })
        )
    }
    
    fn convert_instruction(&self, instruction: &GMInstruction) -> Result<ModInstruction, String> {
        let kind: ModInstructionKind = match instruction {
            GMInstruction::SingleType(i) => ModInstructionKind::SingleType(
                convert_data_type(i.data_type)?,
            ),
            GMInstruction::DoubleType(i) => ModInstructionKind::DoubleType(
                convert_data_type(i.type1)?,
                convert_data_type(i.type2)?,
            ),
            GMInstruction::Comparison(i) => ModInstructionKind::Comparison(
                convert_data_type(i.type1)?,
                convert_data_type(i.type2)?,
                convert_comparison_type(i.comparison_type),
            ),
            GMInstruction::Goto(i) => ModInstructionKind::Goto(
                if i.popenv_exit_magic {
                    ModGotoTarget::PopenvMagic
                } else {
                    ModGotoTarget::Offset(i.jump_offset)
                }
            ),
            GMInstruction::Pop(i) => ModInstructionKind::Pop(
                convert_data_type(i.type1)?,
                convert_data_type(i.type2)?,
                self.convert_instance_type(&i.instance_type)?,
                self.convert_code_variable(&i.destination)?,
                
            ),
            GMInstruction::Push(i) => ModInstructionKind::Push(
                convert_data_type(i.data_type)?,
                self.convert_value(&i.value)?,
            ),
            GMInstruction::Call(i) => ModInstructionKind::Call(
                convert_data_type(i.data_type)?,
                self.convert_function_ref(&i.function)?,
                i.arguments_count,
            ),
            GMInstruction::Break(i) => ModInstructionKind::Break(
                convert_data_type(i.data_type)?,
                i.value,
                i.int_argument,
            ),
        };
        
        let opcode: GMOpcode = match instruction {
            GMInstruction::SingleType(i) => i.opcode,
            GMInstruction::DoubleType(i) => i.opcode,
            GMInstruction::Comparison(i) => i.opcode,
            GMInstruction::Goto(i) => i.opcode,
            GMInstruction::Pop(i) => i.opcode,
            GMInstruction::Push(i) => i.opcode,
            GMInstruction::Call(i) => i.opcode,
            GMInstruction::Break(i) => i.opcode,
        };

        Ok(ModInstruction {
            opcode,
            kind,
        })
    }
    
    pub fn convert_instance_type(&self, i: &GMInstanceType) -> Result<ModInstanceType, String> {
        match i {
            GMInstanceType::Undefined => Ok(ModInstanceType::Undefined),
            GMInstanceType::Instance(obj_ref) => Ok(ModInstanceType::Instance(self.convert_game_object_ref_opt(obj_ref)?)),
            GMInstanceType::Other => Ok(ModInstanceType::Other),
            GMInstanceType::All => Ok(ModInstanceType::All),
            GMInstanceType::None => Ok(ModInstanceType::None),
            GMInstanceType::Global => Ok(ModInstanceType::Global),
            GMInstanceType::Builtin => Ok(ModInstanceType::Builtin),
            GMInstanceType::Local => Ok(ModInstanceType::Local),
            GMInstanceType::StackTop => Ok(ModInstanceType::StackTop),
            GMInstanceType::Argument => Ok(ModInstanceType::Argument),
            GMInstanceType::Static => Ok(ModInstanceType::Static),
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
                GMVariableType::MultiPush => ModVariableType::MultiPush,
                GMVariableType::MultiPushPop => ModVariableType::MultiPushPop,
            },
        })
    }

    fn convert_value(&self, val: &GMValue) -> Result<ModValue, String> {
        Ok(match val {
            GMValue::Double(i) => ModValue::Double(*i),
            GMValue::Float(i) => ModValue::Float(*i),
            GMValue::Int16(i) => ModValue::Int16(*i),
            GMValue::Int32(i) => ModValue::Int32(*i),
            GMValue::Int64(i) => ModValue::Int64(*i),
            GMValue::Boolean(i) => ModValue::Boolean(*i),
            GMValue::String(i) => ModValue::String(self.convert_string_ref(i)?),
            GMValue::Variable(i) => ModValue::Variable(self.convert_code_variable(i)?),
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
        GMDataType::Instance => Err("Code Data Type \"Instance\" is not (yet) supported".to_string()),
        GMDataType::Delete => Err("Code Data Type \"Delete\" is not (yet) supported".to_string()),
        GMDataType::Undefined => Err("Code Data Type \"Undefined\" is not (yet) supported".to_string()),
        GMDataType::UnsignedInt => Err("Code Data Type \"UnsignedInt\" is not (yet) supported".to_string()),
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

