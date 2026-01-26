use std::fmt::{Display, Formatter};

use crate::{
    gamemaker::elements::game_object::GMGameObject, gml::instruction::VariableType, prelude::*,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum InstanceType {
    /// Represents the first (?) instance of an object.
    /// This is typically an object that should only have one instance.
    GameObject(GMRef<GMGameObject>),

    /// Instance ID in the Room -100000; used when the Variable Type is [`VariableType::Instance`].
    /// This doesn't exist in UTMT.
    RoomInstance(i16),

    /// Represents the current `self` instance.
    ///
    /// (should this be default?)
    #[default]
    Self_,

    /// Represents the `other` context, which has multiple definitions based on the location used.
    Other,

    /// Represents all active object instances.
    /// Assignment operations can perform a loop.
    All,

    /// Represents no object/instance.
    None,

    /// Used for global variables.
    Global,

    /// Used for GML built-in variables.
    Builtin,

    /// Used for local variables; local to their code script.
    Local,

    /// Instance is stored in a Variable data type on the top of the stack.
    StackTop,

    /// Used for function argument variables in GMS 2.3+.
    Argument,

    /// Used for static variables.
    Static,
}

impl Display for InstanceType {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::Self_ => write!(f, "Self"),
            Self::GameObject(reference) => {
                write!(f, "GameObject<{}>", reference.index)
            },
            Self::RoomInstance(instance_id) => {
                write!(f, "RoomInstanceID<{instance_id}>")
            },
            Self::Other => write!(f, "Other"),
            Self::All => write!(f, "All"),
            Self::None => write!(f, "None"),
            Self::Global => write!(f, "Global"),
            Self::Builtin => write!(f, "Builtin"),
            Self::Local => write!(f, "Local"),
            Self::StackTop => write!(f, "StackTop"),
            Self::Argument => write!(f, "Argument"),
            Self::Static => write!(f, "Static"),
        }
    }
}

impl InstanceType {
    /// Parse an instance type from the given raw value.
    /// The variable type is needed because [`VariableType::Instance`] signifies
    /// a [`InstanceType::RoomInstance`] instead of a [`InstanceType::GameObject`].
    pub fn parse(raw: i16, var_type: VariableType) -> Result<Self> {
        if raw > 0 {
            return Ok(if var_type == VariableType::Instance {
                Self::RoomInstance(raw)
            } else {
                Self::GameObject(GMRef::new(raw as u32))
            });
        }

        Ok(match raw {
            -1 => Self::Self_,
            -2 => Self::Other,
            -3 => Self::All,
            -4 => Self::None,
            -5 => Self::Global,
            -6 => Self::Builtin,
            -7 => Self::Local,
            -9 => Self::StackTop,
            -15 => Self::Argument,
            -16 => Self::Static,
            _ => bail!("Invalid instance type {raw} (0x{raw:04X})"),
        })
    }

    /// Parse an instance type from the given raw value,
    /// assuming that this is not a `RoomInstance` instance type.
    pub fn parse_normal(raw: i16) -> Result<Self> {
        Self::parse(raw, VariableType::Normal)
    }

    /// Serialize this instance type into an i16.
    ///
    /// If the game object reference is erroneously higher than [`i16::MAX`], you will be boiled.
    ///
    /// TODO: limit gmref to i16 prolly
    #[must_use]
    pub const fn build(self) -> i16 {
        match self {
            Self::GameObject(game_object_ref) => game_object_ref.index as i16,
            Self::RoomInstance(instance_id) => instance_id,
            Self::Self_ => -1,
            Self::Other => -2,
            Self::All => -3,
            Self::None => -4,
            Self::Global => -5,
            Self::Builtin => -6,
            Self::Local => -7,
            Self::StackTop => -9,
            Self::Argument => -15,
            Self::Static => -16,
        }
    }

    /// Convert an instance type to the "VARI version".
    /// In other words, convert the instance type to what
    /// it would be if it was in the 'VARI' chunk (`GMVariable.instance_type`)
    /// instead of in an instruction (`CodeVariable.instance_type`).
    #[must_use]
    pub const fn as_vari(self) -> Self {
        match self {
            Self::GameObject(_)
            | Self::RoomInstance(_)
            | Self::Other
            | Self::Builtin
            | Self::StackTop => Self::Self_,
            Self::Argument => Self::Builtin,
            _ => self,
        }
    }
}
