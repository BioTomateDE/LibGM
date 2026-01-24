use std::fmt::{Display, Formatter};

use crate::{gamemaker::elements::game_object::GMGameObject, prelude::GMRef};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InstanceType {
    /// Represents the current `self` instance.
    Self_,

    /// Represents the first (?) instance of an object.
    /// This is typically an object that should only have one instance.
    GameObject(GMRef<GMGameObject>),

    /// Instance ID in the Room -100000; used when the Variable Type is [`VariableType::Instance`].
    /// This doesn't exist in UTMT.
    RoomInstance(i16),

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
