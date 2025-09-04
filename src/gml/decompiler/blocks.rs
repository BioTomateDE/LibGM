use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use crate::gamemaker::elements::code::{get_instruction_size, GMInstruction};


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
    pub start_index: usize,
    pub end_index: usize,
    /// Addresses of blocks that can jump to this block (sources)
    pub predecessors: Vec<usize>,
    /// Addresses of blocks this block can jump to (targets)
    pub successors: Vec<usize>,
}

impl<'a> BasicBlock<'a> {
    fn new(instructions: &'a [GMInstruction], start: usize, end: usize) -> Self {
        Self {
            instructions: &instructions[start..end],
            start_index: start,
            end_index: end,
            predecessors: Vec::new(),
            successors: Vec::new(),
        }
    }
}

impl Display for BasicBlock<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let predecessors: String = self.predecessors.iter().map(|&i| i.to_string()).collect::<Vec<_>>().join(", ");
        let successors: String = self.successors.iter().map(|&i| i.to_string()).collect::<Vec<_>>().join(", ");
        write!(f, "({:<3}-{:>3}) pred[{:<1}] succ[{:<1}]", self.start_index, self.end_index, predecessors, successors)
    }
}



fn generate_address_map(instructions: &[GMInstruction]) -> HashMap<u32, usize> {
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


pub fn find_basic_blocks(instructions: &[GMInstruction]) -> Result<Vec<BasicBlock>, BlockError> {
    if instructions.is_empty() {
        return Ok(vec![])
    }

    // 1. generate address map immediately
    // 2. go through all instructions again
    //   - populate hashset of nodes (convert to sorted vec later)
    //   - generate branch table
    // 3. generate blocks (empty predecessors/successor)
    // 4. generate predecessors and successors
    //   - respect fallthrough and branching cases
    //   - resolve instruction index to block index (using binary search)

    // 1. Generate address map
    let address_map: HashMap<u32, usize> = generate_address_map(instructions);

    // 2. Get nodes and branch table
    let mut current_address: u32 = 0;
    let mut nodes_set: HashSet<usize> = HashSet::new();
    let mut branch_table: HashMap<usize, usize> = HashMap::new();

    for (current_index, instruction) in instructions.iter().enumerate() {
        match instruction {
            GMInstruction::Exit(_) |
            GMInstruction::Return(_) => {
                nodes_set.insert(current_index);
            }
            GMInstruction::Branch(instr) |
            GMInstruction::BranchIf(instr) |
            GMInstruction::BranchUnless(instr) |
            GMInstruction::PushWithContext(instr) |
            GMInstruction::PopWithContext(instr) => {
                let target_address: u32 = (current_address as i32 + instr.jump_offset) as u32;
                let target_index: usize = *address_map.get(&target_address)
                    .ok_or(BlockError::InvalidBranchTarget(target_address))?;
                branch_table.insert(current_index, target_index);
                nodes_set.insert(current_index + 1);
                nodes_set.insert(target_index);
            }
            _ => {}
        }
        current_address += get_instruction_size(instruction);
    }

    // Convert nodes to sorted vec, push end node
    let mut nodes_list: Vec<usize> = nodes_set.into_iter().collect();
    nodes_list.sort_unstable();
    nodes_list.push(instructions.len());

    // 3. Create blocks
    let mut blocks: Vec<BasicBlock> = Vec::with_capacity(nodes_list.len());
    let mut current_index: usize = 0;

    for instruction_address in nodes_list {
        if instruction_address > current_index {
            blocks.push(BasicBlock::new(instructions, current_index, instruction_address));
        }
        current_index = instruction_address;
    }

    // Populate predecessor and successor fields of blocks
    for block_index in 0..blocks.len() {
        let last_instruction: &GMInstruction = blocks[block_index].instructions.last().unwrap();
        let last_instruction_index: usize = blocks[block_index].end_index - 1;

        if matches!(last_instruction, GMInstruction::Exit(_) | GMInstruction::Return(_)) {
            // This terminates code execution, so this block won't have any successors
            continue
        }

        // Insert branch target to predecessor/successor list
        if let Some(&target_index) = branch_table.get(&last_instruction_index) {
            if target_index == instructions.len() {
                // If instruction branches to the end of the code, insert fictional end block
                let end_block_index: usize = blocks.len();
                blocks[block_index].successors.push(end_block_index);
            } else {
                let target_block_index: usize = get_block_index(&blocks, target_index);
                blocks[target_block_index].predecessors.push(block_index);
                blocks[block_index].successors.push(target_block_index);
            }
        }

        // For unconditional branch instructions, do not add fallthrough case
        if matches!(last_instruction, GMInstruction::Branch(_)) {
            // TODO: Are PushEnv and PopEnv conditional???
            continue
        }

        // Do not insert sequential successor block if it is the last one
        if block_index == blocks.len() - 1 {
            continue
        }

        // Insert fall through case (or normal progression for non branch instructions, i guess)
        blocks[block_index + 1].predecessors.push(block_index);
        blocks[block_index].successors.push(block_index + 1);
    }

    Ok(blocks)
}


/// Convert an instruction index (not raw address) to a block index.
/// In other words, find the block this instruction belongs to.
/// # Panic Safety
/// This function will panic if `blocks` is empty, unordered,
/// or if the block `start_index`/`end_index` fields are malformed.
fn get_block_index(blocks: &[BasicBlock], instruction_index: usize) -> usize {
    if blocks.is_empty() {
        panic!("Cannot find block in empty blocks array")
    }

    let mut low: usize = 0;
    let mut high: usize = blocks.len() - 1;

    while low <= high {
        let mid: usize = (low + high) / 2;
        let block: &BasicBlock = &blocks[mid];

        if instruction_index < block.start_index {
            high = mid - 1;
        } else if instruction_index >= block.end_index {
            low = mid + 1;
        } else {
            return mid
        }
    }

    panic!(
        "Could not find block for instruction index {} in {} blocks (range: {}-{})",
        instruction_index,
        blocks.len(),
        blocks[0].start_index,
        blocks.last().unwrap().end_index,
    );
}

