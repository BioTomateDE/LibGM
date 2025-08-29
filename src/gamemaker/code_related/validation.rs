use std::fmt::{Display, Formatter};
use bimap::{BiHashMap, BiMap};
use crate::gamemaker::elements::code::{get_data_type_from_value, GMCode, GMCodeValue, GMCodes, GMDataType, GMInstanceType, GMInstruction, GMVariableType};

#[derive(Debug)]
pub enum CodeValidationError {
    CodeEndWithoutExit(usize, usize),
    PopStackEmpty,
    PeekStackEmpty,
    TypeMismatch,
    ParentCodeUnresolvable(u32),
    InvalidInitialOffset(u32),
    InvalidBranchTarget(u32),
    ExitStackLeftover(VMStack),
    BranchStackLeftover(VMStack),
    PushImmediateNotInt16(GMDataType),
    PushVarWrongDataType(GMDataType),
    PushVarWrongInstanceType(GMInstanceType),
    CallArgumentNotVar(GMDataType),
}

impl Display for CodeValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CodeValidationError::CodeEndWithoutExit(a, b) => write!(f, "Code instructions ended without Exit or Return instruction ({a} > {b})"),
            CodeValidationError::PopStackEmpty => write!(f, "Tried to pop value from empty stack"),
            CodeValidationError::PeekStackEmpty => write!(f, "Tried to peek value from empty stack"),
            CodeValidationError::TypeMismatch => write!(f, "Data type on stack does not match data type of instruction"),
            CodeValidationError::ParentCodeUnresolvable(idx) => write!(f, "Could not resolve parent code with index {idx}"),
            CodeValidationError::InvalidInitialOffset(offset) => write!(f, "Child code instruction offset {offset} does not point to the start of an instruction"),
            CodeValidationError::InvalidBranchTarget(addr) => write!(f, "Branch instruction with target address {addr} does not point to the start of an instruction"),
            CodeValidationError::ExitStackLeftover(stack) => write!(f, "Code exited with leftover stack data: {:?}", stack.items),
            CodeValidationError::BranchStackLeftover(stack) => write!(f, "Code branched with leftover stack data (this is an internal issue): {:?}", stack.items),
            CodeValidationError::PushImmediateNotInt16(dtype) => write!(f, "PushImmediate instruction's data type is {dtype:?} instead of Int16"),
            CodeValidationError::PushVarWrongDataType(dtype) => write!(f, "Push instruction for variable (local/global/builtin) has data type {dtype:?} instead Variable"),
            CodeValidationError::PushVarWrongInstanceType(inst) => write!(f, "Push instruction for variable (local/global/builtin) wrong instance type {inst}"),
            CodeValidationError::CallArgumentNotVar(dtype) => write!(f, "Stack value popped as argument for function call has data type {dtype:?} instead of Variable"),
        }
    }
}


pub fn validate_code(code: &GMCode, all_codes: &GMCodes) -> Result<(), CodeValidationError> {
    let mut instruction_address: u32 = 0;
    let mut instructions: &Vec<GMInstruction> = &code.instructions;

    if let Some(b15_info) = &code.bytecode15_info {
        if let Some(parent_ref) = b15_info.parent {
            let parent: &GMCode = parent_ref.resolve(&all_codes.codes)
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

    validate_instructions(&address_map, instructions, instruction_index)?;
    Ok(())
}


fn validate_instructions(
    address_map: &BiMap<usize, u32>,
    instructions: &Vec<GMInstruction>,
    mut instruction_index: usize,
) -> Result<(), CodeValidationError> {
    let mut stack = VMStack::new();

    while instruction_index < instructions.len() {
        // TODO: clean this part up
        let instruction: &GMInstruction = instructions.get(instruction_index)
            .ok_or(CodeValidationError::CodeEndWithoutExit(instruction_index, instructions.len()))?;
        let instruction_address: u32 = *address_map.get_by_left(&instruction_index).unwrap();
        log::debug!("{} {} {:?}", instruction_index, instruction_address, instruction);

        match instruction {
            GMInstruction::Convert(instr) => {
                let origin_type: GMDataType = stack.pop()?;
                assert_type(instr.type1, origin_type)?;
                stack.push(instr.type2);
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
                let type2: GMDataType = stack.pop()?;
                let type1: GMDataType = stack.pop()?;
                assert_type(instr.type1, type1)?;
                assert_type(instr.type2, type2)?;
                stack.push(instr.type2);
            }

            GMInstruction::Negate(instr) |
            GMInstruction::Not(instr) => {
                let stack_type: GMDataType = stack.pop()?;
                assert_type(instr.data_type, stack_type)?;
                stack.push(instr.data_type);
            }

            GMInstruction::Compare(instr) => {
                let type2: GMDataType = stack.pop()?;
                let type1: GMDataType = stack.pop()?;
                assert_type(instr.type1, type1)?;
                assert_type(instr.type2, type2)?;
                stack.push(GMDataType::Boolean);
            }

            GMInstruction::Pop(instr) => {
                let stack_type: GMDataType = stack.pop()?;
                if instr.destination.variable_type != GMVariableType::Normal {
                    assert_type(instr.type1, GMDataType::Variable)?;    // TODO assumption
                } else {
                    assert_type(instr.type1, stack_type)?;
                }

                // TODO: assumption
                if instr.destination.is_int32 {
                    assert_type(instr.type2, GMDataType::Int32)?;
                } else {
                    assert_type(instr.type2, GMDataType::Variable)?;
                }
            }

            GMInstruction::PopSwap(instr) => {
                unimplemented!("popswap not yet implemented")
            }

            GMInstruction::Duplicate(instr) => {
                // TODO: this is an assumption. idk how dup works.
                let data_type: GMDataType = stack.peek()?;
                assert_type(instr.data_type, data_type)?;
                for _ in 0..instr.size+1 {
                    stack.push(data_type);
                }
            }

            GMInstruction::DuplicateSwap(instr) => {
                unimplemented!("dupswap not yet implemented")
            }

            GMInstruction::Return(instr) => {
                let ret_type: GMDataType = stack.pop()?;
                assert_type(instr.data_type, ret_type)?;
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
                let stack_type: GMDataType = stack.pop()?;
                assert_type(instr.data_type, stack_type)?;
            }

            GMInstruction::Branch(instr) |
            GMInstruction::BranchIf(instr) |
            GMInstruction::BranchUnless(instr) => {
                if !stack.is_empty() {
                    return Err(CodeValidationError::BranchStackLeftover(stack))
                }

                let branch_target_index: usize;
                if let Some(address_offset) = instr.jump_offset {
                    let address_target: u32 = (instruction_address as i32 + address_offset) as u32;
                    branch_target_index = *address_map.get_by_right(&address_target)
                        .ok_or(CodeValidationError::InvalidBranchTarget(address_target))?;
                } else {
                    unimplemented!("popenv exit magic not yet implemented")
                }

                if matches!(instruction, GMInstruction::BranchIf(_) | GMInstruction::BranchUnless(_)) {
                    // perform branch on recursive call; skip branch in this execution
                    validate_instructions(address_map, instructions, branch_target_index)?;
                    continue
                }

                instruction_index = branch_target_index;
                continue
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
                let mut data_type: GMDataType = get_data_type_from_value(&instr.value);

                if matches!(instruction, GMInstruction::PushLocal(_) | GMInstruction::PushGlobal(_) | GMInstruction::PushBuiltin(_)) {
                    let GMCodeValue::Variable(code_variable) = &instr.value else {
                        return Err(CodeValidationError::PushVarWrongDataType(data_type))
                    };
                    match (instruction, &code_variable.instance_type) {
                        (GMInstruction::PushLocal(_), GMInstanceType::Local) => {}
                        (GMInstruction::PushGlobal(_), GMInstanceType::Global) => {}
                        (GMInstruction::PushBuiltin(_), GMInstanceType::Builtin) => {}
                        (_, _) => return Err(CodeValidationError::PushVarWrongInstanceType(code_variable.instance_type.clone()))
                    }
                }

                if matches!(instruction, GMInstruction::PushImmediate(_)) && data_type != GMDataType::Int16 {
                    return Err(CodeValidationError::PushImmediateNotInt16(data_type))
                }

                // Convert Int16 to Int32 when pushing
                if data_type == GMDataType::Int16 {
                    data_type = GMDataType::Int32;
                }
                stack.push(data_type);
            }

            GMInstruction::Call(instr) => {
                for _ in 0..instr.arguments_count {
                    let data_type: GMDataType = stack.pop()?;
                    if data_type != GMDataType::Variable {
                        return Err(CodeValidationError::CallArgumentNotVar(data_type))
                    }
                }
                // TODO: what does instr.data_type do??? it's always Int32
                // TODO: return type is an assumption
                stack.push(GMDataType::Variable);
            }

            GMInstruction::CallVariable(instr) => {
                unimplemented!("callvar not yet implemeneted")
            }

            GMInstruction::Extended16(instr) => {
                unimplemented!("extended16 not yet implemeneted")
            }

            GMInstruction::Extended32(instr) => {
                unimplemented!("extended32 not yet implemeneted")
            }

            GMInstruction::ExtendedFunc(instr) => {
                unimplemented!("extendedfunc not yet implemeneted")
            }
        }

        instruction_index += 1;
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


#[derive(Debug)]
pub struct VMStack {
    items: Vec<GMDataType>,
}

impl VMStack {
    fn new() -> Self {
        Self { items: Vec::with_capacity(3) }
    }

    fn push(&mut self, data_type: GMDataType) {
        self.items.push(data_type);
    }

    fn pop(&mut self) -> Result<GMDataType, CodeValidationError> {
        self.items.pop().ok_or(CodeValidationError::PopStackEmpty)
    }

    fn peek(&self) -> Result<GMDataType, CodeValidationError> {
        self.items.last().copied().ok_or(CodeValidationError::PeekStackEmpty)
    }

    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

fn assert_type(instruction_type: GMDataType, stack_type: GMDataType) -> Result<(), CodeValidationError> {
    if instruction_type == stack_type {
        return Ok(())
    }
    Err(CodeValidationError::TypeMismatch)
}
