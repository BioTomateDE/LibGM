use crate::gml::decompiler::control_flow::blocks::Block;
use crate::gml::decompiler::control_flow::fragments::Fragment;
use crate::gml::decompiler::control_flow::loops::Loop;
use crate::gml::decompiler::control_flow::static_inits::StaticInit;
use crate::gml::decompiler::decompile_context::DecompileContext;
use crate::prelude::*;
use std::fmt::{Display, Formatter};

pub mod blocks;
pub mod fragments;
pub mod loops;
pub mod short_circuits;
pub mod static_inits;

macro_rules! delegate_to_node {
    // For methods that return mutable references
    ($field:ident, $meth:ident -> &mut $return_type:ty) => {
        pub fn $meth<'a>(&self, cfg: &'a mut ControlFlowGraph) -> &'a mut $return_type {
            let idx = self.index();
            match self.node_type() {
                NodeType::Empty => &mut cfg.empty_nodes[idx].$field,
                NodeType::Block => &mut cfg.blocks[idx].$field,
                NodeType::Fragment => &mut cfg.fragments[idx].$field,
                NodeType::StaticInit => &mut cfg.static_inits[idx].$field,
                NodeType::Loop => &mut cfg.loops[idx].$field,
            }
        }
    };

    // For methods that return immutable references
    ($field:ident, $meth:ident -> &$return_type:ty) => {
        pub fn $meth<'a>(&self, cfg: &'a ControlFlowGraph) -> &'a $return_type {
            let idx = self.index();
            match self.node_type() {
                NodeType::Empty => &cfg.empty_nodes[idx].$field,
                NodeType::Block => &cfg.blocks[idx].$field,
                NodeType::Fragment => &cfg.fragments[idx].$field,
                NodeType::StaticInit => &cfg.static_inits[idx].$field,
                NodeType::Loop => &cfg.loops[idx].$field,
            }
        }
    };

    // For methods that return values by copy
    ($field:ident, $meth:ident -> $return_type:ty) => {
        pub fn $meth(&self, cfg: &ControlFlowGraph) -> $return_type {
            let idx = self.index();
            match self.node_type() {
                NodeType::Empty => cfg.empty_nodes[idx].$field,
                NodeType::Block => cfg.blocks[idx].$field,
                NodeType::Fragment => cfg.fragments[idx].$field,
                NodeType::StaticInit => cfg.static_inits[idx].$field,
                NodeType::Loop => cfg.loops[idx].$field,
            }
        }
    };
}

#[derive(Debug, Clone)]
pub struct BaseNode {
    /// The address of the first instruction from the original bytecode, where this node begins.
    pub start_address: u32,

    /// The address of the instruction after this node ends (that is, exclusive).
    pub end_address: u32,

    /// All nodes which precede this one in the control flow graph.
    pub predecessors: Vec<NodeRef>,

    /// All nodes which succeed this one in the control flow graph.
    pub successors: Successors,

    /// If disconnected from the rest of the graph, e.g. at the start of a high-level
    /// control flow structure like a loop, this points to the enveloping structure.
    pub parent: Option<NodeRef>,
}

impl BaseNode {
    pub fn new(start_address: u32, end_address: u32) -> Self {
        Self {
            start_address,
            end_address,
            predecessors: vec![],
            successors: Successors::none(),
            parent: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ControlFlowGraph<'a> {
    pub context: DecompileContext<'a>,
    pub empty_nodes: Vec<BaseNode>,
    pub blocks: Vec<Block<'a>>,
    pub fragments: Vec<Fragment<'a>>,
    pub static_inits: Vec<StaticInit>,
    pub short_circuit_blocks: Vec<NodeRef>,
    pub loops: Vec<Loop>,
}

impl ControlFlowGraph<'_> {
    pub fn new_empty_node(&mut self, address: u32) -> NodeRef {
        let node_ref = NodeRef::new(NodeType::Empty, self.empty_nodes.len());
        self.empty_nodes.push(BaseNode::new(address, address));
        node_ref
    }

    pub fn disconnect_branch_successor(&mut self, node: NodeRef) -> Result<()> {
        let successors: &mut Successors = node.successors_mut(self);
        let old_successor: NodeRef = successors
            .branch_target
            .ok_or("Branch successor was not set in the first place")?;
        successors.branch_target = None;
        remove_predecessor(old_successor.predecessors_mut(self), node)?;
        Ok(())
    }

    pub fn disconnect_fallthrough_successor(&mut self, node: NodeRef) -> Result<()> {
        let successors: &mut Successors = node.successors_mut(self);
        let old_successor: NodeRef = successors
            .fall_through
            .ok_or("Fallthrough successor was not set in the first place")?;
        successors.fall_through = None;
        remove_predecessor(old_successor.predecessors_mut(self), node)?;
        Ok(())
    }

    /// TODO: i dont like this function, replace all calls to it if possible
    pub fn disconnect_predecessor(&mut self, node: NodeRef, predecessor_index: usize) -> Result<()> {
        let predecessors: &mut Vec<NodeRef> = node.predecessors_mut(self);
        let old_predecessor: NodeRef = *predecessors
            .get(predecessor_index)
            .ok_or("Predecessor index out of range")?;
        predecessors.remove(predecessor_index);
        old_predecessor.successors_mut(self).remove(node);
        Ok(())
    }

    pub fn disconnect_all_predecessors(&mut self, node: NodeRef) -> Result<()> {
        for pred in node.predecessors(self).clone() {
            pred.successors_mut(self).remove(node);
        }
        *node.predecessors_mut(self) = vec![];
        Ok(())
    }

    /// Utility function to insert a new node to the control flow graph, which is a
    /// sole predecessor of "node", and takes on all predecessors of "node" that are
    /// within a range of addresses, ending at "node"'s address.
    pub fn insert_predecessors(&mut self, node: NodeRef, new_predecessor: NodeRef, start_address: u32) -> Result<()> {
        // Reroute all earlier predecessors of [node] to [new_predecessor]
        let node_start: u32 = node.start_address(self);
        let mut i: usize = 0;

        while let Some(curr_pred) = node.predecessors_mut(self).get(i).copied() {
            let curr_start: u32 = curr_pred.start_address(self);
            if curr_start >= start_address && curr_start < node_start {
                new_predecessor.predecessors_mut(self).push(curr_pred);
                let successors = curr_pred.successors_mut(self);
                successors.replace(node, new_predecessor)?;

                node.predecessors_mut(self).remove(i);
                continue;
            }
            i += 1;
        }
        Ok(())
    }

    pub fn insert_structure(&mut self, start: NodeRef, after: NodeRef, new_structure: NodeRef) -> Result<()> {
        // Reroute all nodes going into [start] to instead go into [new_structure]
        let mut i: usize = 0;
        while let Some(curr_pred) = start.predecessors_mut(self).get(i).copied() {
            new_structure.predecessors_mut(self).push(curr_pred);
            curr_pred.successors_mut(self).replace(start, new_structure)?;
            i += 1;
        }
        start.predecessors_mut(self).clear();

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

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodeRef(u32);
impl NodeRef {
    const TYPE_BITS: u32 = 5; // 5 bits = 32 variants max
    const INDEX_BITS: u32 = 27; // 27 bits = ~134 million nodes
    const TYPE_MASK: u32 = 0b11111;
    const INDEX_MASK: u32 = (1 << 27) - 1;

    pub const fn new(node_type: NodeType, index: usize) -> Self {
        debug_assert!(index < (1 << Self::INDEX_BITS), "Index too large");
        let type_bits = node_type as u32 & Self::TYPE_MASK;
        let index_bits = index as u32 & Self::INDEX_MASK;
        Self(type_bits | (index_bits << Self::TYPE_BITS))
    }
    pub const fn node_type(&self) -> NodeType {
        unsafe { std::mem::transmute((self.0 & Self::TYPE_MASK) as u8) }
    }
    pub const fn index(&self) -> usize {
        ((self.0 >> Self::TYPE_BITS) & Self::INDEX_MASK) as usize
    }

    pub const fn next_sequentially(&self) -> Self {
        Self::new(self.node_type(), self.index() + 1)
    }
}

impl Display for NodeRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}<{}>", self.node_type(), self.index())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum NodeType {
    Empty,
    Block,
    Fragment,
    StaticInit,
    Loop,
}

impl NodeRef {
    delegate_to_node!(start_address, start_address -> u32);
    delegate_to_node!(end_address, end_address -> u32);
    delegate_to_node!(predecessors, predecessors -> &Vec<NodeRef>);
    delegate_to_node!(predecessors, predecessors_mut -> &mut Vec<NodeRef>);
    delegate_to_node!(successors, successors -> &Successors);
    delegate_to_node!(successors, successors_mut -> &mut Successors);
    delegate_to_node!(parent, parent -> Option<NodeRef>);
    delegate_to_node!(parent, parent_mut -> &mut Option<NodeRef>);

    pub const fn block(index: usize) -> Self {
        NodeRef::new(NodeType::Block, index)
    }
    pub const fn fragment(index: usize) -> Self {
        NodeRef::new(NodeType::Fragment, index)
    }
    pub const fn static_init(index: usize) -> Self {
        NodeRef::new(NodeType::StaticInit, index)
    }
    pub const fn r#loop(index: usize) -> Self {
        NodeRef::new(NodeType::Loop, index)
    }

    pub fn as_block<'c, 'd>(&self, cfg: &'c ControlFlowGraph<'d>) -> Option<&'c Block<'d>> {
        match self.node_type() {
            NodeType::Block => cfg.blocks.get(self.index()),
            _ => None,
        }
    }
    pub fn as_block_mut<'c, 'd>(&self, cfg: &'c mut ControlFlowGraph<'d>) -> Option<&'c mut Block<'d>> {
        match self.node_type() {
            NodeType::Block => cfg.blocks.get_mut(self.index()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Successors {
    /// The next node to execute when the control flow continues sequentially.
    /// This value is [`None`] if one of these apply:
    /// - The node branches unconditionally (`Branch` instruction).
    /// - The node ends in a `Exit` or `Return` instruction.
    /// - It is the last node in the code.
    pub fall_through: Option<NodeRef>,

    /// The node to jump to if the branch condition is true.
    /// This value is [`Some`] if the node's last instruction is
    /// either a`(Branch(If|Unless)?|(Push|Pop)WithContext)` instruction,
    /// or a try-block, in which case this is value represents the `finally` block.
    pub branch_target: Option<NodeRef>,

    /// The node that will be executed if the try-catch block failed.
    /// This value is only [`Some`] if the node is a try-block (obviously).
    pub catch: Option<NodeRef>,
}

impl Successors {
    pub fn none() -> Self {
        Self { fall_through: None, branch_target: None, catch: None }
    }

    pub fn replace(&mut self, search: NodeRef, replace: NodeRef) -> Result<()> {
        let mut found: bool = false;
        if self.branch_target == Some(search) {
            self.branch_target = Some(replace.clone());
            found = true;
        }
        if self.fall_through == Some(search) {
            self.fall_through = Some(replace.clone());
            found = true;
        }
        if self.catch == Some(search) {
            self.catch = Some(replace);
            found = true;
        }
        if !found {
            bail!("Could not find {search} successor");
        }
        Ok(())
    }

    pub fn remove(&mut self, search: NodeRef) {
        if self.branch_target == Some(search) {
            self.branch_target = None;
        }
        if self.fall_through == Some(search) {
            self.fall_through = None;
        }
        if self.catch == Some(search) {
            self.catch = None;
        }
    }
}

pub fn remove_predecessor(predecessors: &mut Vec<NodeRef>, successor: NodeRef) -> Result<()> {
    let index: usize = predecessors
        .iter()
        .position(|&n| n == successor)
        .ok_or("Successor's predecessor was not set in the first place")?;
    predecessors.remove(index);
    Ok(())
}
