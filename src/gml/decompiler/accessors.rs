use crate::gml::decompiler::control_flow::blocks::Block;
use crate::gml::decompiler::control_flow::fragments::Fragment;
use crate::gml::decompiler::control_flow::loops::Loop;
use crate::gml::decompiler::control_flow::node::{Node, NodeData};
use crate::gml::decompiler::control_flow::node_ref::NodeRef;
use crate::gml::decompiler::decompile_context::DecompileContext;
use std::ops::{Deref, DerefMut};

impl<'d> Deref for Node<'d> {
    type Target = NodeData<'d>;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'d> DerefMut for Node<'d> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<'d> Node<'d> {
    pub fn block(&self) -> &Block<'d> {
        self.as_block().expect("expected Block variant")
    }
    pub fn block_mut(&mut self) -> &mut Block<'d> {
        self.as_block_mut().expect("expected Block variant")
    }
    pub fn fragment(&self) -> &Fragment<'d> {
        self.as_fragment().expect("expected Fragment variant")
    }
    pub fn fragment_mut(&mut self) -> &mut Fragment<'d> {
        self.as_fragment_mut().expect("expected Fragment variant")
    }
    pub fn r#loop(&self) -> &Loop {
        self.as_loop().expect("expected Loop variant")
    }
    pub fn loop_mut(&mut self) -> &mut Loop {
        self.as_loop_mut().expect("expected Loop variant")
    }
}

impl NodeRef {
    pub fn block<'c, 'd>(&self, ctx: &'c DecompileContext<'d>) -> &'c Block<'d> {
        self.node(ctx).block()
    }
    pub fn block_mut<'c, 'd>(&self, ctx: &'c mut DecompileContext<'d>) -> &'c mut Block<'d> {
        self.node_mut(ctx).block_mut()
    }
    pub fn fragment<'c, 'd>(&self, ctx: &'c DecompileContext<'d>) -> &'c Fragment<'d> {
        self.node(ctx).fragment()
    }
    pub fn fragment_mut<'c, 'd>(&self, ctx: &'c mut DecompileContext<'d>) -> &'c mut Fragment<'d> {
        self.node_mut(ctx).fragment_mut()
    }
    pub fn r#loop<'c, 'd>(&self, ctx: &'c DecompileContext<'d>) -> &'c Loop {
        self.node(ctx).r#loop()
    }
    pub fn loop_mut<'c, 'd>(&self, ctx: &'c mut DecompileContext<'d>) -> &'c mut Loop {
        self.node_mut(ctx).loop_mut()
    }
}
