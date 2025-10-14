// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.
use crate::gamemaker::elements::code::GMInstruction;
use crate::gml::decompiler::control_flow::blocks::Block;
use crate::gml::decompiler::control_flow::node::{Node, NodeData};
use crate::gml::decompiler::control_flow::node_ref::NodeRef;
use crate::gml::decompiler::decompile_context::DecompileContext;
use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct StaticInit {
    pub head: NodeRef,
}

impl StaticInit {
    pub fn new<'d>(start_address: u32, end_address: u32, head: NodeRef) -> Node<'d> {
        Node::new(start_address, end_address, NodeData::StaticInit(StaticInit { head }))
    }
}

// TODO: remove panicking

pub fn find_static_inits(ctx: &mut DecompileContext) -> Result<()> {
    for block_ref in ctx.blocks.clone() {
        let block = block_ref.node_mut(ctx);
        let Some([GMInstruction::HasStaticInitialized, GMInstruction::BranchIf(..)]) =
            block.block().instructions.last_chunk()
        else {
            continue;
        };
        let fall_through: NodeRef = block.successors.fall_through.unwrap();
        let branch_target: NodeRef = block.successors.branch_target.unwrap();

        let mut static_init = StaticInit::new(block.end_address, branch_target.node(ctx).start_address, fall_through);
        let cur_node = ctx.next_node_ref();

        block_ref.block_mut(ctx).pop_last_instructions(2)?; // Pop BranchIf and HasStaticInitialized

        // Remove instruction from ending block, if it's the right one (changes depending on version)
        let branch_block: &mut Block = branch_target
            .node_mut(ctx)
            .as_block_mut()
            .context("StaticInit Branch successor is not a block")?; // Utmt: no error
        let first_instruction: &GMInstruction = branch_block
            .instructions
            .first()
            .context("StaticInit Branch successor block has no instructions")?; // Utmt: no error
        if *first_instruction == GMInstruction::SetStaticInitialized {
            branch_block.pop_first_instruction()?;
        }

        // Disconnect predecessors of the head and our after block
        ctx.disconnect_all_predecessors(branch_target)?;
        ctx.disconnect_fallthrough_successor(block_ref)?;

        // Insert into control flow graph (done manually, here)
        branch_target.node_mut(ctx).predecessors.push(cur_node);
        let fall_through_parent = fall_through.node(ctx).parent;
        block_ref.node_mut(ctx).successors.fall_through = Some(cur_node);
        static_init.predecessors.push(block_ref);
        static_init.successors.branch_target = Some(branch_target);

        // Update parent status of head and this structure
        static_init.parent = fall_through_parent;
        fall_through.node_mut(ctx).parent = Some(cur_node);

        // // Insert into control flow graph (done manually, here) TODO
        // cfg.blocks[i].successors.fall_through = Some(static_init_ref);
        // cfg.static_inits[static_init_ref.index()].predecessors.push(NodeRef::block(i));
        //
        // branch_target.predecessors_mut(cfg).push(static_init_ref);
        // cfg.static_inits[static_init_ref.index()].successors.branch_target = Some(branch_target);
        //
        // // Update parent status of head and this structure
        // cfg.static_inits[static_init_ref.index()].parent = *fall_through.parent(cfg);
        // *fall_through.parent_mut(cfg) = Some(static_init_ref);

        ctx.nodes.push(static_init);
    }
    Ok(())
}
