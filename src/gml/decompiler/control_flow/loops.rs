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
    pub fn new(start_address: u32, end_address: u32, loop_index: usize, head: NodeRef, tail: NodeRef, after: NodeRef) -> Self {
        Self {
            base_node: BaseNode::new(start_address, end_address),
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

impl Deref for BaseLoop {
    type Target = BaseNode;
    fn deref(&self) -> &Self::Target {
        &self.base_node
    }
}
impl DerefMut for BaseLoop {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base_node
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


pub fn find_loops<'c, 'd>(cfg: &'c mut ControlFlowGraph<'c>) -> Result<(), String> {
    let mut while_loops_found = HashSet::new();
    let mut loops = Vec::new();

    // Search for different loops types based on instruction patterns
    // Do this in reverse order, because we want to find the ends of loops first
    for idx in (0..cfg.blocks.len()).rev() {
        let block = &cfg.blocks[idx];
        let Some(last_instruction) = block.instructions.last() else {continue};

        // Check last instruction (where branches are located)
        match last_instruction {
            GMInstruction::Branch(instr) => push_while(cfg, &mut loops, &mut while_loops_found, idx, instr),
            GMInstruction::BranchUnless(instr) if instr.jump_offset < 0 => push_do_until(cfg, &mut loops, idx),
            GMInstruction::BranchIf(instr) if instr.jump_offset < 0 => push_repeat(cfg, &mut loops, idx),
            GMInstruction::PushWithContext(_) => push_with(cfg, &mut loops, idx)?,
            _ => {}
        }
    }

    for (i, r#loop) in loops.iter_mut().enumerate() {
        let loop_node = NodeRef::r#loop(i);
        match r#loop {
            Loop::While(base_loop) => update_while(cfg, base_loop, loop_node)?,
            Loop::DoUntil(base_loop) => update_do_until(cfg, base_loop, loop_node)?,
            Loop::Repeat(base_loop) => update_repeat(cfg, base_loop, loop_node)?,
            Loop::With(base_loop) => update_with(cfg, base_loop, loop_node)?,
        }
    }

    cfg.loops = loops;
    Ok(())
}



fn push_while(cfg: &mut ControlFlowGraph, loops: &mut Vec<Loop>, while_loops_found: &mut HashSet<u32>, idx: usize, instr: &GMGotoInstruction) {
    let block = &cfg.blocks[idx];

    // While loop detected - only add if this is the first time we see it (in reverse)
    // If not done only once, then this misfires on "continue" statements
    let condition_address = (block.end_address as i32 - 1 + instr.jump_offset) as u32;
    if while_loops_found.insert(condition_address) {
        let head: NodeRef = block.successors.branch_target.clone().unwrap();
        let while_loop = BaseLoop::new(condition_address, block.end_address, cfg.loops.len(), head, NodeRef::block(idx), NodeRef::block(idx +1));
        loops.push(Loop::While(while_loop));
    }
}


fn push_do_until(cfg: &mut ControlFlowGraph, loops: &mut Vec<Loop>, idx: usize) {
    let block = &cfg.blocks[idx];
    let head: NodeRef = block.successors.fall_through.clone().unwrap();
    let after: NodeRef = block.successors.branch_target.clone().unwrap();
    let start_address: u32 = head.start_address(cfg);
    let do_until_loop = BaseLoop::new(start_address, block.end_address, loops.len(), head, NodeRef::block(idx), after);
    loops.push(Loop::DoUntil(do_until_loop));
}


fn push_repeat(cfg: &mut ControlFlowGraph, loops: &mut Vec<Loop>, idx: usize) {
    let block = &cfg.blocks[idx];
    let head: NodeRef = block.successors.fall_through.clone().unwrap();
    let after: NodeRef = block.successors.branch_target.clone().unwrap();
    let start_address: u32 = head.start_address(cfg);
    let do_until_loop = BaseLoop::new(start_address, block.end_address, loops.len(), head, NodeRef::block(idx), after);
    loops.push(Loop::DoUntil(do_until_loop));
}


fn push_with(cfg: &mut ControlFlowGraph, loops: &mut Vec<Loop>, index: usize) -> Result<(), String> {
    let block = &cfg.blocks[index];

    // With loop detected - need to additionally check for break block
    let head: NodeRef = block.successors.branch_target.clone().unwrap();
    let tail: NodeRef = block.successors.fall_through.clone().ok_or("PushEnv does not have a fallthrough successor")?;
    let after: NodeRef = tail.successors(cfg).branch_target.clone()
        .ok_or("PushEnv's fallthrough successor does not have a branch successor")?;
    let succ_block: &Block = after.as_block(cfg).ok_or("Expected 2nd successor of PushEnv to be a block")?;

    let mut break_block = None;
    if let [GMInstruction::Branch(_)] = succ_block.instructions {
        let potential_break_block = after.next_sequentially().as_block(cfg)
            .ok_or("Expected potential break block after 2nd successor of PushEnv")?;
        let succ_start: u32 = succ_block.successors.branch_target.clone().unwrap().start_address(cfg);
        if potential_break_block.end_address == succ_start {
            if let [GMInstruction::PopWithContextExit(_)] = potential_break_block.instructions {
                break_block = Some(after.next_sequentially());
            }
        }
    }

    let end_address: u32 = tail.start_address(cfg);
    let mut with_loop = BaseLoop::new(block.end_address, end_address, loops.len(), head, tail, after);
    with_loop.before = Some(NodeRef::block(index));
    with_loop.break_block = break_block;
    loops.push(Loop::With(with_loop));
    Ok(())
}



/// Add a new node that is branched to at the end, to keep control flow internal
fn update_control_flow<'c, 'd>(cfg: &'c mut ControlFlowGraph<'d>, base_loop: &'c mut BaseLoop, loop_node: NodeRef) -> Result<(), String> {
    let old_after = base_loop.after.clone();
    let new_after = cfg.new_empty_node(base_loop.after.start_address(cfg));
    cfg.insert_predecessors(base_loop.after, new_after, base_loop.head.end_address(cfg))?;
    *new_after.parent_mut(cfg) = Some(loop_node);
    base_loop.after = new_after;

    // Insert structure into graph
    cfg.insert_structure(base_loop.head.clone(), old_after, loop_node)?;

    // Update parent status of Head, as well as this loop, for later operation
    base_loop.parent = base_loop.head.parent(cfg);
    *base_loop.head.parent_mut(cfg) = Some(loop_node);
    Ok(())
}


fn update_while<'c, 'd>(cfg: &'c mut ControlFlowGraph<'d>, base_loop: &'c mut BaseLoop, loop_node: NodeRef) -> Result<(), String> {
    // Get rid of jump from tail
    cfg.disconnect_branch_successor(base_loop.tail)?;
    let tail_block = base_loop.tail.as_block_mut(cfg)
        .ok_or("Expected while loop's tail to be block")?;
    tail_block.pop_last_instruction()?;

    // Find first branch location after head
    let branch_node: NodeRef = base_loop.after
        .predecessors(cfg)
        .iter()
        .find(|n| n.start_address(cfg) < base_loop.head.start_address(cfg) && n.node_type() != NodeType::Block)
        .ok_or("Failed to find while loop's first branch location after head")?
        .clone();

    let branch_block = branch_node.as_block_mut(cfg).unwrap();
    if !matches!(branch_block.instructions.last(), Some(GMInstruction::BranchUnless(_))) {
        return Err("Expected BranchUnless for while loop's first branch location after head".to_string())
    }

    // Identify body node by using branch location's first target (the one that doesn't jump)
    base_loop.body = branch_block.successors.branch_target.clone();

    // Get rid of jumps from branch location
    branch_block.pop_last_instruction()?;
    cfg.disconnect_fallthrough_successor(branch_node)?;
    cfg.disconnect_branch_successor(branch_node)?;

    update_control_flow(cfg, base_loop, loop_node)?;
    Ok(())
}


fn update_do_until<'c, 'd>(cfg: &'c mut ControlFlowGraph<'d>, base_loop: &'c mut BaseLoop, loop_node: NodeRef) -> Result<(), String> {
    // Get rid of jumps from tail
    cfg.disconnect_branch_successor(base_loop.tail)?;
    cfg.disconnect_fallthrough_successor(base_loop.tail)?;
    let tail_block = base_loop.tail.as_block_mut(cfg)
        .ok_or("Expected while loop's tail to be block")?;
    tail_block.pop_last_instruction()?;

    update_control_flow(cfg, base_loop, loop_node)?;
    Ok(())
}


fn update_repeat<'c, 'd>(cfg: &'c mut ControlFlowGraph<'d>, base_loop: &'c mut BaseLoop, loop_node: NodeRef) -> Result<(), String> {
    // Get rid of branch (and unneeded logic) from branch into Head
    // The (first) predecessor of Head should always be a Block, as it has logic
    let head_pred: NodeRef = base_loop.head.predecessors(cfg)[0].clone();
    let head_pred_block: &mut Block = head_pred.as_block_mut(cfg)
        .ok_or("Expected repeat loop's first predecessor to be block")?;
    head_pred_block.pop_last_instructions(4)?;
    cfg.disconnect_fallthrough_successor(head_pred)?;

    // Get rid of jumps (and unneeded logic) from Tail
    cfg.disconnect_branch_successor(base_loop.tail)?;
    cfg.disconnect_fallthrough_successor(base_loop.tail)?;
    let tail_block = base_loop.tail.as_block_mut(cfg)
        .ok_or("Expected repeat loop's tail to be block")?;

    if let [.., GMInstruction::Convert(_), GMInstruction::BranchIf(_)] = tail_block.instructions {
        // We have a Convert instruction before branching at the end (older GML output)
        tail_block.pop_last_instructions(5)?;
    } else {
        // We don't have any Convert instruction before branching at the end (more recent GML output)
        tail_block.pop_last_instructions(4)?;
    }

    // Remove unneeded logic from After (should also always be a Block)
    let after_block = base_loop.after.as_block_mut(cfg)
        .ok_or("Expected repeat loop's after to be block")?;
    after_block.pop_first_instruction()?;

    update_control_flow(cfg, base_loop, loop_node)?;
    Ok(())
}


fn update_with<'c, 'd>(cfg: &'c mut ControlFlowGraph<'d>, base_loop: &'c mut BaseLoop, loop_node: NodeRef) -> Result<(), String> {
    // TODO: Ensure Head is the outermost parent (not an inner block of a loop, for instance)

    // Add a new node that is branched to at the end, to keep control flow internal
    let old_after = base_loop.after;
    let new_after = cfg.new_empty_node(base_loop.after.start_address(cfg));
    cfg.insert_predecessors(base_loop.after, new_after, base_loop.head.end_address(cfg))?;
    *new_after.parent_mut(cfg) = Some(loop_node);
    base_loop.after = new_after;

    // Get rid of jumps from [tail]
    cfg.disconnect_branch_successor(base_loop.tail)?;
    cfg.disconnect_fallthrough_successor(base_loop.tail)?;

    let mut end_node = old_after;
    if let Some(break_block) = base_loop.break_block {
        // Reroute everything going into [break_block] to instead go into [new_after]
        for pred in break_block.predecessors_mut(cfg).clone() {
            new_after.predecessors_mut(cfg).push(pred);
            pred.successors_mut(cfg).replace(break_block, new_after)?;
        }

        // Disconnect [break_block] completely (and use the node after it as our new end location)
        *break_block.predecessors_mut(cfg) = vec![];
        end_node = NodeRef::block(break_block.index() + 1);
        *break_block.successors_mut(cfg) = Successors::none();

        // Get rid of branch instruction from [old_after]
        let old_after_block = old_after.as_block_mut(cfg)
            .ok_or("Expected with loop's old after to be block")?;
        old_after_block.pop_last_instruction()?;

        // Reroute branch successor of [after] to instead go to [end_node]
        let after_successors = base_loop.after.successors_mut(cfg);
        after_successors.branch_target = Some(end_node);
        cfg.disconnect_branch_successor(base_loop.after)?;
        end_node.predecessors_mut(cfg).push(base_loop.after);
    }

    // Insert structure into graph. Don't reroute backwards branches to [head] though (as other loop headers could be there)
    cfg.insert_with_loop(base_loop.head, end_node, loop_node)?;

    // Redirect [before] into this loop
    let before= base_loop.before.unwrap();
    cfg.disconnect_fallthrough_successor(before)?;
    cfg.disconnect_branch_successor(before)?;
    before.successors_mut(cfg).fall_through = Some(loop_node);
    base_loop.predecessors.push(before);

    // Remove all predecessors of [tail] that are before this loop
    let mut to_remove = Vec::new();
    for (i, pred) in base_loop.tail.predecessors(cfg).iter().enumerate() {
        if pred.start_address(cfg) < base_loop.start_address {
            to_remove.push(i);
        }
    }
    // I love constantly having to fight the borrow checker
    for i in to_remove.into_iter().rev() {
        let pred = base_loop.tail.predecessors(cfg)[i];
        base_loop.tail.predecessors_mut(cfg).remove(i);
        pred.successors_mut(cfg).remove(base_loop.tail);
    }

    // Update parent status of [head], as well as this loop, for later operation
    base_loop.parent = base_loop.head.parent(cfg);
    *base_loop.head.parent_mut(cfg) = Some(loop_node);

    Ok(())
}


impl<'d> ControlFlowGraph<'d> {
    fn insert_with_loop(&mut self, start: NodeRef, after: NodeRef, new_structure: NodeRef) -> Result<(), String> {
        let start_address = start.start_address(self);

        // Reroute all nodes going into [start] to instead go into [new_structure]
        for _ in 0..start.predecessors(self).len() {
            let pred = start.predecessors(self)[0];
            if pred.start_address(self) >= start_address {
                // If not rerouting backwards branches to "start", then ignore predecessors that come after, by address
                continue
            }
            new_structure.predecessors_mut(self).push(pred);
            pred.successors_mut(self).replace(start, new_structure)?;
        }

        // TODO: parent children

        // Reroute predecessor at index 0 from [after] to instead come from [new_structure]
        let after_preds = after.predecessors_mut(self);
        if let Some(pred) = after_preds.first_mut() {
            *pred = new_structure;
        } else {
            after_preds.push(new_structure);
        }

        new_structure.successors_mut(self).branch_target = Some(after);
        Ok(())
    }
}

