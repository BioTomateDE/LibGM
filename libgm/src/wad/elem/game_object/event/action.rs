// SPDX-License-Identifier: GPL-3.0-only
//! Actions that will be ran when an event is executed.
//!
//!
//! All these unused values seem to be provided for compatibility only.
//! In older versions of GM:S they stored the drag and drop blocks,
//! but newer versions compile them down to GML bytecode anyway.
//!
//! The layout of an action is as follows:
//! * (u32) Lib ID = 1
//! * (u32) ID = 603 | 601
//! * (u32) Kind = 7
//! * (bool) Use Relative = false
//! * (bool) Is Question = false
//! * (bool) Use Apply To = true
//! * (u32) Exe Type = 2
//! * (string) Action Name = "" | null
//! * (code ref) Code = <a GMCode ID>
//! * (u32) Argument Count = 1
//! * (i32) Who = -1
//! * (bool) Relative = false
//! * (bool) Is Not = false
//! * (u32) Unknown = 0
//!
//! All of these fields are useless, except for code.
//! The value after by the equals sign denotes the "usual" value
//! of this field. It may be a different value, depending on the
//! GameMaker Version and maybe some other stuff.
//! Either way, these fields seem to be completely ignored by the runner.
use std::fmt;

use crate::gml::GMCode;
use crate::prelude::*;
use crate::wad::GMRef;
use crate::wad::build::builder::DataBuilder;
use crate::wad::elem::GMElement;
use crate::wad::parse::reader::DataReader;

#[derive(Clone, PartialEq)]
pub struct Action {
    lib_id: u32,                 // usually 603, sometimes 601, sometimes other
    kind: u32,                   // usually 7
    use_relative: bool,          // usually false
    is_question: bool,           // usually false
    use_apply_to: bool,          // usually true
    exe_type: u32,               // usually 2
    action_name: Option<String>, // Some("") or None

    /// The code that will be executed when this action is ran.
    pub code: Option<GMRef<GMCode>>,

    argument_count: u32, // usually 1
    who: i32,            // usually -1
    is_not: bool,        // usually false
    unknown: u32,        // usually 0
}

impl Action {
    #[must_use]
    pub const fn new(code: GMRef<GMCode>) -> Self {
        Self {
            lib_id: 603,
            kind: 7,
            use_relative: false,
            is_question: false,
            use_apply_to: true,
            exe_type: 2,
            action_name: None,
            code: Some(code),
            argument_count: 1,
            who: -1,
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
        let kind = reader.read_u32()?;
        let use_relative = reader.read_bool32()?;
        let is_question = reader.read_bool32()?;
        let use_apply_to = reader.read_bool32()?;
        let exe_type = reader.read_u32()?;
        let action_name = reader.read_gm_string_opt()?;
        let code = reader.read_resource_by_id_opt()?;
        let argument_count = reader.read_u32()?;
        let who = reader.read_i32()?;
        let is_not = reader.read_bool32()?;
        let unknown = reader.read_u32()?;
        Ok(Self {
            lib_id,
            kind,
            use_relative,
            is_question,
            use_apply_to,
            exe_type,
            action_name,
            code,
            argument_count,
            who,
            is_not,
            unknown,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_u32(self.lib_id);
        builder.write_u32(self.kind);
        builder.write_bool32(self.use_relative);
        builder.write_bool32(self.is_question);
        builder.write_bool32(self.use_apply_to);
        builder.write_u32(self.exe_type);
        builder.write_gm_string_opt(&self.action_name);
        builder.write_resource_id_opt(self.code);
        builder.write_u32(self.argument_count);
        builder.write_i32(self.who);
        builder.write_bool32(self.is_not);
        builder.write_u32(self.unknown);
        Ok(())
    }
}
