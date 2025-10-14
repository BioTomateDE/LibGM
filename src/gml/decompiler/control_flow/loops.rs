// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.
use crate::gamemaker::elements::code::{GMGotoInstruction, GMInstruction};
use crate::gml::decompiler::control_flow::blocks::Block;
use crate::gml::decompiler::control_flow::node::{Node, NodeData};
use crate::gml::decompiler::control_flow::node_ref::NodeRef;
use crate::gml::decompiler::control_flow::successors::Successors;
use crate::gml::decompiler::decompile_context::DecompileContext;
use crate::prelude::*;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct Loop {
    pub loop_type: LoopType,

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

impl Loop {
    pub fn new<'d>(
        loop_type: LoopType,
        start_address: u32,
        end_address: u32,
        head: NodeRef,
        tail: NodeRef,
        after: NodeRef,
    ) -> Node<'d> {
        Node::new(
            start_address,
            end_address,
            NodeData::Loop(Self {
                loop_type,
                head,
                tail,
                after,
                body: None,
                before: None,
                break_block: None,
                for_loop_incrementor: None,
            }),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoopType {
    While,
    DoUntil,
    Repeat,
    With,
}

pub fn find_loops(ctx: &mut DecompileContext) -> Result<()> {
    let mut whiles = HashSet::new();
    let start_idx = ctx.nodes.len() as u32;

    // Search for different loops types based on instruction patterns
    // Do this in reverse order, because we want to find the ends of loops first
    for block in ctx.blocks.clone().into_iter().rev() {
        let Some(last_instruction) = block.block(ctx).instructions.last() else {
            continue;
        };

        // Check last instruction (where branches are located)
        match last_instruction {
            GMInstruction::Branch(instr) if instr.jump_offset < 0 => push_while(ctx, &mut whiles, block, instr),
            GMInstruction::BranchUnless(instr) if instr.jump_offset < 0 => push_do_until(ctx, block),
            GMInstruction::BranchIf(instr) if instr.jump_offset < 0 => push_repeat(ctx, block),
            GMInstruction::PushWithContext(_) => push_with(ctx, block)?,
            _ => {}
        }
    }

    let end_idx = ctx.nodes.len() as u32;
    for idx in start_idx..end_idx {
        let node = NodeRef::from(idx);
        match node.r#loop(ctx).loop_type {
            LoopType::While => update_while(ctx, node)?,
            LoopType::DoUntil => update_do_until(ctx, node)?,
            LoopType::Repeat => update_repeat(ctx, node)?,
            LoopType::With => update_with(ctx, node)?,
        }
    }

    Ok(())
}

fn push_while(ctx: &mut DecompileContext, whiles: &mut HashSet<u32>, block: NodeRef, instr: &GMGotoInstruction) {
    // While loop detected - only add if this is the first time we see it (in reverse)
    // If not done only once, then this misfires on "continue" statements
    let condition_address = (block.node(ctx).end_address as i32 - 1 + instr.jump_offset) as u32;
    if !whiles.insert(condition_address) {
        return;
    }

    let head: NodeRef = block.node(ctx).successors.branch_target.unwrap();
    ctx.nodes.push(Loop::new(
        LoopType::While,
        condition_address,
        block.node(ctx).end_address,
        head,
        block,
        block.next_sequentially(),
    ));
}

fn push_do_until(ctx: &mut DecompileContext, block: NodeRef) {
    let head: NodeRef = block.node(ctx).successors.fall_through.unwrap();
    let after: NodeRef = block.node(ctx).successors.branch_target.unwrap();
    ctx.nodes.push(Loop::new(
        LoopType::DoUntil,
        head.node(ctx).start_address,
        block.node(ctx).end_address,
        head,
        block,
        after,
    ));
}

fn push_repeat(ctx: &mut DecompileContext, block: NodeRef) {
    let head: NodeRef = block.node(ctx).successors.fall_through.unwrap();
    let after: NodeRef = block.node(ctx).successors.branch_target.unwrap();
    ctx.nodes.push(Loop::new(
        LoopType::Repeat,
        head.node(ctx).start_address,
        block.node(ctx).end_address,
        head,
        block,
        after,
    ));
}

fn push_with(ctx: &mut DecompileContext, block: NodeRef) -> Result<()> {
    // With loop detected - need to additionally check for break block
    let head: NodeRef = block.node(ctx).successors.branch_target.unwrap();
    let tail: NodeRef = block
        .node(ctx)
        .successors
        .fall_through
        .clone()
        .context("PushEnv does not have a fallthrough successor")?;
    let after: NodeRef = tail
        .node(ctx)
        .successors
        .fall_through
        .clone()
        .context("PushEnv's fallthrough successor does not have a fallthrough successor")?;
    let succ_block: &Block = after
        .node(ctx)
        .as_block()
        .context("Expected 2nd successor of PushEnv to be a block")?;

    let mut break_block = None;
    if let [GMInstruction::Branch(_)] = succ_block.instructions {
        let potential_break_block = after.next_sequentially().node(ctx);
        let p = potential_break_block
            .as_block()
            .context("Expected potential break block after 2nd successor of PushEnv")?;

        let succ_start: u32 = after
            .node(ctx)
            .successors
            .branch_target
            .unwrap()
            .node(ctx)
            .start_address;
        if potential_break_block.end_address == succ_start {
            if let [GMInstruction::PopWithContextExit(_)] = p.instructions {
                break_block = Some(after.next_sequentially());
            }
        }
    }

    let end_address: u32 = tail.node(ctx).start_address;
    let mut with_loop = Loop::new(
        LoopType::With,
        block.node(ctx).end_address,
        end_address,
        head,
        tail,
        after,
    );
    with_loop.loop_mut().before = Some(block);
    with_loop.loop_mut().break_block = break_block;
    ctx.nodes.push(with_loop);
    Ok(())
}

/// Add a new node that is branched to at the end, to keep control flow internal
fn update_control_flow(ctx: &mut DecompileContext, node: NodeRef) -> Result<()> {
    let head = node.r#loop(ctx).head;
    let old_after = node.r#loop(ctx).after;

    let start_addr = old_after.node(ctx).start_address;
    let new_after = ctx.make_node(Node::new_empty(start_addr));
    ctx.insert_predecessors(old_after, new_after, head.node(ctx).end_address)?;
    new_after.node_mut(ctx).parent = Some(node);
    node.loop_mut(ctx).after = new_after;

    // Insert structure into graph
    ctx.insert_structure(head, old_after, node)?;

    // Update parent status of Head, as well as this loop, for later operation
    node.node_mut(ctx).parent = head.node(ctx).parent;
    node.loop_mut(ctx).head.node_mut(ctx).parent = Some(node);
    Ok(())
}

fn update_while(ctx: &mut DecompileContext, node: NodeRef) -> Result<()> {
    let head = node.r#loop(ctx).head;
    let tail = node.r#loop(ctx).tail;
    let after = node.r#loop(ctx).after;

    // Get rid of jump from tail
    ctx.disconnect_branch_successor(tail)?;
    let tail_block = tail
        .node_mut(ctx)
        .as_block_mut()
        .context("Expected while loop's tail to be block")?;
    tail_block.pop_last_instruction()?;

    // Find first branch location after head
    let branch_target: NodeRef = after
        .node(ctx)
        .predecessors
        .iter()
        .find(|pred| {
            matches!(pred.node(ctx).data, NodeData::Block(_))
                && pred.node(ctx).start_address >= head.node(ctx).start_address
        })
        .context("Failed to find while loop's first branch location after head")?
        .clone();

    // Identify body node by using branch location's first target (the one that doesn't jump)
    node.loop_mut(ctx).body = branch_target.node(ctx).successors.branch_target;

    let branch_block = branch_target.block_mut(ctx);
    if !matches!(branch_block.instructions.last(), Some(GMInstruction::BranchUnless(_))) {
        bail!("Expected BranchUnless for while loop's first branch location after head");
    }

    // Get rid of jumps from branch location
    branch_block.pop_last_instruction()?;
    ctx.disconnect_fallthrough_successor(branch_target)?;
    ctx.disconnect_branch_successor(branch_target)?;

    update_control_flow(ctx, node)?;
    Ok(())
}

fn update_do_until(ctx: &mut DecompileContext, node: NodeRef) -> Result<()> {
    let tail = node.loop_mut(ctx).tail;

    // Get rid of jumps from tail
    ctx.disconnect_branch_successor(tail)?;
    ctx.disconnect_fallthrough_successor(tail)?;
    let tail_block = tail
        .node_mut(ctx)
        .as_block_mut()
        .context("Expected while loop's tail to be block")?;
    tail_block.pop_last_instruction()?;

    update_control_flow(ctx, node)?;
    Ok(())
}

fn update_repeat(ctx: &mut DecompileContext, node: NodeRef) -> Result<()> {
    let head = node.r#loop(ctx).head;
    let tail = node.r#loop(ctx).tail;
    let after = node.r#loop(ctx).after;

    // Get rid of branch (and unneeded logic) from branch into Head
    // The (first) predecessor of Head should always be a Block, as it has logic
    let head_pred: NodeRef = head.node(ctx).predecessors[0].clone();
    let head_pred_block: &mut Block = head_pred
        .node_mut(ctx)
        .as_block_mut()
        .context("Expected repeat loop's first predecessor to be block")?;
    head_pred_block.pop_last_instructions(4)?;
    ctx.disconnect_fallthrough_successor(head_pred)?;

    // Get rid of jumps (and unneeded logic) from Tail
    ctx.disconnect_branch_successor(tail)?;
    ctx.disconnect_fallthrough_successor(tail)?;
    let tail_block = tail
        .node_mut(ctx)
        .as_block_mut()
        .context("Expected repeat loop's tail to be block")?;

    if let [.., GMInstruction::Convert(_), GMInstruction::BranchIf(_)] = tail_block.instructions {
        // We have a Convert instruction before branching at the end (older GML output)
        tail_block.pop_last_instructions(5)?;
    } else {
        // We don't have any Convert instruction before branching at the end (more recent GML output)
        tail_block.pop_last_instructions(4)?;
    }

    // Remove unneeded logic from After (should also always be a Block)
    let after_block = after
        .node_mut(ctx)
        .as_block_mut()
        .context("Expected repeat loop's after to be block")?;
    after_block.pop_first_instruction()?;

    update_control_flow(ctx, node)?;
    Ok(())
}

fn update_with(ctx: &mut DecompileContext, loop_node: NodeRef) -> Result<()> {
    let base_loop = loop_node.loop_mut(ctx);
    let before = base_loop.before.unwrap();
    let head = base_loop.head;
    let tail = base_loop.tail;
    let after = base_loop.after;
    let break_block = base_loop.break_block;

    // TODO: Ensure Head is the outermost parent (not an inner block of a loop, for instance)

    // Add a new node that is branched to at the end, to keep control flow internal
    let new_after = ctx.make_node(Node::new_empty(after.node(ctx).start_address));
    ctx.insert_predecessors(after, new_after, head.node(ctx).end_address)?;
    new_after.node_mut(ctx).parent = Some(loop_node);
    loop_node.loop_mut(ctx).after = new_after;

    // Get rid of jumps from [tail]
    ctx.disconnect_branch_successor(tail)?;
    ctx.disconnect_fallthrough_successor(tail)?;

    let mut end_node = after;
    if let Some(break_block) = break_block {
        // Reroute everything going into [break_block] to instead go into [new_after]
        for pred in std::mem::take(&mut break_block.node_mut(ctx).predecessors) {
            new_after.node_mut(ctx).predecessors.push(pred);
            pred.node_mut(ctx).successors.replace(break_block, new_after)?;
        }

        // Disconnect [break_block] completely (and use the node after it as our new end location)
        end_node = break_block.next_sequentially();
        break_block.node_mut(ctx).successors = Successors::none();

        // Get rid of branch instruction from [old_after]
        let old_after_block = after
            .node_mut(ctx)
            .as_block_mut()
            .context("Expected with loop's old after to be block")?;
        old_after_block.pop_last_instruction()?;

        // Reroute branch successor of [after] to instead go to [end_node]
        let after_successors = &mut after.node_mut(ctx).successors;
        after_successors.branch_target = Some(end_node);
        ctx.disconnect_branch_successor(after)?;
        end_node.node_mut(ctx).predecessors.push(after);
    }

    // Insert structure into graph. Don't reroute backwards branches to [head] though (as other loop headers could be there)
    ctx.insert_with_loop(head, end_node, loop_node)?;

    // Redirect [before] into this loop
    ctx.disconnect_fallthrough_successor(before)?;
    ctx.disconnect_branch_successor(before)?;
    before.node_mut(ctx).successors.fall_through = Some(loop_node);
    loop_node.node_mut(ctx).predecessors.push(before);

    // Remove all predecessors of [tail] that are before this loop
    for (i, pred) in tail.node(ctx).predecessors.clone().into_iter().enumerate().rev() {
        if pred.node(ctx).start_address < loop_node.node(ctx).start_address {
            tail.node_mut(ctx).predecessors.remove(i);
            pred.node_mut(ctx).successors.remove(tail);
        }
    }

    // Update parent status of [head], as well as this loop, for later operation
    loop_node.node_mut(ctx).parent = head.node(ctx).parent;
    head.node_mut(ctx).parent = Some(loop_node);

    Ok(())
}

impl<'d> DecompileContext<'d> {
    fn insert_with_loop(&mut self, start: NodeRef, after: NodeRef, new_structure: NodeRef) -> Result<()> {
        let start_address = start.node(self).start_address;

        // Reroute all nodes going into [start] to instead go into [new_structure]
        for _ in 0..start.node(self).predecessors.len() {
            let pred = start.node(self).predecessors[0];
            if pred.node(self).start_address >= start_address {
                // If not rerouting backwards branches to "start", then ignore predecessors that come after, by address
                continue;
            }
            new_structure.node_mut(self).predecessors.push(pred);
            pred.node_mut(self).successors.replace(start, new_structure)?;
        }

        // TODO: parent children

        // Reroute predecessor at index 0 from [after] to instead come from [new_structure]
        let after_preds = &mut after.node_mut(self).predecessors;
        if let Some(pred) = after_preds.first_mut() {
            *pred = new_structure;
        } else {
            after_preds.push(new_structure);
        }

        new_structure.node_mut(self).successors.branch_target = Some(after);
        Ok(())
    }
}
