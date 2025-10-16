use crate::gml::decompiler::control_flow::node_ref::NodeRef;
use crate::prelude::*;

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

    pub fn replace(&mut self, search: NodeRef, replace: NodeRef) {
        if self.branch_target == Some(search) {
            self.branch_target = Some(replace);
        }
        if self.fall_through == Some(search) {
            self.fall_through = Some(replace);
        }
        if self.catch == Some(search) {
            self.catch = Some(replace);
        }
    }

    pub fn remove(&mut self, search: NodeRef) -> Result<()> {
        let mut found: bool = false;
        if self.branch_target == Some(search) {
            self.branch_target = None;
            found = true;
        }
        if self.fall_through == Some(search) {
            self.fall_through = None;
            found = true;
        }
        if self.catch == Some(search) {
            self.catch = None;
            found = true;
        }
        if !found {
            bail!("Could not find {search} successor");
        }
        Ok(())
    }
}
