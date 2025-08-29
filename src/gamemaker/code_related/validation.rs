// use std::collections::HashMap;
// use bimap::{BiHashMap, BiMap};
// use crate::gamemaker::data::GMData;
// use crate::gamemaker::elements::code::{GMCode, GMCodeValue, GMInstruction};
//
// #[derive(Debug)]
// pub enum CodeValidationError {
//     ParentCodeUnresolvable(u32),
//     InitialOffsetInvalid(u32),
//     CodeEndWithoutExit,
// }
//
//
// pub fn validate_code(code: &GMCode, gm_data: &GMData) -> Result<(), CodeValidationError> {
//     let mut initial_offset: u32 = 0;
//     let mut instructions: &Vec<GMInstruction> = &code.instructions;
//
//     if let Some(b15_info) = &code.bytecode15_info {
//         if let Some(parent_ref) = b15_info.parent {
//             let parent: &GMCode = parent_ref.resolve(&gm_data.codes.codes)
//                 .map_err(|_| CodeValidationError::ParentCodeUnresolvable(parent_ref.index))?;
//             instructions = &parent.instructions;
//             initial_offset = b15_info.offset;
//         }
//     }
//
//     let address_map: BiMap<usize, u32> = generate_address_map(instructions);
//     let mut instruction_index: usize = *address_map.get_by_right(&initial_offset)
//         .ok_or(CodeValidationError::InitialOffsetInvalid(initial_offset))?;
//
//     loop {
//         let instruction: &GMInstruction = instructions.get(instruction_index)
//             .ok_or(CodeValidationError::CodeEndWithoutExit)?;
//         // match instruction {
//         // }
//     }
//
//     Ok(())
// }
//
//
// fn generate_address_map(instructions: &[GMInstruction]) -> BiMap<usize, u32> {
//     let mut map: BiHashMap<usize, u32> = BiMap::with_capacity(instructions.len());
//     let mut current_address: u32 = 0;
//
//     for (i, instruction) in instructions.iter().enumerate() {
//         map.insert(i, current_address);
//         current_address += get_instruction_size(instruction);
//     }
//
//     // // insert end block
//     // map.insert(current_address, instructions.len());
//
//     map
// }
//
//
// fn get_instruction_size(instruction: &GMInstruction) -> u32 {
//     // TODO: maybe caching these sizes is faster (store in list)? but needs benchmarks
//     match &instruction.kind {
//         GMInstructionData::Empty => 1,
//         GMInstructionData::SingleType(_) => 1,
//         GMInstructionData::Duplicate(_) => 1,
//         GMInstructionData::DuplicateSwap(_) => 1,
//         GMInstructionData::CallVariable(_) => 1,
//         GMInstructionData::DoubleType(_) => 1,
//         GMInstructionData::Comparison(_) => 1,
//         GMInstructionData::Goto(_) => 1,
//         GMInstructionData::Pop(_) => 2,
//         GMInstructionData::PopSwap(_) => 2,
//         GMInstructionData::Push(instr) => match instr.value {
//             GMCodeValue::Int16(_) => 1,
//             GMCodeValue::Int32(_) => 2,
//             GMCodeValue::Float(_) => 2,
//             GMCodeValue::Boolean(_) => 2,
//             GMCodeValue::String(_) => 2,
//             GMCodeValue::Variable(_) => 2,
//             GMCodeValue::Function(_) => 2,
//             GMCodeValue::Int64(_) => 3,
//             GMCodeValue::Double(_) => 3,
//         }
//         GMInstructionData::Call(_) => 2,
//         GMInstructionData::Extended16(_) => 1,
//         GMInstructionData::Extended32(_) => 2,
//         GMInstructionData::ExtendedFunc(_) => 2,
//     }
// }
//
//
// mod extract_instruction_data {
//     use crate::gamemaker::elements::code::{GMInstructionData, GMSingleTypeInstruction};
//
//     #[derive(Debug)]
//     pub enum ExpectedInstructionData<'a> {
//         Empty(&'a GMInstructionData),
//         SingleType(&'a GMInstructionData),
//     }
//
//     fn empty(instruction_data: &GMInstructionData) -> Result<(), ExpectedInstructionData> {
//         match instruction_data {
//             GMInstructionData::Empty => Ok(()),
//             _ => Err(ExpectedInstructionData::Empty(instruction_data))
//         }
//     }
//
//
//     fn single_type(instruction_data: &GMInstructionData) -> Result<&GMSingleTypeInstruction, ExpectedInstructionData> {
//         match instruction_data {
//             GMInstructionData::SingleType(instr) => Ok(instr),
//             _ => Err(ExpectedInstructionData::SingleType(instruction_data))
//         }
//     }
// }
//
