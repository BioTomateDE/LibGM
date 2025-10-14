// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.
use crate::gml::decompiler::control_flow::blocks::Block;
use crate::gml::decompiler::control_flow::fragments::Fragment;
use crate::gml::decompiler::control_flow::loops::Loop;
use crate::gml::decompiler::control_flow::node_ref::NodeRef;
use crate::gml::decompiler::control_flow::static_inits::StaticInit;
use crate::gml::decompiler::control_flow::successors::Successors;
use enum_as_inner::EnumAsInner;
use std::fmt::{Debug, Formatter};

#[derive(Debug, Clone)]
pub struct Node<'d> {
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

    /// The specific node data for this node kind.
    pub data: NodeData<'d>,
}

impl<'d> Node<'d> {
    pub fn new(start_address: u32, end_address: u32, data: NodeData<'d>) -> Self {
        Self {
            start_address,
            end_address,
            predecessors: vec![],
            successors: Successors::none(),
            parent: None,
            data,
        }
    }

    pub fn new_empty(address: u32) -> Self {
        Self::new(address, address, NodeData::Empty)
    }
}

#[derive(Clone, EnumAsInner)]
pub enum NodeData<'d> {
    Empty,
    Block(Block<'d>),
    Fragment(Fragment<'d>),
    StaticInit(StaticInit),
    Loop(Loop),
}

impl Debug for NodeData<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                NodeData::Empty => "Empty",
                NodeData::Block(_) => "Block",
                NodeData::Fragment(_) => "Fragment",
                NodeData::StaticInit(_) => "StaticInit",
                NodeData::Loop(_) => "Loop",
            }
        )
    }
}
