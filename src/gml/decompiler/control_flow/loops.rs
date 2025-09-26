use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use crate::gamemaker::elements::code::{GMGotoInstruction, GMInstruction};
use crate::gml::decompiler::control_flow::blocks::Block;
use crate::gml::decompiler::control_flow::{BaseNode, ControlFlowGraph, NodeRef, NodeType, Successors};

#[derive(Debug, Clone)]
pub struct BaseLoop {
    pub base_node: BaseNode,

    /// The index of this loop, as assigned in order of when the loop was first discovered (from top to bottom of code).
    pub loop_index: usize,

    /// The node before this loop; usually a block with [GMInstruction::PushWithContext] after it.
    /// *This is only set for [Loop::With].*
    pub before: Option<NodeRef>,

    /// The top loop point of the loop. This is where the loop condition begins to be evaluated.
    pub head: NodeRef,

    /// The bottom loop point of the loop. This is where the jump back to the loop head/condition is located.
    pub tail: NodeRef,

    /// The "sink" location of the loop. The loop condition being false or "break" statements will lead to this location.
    pub after: NodeRef,

    /// The start of the body of the loop, as written in the source code. That is, this does not include the loop condition.
    pub body: Option<NodeRef>,

    /// If [Some], this is a special block jumped to from within the with statement for "break" statements.
    /// *This is only set for [Loop::With].*
    pub break_block: Option<NodeRef>,

    /// If [Some], then it was detected that this while loop must be written as a for loop.
    /// This can occur when "continue" statements are used within the loop, which otherwise
    /// could not be written using normal if/else statements.
    /// This points to the start of the "incrementing" code of the for loop.
    /// *This is only set for [Loop::While].*
    pub for_loop_incrementor: Option<NodeRef>,
}

impl BaseLoop {
    pub fn new(loop_index: usize, head: NodeRef, tail: NodeRef, after: NodeRef, start_address: u32, end_address: u32) -> Self {
        Self {
            base_node: BaseNode {
                start_address,
                end_address,
                predecessors: vec![],
                successors: Successors::none(),
            },
            loop_index,
            head,
            tail,
            after,
            body: None,
            before: None,
            break_block: None,
            for_loop_incrementor: None,
        }
    }
}


#[derive(Debug, Clone)]
pub enum Loop {
    While(BaseLoop),
    DoUntil(BaseLoop),
    Repeat(BaseLoop),
    With(BaseLoop),
}

impl Deref for Loop {
    type Target = BaseNode;
    fn deref(&self) -> &Self::Target {
        match self {
            Loop::While(x) |
            Loop::DoUntil(x) |
            Loop::Repeat(x) |
            Loop::With(x) => &x.base_node,
        }
    }
}
impl DerefMut for Loop {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Loop::While(x) |
            Loop::DoUntil(x) |
            Loop::Repeat(x) |
            Loop::With(x) => &mut x.base_node,
        }
    }
}


pub fn find_loops(cfg: &mut ControlFlowGraph) -> Result<(), String> {
    let mut while_loops_found = HashSet::new();

    // Search for different loops types based on instruction patterns
    // Do this in reverse order, because we want to find the ends of loops first
    for i in (0..cfg.blocks.len()).rev() {
        let block = &cfg.blocks[i];
        let Some(last_instruction) = block.instructions.last() else {continue};

        // Check last instruction (where branches are located)
        match last_instruction {
            GMInstruction::Branch(instr) => push_while(cfg, i, instr, &mut while_loops_found),
            GMInstruction::BranchUnless(instr) if instr.jump_offset < 0 => push_do_until(cfg, i),
            GMInstruction::BranchIf(instr) if instr.jump_offset < 0 => push_repeat(cfg, i),
            GMInstruction::PushWithContext(_) => push_with(cfg, i)?,
            _ => {}
        }
    }



    Ok(())
}


fn push_while(cfg: &mut ControlFlowGraph, i: usize, instr: &GMGotoInstruction, while_loops_found: &mut HashSet<u32>) {
    let block = &cfg.blocks[i];

    // While loop detected - only add if this is the first time we see it (in reverse)
    // If not done only once, then this misfires on "continue" statements
    let condition_address = (block.end_address as i32 - 1 + instr.jump_offset) as u32;
    if while_loops_found.insert(condition_address) {
        let head: NodeRef = block.successors.branch_target.clone().unwrap();
        let while_loop = BaseLoop::new(cfg.loops.len(), head, NodeRef::block(i), NodeRef::block(i+1), condition_address, block.end_address);
        cfg.loops.push(Loop::While(while_loop));
    }
}


fn push_do_until(cfg: &mut ControlFlowGraph, i: usize) {
    let block = &cfg.blocks[i];
    let head: NodeRef = block.successors.fall_through.clone().unwrap();
    let after: NodeRef = block.successors.branch_target.clone().unwrap();
    let start_address: u32 = head.start_address(cfg);
    let do_until_loop = BaseLoop::new(cfg.loops.len(), head, NodeRef::block(i), after, start_address, block.end_address);
    cfg.loops.push(Loop::DoUntil(do_until_loop));
}


fn push_repeat(cfg: &mut ControlFlowGraph, i: usize) {
    let block = &cfg.blocks[i];
    let head: NodeRef = block.successors.fall_through.clone().unwrap();
    let after: NodeRef = block.successors.branch_target.clone().unwrap();
    let start_address: u32 = head.start_address(cfg);
    let do_until_loop = BaseLoop::new(cfg.loops.len(), head, NodeRef::block(i), after, start_address, block.end_address);
    cfg.loops.push(Loop::DoUntil(do_until_loop));
}


fn push_with(cfg: &mut ControlFlowGraph, i: usize) -> Result<(), String> {
    let block = &cfg.blocks[i];

    // With loop detected - need to additionally check for break block
    let head: NodeRef = block.successors.branch_target.clone().unwrap();
    let tail: NodeRef = block.successors.fall_through.clone().ok_or("PushEnv does not have a fallthrough successor")?;
    let after: NodeRef = tail.successors(cfg).branch_target.clone()
        .ok_or("PushEnv's fallthrough successor does not have a branch successor")?;
    let succ_block: &Block = after.as_block(cfg).ok_or("Expected 2nd successor of PushEnv to be a block")?;

    let mut break_block = None;
    if let [GMInstruction::Branch(_)] = succ_block.instructions {
        let potential_break_block = cfg.blocks.get(after.index+1)
            .ok_or("Expected potential break block after 2nd successor of PushEnv")?;
        let succ_start: u32 = succ_block.successors.branch_target.clone().unwrap().start_address(cfg);
        if potential_break_block.end_address == succ_start {
            if let [GMInstruction::PopWithContextExit(_)] = potential_break_block.instructions {
                break_block = Some(NodeRef::block(after.index + 1));
            }
        }
    }

    let end_address: u32 = tail.start_address(cfg);
    let mut with_loop = BaseLoop::new(cfg.loops.len(), head, tail, after, block.end_address, end_address);
    with_loop.before = Some(NodeRef::block(i));
    with_loop.break_block = break_block;
    Ok(())
}


fn update_while<'a>(cfg: &'a mut ControlFlowGraph<'a>, base_loop: &mut BaseLoop, index: usize) -> Result<(), String> {
    // Get rid of jump from tail
    cfg.disconnect_branch_successor(&base_loop.tail)?;
    let tail_block = base_loop.tail.as_block_mut(cfg).ok_or("Expected while loop's tail to be block")?;
    tail_block.pop_last_instruction();

    // Find first branch location after head
    let branch_node: NodeRef = base_loop.after
        .predecessors(cfg)
        .iter()
        .find(|n| n.start_address(cfg) < base_loop.head.start_address(cfg) && n.node_type != NodeType::Block)
        .ok_or("Failed to find while loop's first branch location after head")?
        .clone();

    let branch_block = branch_node.as_block_mut(cfg).unwrap();
    if !matches!(branch_block.instructions.last(), Some(GMInstruction::BranchUnless(_))) {
        return Err("Expected BranchUnless for while loop's first branch location after head".to_string())
    }

    // Identify body node by using branch location's first target (the one that doesn't jump)
    base_loop.body = branch_block.successors.branch_target.clone();

    // Get rid of jumps from branch location
    branch_block.pop_last_instruction();
    cfg.disconnect_fallthrough_successor(&branch_node)?;
    cfg.disconnect_branch_successor(&branch_node)?;

    // Add a new node that is branched to at the end, to keep control flow internal
    let old_after = base_loop.after.clone();
    let new_after = cfg.new_empty_node(base_loop.after.start_address(cfg));
    cfg.insert_predecessors(&base_loop.after, &new_after, base_loop.head.end_address(cfg))?;
    base_loop.after = new_after;

    // Insert structure into graph
    cfg.insert_structure(base_loop.head.clone(), old_after, NodeRef::r#loop(index))?;

    // Update parent status of Head, as well as this loop, for later operation
    //TODO

    Ok(())
}

