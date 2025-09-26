use crate::gamemaker::elements::code::{get_instruction_size, GMCodeValue, GMDataType, GMDoubleTypeInstruction, GMInstruction, GMPushInstruction, GMSingleTypeInstruction};
use crate::gml::decompiler::control_flow::{BaseNode, ControlFlowGraph, NodeRef};
use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};
use crate::gamemaker::data::GMData;
use crate::gamemaker::deserialize::GMRef;
use crate::gamemaker::elements::functions::GMFunction;
use crate::gml::decompiler::vm_constants;
use crate::gml::disassembler::disassemble_instructions;

#[derive(Debug, Clone)]
pub struct Block<'a> {
    pub base_node: BaseNode,
    pub instructions: &'a [GMInstruction],
}

impl<'a> Block<'a> {
    pub fn new(start_address: u32) -> Self {
        Self {
            base_node: BaseNode::new(start_address, start_address),
            instructions: &[],
        }
    }

    pub fn pop_first_instruction(&mut self) {
        self.instructions = &self.instructions[1..];
    }
    pub fn pop_last_instruction(&mut self) {
        self.instructions = &self.instructions[0..self.instructions.len() - 1];
    }
}

impl<'a> Deref for Block<'a> {
    type Target = BaseNode;
    fn deref(&self) -> &Self::Target {
        &self.base_node
    }
}
impl<'a> DerefMut for Block<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base_node
    }
}


pub fn find_blocks<'a>(cfg: &mut ControlFlowGraph<'a>, instructions: &'a [GMInstruction]) -> Result<(), String> {
    // Get nodes, address map, and try blocks
    let mut nodes: HashSet<u32> = HashSet::new();
    let mut address_map: HashMap<u32, usize> = HashMap::with_capacity(instructions.len());
    let mut try_blocks: Vec<(u32, u32, Option<u32>)> = Vec::new();
    let mut current_address: u32 = 0;

    for (i, instruction) in instructions.iter().enumerate() {
        let instruction_size: u32 = get_instruction_size(instruction);
        match instruction {
            GMInstruction::Exit(_) |
            GMInstruction::Return(_) => {
                nodes.insert(current_address + instruction_size);
            }
            GMInstruction::Branch(instr) |
            GMInstruction::BranchIf(instr) |
            GMInstruction::BranchUnless(instr) |
            GMInstruction::PushWithContext(instr) |
            GMInstruction::PopWithContext(instr) => {
                let target_address: u32 = (current_address as i32 + instr.jump_offset) as u32;
                nodes.insert(target_address);
                nodes.insert(current_address + instruction_size);
            }
            GMInstruction::Call(instr) if is_function_try_hook(cfg.context.gm_data, instr.function)? => {
                // Handle @@try_hook@@ call
                const CONV: GMInstruction = GMInstruction::Convert(GMDoubleTypeInstruction { right: GMDataType::Int32, left: GMDataType::Int32 });
                let pattern = instructions.get(i.saturating_sub(4)..=i)
                    .ok_or_else(|| format!("@@try_hook@@ pattern extends beyond instruction bounds at index {}", i))?;
                let (finally_block, catch_block) = match pattern {
                    [
                        GMInstruction::Push(GMPushInstruction {value: GMCodeValue::Int32(finally)} ),
                        CONV,
                        GMInstruction::Push(GMPushInstruction {value: GMCodeValue::Int32(catch)} ),
                        CONV,
                        _,
                        GMInstruction::PopDiscard(GMSingleTypeInstruction{ data_type: GMDataType::Variable })
                    ] => (*finally, *catch),
                    _ => {
                        let actual_code: String = disassemble_instructions(cfg.context.gm_data, pattern)
                            .unwrap_or_else(|e| format!("<invalid instructions: \"{e}\">"));
                        return Err(format!(
                            "Malformed @@try_hook@@ pattern at instruction {i}: expected \
                            [push.i, conv.i.v, push.i, conv.i.v, call, popz.v], \
                            found [{actual_code}]",
                        ))
                    }
                };

                nodes.insert(current_address + instruction_size);
                nodes.insert(finally_block as u32);
                let catch_block_opt = if catch_block != -1 {
                    let addr = catch_block as u32;
                    nodes.insert(addr);
                    Some(addr)
                } else { None };
                try_blocks.push((current_address+instruction_size, finally_block as u32, catch_block_opt));

                // Split this try hook into its own block - removes edge cases in later graph operations
                nodes.insert(current_address - 24);  // address of finally instruction
                nodes.insert(current_address + 12);  // end address of popz instruction
            }
            _ => {}
        }
        address_map.insert(current_address, i);
        current_address += instruction_size;
    }
    let code_end_address: u32 = current_address;

    // Convert nodes to sorted vec, push end node
    let mut nodes: Vec<u32> = nodes.into_iter().collect();
    nodes.sort_unstable();
    nodes.push(code_end_address);

    // Create blocks
    cfg.blocks = Vec::with_capacity(nodes.len());
    let mut current_address: u32 = 0;
    let mut current_index: usize = 0;

    for instruction_address in nodes {
        // End previous block
        if let Some(last) = cfg.blocks.last_mut() {
            last.end_address = instruction_address;
            let start_index = address_map[&last.start_address];
            last.instructions = &instructions[start_index..current_index];
        }

        cfg.blocks.push(Block::new(current_address));
        current_address = instruction_address;
        current_index += 1;
    }

    // Populate predecessor and successor fields of blocks
    for block_index in 0..cfg.blocks.len() {
        let block = &cfg.blocks[block_index];
        let Some(last_instruction) = block.instructions.last() else {continue};

        match last_instruction {
            // This terminates code execution, so this block won't have any successors
            GMInstruction::Exit(_) | GMInstruction::Return(_) => continue,

            // Insert branch target to predecessor/successor list
            GMInstruction::Branch(instr) |
            GMInstruction::BranchIf(instr) |
            GMInstruction::BranchUnless(instr) |
            GMInstruction::PushWithContext(instr) |
            GMInstruction::PopWithContext(instr) => {
                let target_address: u32 = (block.end_address as i32 - 1 + instr.jump_offset) as u32;
                let target_block_index: usize = get_block_index(&cfg.blocks, target_address);

                let predecessor = NodeRef::block(block_index);
                let successor = NodeRef::block(target_block_index);

                successor.predecessors_mut(cfg).push(predecessor.clone());
                predecessor.successors_mut(cfg).branch_target = Some(successor);
            }

            GMInstruction::PopDiscard(_) if block.instructions.len() == 6 => {
                if let GMInstruction::Call(instr) = &block.instructions[4] {
                    if is_function_try_hook(cfg.context.gm_data, instr.function)? {
                        // We've found a try hook - connect to targets
                        let (finally_address, catch_address) = get_try_catch(&try_blocks, block.end_address);

                        // Connect finally block
                        let finally_block_index: usize = get_block_index(&cfg.blocks, finally_address);
                        cfg.blocks[block_index].successors.fall_through = Some(NodeRef::block(finally_block_index));
                        cfg.blocks[finally_block_index].predecessors.push(NodeRef::block(block_index));

                        // Connect catch block, if available
                        if let Some(catch_addr) = catch_address {
                            let catch_block_index: usize = get_block_index(&cfg.blocks, catch_addr);
                            cfg.blocks[block_index].successors.catch = Some(NodeRef::block(catch_block_index));
                            cfg.blocks[catch_block_index].predecessors.push(NodeRef::block(block_index));
                        }
                    }
                }
            }
            _ => {}
        }

        // Only add sequential successor block (fallthrough) if the block is not
        // the last one in the code, and it isn't unconditional branch.
        if block_index < cfg.blocks.len() - 1 && !matches!(last_instruction, GMInstruction::Branch(_)) {
            cfg.blocks[block_index + 1].predecessors.push(NodeRef::block(block_index));
            cfg.blocks[block_index].successors.fall_through = Some(NodeRef::block(block_index + 1));
        }

        // Compute blocks that are unreachable
        for block in &mut cfg.blocks {
            if block.predecessors.is_empty() {
                block.unreachable = true;
            }
        }
    }

    Ok(())
}


/// Convert an instruction address to a block index.
/// In other words, find the block this instruction belongs to.
/// # Panic Safety
/// This function will panic if `blocks` is empty, unordered,
/// or if the block `start_index`/`end_index` fields are malformed.
fn get_block_index(blocks: &[Block], instruction_address: u32) -> usize {
    if blocks.is_empty() {
        unreachable!("Cannot find block in empty blocks array")
    }

    let mut low: usize = 0;
    let mut high: usize = blocks.len() - 1;

    while low <= high {
        let mid: usize = (low + high) / 2;
        let block: &Block = &blocks[mid];

        if instruction_address < block.start_address {
            high = mid - 1;
        } else if instruction_address >= block.end_address {
            low = mid + 1;
        } else {
            return mid
        }
    }

    unreachable!(
        "Could not find block for instruction index {} in {} blocks (range: {}-{})",
        instruction_address,
        blocks.len(),
        blocks[0].start_address,
        blocks.last().unwrap().end_address,
    );
}


fn get_try_catch(try_blocks: &Vec<(u32, u32, Option<u32>)>, pop_delete_address: u32) -> (u32, Option<u32>) {
    // Binary search is probably faster but idrc because try catches are extremely rare (in deltarune code)
    for (pop_delete_addr, finally_addr, catch_addr) in try_blocks {
        if *pop_delete_addr == pop_delete_address {
            return (*finally_addr, *catch_addr)
        }
    }

    unreachable!("Could not find try-catch block for PopDelete instruction at {pop_delete_address}");
}


fn is_function_try_hook(gm_data: &GMData, function_ref: GMRef<GMFunction>) -> Result<bool, String> {
    let function: &GMFunction = function_ref.resolve(&gm_data.functions.functions)?;
    let name: &String = function.name.resolve(&gm_data.strings.strings)?;
    Ok(name == vm_constants::functions::TRY_HOOK)
}

