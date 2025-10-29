use crate::gml::decompiler::control_flow::node::Node;
use crate::gml::decompiler::decompile_context::DecompileContext;
use std::fmt::{Debug, Display, Formatter};

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq)]
/// TODO: tell rustc that this will never be u32::MAX for Option optimisation
pub struct NodeRef {
    pub index: u32,
}

impl NodeRef {
    pub fn try_node<'c, 'd>(self, ctx: &'c DecompileContext<'d>) -> Option<&'c Node<'d>> {
        ctx.nodes.get(self.index as usize)
    }

    pub fn try_node_mut<'c, 'd>(self, ctx: &'c mut DecompileContext<'d>) -> Option<&'c mut Node<'d>> {
        ctx.nodes.get_mut(self.index as usize)
    }

    pub fn node<'c, 'd>(self, ctx: &'c DecompileContext<'d>) -> &'c Node<'d> {
        &ctx.nodes[self.index as usize]
    }

    pub fn node_mut<'c, 'd>(self, ctx: &'c mut DecompileContext<'d>) -> &'c mut Node<'d> {
        &mut ctx.nodes[self.index as usize]
    }

    pub const fn next_sequentially(self) -> Self {
        Self { index: self.index + 1 }
    }

    pub const fn previous_sequentially(self) -> Self {
        Self { index: self.index - 1 }
    }
}

impl From<u32> for NodeRef {
    fn from(index: u32) -> Self {
        Self { index }
    }
}

impl From<usize> for NodeRef {
    fn from(index: usize) -> Self {
        Self { index: index as u32 }
    }
}

impl Display for NodeRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Node#{}", self.index)
    }
}

impl Debug for NodeRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}
