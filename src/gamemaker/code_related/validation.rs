use std::fmt::{Display, Formatter};
use bimap::{BiHashMap, BiMap};
use crate::gamemaker::code_related::disassembler::disassemble_instruction;
use crate::gamemaker::data::GMData;
use crate::gamemaker::elements::code::{get_data_type_from_value, parse_instance_type, GMCode, GMCodeValue, GMCodes, GMDataType, GMExtendedKind, GMInstanceType, GMInstruction, GMVariableType};

#[derive(Debug)]
pub enum CodeValidationError {
    CodeEndWithoutExit(usize, usize),
    PopStackEmpty,
    PeekStackEmpty,
    TypeMismatch(VMStackItem, GMDataType),
    NormalPopFirstNotVar(GMDataType),
    UnnormalPopFirstNotInt32(GMVariableType, GMDataType),
    UnnormalPopSecondNotVar(GMVariableType, GMDataType),
    VariableUnresolvable(u32),
    ParentCodeUnresolvable(u32),
    InvalidInitialOffset(u32),
    InvalidBranchTarget(u32),
    ExitStackLeftover(VMStack),
    BranchStackLeftover(VMStack),
    PushImmediateNotInt16(GMDataType),
    PushVarWrongDataType(GMDataType),
    PushVarWrongInstanceType(GMInstanceType),
    CallArgumentNotVar(VMStackItem),
    SpecialVarTypeNotInt32(GMVariableType, GMDataType),
    BranchConditionNotBool(VMStackItem),
    StackTopNotInt32(VMStackItem),
    SetOwnerNotInt32(VMStackItem),
    Int16OutOfBounds(i32),
    InvalidInt16InstanceType(i16),
    UnacceptableInstanceType(GMInstanceType),
}

impl Display for CodeValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CodeValidationError::CodeEndWithoutExit(a, b) => write!(f, "Code instructions ended without Exit or Return instruction ({a} >= {b})"),
            CodeValidationError::PopStackEmpty => write!(f, "Tried to pop value from empty stack"),
            CodeValidationError::PeekStackEmpty => write!(f, "Tried to peek value from empty stack"),
            CodeValidationError::TypeMismatch(item, dtype) => write!(f, "Stack item {item:?} does not match data type of instruction {dtype:?}"),
            CodeValidationError::NormalPopFirstNotVar(dtype) => write!(f, "Pop instruction with Normal variable type has type1 set to {dtype:?} instead of Variable"),
            CodeValidationError::UnnormalPopFirstNotInt32(vtype, dtype) => write!(f, "Pop instruction with variable type {vtype:?} has type1 set to {dtype:?} instead of Int32"),
            CodeValidationError::UnnormalPopSecondNotVar(vtype, dtype) => write!(f, "Pop instruction with variable type {vtype:?} has type2 set to {dtype:?} instead of Variable"),
            CodeValidationError::VariableUnresolvable(idx) => write!(f, "Could not resolve variable with index {idx}"),
            CodeValidationError::ParentCodeUnresolvable(idx) => write!(f, "Could not resolve parent code with index {idx}"),
            CodeValidationError::InvalidInitialOffset(offset) => write!(f, "Child code instruction offset {offset} does not point to the start of an instruction"),
            CodeValidationError::InvalidBranchTarget(addr) => write!(f, "Branch instruction with target address {addr} does not point to the start of an instruction"),
            CodeValidationError::ExitStackLeftover(stack) => write!(f, "Code exited with leftover stack data: {:?}", stack.items),
            CodeValidationError::BranchStackLeftover(stack) => write!(f, "Code branched with leftover stack data (this is an internal issue): {:?}", stack.items),
            CodeValidationError::PushImmediateNotInt16(dtype) => write!(f, "PushImmediate instruction's data type is {dtype:?} instead of Int16"),
            CodeValidationError::PushVarWrongDataType(dtype) => write!(f, "Push instruction for variable (local/global/builtin) has data type {dtype:?} instead Variable"),
            CodeValidationError::PushVarWrongInstanceType(inst) => write!(f, "Push instruction for variable (local/global/builtin) wrong instance type {inst}"),
            CodeValidationError::CallArgumentNotVar(dtype) => write!(f, "Stack value popped as argument for function call has data type {dtype:?} instead of Variable"),
            CodeValidationError::SpecialVarTypeNotInt32(vtype, dtype) => write!(f, "Top data type is {dtype:?} instead of Int32 for {vtype:?} Pop instruction"),
            CodeValidationError::BranchConditionNotBool(item) => write!(f, "Top data type is {item:?} instead of Boolean for conditional Branch instruction"),
            CodeValidationError::StackTopNotInt32(item) => write!(f, "Top data type is {item:?} instead of Int32 (with value) for stacktop variable type"),
            CodeValidationError::SetOwnerNotInt32(item) => write!(f, "Top data type is {item:?} instead of Int32 (with value) for extended SetOwner instruction"),
            CodeValidationError::Int16OutOfBounds(int32) => write!(f, "Int32 {int32} could not be converted to an Int16; out of bounds"),
            CodeValidationError::InvalidInt16InstanceType(int16) => write!(f, "Int16 {int16} is not a valid instance type"),
            CodeValidationError::UnacceptableInstanceType(inst) => write!(f, "Instance Type {inst} is not an instance type in this context"),
        }
    }
}


pub fn validate_code(code: &GMCode, gm_data: &GMData) -> Result<(), CodeValidationError> {
    let mut instruction_address: u32 = 0;
    let mut instructions: &Vec<GMInstruction> = &code.instructions;

    if let Some(b15_info) = &code.bytecode15_info {
        if let Some(parent_ref) = b15_info.parent {
            let parent: &GMCode = parent_ref.resolve(&gm_data.codes.codes)
                .map_err(|_| CodeValidationError::ParentCodeUnresolvable(parent_ref.index))?;
            instructions = &parent.instructions;
            if b15_info.offset % 4 != 0 {
                return Err(CodeValidationError::InvalidInitialOffset(instruction_address));
            }
            instruction_address = b15_info.offset / 4;
        }
    }

    let address_map: BiMap<usize, u32> = generate_address_map(instructions);
    let instruction_index: usize = *address_map.get_by_right(&instruction_address)
        .ok_or(CodeValidationError::InvalidInitialOffset(instruction_address))?;

    let stack = VMStack::new();
    validate_instructions(gm_data, &address_map, instructions, stack, instruction_index)?;
    Ok(())
}


fn validate_instructions(
    gm_data: &GMData,
    address_map: &BiMap<usize, u32>,
    instructions: &Vec<GMInstruction>,
    mut stack: VMStack,
    mut instruction_index: usize,
) -> Result<(), CodeValidationError> {
    while instruction_index < instructions.len() {
        // TODO: clean this part up
        let instruction: &GMInstruction = instructions.get(instruction_index)
            .ok_or(CodeValidationError::CodeEndWithoutExit(instruction_index, instructions.len()))?;
        let instruction_address: u32 = *address_map.get_by_left(&instruction_index).unwrap();
        log::debug!("{} {}", stack.items.len(), disassemble_instruction(gm_data, instruction).unwrap());
        instruction_index += 1;

        match instruction {
            GMInstruction::Convert(instr) => {
                let item: VMStackItem = stack.pop()?;
                item.assert_data_type(instr.type1)?;
                stack.push_data_type(instr.type2);
            }

            GMInstruction::Multiply(instr) |
            GMInstruction::Divide(instr) |
            GMInstruction::Remainder(instr) |
            GMInstruction::Modulus(instr) |
            GMInstruction::Add(instr) |
            GMInstruction::Subtract(instr) |
            GMInstruction::And(instr) |
            GMInstruction::Or(instr) |
            GMInstruction::Xor(instr) |
            GMInstruction::ShiftLeft(instr) |
            GMInstruction::ShiftRight(instr) => {
                let item2: VMStackItem = stack.pop()?;
                let item1: VMStackItem = stack.pop()?;
                item1.assert_data_type(instr.type1)?;
                item2.assert_data_type(instr.type2)?;
                stack.push_data_type(instr.type2);
            }

            GMInstruction::Negate(instr) |
            GMInstruction::Not(instr) => {
                let item: VMStackItem = stack.pop()?;
                item.assert_data_type(instr.data_type)?;
                stack.push_data_type(instr.data_type);
            }

            GMInstruction::Compare(instr) => {
                let item1: VMStackItem = stack.pop()?;
                let item2: VMStackItem = stack.pop()?;
                item1.assert_data_type(instr.type1)?;
                item2.assert_data_type(instr.type2)?;
                stack.push_data_type(GMDataType::Boolean);
            }

            GMInstruction::Pop(instr) => {
                match instr.destination.variable_type {
                    GMVariableType::Normal => {}
                    GMVariableType::StackTop => {
                        let instance_type_item: VMStackItem = stack.pop()?;
                        validate_stacktop(instance_type_item)?;
                    }
                    GMVariableType::Array => {
                        let array_index = stack.pop()?;
                        array_index.assert_data_type(GMDataType::Int32)?;
                    }
                    GMVariableType::Instance => unimplemented!("Variable Type Instance when popping not yet implemented"),
                    GMVariableType::ArrayPushAF => unimplemented!("Variable Type ArrayPushAF when popping not yet implemented"),
                    GMVariableType::ArrayPopAF => unimplemented!("Variable Type ArrayPopAF when popping not yet implemented"),
                }

                let item: VMStackItem = stack.pop()?;
                // todo assumption
                if instr.destination.variable_type == GMVariableType::Normal {
                    if instr.type1 != GMDataType::Variable {
                        return Err(CodeValidationError::NormalPopFirstNotVar(instr.type1))
                    }
                    item.assert_data_type(instr.type2)?;
                } else {
                    if instr.type1 != GMDataType::Int32 {
                        return Err(CodeValidationError::UnnormalPopFirstNotInt32(instr.destination.variable_type, instr.type1))
                    }
                    if instr.type2 != GMDataType::Variable {
                        return Err(CodeValidationError::UnnormalPopSecondNotVar(instr.destination.variable_type, instr.type1))
                    }
                }
            }

            GMInstruction::PopSwap(instr) => {
                unimplemented!("popswap not yet implemented")
            }

            GMInstruction::Duplicate(instr) => {
                // TODO: this is an assumption. idk how dup works.
                let item: VMStackItem = stack.peek()?.clone();
                item.assert_data_type(instr.data_type)?;
                for _ in 0..instr.size+1 {
                    stack.push(item.clone());
                }
            }

            GMInstruction::DuplicateSwap(instr) => {
                unimplemented!("dupswap not yet implemented")
            }

            GMInstruction::Return(instr) => {
                let item: VMStackItem = stack.pop()?;
                item.assert_data_type(instr.data_type)?;
                if !stack.is_empty() {
                    return Err(CodeValidationError::ExitStackLeftover(stack))
                }
                return Ok(())
            }

            GMInstruction::Exit(_) => {
                if !stack.is_empty() {
                    return Err(CodeValidationError::ExitStackLeftover(stack))
                }
                return Ok(())
            }

            GMInstruction::PopDiscard(instr) => {
                let item: VMStackItem = stack.pop()?;
                item.assert_data_type(instr.data_type)?;
            }

            GMInstruction::Branch(instr) |
            GMInstruction::BranchIf(instr) |
            GMInstruction::BranchUnless(instr) => {
                let conditional: bool = matches!(instruction, GMInstruction::BranchIf(_) | GMInstruction::BranchUnless(_));
                if conditional {
                    let item: VMStackItem = stack.pop()?;
                    if item != VMStackItem::Boolean {
                        return Err(CodeValidationError::BranchConditionNotBool(item))
                    }
                }

                // if !stack.is_empty() {
                //     return Err(CodeValidationError::BranchStackLeftover(stack))
                // }

                let branch_target_index: usize;
                if let Some(address_offset) = instr.jump_offset {
                    let address_target: u32 = (instruction_address as i32 + address_offset) as u32;
                    branch_target_index = *address_map.get_by_right(&address_target)
                        .ok_or(CodeValidationError::InvalidBranchTarget(address_target))?;
                } else {
                    unimplemented!("popenv exit magic not yet implemented")
                }

                if conditional {
                    // perform branch on recursive call; skip branch in this execution
                    validate_instructions(gm_data, address_map, instructions, stack.clone(), branch_target_index)?;
                    continue
                }

                instruction_index = branch_target_index;
            }

            GMInstruction::PushWithContext(instr) => {
                unimplemented!("pushenv not yet supported")
            }

            GMInstruction::PopWithContext(instr) => {
                unimplemented!("popenv not yet supported")
            }

            GMInstruction::Push(instr) |
            GMInstruction::PushLocal(instr) |
            GMInstruction::PushGlobal(instr) |
            GMInstruction::PushBuiltin(instr) |
            GMInstruction::PushImmediate(instr) => {
                let data_type: GMDataType = get_data_type_from_value(&instr.value);

                // if let GMCodeValue::Variable(code_var) = &instr.value {
                    // validate_stacktop(code_var.variable_type)
                    // let var = code_var.variable.resolve(&gm_data.variables.variables)
                    //     .map_err(|_| CodeValidationError::VariableUnresolvable(code_var.variable.index))?;
                    // if let Some(b15) = &var.b15_data && b15.instance_type !=
                // }

                if matches!(instruction, GMInstruction::PushLocal(_) | GMInstruction::PushGlobal(_) | GMInstruction::PushBuiltin(_)) {
                    let GMCodeValue::Variable(code_variable) = &instr.value else {
                        return Err(CodeValidationError::PushVarWrongDataType(data_type))
                    };
                    match (instruction, &code_variable.instance_type) {
                        (GMInstruction::PushLocal(_), GMInstanceType::Local) => {}
                        (GMInstruction::PushGlobal(_), GMInstanceType::Global) => {}
                        (GMInstruction::PushBuiltin(_), GMInstanceType::Self_(None)) => {}  //idk
                        (_, _) => return Err(CodeValidationError::PushVarWrongInstanceType(code_variable.instance_type.clone()))
                    }
                }

                if matches!(instruction, GMInstruction::PushImmediate(_)) && data_type != GMDataType::Int16 {
                    return Err(CodeValidationError::PushImmediateNotInt16(data_type))
                }

                if let GMCodeValue::Int16(val) = instr.value {
                    stack.push(VMStackItem::from_int32(i32::from(val)));
                } else if let GMCodeValue::Int32(val) = instr.value {
                    stack.push(VMStackItem::from_int32(val));
                } else {
                    stack.push_data_type(data_type);
                }
            }

            GMInstruction::Call(instr) => {
                for _ in 0..instr.arguments_count {
                    let item: VMStackItem = stack.pop()?;
                    if item != VMStackItem::Variable {
                        return Err(CodeValidationError::CallArgumentNotVar(item))
                    }
                }
                // TODO: what does instr.data_type do??? it's always Int32
                // TODO: return type is an assumption
                stack.push(VMStackItem::Variable);
            }

            GMInstruction::CallVariable(instr) => {
                unimplemented!("callvar not yet implemented")
            }

            GMInstruction::Extended16(instr) => {
                match instr.kind {
                    GMExtendedKind::CheckArrayIndex => unimplemented!("CheckArrayIndex not yet implemented"),
                    GMExtendedKind::PushArrayFinal => unimplemented!("CheckArrayIndex not yet implemented"),
                    GMExtendedKind::PopArrayFinal => unimplemented!("PopArrayFinal not yet implemented"),
                    GMExtendedKind::PushArrayContainer => unimplemented!("PushArrayContainer not yet implemented"),
                    GMExtendedKind::SetArrayOwner => {
                        let item: VMStackItem = stack.pop()?;
                        let VMStackItem::Int32(Some(int32)) = item else {
                            return Err(CodeValidationError::SetOwnerNotInt32(item))
                        };
                        // todo use this int32 glovql variabe thign

                    }
                    GMExtendedKind::HasStaticInitialized => unimplemented!("HasStaticInitialized not yet implemented"),
                    GMExtendedKind::SetStaticInitialized => unimplemented!("SetStaticInitialized not yet implemented"),
                    GMExtendedKind::SaveArrayReference => unimplemented!("SaveArrayReference not yet implemented"),
                    GMExtendedKind::RestoreArrayReference => unimplemented!("RestoreArrayReference not yet implemented"),
                    GMExtendedKind::IsNullishValue => unimplemented!("IsNullishValue not yet implemented"),
                    GMExtendedKind::PushReference => unimplemented!("PushReference not yet implemented"),
                }
            }

            GMInstruction::Extended32(instr) => {
                unimplemented!("extended32 not yet implemented")
            }

            GMInstruction::ExtendedFunc(instr) => {
                unimplemented!("extendedfunc not yet implemented")
            }
        }
    }

    Ok(())
}


fn validate_stacktop(item: VMStackItem) -> Result<(), CodeValidationError> {
    let VMStackItem::Int32(Some(int32)) = item else {
        return Err(CodeValidationError::SetOwnerNotInt32(item))
    };
    let int16: i16 = i16::try_from(int32).map_err(|_| CodeValidationError::Int16OutOfBounds(int32))?;
    let instance_type: GMInstanceType = parse_instance_type(int16, GMVariableType::Array)
        .map_err(|_| CodeValidationError::InvalidInt16InstanceType(int16))?;
    if matches!(instance_type, GMInstanceType::Self_(Some(_)) | GMInstanceType::RoomInstance(_) | GMInstanceType::Undefined) {
        return Err(CodeValidationError::UnacceptableInstanceType(instance_type))
    }
    Ok(())
}


fn generate_address_map(instructions: &[GMInstruction]) -> BiMap<usize, u32> {
    let mut map: BiHashMap<usize, u32> = BiMap::with_capacity(instructions.len());
    let mut current_address: u32 = 0;

    for (i, instruction) in instructions.iter().enumerate() {
        map.insert(i, current_address);
        current_address += get_instruction_size(instruction);
    }

    // // insert end block
    // map.insert(current_address, instructions.len());

    map
}


fn get_instruction_size(instruction: &GMInstruction) -> u32 {
    match instruction {
        GMInstruction::Pop(_) => 2,
        GMInstruction::Push(instr) |
        GMInstruction::PushLocal(instr) |
        GMInstruction::PushGlobal(instr) |
        GMInstruction::PushBuiltin(instr) |
        GMInstruction::PushImmediate(instr) => match instr.value {
            GMCodeValue::Int16(_) => 1,
            GMCodeValue::Int64(_) => 3,
            GMCodeValue::Double(_) => 3,
            _ => 2,
        }
        GMInstruction::Call(_) => 2,
        GMInstruction::Extended32(_) => 2,
        GMInstruction::ExtendedFunc(_) => 2,
        _ => 1,
    }
}


#[derive(Debug, Clone)]
pub struct VMStack {
    items: Vec<VMStackItem>,
}

impl VMStack {
    fn new() -> Self {
        Self { items: Vec::with_capacity(3) }
    }

    fn push(&mut self, item: VMStackItem) {
        self.items.push(item);
    }

    fn push_data_type(&mut self, data_type: GMDataType) {
        self.push(VMStackItem::from_data_type(data_type));
    }

    fn pop(&mut self) -> Result<VMStackItem, CodeValidationError> {
        self.items.pop().ok_or(CodeValidationError::PopStackEmpty)
    }

    fn peek(&self) -> Result<&VMStackItem, CodeValidationError> {
        self.items.last().ok_or(CodeValidationError::PeekStackEmpty)
    }

    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}


#[derive(Debug, Clone, PartialEq)]
enum VMStackItem {
    Int32(Option<i32>),
    Int64,
    Float,
    Double,
    Boolean,
    Variable,
    String,
}

impl VMStackItem {
    fn from_data_type(data_type: GMDataType) -> Self {
        match data_type {
            GMDataType::Int32 | GMDataType::Int16 => Self::Int32(None),
            GMDataType::Int64 => Self::Int64,
            GMDataType::Float => Self::Float,
            GMDataType::Double => Self::Double,
            GMDataType::Boolean => Self::Boolean,
            GMDataType::Variable => Self::Variable,
            GMDataType::String => Self::String,
        }
    }

    fn from_int32(value: i32) -> Self {
        Self::Int32(Some(value))
    }

    fn assert_data_type(&self, data_type: GMDataType) -> Result<(), CodeValidationError> {
        match (self, data_type) {
            (Self::Int32(_), GMDataType::Int16) => {}
            (Self::Int32(_), GMDataType::Int32) => {}
            (Self::Int64, GMDataType::Int64) => {}
            (Self::Float, GMDataType::Float) => {}
            (Self::Double, GMDataType::Double) => {}
            (Self::Boolean, GMDataType::Boolean) => {}
            (Self::Variable, GMDataType::Variable) => {}
            (Self::String, GMDataType::String) => {}
            (_, _) => return Err(CodeValidationError::TypeMismatch(self.clone(), data_type)) //todo
        }
        Ok(())
    }
}


