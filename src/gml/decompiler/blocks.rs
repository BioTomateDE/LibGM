use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use many_to_many::ManyToMany;
use crate::gamemaker::elements::code::{get_instruction_size, GMGotoInstruction, GMInstruction};


pub enum BlockError {
    InvalidBranchTarget(u32),
}

impl Display for BlockError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockError::InvalidBranchTarget(target_address) => write!(f, "Invalid branch target with address {target_address}"),
        }
    }
}


#[derive(Debug)]
pub struct BasicBlock<'a> {
    pub instructions: &'a [GMInstruction],
    pub start_address: usize,
    pub end_address: usize,
    /// Addresses of blocks this block can jump to
    pub successors: Vec<usize>,
    /// Addresses of blocks that can jump to this block
    pub predecessors: Vec<usize>,
}

impl<'a> BasicBlock<'a> {
    fn new(instructions: &'a [GMInstruction], start: usize, end: usize) -> Self {
        Self {
            instructions: &instructions[start..end],
            start_address: start,
            end_address: end,
            successors: vec![],
            predecessors: vec![],
        }
    }
}

impl Display for BasicBlock<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let predecessors: String = self.predecessors.iter().map(|&i| i.to_string()).collect::<Vec<_>>().join(", ");
        let successors: String = self.successors.iter().map(|&i| i.to_string()).collect::<Vec<_>>().join(", ");
        write!(f, "({:<3}-{:>3}) pred[{:<3}] succ[{:<3}]", self.start_address, self.end_address, predecessors, successors)
    }
}



fn get_branch_instr(instruction: &GMInstruction) -> Option<&GMGotoInstruction> {
    match instruction {
        GMInstruction::Branch(instr) |
        GMInstruction::BranchIf(instr) |
        GMInstruction::BranchUnless(instr) |
        GMInstruction::PushWithContext(instr) |
        GMInstruction::PopWithContext(instr) => Some(instr),
        _ => None
    }
}


/// Control Flow Analysis
pub fn find_basic_blocks(instructions: &[GMInstruction]) -> Result<Vec<BasicBlock>, BlockError> {
    let mut current_address: u32 = 0;
    let mut address_map: HashMap<u32, usize> = HashMap::with_capacity(instructions.len());
    let mut goto_instructions: Vec<(&GMGotoInstruction, u32)> = Vec::new();

    // Get end nodes and generate address map
    let mut nodes: HashSet<usize> = HashSet::new();
    for (current_index, instruction) in instructions.iter().enumerate() {
        address_map.insert(current_address, current_index);
        if let Some(instr) = get_branch_instr(instruction) {
            nodes.insert(current_index + 1);
            goto_instructions.push((instr, current_address));
        }
        current_address += get_instruction_size(instruction);
    }

    // Insert node for code end
    address_map.insert(current_address, instructions.len());

    // Get start nodes and generate edges
    let mut edges: ManyToMany<usize, usize> = ManyToMany::new();

    for (instr, instr_address) in goto_instructions {
        let target_address: u32 = (instr_address as i32 + instr.jump_offset) as u32;
        let target_index: usize = *address_map.get(&target_address)
            .ok_or(BlockError::InvalidBranchTarget(target_address))?;
        nodes.insert(target_index);

        let instr_index: usize = *address_map.get(&instr_address).unwrap();
        edges.insert(instr_index, target_index);
    }

    let mut nodes: Vec<usize> = nodes.into_iter().collect();
    nodes.sort_unstable();

    // Create blocks
    let mut blocks: Vec<BasicBlock> = Vec::with_capacity(nodes.len());
    let mut current_index: usize = 0;

    for instruction_address in nodes {
        if instruction_address > current_index {
            blocks.push(BasicBlock::new(instructions, current_index, instruction_address));
        }
        current_index = instruction_address;
    }

    // Add final block if there are remaining instructions
    if current_index < instructions.len() {
        blocks.push(BasicBlock::new(instructions, current_index, instructions.len()));
    }

    // Populate predecessor and successor fields of blocks
    for i in 0..blocks.len() {
        let source_indexes: Vec<usize> = edges.get_right(&blocks[i].start_address).unwrap_or_default();
        let target_indexes: Vec<usize> = edges.get_left(&(blocks[i].end_address - 1)).unwrap_or_default();

        blocks[i].predecessors = addresses_to_block_indexes(&blocks, source_indexes);
        blocks[i].successors = addresses_to_block_indexes(&blocks, target_indexes);
    }

    Ok(blocks)
}


fn address_to_block_index(blocks: &[BasicBlock], address: usize) -> usize {
    for (i, block) in blocks.iter().enumerate() {
        if address >= block.start_address && address < block.end_address {
            return i
        }
    }
    // handle end case
    if address == blocks.last().unwrap().end_address {
        return blocks.len()
    }
    unreachable!("Somehow, the instruction address {address} couldn't be resolved to a block")
}


fn addresses_to_block_indexes(blocks: &Vec<BasicBlock>, addresses: Vec<usize>) -> Vec<usize> {
    addresses.into_iter().map(|i| address_to_block_index(blocks, i)).collect()
}

// TODO: refactor popenv exit magic; can only happen for popenv and doesn't branch