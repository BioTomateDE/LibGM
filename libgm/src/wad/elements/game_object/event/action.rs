use std::fmt;

use crate::gml::GMCode;
use crate::prelude::*;
use crate::wad::GMRef;
use crate::wad::deserialize::reader::DataReader;
use crate::wad::elements::GMElement;
use crate::wad::serialize::builder::DataBuilder;

#[derive(Clone, PartialEq)]
pub struct Action {
    /// The code that will be executed when this action is ran.
    pub code: GMRef<GMCode>,

    // kind of a hack but idc
    pub(super) __exists: bool,
}

impl Action {
    #[must_use]
    pub const fn new(code: GMRef<GMCode>) -> Self {
        Self { code, __exists: true }
    }
}

impl GMElement for Action {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let lib_id = reader.read_u32()?;
        reader.assert_int(lib_id, 1, "Lib ID")?;

        let _id = reader.read_u32()?;
        // Usually 603, sometimes 601 and sometimes some other value (always 603 in
        // modern GM)

        let kind = reader.read_u32()?;
        reader.assert_int(kind, 7, "Kind")?;

        let use_relative = reader.read_bool32()?;
        reader.assert_bool(use_relative, false, "Use Relative")?;

        let is_question = reader.read_bool32()?;
        reader.assert_bool(is_question, false, "Is Question")?;

        let _use_apply_to = reader.read_bool32()?;
        // depends on gamemaker version

        let exe_type = reader.read_u32()?;
        reader.assert_int(exe_type, 2, "Exe Type")?;

        let _action_name: Option<String> = reader.read_gm_string_opt()?;
        // depends on gamemaker version (either None or "")

        let code: Option<GMRef<GMCode>> = reader.read_resource_by_id_opt()?;

        let _argument_count = reader.read_u32()?;
        // depends on gamemaker version (either 0 or 1)

        let who = reader.read_i32()?;
        reader.assert_int(who, -1, "Who")?;

        let relative = reader.read_bool32()?;
        reader.assert_bool(relative, false, "Relative")?;

        let is_not = reader.read_bool32()?;
        reader.assert_bool(is_not, false, "Is Not")?;

        let unknown_always_zero = reader.read_u32()?;
        reader.assert_int(unknown_always_zero, 0, "Unknown always zero")?;

        // this will be handled by `SubEvent::deserialize`
        Ok(Self {
            code: code.unwrap_or(GMRef::new(0)),
            __exists: code.is_some(),
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_u32(1); // Lib ID
        builder.write_u32(603); // ID
        builder.write_u32(7); // Kind
        builder.write_bool32(false); // Use Relative
        builder.write_bool32(false); // Is Question
        builder.write_bool32(false); // Use Apply To 
        builder.write_u32(2); // Exe Type
        builder.write_gm_string(""); // Name
        builder.write_resource_id(self.code);
        builder.write_u32(0); // Argument Count 
        builder.write_i32(-1); // Who
        builder.write_bool32(false); // Relative
        builder.write_bool32(false); // Is Not
        builder.write_u32(0); // UnknownAlwaysZero
        Ok(())
    }
}

impl fmt::Debug for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Action")
            .field("code", &self.code)
            .finish_non_exhaustive()
    }
}
