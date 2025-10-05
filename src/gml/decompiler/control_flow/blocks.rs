use crate::gamemaker::data::GMData;
use crate::gamemaker::deserialize::GMRef;
use crate::gamemaker::elements::code::{
    get_instruction_size, GMCodeValue, GMDataType, GMDoubleTypeInstruction,
    GMInstruction, GMPushInstruction, GMSingleTypeInstruction,
};
use crate::gamemaker::elements::functions::GMFunction;
use crate::gml::decompiler::control_flow::{BaseNode, ControlFlowGraph, NodeRef};
use crate::gml::decompiler::vm_constants;
use crate::gml::disassembler::disassemble_instructions;
use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};


#[derive(Debug, Clone)]
pub struct Block<'a> {
    pub base_node: BaseNode,
    pub instructions: &'a [GMInstruction],
}

impl<'a> Block<'a> {
    pub fn new(address: u32) -> Self {
        Self {
            base_node: BaseNode::new(address, address),
            instructions: &[],
        }
    }

    pub fn pop_first_instruction(&mut self) -> Result<(), String> {
        self.pop_first_instructions(1)
    }

    pub fn pop_first_instructions(&mut self, count: usize) -> Result<(), String> {
        let len = self.instructions.len();
        if count > len {
            return Err(format!("Tried to pop {count} first instructions from block with {len} instructions"))
        }
        self.instructions = &self.instructions[count..];
        Ok(())
    }

    pub fn pop_last_instruction(&mut self) -> Result<(), String> {
        self.pop_last_instructions(1)
    }

    pub fn pop_last_instructions(&mut self, count: usize) -> Result<(), String> {
        let len = self.instructions.len();
        if count > len {
            return Err(format!("Tried to pop {count} last instructions from block with {len} instructions"))
        }
        self.instructions = &self.instructions[..len-count];
        Ok(())
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


/// Information about a try-catch-finally block
struct TryBlock {
    end_address: u32,
    finally_address: u32,
    catch_address: Option<u32>,
}


/// Result of analyzing instructions for block boundaries
struct BlockAnalysis {
    /// Set of addresses where blocks should start
    block_starts: HashSet<u32>,
    /// Map from instruction address to instruction index
    address_map: HashMap<u32, usize>,
    /// Information about try-catch-finally blocks
    try_blocks: Vec<TryBlock>,
    /// Total size of all instructions
    code_size: u32,
}


pub fn find_blocks<'a>(cfg: &mut ControlFlowGraph<'a>, instructions: &'a [GMInstruction]) -> Result<(), String> {
    let analysis = analyze_instructions(cfg.context.gm_data, instructions)?;
    create_blocks(cfg, instructions, analysis)?;
    connect_blocks(cfg)?;
    Ok(())
}


/// Analyze instructions to identify block boundaries and special structures
fn analyze_instructions(gm_data: &GMData, instructions: &[GMInstruction]) -> Result<BlockAnalysis, String> {
    let mut block_starts: HashSet<u32> = HashSet::new();
    let mut address_map: HashMap<u32, usize> = HashMap::with_capacity(instructions.len());
    let mut try_blocks: Vec<TryBlock> = Vec::new();
    let mut addr: u32 = 0;

    for (i, instr) in instructions.iter().enumerate() {
        address_map.insert(addr, i);
        let size = get_instruction_size(instr);

        match instr {
            // Terminal instructions - next instruction starts a new block
            GMInstruction::Exit(_) | GMInstruction::Return(_) => {
                block_starts.insert(addr + size);
            }

            // Branch instructions - both target and fallthrough start new blocks
            GMInstruction::Branch(b)
            | GMInstruction::BranchIf(b)
            | GMInstruction::BranchUnless(b)
            | GMInstruction::PushWithContext(b)
            | GMInstruction::PopWithContext(b) => {
                let target = (addr as i32 + b.jump_offset) as u32;
                block_starts.insert(target);
                block_starts.insert(addr + size);
            }

            // Try hook pattern detection
            GMInstruction::Call(call) if is_try_hook(gm_data, call.function)? => {
                let try_info = extract_try_info(gm_data, instructions, i, addr)?;

                // Mark block boundaries for try structure
                block_starts.insert(try_info.finally_address);
                if let Some(catch_addr) = try_info.catch_address {
                    block_starts.insert(catch_addr);
                }
                block_starts.insert(addr + size);  // After popz.v
                block_starts.insert(addr - 24);    // Start of pattern
                block_starts.insert(addr + 12);    // End of pattern

                try_blocks.push(try_info);
            }

            _ => {}
        }

        addr += size;
    }

    // Add final address for the end block
    address_map.insert(addr, instructions.len());

    Ok(BlockAnalysis {
        block_starts,
        address_map,
        try_blocks,
        code_size: addr,
    })
}


/// Extract try-catch-finally information from instruction pattern
fn extract_try_info(gm_data: &GMData, instructions: &[GMInstruction], call_index: usize, call_addr: u32) -> Result<TryBlock, String> {
    // Pattern: [push.i finally, conv.i.v, push.i catch, conv.i.v, call, popz.v]
    const PATTERN_SIZE: usize = 6;
    const PATTERN_START: usize = 4;

    let start = call_index.saturating_sub(PATTERN_START);
    let end = call_index + 2; // Include popz.v

    let pattern = instructions.get(start..end)
        .ok_or_else(|| format!(
            "@@try_hook@@ pattern extends beyond bounds at index {}",
            call_index
        ))?;

    if pattern.len() != PATTERN_SIZE {
        return Err(format!(
            "Invalid @@try_hook@@ pattern size at index {}: expected {}, got {}",
            call_index, PATTERN_SIZE, pattern.len()
        ));
    }

    // Extract addresses from the pattern
    match pattern {
        [
        GMInstruction::Push(GMPushInstruction { value: GMCodeValue::Int32(finally) }),
        GMInstruction::Convert(GMDoubleTypeInstruction {
                                   right: GMDataType::Int32,
                                   left: GMDataType::Int32
                               }),
        GMInstruction::Push(GMPushInstruction { value: GMCodeValue::Int32(catch) }),
        GMInstruction::Convert(GMDoubleTypeInstruction {
                                   right: GMDataType::Int32,
                                   left: GMDataType::Int32
                               }),
        GMInstruction::Call(_),
        GMInstruction::PopDiscard(GMSingleTypeInstruction {
                                      data_type: GMDataType::Variable
                                  }),
        ] => {
            Ok(TryBlock {
                end_address: call_addr + 12, // After popz.v
                finally_address: *finally as u32,
                catch_address: if *catch != -1 { Some(*catch as u32) } else { None },
            })
        }
        _ => {
            let actual = disassemble_instructions(gm_data, pattern)
                .unwrap_or_else(|e| format!("<invalid: {}>", e));
            Err(format!(
                "Malformed @@try_hook@@ pattern at index {}: expected \
                [push.i, conv.i.v, push.i, conv.i.v, call, popz.v], found [{}]",
                call_index, actual
            ))
        }
    }
}


/// Create blocks from analyzed instruction boundaries
fn create_blocks<'a>(cfg: &mut ControlFlowGraph<'a>, instructions: &'a [GMInstruction], analysis: BlockAnalysis) -> Result<(), String> {
    // Convert to sorted vector for efficient block creation
    let mut boundaries: Vec<u32> = analysis.block_starts.into_iter().collect();
    boundaries.push(0); // Ensure we start from 0
    boundaries.push(analysis.code_size);
    boundaries.sort_unstable();
    boundaries.dedup();

    // Preallocate blocks
    cfg.blocks = Vec::with_capacity(boundaries.len() - 1);

    // Create blocks between consecutive boundaries
    for window in boundaries.windows(2) {
        let start = window[0];
        let end = window[1];

        let mut block = Block::new(start);
        block.end_address = end;

        // Map addresses to instruction slice
        let start_idx = *analysis.address_map.get(&start)
            .ok_or_else(|| format!("Missing address mapping for {}", start))?;
        let end_idx = *analysis.address_map.get(&end)
            .ok_or_else(|| format!("Missing address mapping for {}", end))?;

        block.instructions = &instructions[start_idx..end_idx];
        cfg.blocks.push(block);
    }

    // Store try blocks in cfg if needed
    // (Assuming cfg has a field for this, otherwise pass them to connect_blocks)
    // ^ TODO

    Ok(())
}


/// Connect blocks with predecessor and successor relationships
fn connect_blocks(cfg: &mut ControlFlowGraph) -> Result<(), String> {
    let block_count = cfg.blocks.len();

    for idx in 0..block_count {
        let block = &cfg.blocks[idx];

        // Empty blocks can only fall through
        if block.instructions.is_empty() {
            if idx + 1 < block_count {
                connect_fallthrough(cfg, idx);
            }
            continue
        }

        let last_instr = block.instructions.last().unwrap();

        match last_instr {
            // Terminal instructions have no successors
            GMInstruction::Exit(_) | GMInstruction::Return(_) => {}

            // Unconditional branch
            GMInstruction::Branch(b) => {
                let target_addr = compute_branch_target(block.end_address - 1, b.jump_offset);
                connect_branch_target(cfg, idx, target_addr)?;
            }

            // Conditional branches have both target and fallthrough
            GMInstruction::BranchIf(b)
            | GMInstruction::BranchUnless(b)
            | GMInstruction::PushWithContext(b)
            | GMInstruction::PopWithContext(b) => {
                let target_addr = compute_branch_target(block.end_address - 1, b.jump_offset);
                connect_branch_target(cfg, idx, target_addr)?;
                if idx + 1 < block_count {
                    connect_fallthrough(cfg, idx);
                }
            }

            // Try hook pattern (simplified detection)
            GMInstruction::PopDiscard(_) if is_try_hook_block(&cfg.blocks[idx]) => {
                // This would need access to try_blocks from analysis
                // For now, assuming try blocks are handled separately
                if idx + 1 < block_count {
                    connect_fallthrough(cfg, idx);
                }
            }

            // Default: fall through to next block
            _ => {
                if idx + 1 < block_count {
                    connect_fallthrough(cfg, idx);
                }
            }
        }
    }

    Ok(())
}


/// Helper to connect a block to its fallthrough successor
fn connect_fallthrough(cfg: &mut ControlFlowGraph, block_idx: usize) {
    let successor = NodeRef::block(block_idx + 1);
    cfg.blocks[block_idx].successors.fall_through = Some(successor);
    cfg.blocks[block_idx + 1].predecessors.push(NodeRef::block(block_idx));
}


/// Helper to connect a block to its branch target
fn connect_branch_target(cfg: &mut ControlFlowGraph, block_idx: usize, target_addr: u32) -> Result<(), String> {
    let target_idx = find_block_containing(cfg, target_addr)?;
    let successor = NodeRef::block(target_idx);
    cfg.blocks[block_idx].successors.branch_target = Some(successor);
    cfg.blocks[target_idx].predecessors.push(NodeRef::block(block_idx));
    Ok(())
}


/// Compute the target address of a branch instruction
fn compute_branch_target(instr_addr: u32, offset: i32) -> u32 {
    (instr_addr as i32 + offset) as u32
}


/// Check if a block is a try hook pattern
fn is_try_hook_block(block: &Block) -> bool {
    block.instructions.len() == 6 &&
        matches!(block.instructions.get(4), Some(GMInstruction::Call(_)))
}


/// Find the block containing the given instruction address using binary search
fn find_block_containing(cfg: &ControlFlowGraph, addr: u32) -> Result<usize, String> {
    // Handle edge case: address is at the very end
    if let Some(last) = cfg.blocks.last() {
        if addr == last.end_address && !last.instructions.is_empty() {
            return Ok(cfg.blocks.len() - 1);
        }
    }

    cfg.blocks.binary_search_by(|block| {
        if addr < block.start_address {
            std::cmp::Ordering::Greater
        } else if addr >= block.end_address {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Equal
        }
    }).map_err(|_| format!("Could not find block containing address {addr}"))
}


/// Check if a function reference is the try hook function
fn is_try_hook(gm_data: &GMData, func_ref: GMRef<GMFunction>) -> Result<bool, String> {
    let func = func_ref.resolve(&gm_data.functions.functions)?;
    let name = func.name.resolve(&gm_data.strings.strings)?;
    Ok(name == vm_constants::functions::TRY_HOOK)
}

