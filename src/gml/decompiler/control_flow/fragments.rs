// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.
use crate::gamemaker::data::GMData;
use crate::gamemaker::deserialize::GMRef;
use crate::gamemaker::elements::code::{GMCode, GMInstruction, get_instruction_size};
use crate::gml::decompiler::control_flow::node::{Node, NodeData};
use crate::gml::decompiler::control_flow::node_ref::NodeRef;
use crate::gml::decompiler::decompile_context::DecompileContext;
use crate::prelude::*;
use crate::util::smallmap::SmallMap;

#[derive(Debug, Clone)]
pub struct Fragment<'d> {
    /// The base blocks that this fragment is composed of.
    pub blocks: Vec<NodeRef>,

    /// Code entry that this fragment belongs to.
    pub code_entry: &'d GMCode,

    /// Whether this fragment is a root-scope fragment (that is, it may be a global function when inside a global script).
    pub root_scope: bool,
}

impl<'d> Fragment<'d> {
    pub fn new(start_address: u32, end_address: u32, code_entry: &'d GMCode, root_scope: bool) -> Node<'d> {
        Node::new(
            start_address,
            end_address,
            NodeData::Fragment(Self { blocks: vec![], code_entry, root_scope }),
        )
    }
}

pub fn find_fragments(ctx: &mut DecompileContext, code_ref: GMRef<GMCode>) -> Result<()> {
    let child_start_offsets: SmallMap<u32, &GMCode> = get_child_start_offsets(ctx.gm_data, code_ref)?;
    let code: &GMCode = code_ref.resolve(&ctx.gm_data.codes.codes)?;
    let code_end_address: u32 = get_code_end_address(&code.instructions);

    // Build fragments, using a stack to track hierarchy
    let mut stack: Vec<Node> = Vec::new();
    ctx.nodes.reserve(child_start_offsets.len());
    let mut current = Fragment::new(0, code_end_address, code, false);

    for block in ctx.blocks.clone() {
        let cur_node = ctx.next_node_ref();
        let start_address = block.node(ctx).start_address;

        // Check if our current fragment is ending at this block
        if start_address == current.end_address {
            if stack.is_empty() {
                // We're done processing now. Add last block and exit loops.
                current.fragment_mut().blocks.push(block);
                if start_address != code_end_address {
                    bail!(
                        "Final block starts at address {start_address} but should start at the code's end address {code_end_address}"
                    );
                }
                break;
            }

            // Disconnect predecessor from branch instruction block
            ctx.disconnect_all_predecessors(current.fragment_mut().blocks[0])?;

            // We're an inner fragment; remove the Exit Instruction
            let last_block = current
                .fragment_mut()
                .blocks
                .last()
                .context("Fragment doesn't have any blocks while ending fragment")?
                .block_mut(ctx);

            match last_block.instructions.last() {
                Some(GMInstruction::Exit(_)) => last_block.pop_last_instruction()?,
                Some(instr) => bail!("Expected Exit instruction; got {instr:?}"),
                None => unreachable!("Block doesn't have any instructions"), // TODO: is ts possible? end block
            }

            // Go to the fragment the next level up
            current = stack.pop().context("Fragment stack is empty")?;
        }

        // Check for new fragment starting at this block
        if let Some(child_code) = child_start_offsets.get(&start_address) {
            // Our "current" is now the next level up
            stack.push(current.clone());

            // Compute the end address of this fragment, by looking at previous block
            let previous = block.previous_sequentially();
            let last_instr = previous.block(ctx).instructions.last().unwrap();
            if !matches!(last_instr, GMInstruction::Branch(_)) {
                bail!("Expected unconditional branch instruction before fragment start, got {last_instr:?}");
            }
            let Some(branch_target) = previous.node(ctx).successors.branch_target else {
                unreachable!(
                    "Successor enum of previous block (which ends in a `Branch` instruction) is not `UnconditionalBranch`"
                )
            };

            // Remove previous block's branch instruction
            previous.block_mut(ctx).pop_last_instruction()?;

            // Make our new "current" be this new fragment
            ctx.nodes.push(current);
            current = Fragment::new(
                block.node(ctx).start_address,
                branch_target.node(ctx).start_address,
                child_code,
                stack.len() == 1,
            );

            // Rewire previous block to jump to this fragment, and this fragment
            // To jump to the successor of the previous block.
            let prev_node = block.previous_sequentially();
            previous.node_mut(ctx).successors.branch_target = Some(cur_node);
            let predecessors = &mut branch_target.node_mut(ctx).predecessors;
            let pred_index: usize = predecessors
                .iter()
                .position(|node| *node == prev_node)
                .context("Could not find predecessor for branch target node")?;
            predecessors[pred_index] = cur_node;
            current.predecessors.push(prev_node);
            current.successors.branch_target = Some(branch_target);
        }

        // If we're at the start of the fragment, track parent node on the block
        if current.fragment().blocks.is_empty() {
            block.node_mut(ctx).parent = Some(cur_node);
        }

        // Add this block to our current fragment
        current.fragment_mut().blocks.push(block);
    }

    if !stack.is_empty() {
        bail!("Failed to close all fragments; stack still has {} items", stack.len());
    }

    ctx.nodes.push(current);
    Ok(())
}

fn get_child_start_offsets(gm_data: &GMData, parent_code_ref: GMRef<GMCode>) -> Result<SmallMap<u32, &GMCode>> {
    let mut start_offsets = SmallMap::new();
    for code in &gm_data.codes.codes {
        let Some(b15_info) = &code.bytecode15_info else {
            continue;
        };
        let Some(parent) = b15_info.parent else {
            continue;
        };
        if parent != parent_code_ref {
            continue;
        }

        if b15_info.offset % 4 != 0 {
            bail!(
                "Child code instruction offset {} does not point to the start of an instruction",
                b15_info.offset
            );
        }
        start_offsets.insert(b15_info.offset / 4, code);
    }
    Ok(start_offsets)
}

fn get_code_end_address(instructions: &Vec<GMInstruction>) -> u32 {
    let mut length: u32 = 0;
    for instruction in instructions {
        length += get_instruction_size(instruction);
    }
    length
}
