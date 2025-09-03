use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use bimap::{BiHashMap, BiMap};
use crate::gamemaker::data::GMData;
use crate::gamemaker::elements::code::{get_data_type_from_value, get_instruction_size, parse_instance_type, GMCode, GMCodeValue, GMDataType, GMInstanceType, GMInstruction, GMVariableType};
use crate::gml::disassembler::disassemble_instruction;

#[derive(Debug)]
pub enum CodeValidationError {
    CodeEndWithoutExit(usize, usize),
    PopStackEmpty,
    PeekStackEmpty,
    DupSizeOutOfBounds,
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
    PushEnvNotInt32(VMStackItem),
    StackTopNotInt32(VMStackItem),
    InstanceTypeNotInt32(VMStackItem),
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
            CodeValidationError::DupSizeOutOfBounds => write!(f, "Tried to duplicate more items than the stack currently holds"),
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
            CodeValidationError::PushEnvNotInt32(item) => write!(f, "Expected Int32 value for PushEnv instruction but found {item:?}"),
            CodeValidationError::StackTopNotInt32(item) => write!(f, "Top data type is {item:?} instead of Int32 (with value) for stacktop variable type"),
            CodeValidationError::InstanceTypeNotInt32(item) => write!(f, "Top data type is {item:?} instead of Int32 (with value) for instance type"),
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

    validate_instructions(gm_data, &address_map, instructions, instruction_index)?;
    Ok(())
}


fn validate_instructions(
    gm_data: &GMData,
    address_map: &BiMap<usize, u32>,
    instructions: &Vec<GMInstruction>,
    start_index: usize,
) -> Result<(), CodeValidationError> {
    let mut states_to_visit = Vec::new();
    states_to_visit.push((start_index, VMStack::new()));
    let mut visited_branch_targets = HashSet::new();

    while let Some((mut instruction_index, mut stack)) = states_to_visit.pop() {
        loop {
            if instruction_index == instructions.len() {
                if !stack.is_empty() {
                    return Err(CodeValidationError::ExitStackLeftover(stack))
                }
                break
            }

            let instruction: &GMInstruction = &instructions[instruction_index];
            let instruction_address: u32 = *address_map.get_by_left(&instruction_index).unwrap();
            let _debug_stack = stack.items.iter().map(|i| match i {
                VMStackItem::Int32(_) => 'i',
                VMStackItem::Int64 => 'l',
                VMStackItem::Float => 'f',
                VMStackItem::Double => 'd',
                VMStackItem::Boolean => 'b',
                VMStackItem::Variable => 'v',
                VMStackItem::String => 's',
            }).collect::<String>();
            log::debug!("{} | {} {} | {}", instruction_index, stack.items.len(), _debug_stack, disassemble_instruction(gm_data, instruction).unwrap());
            instruction_index += 1;

            match instruction {
                GMInstruction::Convert(instr) => {
                    let item: VMStackItem = stack.pop()?;
                    item.assert_data_type(instr.right)?;
                    stack.push_data_type(instr.left);
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
                    let right: VMStackItem = stack.pop()?;
                    let left: VMStackItem = stack.pop()?;
                    right.assert_data_type(instr.right)?;
                    left.assert_data_type(instr.left)?;
                    let result_type: GMDataType = binary_operation_result_type(instruction, instr.right, instr.left);
                    stack.push_data_type(result_type);
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
                    let var_type: GMVariableType = instr.destination.variable_type;
                    match var_type {
                        GMVariableType::Normal => {
                            if instr.type1 != GMDataType::Variable {
                                return Err(CodeValidationError::NormalPopFirstNotVar(instr.type1))
                            }
                            stack.pop()?.assert_data_type(instr.type2)?;
                        }
                        GMVariableType::Array => {
                            match instr.type1 {
                                GMDataType::Variable => {
                                    stack.pop()?.assert_data_type(GMDataType::Int32)?;  // index
                                    validate_instance_type(stack.pop()?)?;         // instance type
                                    stack.pop()?.assert_data_type(instr.type2)?;        // actual value
                                }
                                GMDataType::Int32 => {
                                    stack.pop()?.assert_data_type(instr.type2)?;        // actual value
                                    stack.pop()?.assert_data_type(GMDataType::Int32)?;  // index
                                    validate_instance_type(stack.pop()?)?;         // instance type
                                }
                                _ => unimplemented!("unexpected data type 1 for pop array")
                            }
                        }
                        GMVariableType::StackTop => {
                            if instr.type1 == GMDataType::Int32 {
                                stack.pop()?.assert_data_type(instr.type2)?;
                                stack.pop()?.assert_data_type(GMDataType::Int32)?;
                            } else if instr.type1 == GMDataType::Variable {
                                stack.pop()?.assert_data_type(GMDataType::Int32)?;  // instance type / object index
                                stack.pop()?.assert_data_type(instr.type2)?;        // actual value
                            } else {
                                unimplemented!("didnt expect pop type1 (stacktop) to be neither var not int32")
                            }
                        }
                        GMVariableType::Instance => {
                            unimplemented!("pop instance")
                        }
                        GMVariableType::ArrayPushAF => {
                            unimplemented!("pop ArrayPushAF")
                        }
                        GMVariableType::ArrayPopAF => {
                            unimplemented!("pop ArrayPopAF")
                        }
                    }
                }

                GMInstruction::PopSwap(instr) => {
                    unimplemented!("popswap not yet implemented")
                }

                GMInstruction::Duplicate(instr) => {
                    let last_index: usize = stack.len() - 1;
                    if instr.size as usize > last_index {
                        return Err(CodeValidationError::DupSizeOutOfBounds)
                    }
                    for i in (0..=instr.size).rev() {
                        let item: &VMStackItem = &stack.items[last_index - i as usize];
                        item.assert_data_type(instr.data_type)?;
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
                GMInstruction::BranchUnless(instr) |
                GMInstruction::PushWithContext(instr) |
                GMInstruction::PopWithContext(instr) => {
                    let conditional: bool = matches!(instruction, GMInstruction::BranchIf(_) | GMInstruction::BranchUnless(_));
                    if conditional {
                        let item: VMStackItem = stack.pop()?;
                        match item {
                            // this will probably have to be extended to non literal ints too
                            VMStackItem::Int32(Some(1)) => {}
                            VMStackItem::Int32(Some(0)) => {}
                            VMStackItem::Boolean => {}
                            _ => return Err(CodeValidationError::BranchConditionNotBool(item))
                        }
                    } else if matches!(instruction, GMInstruction::PushWithContext(_)) {
                        let item: VMStackItem = stack.pop()?;
                        if !matches!(item, VMStackItem::Int32(_)) {
                            return Err(CodeValidationError::PushEnvNotInt32(item))
                        }
                    }

                    let address_target: u32 = (instruction_address as i32 + instr.jump_offset) as u32;
                    let branch_target_index: usize = *address_map.get_by_right(&address_target)
                        .ok_or(CodeValidationError::InvalidBranchTarget(address_target))?;

                    if conditional {
                        if visited_branch_targets.insert((branch_target_index, stack.clone())) {
                            // conditional branch - never visited; add to list
                            states_to_visit.push((branch_target_index, stack.clone()));
                        }
                    } else if visited_branch_targets.insert((branch_target_index, stack.clone())) {
                        // unconditional branch - never visited; branch now
                        instruction_index = branch_target_index;
                    } else {
                        // unconditional branch - already visited; stop execution
                        break
                    }
                }

                GMInstruction::PopWithContextExit(_) => unimplemented!("popenv exit magic"),

                GMInstruction::Push(instr) |
                GMInstruction::PushLocal(instr) |
                GMInstruction::PushGlobal(instr) |
                GMInstruction::PushBuiltin(instr) |
                GMInstruction::PushImmediate(instr) => {
                    let data_type: GMDataType = get_data_type_from_value(&instr.value);

                    if let GMCodeValue::Variable(code_var) = &instr.value {
                        match code_var.variable_type {
                            GMVariableType::Normal => {}
                            GMVariableType::Array => {
                                stack.pop()?.assert_data_type(GMDataType::Int32)?;  // index
                                validate_instance_type(stack.pop()?)?;         // instance type
                            }
                            GMVariableType::StackTop => {
                                stack.pop()?.assert_data_type(GMDataType::Int32)?;  // instance type / object index
                            }
                            GMVariableType::Instance => unimplemented!("push Instance"),
                            GMVariableType::ArrayPushAF => unimplemented!("push ArrayPushAF"),
                            GMVariableType::ArrayPopAF => unimplemented!("push ArrayPopAF"),
                        }
                        // let var = code_var.variable.resolve(&gm_data.variables.variables)
                        //     .map_err(|_| CodeValidationError::VariableUnresolvable(code_var.variable.index))?;
                        // if let Some(b15) = &var.b15_data && b15.instance_type !=
                    }

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

                GMInstruction::SetArrayOwner => {
                    let item: VMStackItem = stack.pop()?;
                    validate_instance_type(item)?;
                }

                GMInstruction::CheckArrayIndex => unimplemented!("CheckArrayIndex instruction"),
                GMInstruction::PushArrayFinal => unimplemented!("PushArrayFinal instruction"),
                GMInstruction::PopArrayFinal => unimplemented!("PopArrayFinal instruction"),
                GMInstruction::PushArrayContainer => unimplemented!("PushArrayContainer instruction"),
                GMInstruction::HasStaticInitialized => unimplemented!("HasStaticInitialized instruction"),
                GMInstruction::SetStaticInitialized => unimplemented!("SetStaticInitialized instruction"),
                GMInstruction::SaveArrayReference => unimplemented!("SaveArrayReference instruction"),
                GMInstruction::RestoreArrayReference => unimplemented!("RestoreArrayReference instruction"),
                GMInstruction::IsNullishValue => unimplemented!("IsNullishValue instruction"),
                GMInstruction::PushReference(_) => unimplemented!("PushReference instruction"),
            }
        }
    }

    Ok(())
}


fn validate_instance_type(item: VMStackItem) -> Result<(), CodeValidationError> {
    let int32: i32 = match item {
        VMStackItem::Int32(Some(int)) => int,
        VMStackItem::Int32(None) => return Ok(()),
        _ => return Err(CodeValidationError::InstanceTypeNotInt32(item))
    };
    let int16: i16 = i16::try_from(int32).map_err(|_| CodeValidationError::Int16OutOfBounds(int32))?;
    let instance_type: GMInstanceType = parse_instance_type(int16, GMVariableType::Array)
        .map_err(|_| CodeValidationError::InvalidInt16InstanceType(int16))?;
    if matches!(instance_type, GMInstanceType::RoomInstance(_) | GMInstanceType::Undefined) {
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

    // insert end block
    map.insert(instructions.len(), current_address);
    map
}


#[derive(Debug, Clone, PartialEq, Hash, Eq)]
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

    fn len(&self) -> usize {
        self.items.len()
    }

    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}


#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum VMStackItem {
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


fn binary_operation_result_type(instruction: &GMInstruction, right: GMDataType, left: GMDataType) -> GMDataType {
    match instruction {
        GMInstruction::Compare(_) => return GMDataType::Boolean,
        GMInstruction::Subtract(_) |
        GMInstruction::Divide(_) |
        GMInstruction::Modulus(_) |
        GMInstruction::And(_) |
        GMInstruction::Or(_) |
        GMInstruction::Xor(_) |
        GMInstruction::ShiftLeft(_) |
        GMInstruction::ShiftRight(_) => if right == GMDataType::String || left == GMDataType::String {
            return GMDataType::Double
        },
        GMInstruction::Remainder(_) => if (right == GMDataType::String && left != GMDataType::Variable) || left == GMDataType::String {
            return GMDataType::Double
        },
        _ => {}
    }

    // Choose whichever type has a higher bias, or if equal, the smaller numerical data type value.
    match stack_type_bias(left).cmp(&stack_type_bias(right)) {
        Ordering::Greater => left,
        Ordering::Equal => if u8::from(left) < u8::from(right) {left} else {right},
        Ordering::Less => right,
    }
}


fn stack_type_bias(data_type: GMDataType) -> u8 {
    match data_type {
        GMDataType::Variable => 2,
        GMDataType::Double | GMDataType::Int64 => 1,
        _ => 0
    }
}

