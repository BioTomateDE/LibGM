use num_enum::{IntoPrimitive, TryFromPrimitive};
use crate::gamemaker::rooms::{GMRoomGameObject, GMRoomLayerEffectProperty, GMSequenceInstance, GMSpriteInstance, GMTextItemInstance};
use crate::gamemaker::ui_nodes::flex_properties::{AlignmentKind, FlexValue, GMNodeUIFlexInstanceProperties, GMNodeUIFlexProperties};
use crate::gm_deserialize::{DataReader, GMChunkElement, GMElement, GMRef};
use crate::gm_serialize::DataBuilder;
use crate::utility::{num_enum_from, typename_val};

#[derive(Debug, Clone)]
pub struct GMRootUINodes {
    pub ui_root_nodes: Vec<GMNodeUI>,
    pub exists: bool,
}
impl GMChunkElement for GMRootUINodes {
    fn empty() -> Self {
        Self { ui_root_nodes: vec![], exists: false }
    }
    fn exists(&self) -> bool {
        self.exists
    }
}

impl GMElement for GMRootUINodes {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        if reader.get_chunk_length() > 4 {
            log::warn!("UI nodes are untested; issues may occur");
        }
        let ui_root_nodes: Vec<GMNodeUI> = reader.read_pointer_list()?;
        Ok(Self { ui_root_nodes, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        if !self.exists { return Ok(()) }
        builder.write_pointer_list(&self.ui_root_nodes)?;
        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct GMNodeUI {
    pub node: GMNodeUIData,
    pub children: Vec<Self>,
}

#[derive(Debug, Clone)]
pub enum GMNodeUIData {
    Layer(GMNodeUILayer),
    FlexPanel(GMNodeUIFlexPanel),
    GameObject(GMNodeUIGameObject),
    SequenceInstance(GMNodeUISequenceInstance),
    SpriteInstance(GMNodeUISpriteInstance),
    TextItemInstance(GMNodeUITextItemInstance),
    EffectLayer(GMNodeUIEffectLayer),
}
impl GMElement for GMNodeUI {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let type_id: i32 = reader.read_i32()?;
        let data_pointer: usize = reader.read_usize()?;

        let mut children: Vec<Self> = Vec::new();
        if matches!(type_id, 0|1) {    // container; Layer or FlexPanel
            children = reader.read_pointer_list()?;
        } else {
            let always_zero = reader.read_i32()?;
            if always_zero != 0 {
                return Err(format!("Expected zero for non-container UI Node's child count but got {always_zero} (0x{always_zero:08X})"))
            }
        }

        reader.assert_pos(data_pointer, "UI Node data")?;
        
        let node: GMNodeUIData = match type_id {
            0 => GMNodeUIData::Layer(GMNodeUILayer::deserialize(reader)?),
            1 => GMNodeUIData::FlexPanel(GMNodeUIFlexPanel::deserialize(reader)?),
            3 => GMNodeUIData::GameObject(GMNodeUIGameObject::deserialize(reader)?),
            4 => GMNodeUIData::SequenceInstance(GMNodeUISequenceInstance::deserialize(reader)?),
            5 => GMNodeUIData::SpriteInstance(GMNodeUISpriteInstance::deserialize(reader)?),
            6 => GMNodeUIData::TextItemInstance(GMNodeUITextItemInstance::deserialize(reader)?),
            7 => GMNodeUIData::EffectLayer(GMNodeUIEffectLayer::deserialize(reader)?),
            _ => return Err(format!("Unknown UI Node type {type_id}"))
        };
        
        Ok(GMNodeUI { node, children })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        let type_id: i32 = match self.node {
            GMNodeUIData::Layer(_) => 0,
            GMNodeUIData::FlexPanel(_) => 1,
            GMNodeUIData::GameObject(_) => 3,
            GMNodeUIData::SequenceInstance(_) => 4,
            GMNodeUIData::SpriteInstance(_) => 5,
            GMNodeUIData::TextItemInstance(_) => 6,
            GMNodeUIData::EffectLayer(_) => 7,
        };
        builder.write_i32(type_id);
        builder.write_pointer(&self.node)?;
        
        // write children if container node
        if matches!(self.node, GMNodeUIData::Layer(_) | GMNodeUIData::FlexPanel(_)) {
            builder.write_pointer_list(&self.children)?;
        } else {
            if !self.children.is_empty() {
                return Err(format!(
                    "Expected non-container UI Node type {} to not have child nodes, but actually has {} children",
                    typename_val(&self.node), self.children.len(),
                ))
            }
            builder.write_i32(0);
        }
        
        builder.resolve_pointer(&self.node)?;
        match &self.node {
            GMNodeUIData::Layer(node) => node.serialize(builder)?,
            GMNodeUIData::FlexPanel(node) => node.serialize(builder)?,
            GMNodeUIData::GameObject(node) => node.serialize(builder)?,
            GMNodeUIData::SequenceInstance(node) => node.serialize(builder)?,
            GMNodeUIData::SpriteInstance(node) => node.serialize(builder)?,
            GMNodeUIData::TextItemInstance(node) => node.serialize(builder)?,
            GMNodeUIData::EffectLayer(node) => node.serialize(builder)?,
        }
        
        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct GMNodeUILayer {
    pub name: GMRef<String>,
    pub draw_space: GMNodeUILayerDrawSpaceKind,
    pub visible: bool,
}
impl GMElement for GMNodeUILayer {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let draw_space: GMNodeUILayerDrawSpaceKind = num_enum_from(reader.read_i32()?)?;
        let visible: bool = reader.read_bool32()?;
        Ok(Self { name, draw_space, visible })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_gm_string(&self.name)?;
        builder.write_i32(self.draw_space.into());
        builder.write_bool32(self.visible);
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, TryFromPrimitive, IntoPrimitive)]
#[repr(i32)]
pub enum GMNodeUILayerDrawSpaceKind {
    GUI = 1,
    View = 2,
}


#[derive(Debug, Clone)]
pub struct GMNodeUIFlexPanel {
    pub name: GMRef<String>,
    pub width: FlexValue,
    pub height: FlexValue,
    pub minimum_width: FlexValue,
    pub minimum_height: FlexValue,
    pub maximum_width: FlexValue,
    pub maximum_height: FlexValue,
    pub offset_left: FlexValue,
    pub offset_right: FlexValue,
    pub offset_top: FlexValue,
    pub offset_bottom: FlexValue,
    pub clips_contents: bool,
    pub position_type: GMNodeUIFlexPanelPositionKind,
    pub align_self: AlignmentKind,
    pub margin_left: FlexValue,
    pub margin_right: FlexValue,
    pub margin_top: FlexValue,
    pub margin_bottom: FlexValue,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_properties: GMNodeUIFlexProperties,
}
impl GMElement for GMNodeUIFlexPanel {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let name: GMRef<String> = reader.read_gm_string()?;
        let width = FlexValue::deserialize(reader)?;
        let height = FlexValue::deserialize(reader)?;
        let minimum_width = FlexValue::deserialize(reader)?;
        let minimum_height = FlexValue::deserialize(reader)?;
        let maximum_width = FlexValue::deserialize(reader)?;
        let maximum_height = FlexValue::deserialize(reader)?;
        let offset_left = FlexValue::deserialize(reader)?;
        let offset_right = FlexValue::deserialize(reader)?;
        let offset_top = FlexValue::deserialize(reader)?;
        let offset_bottom = FlexValue::deserialize(reader)?;
        let clips_contents: bool = reader.read_bool32()?;
        let position_type: GMNodeUIFlexPanelPositionKind = num_enum_from(reader.read_i32()?)?;
        let align_self: AlignmentKind = num_enum_from(reader.read_i32()?)?;
        let margin_left = FlexValue::deserialize(reader)?;
        let margin_right = FlexValue::deserialize(reader)?;
        let margin_top = FlexValue::deserialize(reader)?;
        let margin_bottom = FlexValue::deserialize(reader)?;
        let flex_grow: f32 = reader.read_f32()?;
        let flex_shrink: f32 = reader.read_f32()?;
        let flex_properties = GMNodeUIFlexProperties::deserialize(reader)?;
        Ok(Self {
            name,
            width,
            height,
            minimum_width,
            minimum_height,
            maximum_width,
            maximum_height,
            offset_left,
            offset_right,
            offset_top,
            offset_bottom,
            clips_contents,
            position_type,
            align_self,
            margin_left,
            margin_right,
            margin_top,
            margin_bottom,
            flex_grow,
            flex_shrink,
            flex_properties,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_gm_string(&self.name)?;
        self.width.serialize(builder)?;
        self.height.serialize(builder)?;
        self.minimum_width.serialize(builder)?;
        self.minimum_height.serialize(builder)?;
        self.maximum_width.serialize(builder)?;
        self.maximum_height.serialize(builder)?;
        self.offset_left.serialize(builder)?;
        self.offset_right.serialize(builder)?;
        self.offset_top.serialize(builder)?;
        self.offset_bottom.serialize(builder)?;
        self.clips_contents.serialize(builder)?;
        builder.write_i32(self.position_type.into());
        builder.write_i32(self.align_self.into());
        self.margin_left.serialize(builder)?;
        self.margin_right.serialize(builder)?;
        self.margin_top.serialize(builder)?;
        self.margin_bottom.serialize(builder)?;
        builder.write_f32(self.flex_grow);
        builder.write_f32(self.flex_shrink);
        self.flex_properties.serialize(builder)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, TryFromPrimitive, IntoPrimitive)]
#[repr(i32)]
pub enum GMNodeUIFlexPanelPositionKind {
    Static = 0,
    Relative = 1,
    Absolute = 2,
}


#[derive(Debug, Clone)]
pub struct GMNodeUIGameObject {
    pub flex_instance_properties: GMNodeUIFlexInstanceProperties,
    pub room_game_object: GMRoomGameObject,
}
impl GMElement for GMNodeUIGameObject {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let room_game_object = GMRoomGameObject::deserialize(reader)?;
        let flex_instance_properties = GMNodeUIFlexInstanceProperties::deserialize(reader)?;
        Ok(Self { flex_instance_properties, room_game_object })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        self.room_game_object.serialize(builder)?;
        self.flex_instance_properties.serialize(builder)?;
        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct GMNodeUISequenceInstance {
    pub flex_instance_properties: GMNodeUIFlexInstanceProperties,
    pub sequence_instance: GMSequenceInstance,
}
impl GMElement for GMNodeUISequenceInstance {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let sequence_instance = GMSequenceInstance::deserialize(reader)?;
        let flex_instance_properties = GMNodeUIFlexInstanceProperties::deserialize(reader)?;
        Ok(Self { flex_instance_properties, sequence_instance })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        self.sequence_instance.serialize(builder)?;
        self.flex_instance_properties.serialize(builder)?;
        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct GMNodeUISpriteInstance {
    pub flex_instance_properties: GMNodeUIFlexInstanceProperties,
    pub sprite_instance: GMSpriteInstance,
}
impl GMElement for GMNodeUISpriteInstance {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let sprite_instance = GMSpriteInstance::deserialize(reader)?;
        let flex_instance_properties = GMNodeUIFlexInstanceProperties::deserialize(reader)?;
        Ok(Self { flex_instance_properties, sprite_instance })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        self.sprite_instance.serialize(builder)?;
        self.flex_instance_properties.serialize(builder)?;
        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct GMNodeUITextItemInstance {
    pub flex_instance_properties: GMNodeUIFlexInstanceProperties,
    pub text_item_instance: GMTextItemInstance,
}
impl GMElement for GMNodeUITextItemInstance {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let text_item_instance = GMTextItemInstance::deserialize(reader)?;
        let flex_instance_properties = GMNodeUIFlexInstanceProperties::deserialize(reader)?;
        Ok(Self { flex_instance_properties, text_item_instance })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        self.text_item_instance.serialize(builder)?;
        self.flex_instance_properties.serialize(builder)?;
        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct GMNodeUIEffectLayer {
    pub enabled: bool,
    pub effect_type: GMRef<String>,
    pub properties: Vec<GMRoomLayerEffectProperty>,
}
impl GMElement for GMNodeUIEffectLayer {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let enabled: bool = reader.read_bool32()?;
        let effect_type: GMRef<String> = reader.read_gm_string()?;
        let properties: Vec<GMRoomLayerEffectProperty> = reader.read_pointer_list()?;
        Ok(Self { enabled, effect_type, properties })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_bool32(self.enabled);
        builder.write_gm_string(&self.effect_type)?;
        builder.write_pointer_list(&self.properties)?;
        Ok(())
    }
}


mod flex_properties {
    use num_enum::{IntoPrimitive, TryFromPrimitive};
    use crate::gm_deserialize::{DataReader, GMElement};
    use crate::gm_serialize::DataBuilder;
    use crate::utility::num_enum_from;

    #[derive(Debug, Clone)]
    pub struct GMNodeUIFlexProperties {
        align_items: AlignmentKind,
        flex_direction: FlexDirectionKind,
        flex_wrap: WrapKind,
        align_content: AlignmentKind,
        gap_row: f32,
        gap_column: f32,
        padding_left: FlexValue,
        padding_right: FlexValue,
        padding_top: FlexValue,
        padding_bottom: FlexValue,
        justify_content: JustifyKind,
        layout_direction: LayoutDirectionKind,
    }
    impl GMElement for GMNodeUIFlexProperties {
        fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
            let align_items = num_enum_from(reader.read_i32()?)?;
            let flex_direction = num_enum_from(reader.read_i32()?)?;
            let flex_wrap = num_enum_from(reader.read_i32()?)?;
            let align_content = num_enum_from(reader.read_i32()?)?;
            let gap_row = reader.read_f32()?;
            let gap_column = reader.read_f32()?;
            let padding_left = FlexValue::deserialize(reader)?;
            let padding_right = FlexValue::deserialize(reader)?;
            let padding_top = FlexValue::deserialize(reader)?;
            let padding_bottom = FlexValue::deserialize(reader)?;
            let justify_content = num_enum_from(reader.read_i32()?)?;
            let layout_direction = num_enum_from(reader.read_i32()?)?;
            Ok(Self {
                align_items,
                flex_direction,
                flex_wrap,
                align_content,
                gap_row,
                gap_column,
                padding_left,
                padding_right,
                padding_top,
                padding_bottom,
                justify_content,
                layout_direction,
            })
        }

        fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
            builder.write_i32(self.align_items.into());
            builder.write_i32(self.flex_direction.into());
            builder.write_i32(self.flex_wrap.into());
            builder.write_i32(self.align_content.into());
            builder.write_f32(self.gap_row);
            builder.write_f32(self.gap_column);
            self.padding_left.serialize(builder)?;
            self.padding_right.serialize(builder)?;
            self.padding_top.serialize(builder)?;
            self.padding_bottom.serialize(builder)?;
            builder.write_i32(self.justify_content.into());
            builder.write_i32(self.layout_direction.into());
            Ok(())
        }
    }


    #[derive(Debug, Clone)]
    pub struct FlexValue {
        pub value: f32,
        pub unit: FlexValueUnit,
    }
    impl GMElement for FlexValue {
        fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
            let value: f32 = reader.read_f32()?;
            let unit: FlexValueUnit = num_enum_from(reader.read_i32()?)?;
            Ok(Self { value, unit })
        }

        fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
            builder.write_f32(self.value);
            builder.write_i32(self.unit.into());
            Ok(())
        }
    }


    #[derive(Debug, Clone, Copy, TryFromPrimitive, IntoPrimitive)]
    #[repr(i32)]
    pub enum FlexValueUnit {
        Undefined = 0,
        Point = 1,
        Percent = 2,
        Auto = 3,
    }

    #[derive(Debug, Clone, Copy, TryFromPrimitive, IntoPrimitive)]
    #[repr(i32)]
    pub enum AlignmentKind {
        Auto = 0,
        FlexStart = 1,
        Center = 2,
        FlexEnd = 3,
        Stretch = 4,
        Baseline = 5,
        SpaceBetween = 6,
        SpaceAround = 7,
        SpaceEvenly = 8,
    }

    #[derive(Debug, Clone, Copy, TryFromPrimitive, IntoPrimitive)]
    #[repr(i32)]
    pub enum FlexDirectionKind {
        Column = 0,
        ColumnReverse = 1,
        Row = 2,
        RowReverse = 3,
    }

    #[derive(Debug, Clone, Copy, TryFromPrimitive, IntoPrimitive)]
    #[repr(i32)]
    pub enum WrapKind {
        NoWrap = 0,
        Wrap = 1,
        WrapReverse = 2,
    }

    #[derive(Debug, Clone, Copy, TryFromPrimitive, IntoPrimitive)]
    #[repr(i32)]
    pub enum JustifyKind {
        FlexStart = 0,
        Center = 1,
        FlexEnd = 2,
        SpaceBetween = 3,
        SpaceAround = 4,
        SpaceEvenly = 5,
    }

    #[derive(Debug, Clone, Copy, TryFromPrimitive, IntoPrimitive)]
    #[repr(i32)]
    pub enum LayoutDirectionKind {
        Inherit = 0,
        LTR = 1,
        RTL = 2
    }


    #[derive(Debug, Clone)]
    pub struct GMNodeUIFlexInstanceProperties {
        visible: bool,
        anchor: i32,
        stretch_width: bool,
        stretch_height: bool,
        tile_h: bool,
        tile_v: bool,
        keep_aspect_ratio: bool,
    }
    impl GMElement for GMNodeUIFlexInstanceProperties {
        fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
            let visible: bool = reader.read_bool32()?;
            let anchor: i32 = reader.read_i32()?;
            let stretch_width: bool = reader.read_bool32()?;
            let stretch_height: bool = reader.read_bool32()?;
            let tile_h: bool = reader.read_bool32()?;
            let tile_v: bool = reader.read_bool32()?;
            let keep_aspect_ratio: bool = reader.read_bool32()?;
            Ok(Self {
                visible,
                anchor,
                stretch_width,
                stretch_height,
                tile_h,
                tile_v,
                keep_aspect_ratio,
            })
        }

        fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
            builder.write_bool32(self.visible);
            builder.write_i32(self.anchor);
            builder.write_bool32(self.stretch_width);
            builder.write_bool32(self.stretch_height);
            builder.write_bool32(self.tile_h);
            builder.write_bool32(self.tile_v);
            builder.write_bool32(self.keep_aspect_ratio);
            Ok(())
        }
    }
}

