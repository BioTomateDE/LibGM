use std::ops::{Deref, DerefMut};
use std::os::linux::raw::stat;
use crate::gamemaker::elements::code::GMInstruction;
use crate::gml::decompiler::control_flow::blocks::Block;
use crate::gml::decompiler::control_flow::{BaseNode, ControlFlowGraph, NodeRef, Successors};


#[derive(Debug, Clone)]
pub struct StaticInit {
    pub base_node: BaseNode,
    pub head: NodeRef,
}

impl StaticInit {
    pub fn new(start_address: u32, end_address: u32, head: NodeRef) -> Self {
        Self {
            base_node: BaseNode::new(start_address, end_address),
            head,
        }
    }
}

impl Deref for StaticInit {
    type Target = BaseNode;
    fn deref(&self) -> &Self::Target {
        &self.base_node
    }
}

impl DerefMut for StaticInit {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base_node
    }
}


// TODO: remove panicking

pub fn find_static_inits(cfg: &mut ControlFlowGraph) -> Result<(), String> {
    for i in 0..cfg.blocks.len() {
        let block = &cfg.blocks[i];
        let Some([GMInstruction::HasStaticInitialized, GMInstruction::BranchIf(..)]) = block.instructions.last_chunk() else {continue};
        let fall_through: NodeRef = block.successors.fall_through.clone().unwrap();
        let branch_target: NodeRef = block.successors.branch_target.clone().unwrap();

        let static_init = StaticInit::new(block.end_address, branch_target.start_address(cfg), fall_through.clone());
        let static_init_ref = NodeRef::static_init(cfg.static_inits.len());
        cfg.static_inits.push(static_init);

        cfg.blocks[i].pop_last_instruction();   // Pop BranchIf
        cfg.blocks[i].pop_last_instruction();   // Pop HasStaticInitialized

        // Remove instruction from ending block, if it's the right one (changes depending on version)
        let branch_block: &mut Block = branch_target.as_block_mut(cfg).expect("StaticInit Branch successor is not a block"); // utmt: no error
        let first_instruction: &GMInstruction = branch_block.instructions.first().expect("StaticInit Branch successor block has no instructions"); // utmt: no error
        if *first_instruction == GMInstruction::SetStaticInitialized {
            branch_block.pop_first_instruction();
        }

        // Disconnect predecessors of the head and our after block
        cfg.disconnect_all_predecessors(&branch_target)?;
        cfg.disconnect_fallthrough_successor(&NodeRef::block(i))?;

        // Insert into control flow graph (done manually, here)
        cfg.blocks[i].successors.fall_through = Some(static_init_ref.clone());
        cfg.static_inits[static_init_ref.index].predecessors.push(NodeRef::block(i));

        branch_target.predecessors_mut(cfg).push(static_init_ref.clone());
        cfg.static_inits[static_init_ref.index].successors.branch_target = Some(branch_target);

        // Update parent status of head and this structure
        cfg.static_inits[static_init_ref.index].parent = fall_through.parent(cfg).clone();
        *fall_through.parent_mut(cfg) = Some(static_init_ref);
    }
    Ok(())
}

