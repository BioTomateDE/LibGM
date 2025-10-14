// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.
use crate::gamemaker::data::GMData;
use crate::gml::decompiler::control_flow::node::Node;
use crate::gml::decompiler::control_flow::node_ref::NodeRef;
use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct DecompileContext<'d> {
    pub gm_data: &'d GMData,
    pub nodes: Vec<Node<'d>>,
    pub blocks: Vec<NodeRef>,
    pub short_circuit_blocks: Vec<NodeRef>,
}

impl<'d> DecompileContext<'d> {
    pub fn next_node_ref(&self) -> NodeRef {
        NodeRef::from(self.nodes.len())
    }

    pub fn make_node(&mut self, node: Node<'d>) -> NodeRef {
        let node_ref = self.next_node_ref();
        self.nodes.push(node);
        node_ref
    }

    pub fn disconnect_branch_successor(&mut self, node: NodeRef) -> Result<()> {
        let successors = &mut node.node_mut(self).successors;
        let old_successor: NodeRef = successors
            .branch_target
            .ok_or("Branch successor was not set in the first place")?;
        successors.branch_target = None;
        remove_predecessor(&mut old_successor.node_mut(self).predecessors, node)?;
        Ok(())
    }

    pub fn disconnect_fallthrough_successor(&mut self, node: NodeRef) -> Result<()> {
        let successors = &mut node.node_mut(self).successors;
        let old_successor: NodeRef = successors
            .fall_through
            .ok_or("Fallthrough successor was not set in the first place")?;
        successors.fall_through = None;
        remove_predecessor(&mut old_successor.node_mut(self).predecessors, node)?;
        Ok(())
    }

    /// TODO: i dont like this function, replace all calls to it if possible
    pub fn disconnect_predecessor(&mut self, node: NodeRef, predecessor_index: usize) -> Result<()> {
        let predecessors: &mut Vec<NodeRef> = &mut node.node_mut(self).predecessors;
        let old_predecessor: NodeRef = *predecessors
            .get(predecessor_index)
            .ok_or("Predecessor index out of range")?;
        predecessors.remove(predecessor_index);
        old_predecessor.node_mut(self).successors.remove(node);
        Ok(())
    }

    pub fn disconnect_all_predecessors(&mut self, node: NodeRef) -> Result<()> {
        for pred in node.node(self).predecessors.clone() {
            pred.node_mut(self).successors.remove(node);
        }
        node.node_mut(self).predecessors = vec![];
        Ok(())
    }

    /// Utility function to insert a new node to the control flow graph, which is a
    /// sole predecessor of "node", and takes on all predecessors of "node" that are
    /// within a range of addresses, ending at "node"'s address.
    pub fn insert_predecessors(&mut self, node: NodeRef, new_predecessor: NodeRef, start_address: u32) -> Result<()> {
        // Reroute all earlier predecessors of [node] to [new_predecessor]
        let node_start: u32 = node.node(self).start_address;
        let mut i: usize = 0;

        while let Some(curr_pred) = node.node(self).predecessors.get(i).copied() {
            let curr_start: u32 = curr_pred.node(self).start_address;
            if curr_start >= start_address && curr_start < node_start {
                new_predecessor.node_mut(self).predecessors.push(curr_pred);
                let successors = &mut curr_pred.node_mut(self).successors;
                successors.replace(node, new_predecessor)?;

                node.node_mut(self).predecessors.remove(i);
                continue;
            }
            i += 1;
        }
        Ok(())
    }

    pub fn insert_structure(&mut self, start: NodeRef, after: NodeRef, new_structure: NodeRef) -> Result<()> {
        // Reroute all nodes going into [start] to instead go into [new_structure]
        let mut i: usize = 0;
        while let Some(curr_pred) = start.node(self).predecessors.get(i).copied() {
            new_structure.node_mut(self).predecessors.push(curr_pred);
            curr_pred.node_mut(self).successors.replace(start, new_structure)?;
            i += 1;
        }
        start.node_mut(self).predecessors.clear();

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

pub fn remove_predecessor(predecessors: &mut Vec<NodeRef>, successor: NodeRef) -> Result<()> {
    let index: usize = predecessors
        .iter()
        .position(|&n| n == successor)
        .ok_or("Successor's predecessor was not set in the first place")?;
    predecessors.remove(index);
    Ok(())
}
