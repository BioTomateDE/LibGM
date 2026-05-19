use std::fmt;

use crate::gml::GMCode;
use crate::prelude::*;
use crate::wad::deserialize::reader::DataReader;
use crate::wad::elements::GMElement;
use crate::wad::serialize::builder::DataBuilder;
use crate::wad::GMRef;

#[derive(Clone, PartialEq)]
pub struct LibId(pub u32);

impl LibId {
    pub const NORMAL: Self = Self(1);
}

#[derive(Clone, PartialEq)]
pub struct Kind(pub u32);

impl Kind {
    pub const NORMAL: Self = Self(7);
}

#[derive(Clone, PartialEq)]
pub struct ExeType(pub u32);

impl ExeType {
    pub const NORMAL: Self = Self(2);
}

#[derive(Clone, PartialEq)]
pub struct Who(pub i32);

impl Who {
    pub const NORMAL: Self = Self(-1);
}

pub struct RandomActionConstantsIGuess;

impl RandomActionConstantsIGuess {
    pub const ARGUMENT_COUNT: u32 = 0;
    pub const ID: u32 = 603;
    pub const IS_NOT: bool = false;
    pub const IS_QUESTION: bool = false;
    pub const NAME: &str = "";
    pub const RELATIVE: bool = false;
    pub const UNKNOWN_ALWAYS_ZERO: u32 = 0;
    pub const USE_APPLY_TO: bool = false;
    pub const USE_RELATIVE: bool = false;
}

#[derive(Clone, PartialEq)]
pub struct Action {
    /// ???
    pub lib_id: LibId,

    /// ???
    pub kind: Kind,

    /// ???
    pub exe_type: ExeType,

    /// ???
    pub who: Who,

    /// The code that will be executed when this action is ran.
    pub code: GMRef<GMCode>,

    // kind of a hack but idc
    pub(super) __exists: bool,
}

impl Action {
    #[must_use]
    pub const fn new(
        lib_id: LibId,
        kind: Kind,
        exe_type: ExeType,
        who: Who,
        code: GMRef<GMCode>,
    ) -> Self {
        Self {
            lib_id,
            kind,
            exe_type,
            who,
            code,
            __exists: true,
        }
    }
}

impl GMElement for Action {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let lib_id = reader.read_u32()?;

        let _id = reader.read_u32()?;
        // Usually 603, sometimes 601 and sometimes some other value (always 603 in
        // modern GM)

        let kind = reader.read_u32()?;

        let use_relative = reader.read_bool32()?;
        reader.assert_bool(use_relative, false, "Use Relative")?;

        let is_question = reader.read_bool32()?;
        reader.assert_bool(is_question, false, "Is Question")?;

        let _use_apply_to = reader.read_bool32()?;
        // depends on gamemaker version

        let exe_type = reader.read_u32()?;

        let _action_name: Option<String> = reader.read_gm_string_opt()?;
        // depends on gamemaker version (either None or "")

        let code: Option<GMRef<GMCode>> = reader.read_resource_by_id_opt()?;

        let _argument_count = reader.read_u32()?;
        // depends on gamemaker version (either 0 or 1)

        let who = reader.read_i32()?;

        let relative = reader.read_bool32()?;
        reader.assert_bool(relative, false, "Relative")?;

        let is_not = reader.read_bool32()?;
        reader.assert_bool(is_not, false, "Is Not")?;

        let unknown_always_zero = reader.read_u32()?;
        reader.assert_int(unknown_always_zero, 0, "Unknown always zero")?;

        // this will be handled by `SubEvent::deserialize`
        Ok(Self {
            lib_id: LibId(lib_id),
            kind: Kind(kind),
            exe_type: ExeType(exe_type),
            who: Who(who),
            code: code.unwrap_or(GMRef::new(0)),
            __exists: code.is_some(),
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_u32(self.lib_id.0);
        builder.write_u32(RandomActionConstantsIGuess::ID);
        builder.write_u32(self.kind.0);
        builder.write_bool32(RandomActionConstantsIGuess::USE_RELATIVE);
        builder.write_bool32(RandomActionConstantsIGuess::IS_QUESTION);
        builder.write_bool32(RandomActionConstantsIGuess::USE_APPLY_TO);
        builder.write_u32(self.exe_type.0);
        builder.write_gm_string(RandomActionConstantsIGuess::NAME);
        builder.write_resource_id(self.code);
        builder.write_u32(RandomActionConstantsIGuess::ARGUMENT_COUNT);
        builder.write_i32(self.who.0);
        builder.write_bool32(RandomActionConstantsIGuess::RELATIVE);
        builder.write_bool32(RandomActionConstantsIGuess::IS_NOT);
        builder.write_u32(RandomActionConstantsIGuess::UNKNOWN_ALWAYS_ZERO);
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
