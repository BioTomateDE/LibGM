use crate::gamemaker::data::GMData;

use crate::gamemaker::deserialize::resources::GMRef;
use crate::gamemaker::elements::code::{
    GMCodeValue, GMDataType, GMDoubleTypeInstruction, GMInstruction, GMPushInstruction, GMSingleTypeInstruction,
    get_instruction_size,
};
use crate::gamemaker::elements::functions::GMFunction;
use crate::gml::decompiler::control_flow::node::{Node, NodeData};
use crate::gml::decompiler::control_flow::node_ref::NodeRef;
use crate::gml::decompiler::decompile_context::DecompileContext;
use crate::gml::decompiler::vm_constants;
use crate::gml::disassembler::disassemble_instruction;
use crate::prelude::*;
use crate::util::smallmap::SmallMap;
use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub struct Block<'d> {
    pub instructions: &'d [GMInstruction],
}

impl<'d> Block<'d> {
    pub fn new(start: u32, end: u32, instructions: &'d [GMInstruction]) -> Node<'d> {
        Node::new(start, end, NodeData::Block(Block { instructions }))
    }

    pub fn pop_first_instruction(&mut self) -> Result<()> {
        self.pop_first_instructions(1)
    }

    pub fn pop_first_instructions(&mut self, count: usize) -> Result<()> {
        let len = self.instructions.len();
        if count > len {
            bail!("Tried to pop {count} first instructions from block with {len} instructions");
        }
        self.instructions = &self.instructions[count..];
        Ok(())
    }

    pub fn pop_last_instruction(&mut self) -> Result<()> {
        self.pop_last_instructions(1)
    }

    pub fn pop_last_instructions(&mut self, count: usize) -> Result<()> {
        let len = self.instructions.len();
        if count > len {
            bail!("Tried to pop {count} last instructions from block with {len} instructions");
        }
        self.instructions = &self.instructions[..len - count];
        Ok(())
    }
}

/// Information about a try-catch-finally block
struct TryBlock {
    finally_address: u32,
    catch_address: Option<u32>,
}

/// Result of analyzing instructions for block boundaries
struct BlockAnalysis {
    /// Set of addresses where blocks should start
    block_starts: Vec<u32>,
    /// Information about try-catch-finally blocks
    try_blocks: SmallMap<u32, TryBlock>,
    /// Total size of all instructions
    code_size: u32,
}

pub fn find_blocks<'c, 'd>(ctx: &'c mut DecompileContext<'d>, instructions: &'d [GMInstruction]) -> Result<()> {
    if instructions.is_empty() {
        return Ok(());
    }
    let analysis = analyze_instructions(ctx.gm_data, instructions)?;
    let blocks = create_blocks(instructions, analysis.block_starts, analysis.code_size)?;
    // This only works if the node list is empty before.
    ctx.blocks = (0..blocks.len()).map(NodeRef::from).collect();
    ctx.nodes = blocks;
    connect_blocks(ctx, analysis.try_blocks)?;
    Ok(())
}

/// Analyze instructions to identify block boundaries and special structures
fn analyze_instructions(gm_data: &GMData, instructions: &[GMInstruction]) -> Result<BlockAnalysis> {
    let mut block_starts: Vec<u32> = Vec::new();
    let mut try_blocks: SmallMap<u32, TryBlock> = SmallMap::new();
    let mut addr: u32 = 0;

    for (i, instr) in instructions.iter().enumerate() {
        let size = get_instruction_size(instr);

        match instr {
            // Terminal instructions - next instruction starts a new block
            GMInstruction::Exit(_) | GMInstruction::Return(_) => {
                block_starts.push(addr + size);
            }

            // Branch instructions - both target and fallthrough start new blocks
            GMInstruction::Branch(b)
            | GMInstruction::BranchIf(b)
            | GMInstruction::BranchUnless(b)
            | GMInstruction::PushWithContext(b)
            | GMInstruction::PopWithContext(b) => {
                let target = (addr as i32 + b.jump_offset) as u32;
                block_starts.push(target);
                block_starts.push(addr + size);
            }

            // Try hook pattern detection
            GMInstruction::Call(call) if is_try_hook(gm_data, call.function)? => {
                let try_info = extract_try_info(gm_data, instructions, i)?;

                // Mark block boundaries for try structure
                block_starts.push(addr - 6); // Start of pattern
                block_starts.push(addr + 3); // End of pattern
                block_starts.push(try_info.finally_address);
                if let Some(catch_addr) = try_info.catch_address {
                    block_starts.push(catch_addr);
                }
                try_blocks.insert(addr + 3, try_info);
            }

            _ => {}
        }

        addr += size;
    }

    Ok(BlockAnalysis { block_starts, try_blocks, code_size: addr })
}

/// Extract try-catch-finally information from instruction pattern
fn extract_try_info(gm_data: &GMData, instructions: &[GMInstruction], call_index: usize) -> Result<TryBlock> {
    const ERR: &str = "@@try_hook@@ pattern out of bounds";
    let start = call_index.checked_sub(4).ok_or(ERR)?;
    let end = call_index + 2;
    let pattern = instructions.get(start..end).ok_or(ERR)?;

    // Extract addresses from the pattern
    match *pattern {
        [
            GMInstruction::Push(GMPushInstruction { value: GMCodeValue::Int32(finally) }),
            GMInstruction::Convert(GMDoubleTypeInstruction { right: GMDataType::Int32, left: GMDataType::Variable }),
            GMInstruction::Push(GMPushInstruction { value: GMCodeValue::Int32(catch) }),
            GMInstruction::Convert(GMDoubleTypeInstruction { right: GMDataType::Int32, left: GMDataType::Variable }),
            GMInstruction::Call(_),
            GMInstruction::PopDiscard(GMSingleTypeInstruction { data_type: GMDataType::Variable }),
        ] => Ok(TryBlock {
            finally_address: convert_address(finally)?,
            catch_address: if catch != -1 {
                Some(convert_address(catch)?)
            } else {
                None
            },
        }),
        _ => {
            let actual = pattern
                .iter()
                .map(|i| disassemble_instruction(gm_data, i).unwrap_or_else(|e| format!("<invalid: {e}>")))
                .collect::<Vec<_>>()
                .join(", ");
            bail!(
                "Malformed @@try_hook@@ pattern at index {call_index}: expected \
                [push.i, conv.i.v, push.i, conv.i.v, call, popz.v], found [{actual}]"
            );
        }
    }
}

/// Converts from a tryhook target address (in bytes) to a LibGM address (in bytes / 4)
fn convert_address(address: i32) -> Result<u32> {
    let address = address as u32;
    if address % 4 != 0 {
        bail!("Address is not divisible by four: {address}")
    }
    Ok(address / 4)
}

/// Create blocks from analyzed instruction boundaries
fn create_blocks(instructions: &'_ [GMInstruction], block_starts: Vec<u32>, code_size: u32) -> Result<Vec<Node<'_>>> {
    let mut boundaries: Vec<u32> = block_starts;
    boundaries.push(code_size);
    boundaries.sort_unstable();
    boundaries.dedup();

    let mut blocks = Vec::with_capacity(boundaries.len() - 1);
    let mut prev_addr = 0;
    let mut prev_index = 0;
    let mut curr_addr = 0;
    let mut curr_index = 0;

    for address in boundaries {
        if address == 0 {
            continue;
        }
        // Resolve address to instruction index
        loop {
            let instr = instructions
                .get(curr_index)
                .ok_or("Instruction out of bounds while creating blocks")?;
            curr_addr += get_instruction_size(instr);
            curr_index += 1;
            if curr_addr == address {
                break;
            }
        }

        let block = Block::new(prev_addr, curr_addr, &instructions[prev_index..curr_index]);
        blocks.push(block);
        prev_index = curr_index;
        prev_addr = curr_addr;
    }

    Ok(blocks)
}

/// Connect blocks with predecessor and successor relationships
fn connect_blocks(ctx: &mut DecompileContext, try_blocks: SmallMap<u32, TryBlock>) -> Result<()> {
    let block_count = ctx.blocks.len();
    for idx in 0..block_count {
        let node = ctx.blocks[idx]; // TODO: Potential unoptimized bounds check?
        let end_address = node.node(ctx).end_address;
        let block = node.block_mut(ctx);
        let is_last_block = idx == block_count - 1;
        let last_instr = block.instructions.last().expect("Block is somehow empty");

        match last_instr {
            // Terminal instructions have no successors
            GMInstruction::Exit(_) | GMInstruction::Return(_) => {}

            // Unconditional branch
            GMInstruction::Branch(instr) => {
                let target_addr = end_address as i32 - 1 + instr.jump_offset;
                connect_branch_target(ctx, idx, target_addr as u32)?;
            }

            // Conditional branches have both target and fallthrough
            GMInstruction::BranchIf(instr)
            | GMInstruction::BranchUnless(instr)
            | GMInstruction::PushWithContext(instr)
            | GMInstruction::PopWithContext(instr) => {
                let target_addr = end_address as i32 - 1 + instr.jump_offset;
                connect_branch_target(ctx, idx, target_addr as u32)?;
                if !is_last_block {
                    connect_fallthrough(ctx, idx);
                }
            }

            GMInstruction::PopDiscard(_) if try_blocks.contains_key(&end_address) => {
                let try_block = try_blocks.get(&end_address).unwrap();
                if !is_last_block {
                    connect_fallthrough(ctx, idx);
                }
                connect_branch_target(ctx, idx, try_block.finally_address)?;
                if let Some(catch_addr) = try_block.catch_address {
                    connect_catch_target(ctx, idx, catch_addr)?;
                }
            }

            _ if is_last_block => {}
            // Default: fall through to next block
            _ => connect_fallthrough(ctx, idx),
        }
    }

    Ok(())
}

/// Helper to connect a block to its fallthrough successor
fn connect_fallthrough(ctx: &mut DecompileContext, block_idx: usize) {
    let predecessor = ctx.blocks[block_idx];
    let successor = ctx.blocks[block_idx + 1];
    ctx.blocks[block_idx].node_mut(ctx).successors.fall_through = Some(successor);
    ctx.blocks[block_idx + 1].node_mut(ctx).predecessors.push(predecessor);
}

/// Helper to connect a block to its branch target
fn connect_branch_target(ctx: &mut DecompileContext, block_idx: usize, target_addr: u32) -> Result<()> {
    let target_idx = find_block_containing(ctx, target_addr)?;
    ctx.blocks[block_idx].node_mut(ctx).successors.branch_target = Some(NodeRef::from(target_idx));
    ctx.blocks[target_idx]
        .node_mut(ctx)
        .predecessors
        .push(NodeRef::from(block_idx));
    Ok(())
}

/// Helper to connect a block to its catch target (only @@try_hook@@)
fn connect_catch_target(ctx: &mut DecompileContext, block_idx: usize, target_addr: u32) -> Result<()> {
    let target_idx = find_block_containing(ctx, target_addr)?;
    ctx.blocks[block_idx].node_mut(ctx).successors.catch = Some(NodeRef::from(target_idx));
    ctx.blocks[target_idx]
        .node_mut(ctx)
        .predecessors
        .push(NodeRef::from(block_idx));
    Ok(())
}

/// Find the block containing the given instruction address using binary search
fn find_block_containing(ctx: &DecompileContext, addr: u32) -> Result<usize> {
    // Handle edge case: address is at the very end
    if let Some(last) = ctx.blocks.last() {
        if addr == last.node(ctx).end_address && !last.block(ctx).instructions.is_empty() {
            return Ok(ctx.blocks.len() - 1);
        }
    }

    ctx.blocks
        .binary_search_by(|node| {
            let block = node.node(ctx);
            if addr < block.start_address {
                Ordering::Greater
            } else if addr >= block.end_address {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        })
        .ok()
        .with_context(|| format!("Could not find block containing address {addr}"))
}

/// Check if a function reference is the `@@try_hook@@` function
fn is_try_hook(gm_data: &GMData, func_ref: GMRef<GMFunction>) -> Result<bool> {
    let func = func_ref.resolve(&gm_data.functions.functions)?;
    Ok(func.name == vm_constants::functions::TRY_HOOK)
}
