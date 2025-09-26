use std::ops::{Deref, DerefMut};
use crate::gamemaker::data::GMData;
use crate::gamemaker::deserialize::GMRef;
use crate::gamemaker::elements::code::{get_instruction_size, GMCode, GMInstruction};
use crate::gml::decompiler::control_flow::{BaseNode, ControlFlowGraph, NodeRef, NodeType, Successors};
use crate::utility::SmallMap;

#[derive(Debug, Clone)]
pub struct Fragment<'a> {
    pub base_node: BaseNode,
    
    pub children: Vec<NodeRef>,

    /// The base blocks that this fragment is composed of.
    pub blocks: Vec<NodeRef>,

    /// Code entry that this fragment belongs to.
    pub code_entry: &'a GMCode,

    /// Whether this fragment is a root-scope fragment (that is, it may be a global function when inside a global script).
    pub root_scope: bool,
}

impl<'a> Fragment<'a> {
    pub fn new(code_entry: &'a GMCode, root_scope: bool, start_address: u32, end_address: u32) -> Self {
        Self {
            base_node: BaseNode {
                start_address,
                end_address,
                predecessors: vec![],
                successors: Successors::none(),
            },
            children: vec![],
            blocks: vec![],
            code_entry,
            root_scope,
        }
    }
}

impl<'a> Deref for Fragment<'a> {
    type Target = BaseNode;
    fn deref(&self) -> &Self::Target {
        &self.base_node
    }
}
impl<'a> DerefMut for Fragment<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base_node
    }
}



pub fn find_fragments<'a>(cfg: &'a mut ControlFlowGraph<'a>, gm_data: &'a GMData, code_ref: GMRef<GMCode>) -> Result<(), String> {
    let child_start_offsets: SmallMap<u32, &GMCode> = get_child_start_offsets(gm_data, code_ref)?;
    let code: &GMCode = code_ref.resolve(&gm_data.codes.codes)?;
    let code_end_address: u32 = get_code_end_address(&code.instructions);

    // Build fragments, using a stack to track hierarchy
    let mut stack: Vec<Fragment> = Vec::new();
    cfg.fragments = Vec::with_capacity(child_start_offsets.len());
    let mut current = Fragment::new(code, false, 0, code_end_address);

    for i in 0..cfg.blocks.len() {
        let block = &mut cfg.blocks[i];
        // Check if our current fragment is ending at this block
        if block.start_address == current.end_address {
            if stack.is_empty() {
                // We're done processing now. Add last block and exit loops.
                current.children.push(NodeRef::block(i));
                current.blocks.push(NodeRef::block(i));
                if block.start_address != code_end_address {
                    return Err(format!("Final block starts at address {} but should start at the code's end address {}", block.start_address, code_end_address))
                }
                break
            }

            // We're an inner fragment; remove the Exit Instruction
            if let Some(NodeRef {node_type: NodeType::Block, index}) = current.blocks.last() {
                let last_block = &mut cfg.blocks[*index];
                match last_block.instructions.last() {
                    Some(GMInstruction::Exit(_)) => last_block.pop_last_instruction(),
                    Some(instr) => return Err(format!("Expected Exit instruction; got {instr:?}")),
                    None => unreachable!("Block doesn't have any instructions"),    // TODO: is ts possible? end block
                }
            } else {
                return Err("fragment doesn't have any blocks or last block is not a block".to_string());
            }

            // Go to the fragment the next level up
            current = stack.pop().expect("fragment stack is empty");
        }

        // Check for new fragment starting at this block
        if let Some(child_code) = child_start_offsets.get(&cfg.blocks[i].start_address) {
            // Our "current" is now the next level up
            stack.push(current.clone());

            // Compute the end address of this fragment, by looking at previous block
            let previous = &mut cfg.blocks[i - 1];
            let last_instr = previous.instructions.last().unwrap();
            if !matches!(last_instr, GMInstruction::Branch(_)) {
                return Err(format!("Expected unconditional branch instruction before fragment start, got {last_instr:?}"))
            }
            let Some(branch_target_node) = previous.successors.branch_target.clone() else {
                unreachable!("Successor enum of previous block (which ends in a `Branch` instruction) is not `UnconditionalBranch`")
            };

            // Remove previous block's branch instruction
            previous.pop_last_instruction();

            // Make our new "current" be this new fragment
            cfg.fragments.push(current);
            current = Fragment::new(child_code, stack.len() == 1, cfg.blocks[i].start_address, branch_target_node.start_address(cfg));

            // Rewire previous block to jump to this fragment, and this fragment
            // to jump to the successor of the previous block.
            let previous = &mut cfg.blocks[i - 1];
            let cur_node_idx = NodeRef::fragment(cfg.fragments.len());
            let prev_node_idx = NodeRef::block(i - 1);
            previous.successors.branch_target = Some(cur_node_idx.clone());
            let predecessors = branch_target_node.predecessors_mut(cfg);
            let pred_index: usize = predecessors.iter().position(|node| *node == prev_node_idx)
                .ok_or(format!("Could not find predecessor with block index {} for branch target node", i-1))?;
            predecessors[pred_index] = cur_node_idx;
            current.predecessors.push(prev_node_idx);
            current.successors.branch_target = Some(branch_target_node);
        }

        current.children.push(NodeRef::block(i));
        current.blocks.push(NodeRef::block(i));
    }

    if !stack.is_empty() {
        return Err(format!("Failed to close all fragments; stack still has {} items", stack.len()))
    }

    cfg.fragments.push(current);
    Ok(())
}


fn get_child_start_offsets(gm_data: &GMData, parent_code_ref: GMRef<GMCode>) -> Result<SmallMap<u32, &GMCode>, String> {
    let mut start_offsets = SmallMap::new();
    for code in &gm_data.codes.codes {
        let Some(b15_info) = &code.bytecode15_info else {continue};
        let Some(parent) = b15_info.parent else {continue};
        if parent != parent_code_ref {
            continue
        }

        if b15_info.offset % 4 != 0 {
            return Err(format!("Child code instruction offset {} does not point to the start of an instruction", b15_info.offset))
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

