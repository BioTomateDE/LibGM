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
use macros::list_chunk;
pub use object::GameObject;
pub use sequence::SequenceInstance;
pub use sprite::SpriteInstance;
pub use text_item::TextItemInstance;

use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::GMElement, serialize::builder::DataBuilder,
    },
    prelude::*,
    util::assert::assert_int,
};

#[list_chunk("UILR")]
pub struct GMRootUINodes {
    pub ui_root_nodes: Vec<UINode>,
    pub exists: bool,
}

impl GMElement for GMRootUINodes {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        if reader.chunk.length() > 4 {
            log::warn!("UI nodes are untested; issues may occur");
        }
        let ui_root_nodes: Vec<UINode> = reader.read_pointer_list()?;
        Ok(Self { ui_root_nodes, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_pointer_list(&self.ui_root_nodes)?;
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

        let mut children: Vec<Self> = Vec::new();
        if matches!(type_id, 0 | 1) {
            // Container; Layer or FlexPanel
            children = reader.read_pointer_list()?;
        } else {
            let child_count = reader.read_u32()?;
            assert_int("Non-container UI Node's child count", 0, child_count)?;
        }

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

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        let type_id: i32 = match self.node {
            NodeData::Layer(_) => 0,
            NodeData::FlexPanel(_) => 1,
            NodeData::GameObject(_) => 3,
            NodeData::SequenceInstance(_) => 4,
            NodeData::SpriteInstance(_) => 5,
            NodeData::TextItemInstance(_) => 6,
            NodeData::EffectLayer(_) => 7,
        };
        builder.write_i32(type_id);
        builder.write_pointer(&self.node);

        if !matches!(self.node, NodeData::Layer(_) | NodeData::FlexPanel(_))
            && !self.children.is_empty()
        {
            bail!(
                "Expected non-container UI Node type {} to not \
                have child nodes, but actually has {} children",
                self.node.variant_name(),
                self.children.len(),
            )
        }

        builder.write_pointer_list(&self.children)?;

        builder.resolve_pointer(&self.node)?;
        match &self.node {
            NodeData::Layer(node) => node.serialize(builder)?,
            NodeData::FlexPanel(node) => node.serialize(builder)?,
            NodeData::GameObject(node) => node.serialize(builder)?,
            NodeData::SequenceInstance(node) => node.serialize(builder)?,
            NodeData::SpriteInstance(node) => node.serialize(builder)?,
            NodeData::TextItemInstance(node) => node.serialize(builder)?,
            NodeData::EffectLayer(node) => node.serialize(builder)?,
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
