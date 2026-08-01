// SPDX-License-Identifier: GPL-3.0-only
mod effect;
pub mod flex;
pub mod layer;
mod object;
mod sequence;
mod sprite;
mod text_item;

pub use effect::EffectLayer;
pub use flex::FlexPanel;
pub use layer::Layer;
pub use object::GameObject;
pub use sequence::SequenceInstance;
pub use sprite::SpriteInstance;
pub use text_item::TextItemInstance;

use crate::prelude::*;
use crate::util::assert;
use crate::wad::chunk::gm_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::load::reader::DataReader;
use crate::wad::save::writer::DataWriter;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct UINodes {
    pub elems: Vec<UINode>,
}

// not sure if direct
gm_list_chunk!(UILR, UINodes, UINode);

impl GMElement for UINodes {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        if reader.chunk.length() > 12 {
            log::warn!("UI nodes are untested; issues may occur");
        }
        let elems: Vec<UINode> = reader.read_pointer_list()?;
        Ok(Self { elems })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        writer.write_pointer_list(&self.elems)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UINode {
    pub node: NodeData,
    pub children: Vec<Self>,
}

impl GMElement for UINode {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let type_id = reader.read_i32()?;
        let data_pointer = reader.read_u32()?;

        let children: Vec<Self> = if matches!(type_id, 0 | 1) {
            // Container; Layer or FlexPanel
            reader.read_pointer_list()?
        } else {
            let child_count = reader.read_u32()?;
            assert::int(child_count, 0, "Non-container UI Node's child count")?;
            Vec::new()
        };

        reader.assert_pos(data_pointer, "UI Node data")?;

        let node: NodeData = match type_id {
            0 => NodeData::Layer(Layer::deserialize(reader)?),
            1 => NodeData::FlexPanel(FlexPanel::deserialize(reader)?),
            3 => NodeData::GameObject(GameObject::deserialize(reader)?),
            4 => NodeData::SequenceInstance(SequenceInstance::deserialize(reader)?),
            5 => NodeData::SpriteInstance(SpriteInstance::deserialize(reader)?),
            6 => NodeData::TextItemInstance(TextItemInstance::deserialize(reader)?),
            7 => NodeData::EffectLayer(EffectLayer::deserialize(reader)?),
            _ => bail!("Unknown UI Node type {type_id}"),
        };

        Ok(Self { node, children })
    }

    fn serialize(&self, writer: &mut DataWriter) -> Result<()> {
        let type_id: i32 = match self.node {
            NodeData::Layer(_) => 0,
            NodeData::FlexPanel(_) => 1,
            NodeData::GameObject(_) => 3,
            NodeData::SequenceInstance(_) => 4,
            NodeData::SpriteInstance(_) => 5,
            NodeData::TextItemInstance(_) => 6,
            NodeData::EffectLayer(_) => 7,
        };
        writer.write_i32(type_id);
        writer.write_pointer(&self.node);

        if !matches!(self.node, NodeData::Layer(_) | NodeData::FlexPanel(_))
            && !self.children.is_empty()
        {
            bail!(
                "Expected non-container UI Node type {} to not have child nodes, but actually has \
                 {} children",
                self.node.variant_name(),
                self.children.len(),
            )
        }

        writer.write_pointer_list(&self.children)?;

        writer.resolve_pointer(&self.node)?;
        match &self.node {
            NodeData::Layer(node) => writer.write_elem(node)?,
            NodeData::FlexPanel(node) => writer.write_elem(node)?,
            NodeData::GameObject(node) => writer.write_elem(node)?,
            NodeData::SequenceInstance(node) => writer.write_elem(node)?,
            NodeData::SpriteInstance(node) => writer.write_elem(node)?,
            NodeData::TextItemInstance(node) => writer.write_elem(node)?,
            NodeData::EffectLayer(node) => writer.write_elem(node)?,
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeData {
    Layer(Layer),
    FlexPanel(FlexPanel),
    GameObject(GameObject),
    SequenceInstance(SequenceInstance),
    SpriteInstance(SpriteInstance),
    TextItemInstance(TextItemInstance),
    EffectLayer(EffectLayer),
}

impl NodeData {
    const fn variant_name(&self) -> &'static str {
        match self {
            Self::Layer(_) => "Layer",
            Self::FlexPanel(_) => "FlexPanel",
            Self::GameObject(_) => "GameObject",
            Self::SequenceInstance(_) => "SequenceInstance",
            Self::SpriteInstance(_) => "SpriteInstance",
            Self::TextItemInstance(_) => "TextItemInstance",
            Self::EffectLayer(_) => "EffectLayer",
        }
    }
}
