// SPDX-License-Identifier: GPL-3.0-only
//! Actions that will be ran when an event is executed.
//!
//! All of the (private) [`Action`] fields are useless, except for code.
//! These fields seem to be completely ignored by the runner
//! and seem to only be provided for compatibility with old GameMaker.
//! Comment from UndertaleModTool:
//! > In older versions of GM:S they stored the drag and drop blocks,
//! > but newer versions compile them down to GML bytecode anyway.

use std::fmt;

use crate::gml::GMCode;
use crate::prelude::*;
use crate::wad::GMRef;
use crate::wad::build::builder::DataBuilder;
use crate::wad::elem::GMElement;
use crate::wad::parse::reader::DataReader;

#[derive(Clone, PartialEq)]
pub struct Action {
    lib_id: u32,         // usually 1
    id: u32,             // usually 603, sometimes 601, sometimes other
    kind: u32,           // usually 7
    use_relative: bool,  // usually false
    is_question: bool,   // usually false
    use_apply_to: bool,  // usually true
    exe_type: u32,       // usually 2
    name: GMRef<String>, // Some("") or None

    /// The code that will be executed when this action is ran.
    pub code: GMRef<GMCode>,

    argument_count: u32, // usually 1
    who: i32,            // usually -1
    relative: bool,      // usually false
    is_not: bool,        // usually false
    unknown: u32,        // usually 0
}

impl Action {
    #[must_use]
    pub const fn new(code: GMRef<GMCode>) -> Self {
        Self {
            lib_id: 1,
            id: 603,
            kind: 7,
            use_relative: false,
            is_question: false,
            use_apply_to: true,
            exe_type: 2,
            name: GMRef::none(),
            code,
            argument_count: 1,
            who: -1,
            relative: false,
            is_not: false,
            unknown: 0,
        }
    }
}

impl fmt::Debug for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Action")
            .field("code", &self.code)
            .finish_non_exhaustive()
    }
}

impl GMElement for Action {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let lib_id = reader.read_u32()?;
        let id = reader.read_u32()?;
        let kind = reader.read_u32()?;
        let use_relative = reader.read_bool32()?;
        let is_question = reader.read_bool32()?;
        let use_apply_to = reader.read_bool32()?;
        let exe_type = reader.read_u32()?;
        let name = reader.read_gm_string()?;
        let code = reader.read_resource_by_id()?;
        let argument_count = reader.read_u32()?;
        let who = reader.read_i32()?;
        let relative = reader.read_bool32()?;
        let is_not = reader.read_bool32()?;
        let unknown = reader.read_u32()?;
        Ok(Self {
            lib_id,
            id,
            kind,
            use_relative,
            is_question,
            use_apply_to,
            exe_type,
            name,
            code,
            argument_count,
            who,
            relative,
            is_not,
            unknown,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_u32(self.lib_id);
        builder.write_u32(self.id);
        builder.write_u32(self.kind);
        builder.write_bool32(self.use_relative);
        builder.write_bool32(self.is_question);
        builder.write_bool32(self.use_apply_to);
        builder.write_u32(self.exe_type);
        builder.write_gm_string(self.name)?;
        builder.write_resource_id(self.code);
        builder.write_u32(self.argument_count);
        builder.write_i32(self.who);
        builder.write_bool32(self.relative);
        builder.write_bool32(self.is_not);
        builder.write_u32(self.unknown);
        Ok(())
    }
}
