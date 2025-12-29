use crate::{
    gamemaker::{
        deserialize::reader::DataReader, elements::GMElement, reference::GMRef,
        serialize::builder::DataBuilder,
    },
    gml::instruction::GMCode,
    prelude::*,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Events {
    pub events: Vec<Event>,
}

impl GMElement for Events {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let events: Vec<Event> = reader.read_pointer_list()?;
        Ok(Self { events })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_pointer_list(&self.events)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub subtype: u32,
    pub actions: Vec<Action>,
}

impl GMElement for Event {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let subtype = reader.read_u32()?;
        let actions: Vec<Action> = reader.read_pointer_list()?;
        Ok(Self { subtype, actions })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_u32(self.subtype);
        builder.write_pointer_list(&self.actions)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
/// All fields are useless EXCEPT for `code`.
pub struct Action {
    /// This is the only relevant field.
    pub code: Option<GMRef<GMCode>>,

    pub lib_id: u32,
    pub id: u32,
    pub kind: u32,
    pub use_relative: bool,
    pub use_apply_to: bool,
    pub exe_type: u32,
    pub action_name: Option<String>,
    pub argument_count: u32,
    pub who: i32,
    pub relative: bool,
    pub is_not: bool,
}

impl GMElement for Action {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let lib_id = reader.read_u32()?;
        let id = reader.read_u32()?;
        let kind = reader.read_u32()?;
        let use_relative = reader.read_bool32()?;
        let is_question = reader.read_bool32()?;
        reader.assert_bool(is_question, false, "Is Question")?;
        let use_apply_to = reader.read_bool32()?;
        let exe_type = reader.read_u32()?;
        let action_name: Option<String> = reader.read_gm_string_opt()?;
        let code: Option<GMRef<GMCode>> = reader.read_resource_by_id_opt()?;
        let argument_count = reader.read_u32()?;
        let who = reader.read_i32()?;
        let relative = reader.read_bool32()?;
        let is_not = reader.read_bool32()?;
        let unknown_always_zero = reader.read_u32()?;
        reader.assert_int(unknown_always_zero, 0, "Unknown always zero")?;

        Ok(Self {
            code,
            lib_id,
            id,
            kind,
            use_relative,
            use_apply_to,
            exe_type,
            action_name,
            argument_count,
            who,
            relative,
            is_not,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_u32(self.lib_id);
        builder.write_u32(self.id);
        builder.write_u32(self.kind);
        builder.write_bool32(self.use_relative);
        builder.write_bool32(false); // Is Question
        builder.write_bool32(self.use_apply_to);
        builder.write_u32(self.exe_type);
        builder.write_gm_string_opt(&self.action_name);
        builder.write_resource_id_opt(self.code);
        builder.write_u32(self.argument_count);
        builder.write_i32(self.who);
        builder.write_bool32(self.relative);
        builder.write_bool32(self.is_not);
        builder.write_u32(0); // UnknownAlwaysZero
        Ok(())
    }
}
