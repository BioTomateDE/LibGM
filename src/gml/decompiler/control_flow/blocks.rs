use crate::gamemaker::elements::code::{get_instruction_size, GMInstruction};
use crate::gml::decompiler::control_flow::{BaseNode, ControlFlowGraph, NodeRef, Successors};
use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone)]
pub struct Block<'a> {
    pub base_node: BaseNode,
    pub instructions: &'a [GMInstruction],
}
impl<'a> Block<'a> {
    pub fn new(start_address: u32, end_address: u32, instructions: &'a [GMInstruction]) -> Self {
        Self {
            base_node: BaseNode::new(start_address, end_address),
            instructions,
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


fn generate_address_map(instructions: &[GMInstruction]) -> HashMap<u32, usize> {
    // TODO: use vec with binary searching for performance
    let mut map: HashMap<u32, usize> = HashMap::with_capacity(instructions.len());
    let mut current_address: u32 = 0;

    for (i, instruction) in instructions.iter().enumerate() {
        map.insert(current_address, i);
        current_address += get_instruction_size(instruction);
    }

    // Insert end block
    map.insert(current_address, instructions.len());
    map
}


pub fn find_blocks<'a>(cfg: &mut ControlFlowGraph<'a>, instructions: &'a [GMInstruction]) {
    if instructions.is_empty() {
        return
    }

    // 1. Generate address map
    let address_map: HashMap<u32, usize> = generate_address_map(instructions);

    // 2. Get nodes and branch table
    let mut current_address: u32 = 0;
    let mut nodes_set: HashSet<u32> = HashSet::new();
    let mut branch_table: HashMap<usize, u32> = HashMap::new();

    for (current_index, instruction) in instructions.iter().enumerate() {
        match instruction {
            GMInstruction::Exit(_) |
            GMInstruction::Return(_) => {
                nodes_set.insert(current_address + 1);
            }
            GMInstruction::Branch(instr) |
            GMInstruction::BranchIf(instr) |
            GMInstruction::BranchUnless(instr) |
            GMInstruction::PushWithContext(instr) |
            GMInstruction::PopWithContext(instr) => {
                let target_address: u32 = (current_address as i32 + instr.jump_offset) as u32;
                branch_table.insert(current_index, target_address);
                nodes_set.insert(current_address + 1);
                nodes_set.insert(target_address);
            }
            // TODO: handle call to @@try_hook@@ function
            _ => {}
        }
        current_address += get_instruction_size(instruction);
    }
    let code_end_address: u32 = current_address;

    // Push end node, convert nodes to sorted vec
    nodes_set.insert(instructions.len() as u32);
    let mut nodes_list: Vec<u32> = nodes_set.into_iter().collect();
    nodes_list.sort_unstable();

    // 3. Create blocks
    cfg.blocks = Vec::with_capacity(nodes_list.len());
    let mut current_address: u32 = 0;
    let mut current_index: usize = 0;

    for instruction_address in nodes_list {
        if instruction_address > current_address {
            let instruction_index = address_map[&instruction_address];
            cfg.blocks.push(Block::new(current_address, instruction_address, &instructions[current_index..instruction_index]));
            current_index = instruction_index;
        }
        current_address = instruction_address;
    }

    // Add end block
    let block_count = cfg.blocks.len();
    cfg.blocks.push(Block::new(code_end_address, code_end_address, &[]));

    // 4. Populate predecessor and successor fields of blocks
    for block_index in 0..block_count {
        let block = &cfg.blocks[block_index];
        let last_instruction: &GMInstruction = block.instructions.last().unwrap();
        let last_instruction_address: u32 = block.end_address - 1;

        // TODO: handle call to @@try_hook@@ function (PopDiscard is relevant)

        match last_instruction {
            // This terminates code execution, so this block won't have any successors
            GMInstruction::Exit(_) | GMInstruction::Return(_) => continue,

            // Insert branch target to predecessor/successor list
            GMInstruction::Branch(instr) |
            GMInstruction::BranchIf(instr) |
            GMInstruction::BranchUnless(instr) |
            GMInstruction::PushWithContext(instr) |
            GMInstruction::PopWithContext(instr) => {
                let target_address: u32 = (last_instruction_address as i32 + instr.jump_offset) as u32;
                let target_block_index: usize = get_block_index(&cfg.blocks, target_address);

                let predecessor = NodeRef::block(block_index);
                let successor = NodeRef::block(target_block_index);

                successor.predecessors_mut(cfg).push(predecessor.clone());
                predecessor.successors_mut(cfg).branch_target = Some(successor);
            }
            _ => {}
        }

        // Only add sequential successor block if the block is not the last one
        if block_index < cfg.blocks.len() - 1 {
            cfg.blocks[block_index + 1].predecessors.push(NodeRef::block(block_index));
            cfg.blocks[block_index].successors.fall_through = Some(NodeRef::block(block_index + 1));
        }
    }
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

