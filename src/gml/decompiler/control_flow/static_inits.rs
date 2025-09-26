use std::ops::{Deref, DerefMut};
use crate::gamemaker::elements::code::GMInstruction;
use crate::gml::decompiler::control_flow::blocks::Block;
use crate::gml::decompiler::control_flow::{BaseNode, ControlFlowGraph, NodeRef, Successors};


#[derive(Debug, Clone)]
pub struct StaticInit {
    pub base_node: BaseNode,
    pub children: Vec<NodeRef>,
}

impl StaticInit {
    pub fn new(head: NodeRef, start_address: u32, end_address: u32) -> Self {
        Self {
            base_node: BaseNode {
                start_address,
                end_address,
                predecessors: vec![],
                successors: Successors::none(),
            },
            children: vec![head],
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

pub fn find_static_inits(cfg: &mut ControlFlowGraph) {
    todo!("implement this shit later");
    for block in &mut cfg.blocks {
        let Some([GMInstruction::HasStaticInitialized, GMInstruction::BranchIf(..)]) = block.instructions.last_chunk() else {continue};
        let fall_through: NodeRef = block.successors.fall_through.clone().expect("Static Init Successor block does not have fallthrough successor");
        let branch_target: NodeRef = block.successors.branch_target.clone().expect("Static Init Successor block does not have branch successor");

        let static_init = StaticInit::new(branch_target, block.end_address, fall_through.start_address(cfg));
        cfg.static_inits.push(static_init);

        block.pop_last_instruction();   // Pop BranchIf
        block.pop_last_instruction();   // Pop HasStaticInitialized

        // Remove instruction from ending block, if it's the right one (changes depending on version)
        let branch_block: &mut Block = fall_through.as_block_mut(cfg).expect("Static Init Fallthrough successor is not a block"); // utmt: no error
        let first_instruction: &GMInstruction = branch_block.instructions.first().expect("Static Init Fall through successor block has no instructions"); // utmt: no error
        if *first_instruction == GMInstruction::HasStaticInitialized {
            branch_block.pop_first_instruction();
        }

        // Disconnect predecessors of the head and our after block
        let old_predecessor = branch_block.predecessors[0];
        branch_block.predecessors.remove(0);
    }
}

