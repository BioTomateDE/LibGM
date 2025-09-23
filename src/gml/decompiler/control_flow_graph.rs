use crate::gml::decompiler::control_flow::blocks::Block;
use crate::gml::decompiler::control_flow::fragments::Fragment;


macro_rules! delegate_to_node {
    // For methods that return mutable references
    ($field:ident, $meth:ident -> &mut $return_type:ty) => {
        pub fn $meth<'a>(&self, cfg: &'a mut ControlFlowGraph) -> &'a mut $return_type {
            match self {
                NodeIndex::Block(idx) => &mut cfg.blocks[*idx].$field,
                NodeIndex::Fragment(idx) => &mut cfg.fragments[*idx].$field,
            }
        }
    };

    // For methods that return immutable references
    ($field:ident, $meth:ident -> &$return_type:ty) => {
        pub fn $meth<'a>(&self, cfg: &'a ControlFlowGraph) -> &'a $return_type {
            match self {
                NodeIndex::Block(idx) => &cfg.blocks[*idx].$field,
                NodeIndex::Fragment(idx) => &cfg.fragments[*idx].$field,
            }
        }
    };

    // For methods that return values by copy
    ($field:ident, $meth:ident -> $return_type:ty) => {
        pub fn $meth(&self, cfg: &ControlFlowGraph) -> $return_type {
            match self {
                NodeIndex::Block(idx) => cfg.blocks[*idx].$field,
                NodeIndex::Fragment(idx) => cfg.fragments[*idx].$field,
            }
        }
    };
}


#[derive(Debug, Clone)]
pub struct ControlFlowGraph<'a> {
    pub node_indices: Vec<NodeIndex>,
    pub blocks: Vec<Block<'a>>,
    pub fragments: Vec<Fragment<'a>>,
}

impl ControlFlowGraph<'_> {
    pub fn connect_nodes(&mut self, predecessor: NodeIndex, successor: NodeIndex) {
        match successor {
            NodeIndex::Block(i) => self.blocks[i].predecessors.push(predecessor.clone()),
            NodeIndex::Fragment(i) => self.fragments[i].predecessors.push(predecessor.clone()),
        }

        match predecessor {
            NodeIndex::Block(i) => self.blocks[i].successors.set_branch_target(successor),
            NodeIndex::Fragment(i) => self.fragments[i].successors.set_branch_target(successor),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeIndex {
    Block(usize),
    Fragment(usize),
}

impl NodeIndex {
    delegate_to_node!(start_address, start_address -> u32);
    delegate_to_node!(end_address, end_address -> u32);
    delegate_to_node!(predecessors, predecessors -> &Vec<NodeIndex>);
    delegate_to_node!(predecessors, predecessors_mut -> &mut Vec<NodeIndex>);
    delegate_to_node!(successors, successors -> &Successors);
    delegate_to_node!(successors, successors_mut -> &mut Successors);

    pub fn as_block_mut<'a>(&self, cfg: &'a mut ControlFlowGraph<'a>) -> Option<&'a mut Block<'a>> {
        match self {
            NodeIndex::Block(idx) => Some(&mut cfg.blocks[*idx]),
            _ => None,
        }
    }
}


#[derive(Debug, Clone)]
pub enum Successors {
    /// No successors; node ends with `exit`/`return` instruction or is the code's last node.
    None,
    /// Normal progression, always go to the next node sequentially.
    Next,
    /// Always branch to this node.
    UnconditionalBranch(NodeIndex),
    /// Either progress to the next node sequentially or branch to the specified node.
    ConditionalBranch(NodeIndex),
}

impl Successors {
    pub fn set_branch_target(&mut self, target: NodeIndex) {
        *self = match self {
            Successors::None => Successors::UnconditionalBranch(target),
            Successors::Next => Successors::ConditionalBranch(target),
            Successors::UnconditionalBranch(_) => Successors::UnconditionalBranch(target),
            Successors::ConditionalBranch(_) => Successors::ConditionalBranch(target)
        }
    }
    
    pub fn set_sequential_next(&mut self) {
        *self = match self {
            Successors::None => Successors::Next,
            Successors::Next => Successors::Next,
            Successors::UnconditionalBranch(target) => Successors::ConditionalBranch(target.clone()),
            Successors::ConditionalBranch(target) => Successors::ConditionalBranch(target.clone()),
        }
    }
}

