use std::fmt::{Display, Formatter};
use crate::gml::decompiler::control_flow::blocks::Block;
use crate::gml::decompiler::control_flow::fragments::Fragment;
use crate::gml::decompiler::control_flow::loops::Loop;
use crate::gml::decompiler::control_flow::static_inits::StaticInit;

pub mod blocks;
pub mod fragments;
pub mod loops;
pub mod short_circuits;
pub mod static_inits;


macro_rules! delegate_to_node {
    // For methods that return mutable references
    ($field:ident, $meth:ident -> &mut $return_type:ty) => {
        pub fn $meth<'a>(&self, cfg: &'a mut ControlFlowGraph) -> &'a mut $return_type {
            match self.node_type {
                NodeType::Empty => &mut cfg.empty_nodes[self.index].$field,
                NodeType::Block => &mut cfg.blocks[self.index].$field,
                NodeType::Fragment => &mut cfg.fragments[self.index].$field,
                NodeType::Loop => &mut cfg.loops[self.index].$field,
            }
        }
    };

    // For methods that return immutable references
    ($field:ident, $meth:ident -> &$return_type:ty) => {
        pub fn $meth<'a>(&self, cfg: &'a ControlFlowGraph) -> &'a $return_type {
            match self.node_type {
                NodeType::Empty => &cfg.empty_nodes[self.index].$field,
                NodeType::Block => &cfg.blocks[self.index].$field,
                NodeType::Fragment => &cfg.fragments[self.index].$field,
                NodeType::Loop => &cfg.loops[self.index].$field,
            }
        }
    };

    // For methods that return values by copy
    ($field:ident, $meth:ident -> $return_type:ty) => {
        pub fn $meth(&self, cfg: &ControlFlowGraph) -> $return_type {
            match self.node_type {
                NodeType::Empty => cfg.empty_nodes[self.index].$field,
                NodeType::Block => cfg.blocks[self.index].$field,
                NodeType::Fragment => cfg.fragments[self.index].$field,
                NodeType::Loop => cfg.loops[self.index].$field,
            }
        }
    };
}


#[derive(Debug, Clone)]
pub struct BaseNode {
    pub start_address: u32,
    pub end_address: u32,
    pub predecessors: Vec<NodeRef>,
    pub successors: Successors,
}


#[derive(Debug, Clone)]
pub struct ControlFlowGraph<'a> {
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
        self.empty_nodes.push(BaseNode {
            start_address: address,
            end_address: address,
            predecessors: vec![],
            successors: Successors::none(),
        });
        node_ref
    }

    pub fn disconnect_branch_successor(&mut self, node: &NodeRef) -> Result<(), String> {
        let successors: &mut Successors = node.successors_mut(self);
        let old_successor: NodeRef = successors.branch_target.clone().ok_or("Branch successor was not set in the first place")?;
        successors.branch_target = None;
        remove_successor(old_successor.predecessors_mut(self), node)?;
        Ok(())
    }

    pub fn disconnect_fallthrough_successor(&mut self, node: &NodeRef) -> Result<(), String> {
        let successors: &mut Successors = node.successors_mut(self);
        let old_successor: NodeRef = successors.fall_through.clone().ok_or("Fallthrough successor was not set in the first place")?;
        successors.fall_through = None;
        remove_successor(old_successor.predecessors_mut(self), node)?;
        Ok(())
    }

    /// Utility function to insert a new node to the control flow graph, which is a
    /// sole predecessor of "node", and takes on all predecessors of "node" that are
    /// within a range of addresses, ending at "node"'s address.
    pub fn insert_predecessors(&mut self, node: &NodeRef, new_predecessor: &NodeRef, start_address: u32) -> Result<(), String> {
        // Reroute all earlier predecessors of [node] to [new_predecessor]
        let node_start: u32 = node.start_address(self);
        let mut i: usize = 0;

        while let Some(curr_pred) = node.predecessors_mut(self).get(i).cloned() {
            let curr_start: u32 = curr_pred.start_address(self);
            if curr_start >= start_address && curr_start < node_start {
                new_predecessor.predecessors_mut(self).push(curr_pred.clone());
                let successors = curr_pred.successors_mut(self);
                successors.replace(node, new_predecessor.clone())?;

                node.predecessors_mut(self).remove(i);
                continue
            }
            i += 1;
        }
        Ok(())
    }

    pub fn insert_structure(&mut self, start: NodeRef, after: NodeRef, new_structure: NodeRef) -> Result<(), String> {
        // TODO: check unreachable

        // Reroute all nodes going into [start] to instead go into [new_structure]
        let mut i: usize = 0;
        while let Some(curr_pred) = start.predecessors_mut(self).get(i).cloned() {
            new_structure.predecessors_mut(self).push(curr_pred.clone());
            curr_pred.successors_mut(self).replace(&start, new_structure.clone())?;
            i += 1;
        }

        // TODO: parent children

        // Reroute predecessor at index 0 from [after] to instead come from [new_structure]
        let after_preds = after.predecessors_mut(self);
        if let Some(pred) = after_preds.first_mut() {
            // pred.successors_mut(self).remove(&after);        // is this even needed????
            *pred = new_structure;
        } else {
            after_preds.push(new_structure.clone());
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodeRef {
    pub node_type: NodeType,
    pub index: usize,
}

impl Display for NodeRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}<{}>", self.node_type, self.index)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    Empty,
    Block,
    Fragment,
    Loop,
}

impl NodeRef {
    delegate_to_node!(start_address, start_address -> u32);
    delegate_to_node!(end_address, end_address -> u32);
    delegate_to_node!(predecessors, predecessors -> &Vec<NodeRef>);
    delegate_to_node!(predecessors, predecessors_mut -> &mut Vec<NodeRef>);
    delegate_to_node!(successors, successors -> &Successors);
    delegate_to_node!(successors, successors_mut -> &mut Successors);

    pub const fn new(node_type: NodeType, index: usize) -> NodeRef {
        Self { node_type, index }
    }

    pub const fn block(index: usize) -> Self {
        NodeRef::new(NodeType::Block, index)
    }

    pub const fn fragment(index: usize) -> Self {
        NodeRef::new(NodeType::Fragment, index)
    }

    pub const fn r#loop(index: usize) -> Self {
        NodeRef::new(NodeType::Loop, index)
    }

    pub fn as_block<'c, 'd>(&self, cfg: &'c ControlFlowGraph<'d>) -> Option<&'c Block<'d>> {
        match self.node_type {
            NodeType::Block => Some(&cfg.blocks[self.index]),
            _ => None,
        }
    }
    pub fn as_block_mut<'c, 'd>(&self, cfg: &'c mut ControlFlowGraph<'d>) -> Option<&'c mut Block<'d>> {
        match self.node_type {
            NodeType::Block => Some(&mut cfg.blocks[self.index]),
            _ => None,
        }
    }
}


#[derive(Debug, Clone)]
pub struct Successors {
    pub fall_through: Option<NodeRef>,
    pub branch_target: Option<NodeRef>,
}

impl Successors {
    pub fn none() -> Self {
        Self { fall_through: None, branch_target: None }
    }

    pub fn replace(&mut self, search: &NodeRef, replace: NodeRef) -> Result<(), String> {
        let mut found: bool = false;
        if self.branch_target.as_ref() == Some(search) {
            self.branch_target = Some(replace.clone());
            found = true;
        }
        if self.fall_through.as_ref() == Some(search) {
            self.fall_through = Some(replace);
            found = true;
        }
        if !found {
            return Err(format!("Could not find {search} successor"))
        }
        Ok(())
    }

    pub fn remove(&mut self, search: &NodeRef) {
        if self.branch_target.as_ref() == Some(search) {
            self.branch_target = None;
        }
        if self.fall_through.as_ref() == Some(search) {
            self.fall_through = None;
        }
    }
}


fn remove_successor(predecessors: &mut Vec<NodeRef>, successor: &NodeRef) -> Result<(), String> {
    let index: usize = predecessors.iter().position(|i| i == successor)
        .ok_or("Successor's predecessor was not set in the first place")?;
    predecessors.remove(index);
    Ok(())
}

