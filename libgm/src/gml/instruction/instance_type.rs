// SPDX-License-Identifier: GPL-3.0-only
use std::fmt;

use crate::gml::instruction::ReferenceType;
use crate::gml::instruction::VariableType;
use crate::prelude::*;
use crate::wad::elem::game_object::GameObject;
use crate::wad::elem::room::InstanceID;

/// The scope/owner of a variable.
///
/// Only used in WAD 15+.
///
/// The most notable ones (and less confusing ones) are:
/// * `self`: represents the current game object instance
/// * `global`: globally shared across entire game
/// * `local`: local to this code entry execution, destroyed afterward
/// * `builtin`: special GameMaker variables such as `x` or `image_speed` (self
///   scope)
///
/// These instance types are used in two ways:
/// The first is in a variable reference (such as in a push or pop instruction).
/// The second is in the declaration of all variables in the variable chunk
/// `VARI` (WAD 15+). In here, not all instance types are available, and they
/// may also be represented slightly differently: There are no references to
/// specific game objects or room instances. Also, certain things like  `other`,
/// `builtin` or `stacktop` are instead listed as `self`. You can convert a
/// variable reference instance type to a `VARI` one using
/// [`InstanceType::as_vari`].
///
/// In code, you can change your instance context by using a `with` loop.
/// For more information, see [`PushWithContext`].
///
/// [`PushWithContext`]: crate::gml::Instruction::PushWithContext
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum InstanceType {
    /// Represents the first (?) instance of an object.
    /// This is typically an object that should only have one instance.
    ///
    /// Note: Since instance types are encoded using only 16 bits,
    /// this may limit the number of game objects to 32767.
    GameObject(GMRef<GameObject>),

    /// Instance ID in the Room -100000; used when the
    /// Variable Type is [`ReferenceType::Instance`].
    ///
    /// Both this and [`InstanceType::GameObject`] are represented using
    /// positive integers. If the specified variable type was
    /// [`ReferenceType::Instance`], the integer gets interpreted as a Room
    /// Instance ID instead of a game object reference.
    Instance(InstanceID),

    /// Represents the current `self` instance.
    ///
    /// This corresponds to the game object instance in event action code.
    /// This `self` context is also kept when scripts (user created functions)
    /// are called.
    ///
    /// In room creation code and other contexts, I don't really know what this
    /// represents.
    ///
    /// (should this be default?)
    #[default]
    Self_,

    /// Represents the `other` context, which has multiple definitions based on
    /// the location used.
    ///
    /// This is commonly used in physics events, where `other` represents the
    /// "other" object which the "self" object collided with.
    Other,

    /// Represents all active object instances.
    ///
    /// Assignment operations can perform a loop.
    All,

    /// Represents no object/instance.
    NoOne,

    /// Used for global variables, which are shared globally across the entire game state.
    Global,

    /// Used for GML built-in variables such as `x`, `y`, `image_speed`, etc.
    Builtin,

    /// Used for local variables; local to their code script.
    ///
    /// These variables are destroyed/reset after the code execution ends.
    Local,

    /// Instance is stored in a Variable data type on the top of the stack.
    ///
    /// I don't know why this exists.
    ///
    /// Theoretically, it allows dynamic variable scope access.
    /// However, it is only over used in conjunction with [`PushImmediate`],
    /// so the instance is constant anyway.
    ///
    /// [`PushImmediate`]: crate::gml::Instruction::PushImmediate
    StackTop,

    /// Used for function argument variables in GMS 2.3+.
    Argument,

    /// Used for static variables.
    Static,
}

impl fmt::Debug for InstanceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Self_ => write!(f, "Self"),
            Self::GameObject(reference) => {
                write!(f, "GameObject<{}>", reference.index)
            }
            Self::Instance(instance_id) => {
                write!(f, "InstanceID<{}>", instance_id.0)
            }
            Self::Other => write!(f, "Other"),
            Self::All => write!(f, "All"),
            Self::NoOne => write!(f, "NoOne"),
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
    /// Decodes an instance type from the given raw value.
    /// The variable type is needed because [`ReferenceType::Instance`] signifies
    /// a [`InstanceType::Instance`] instead of a
    /// [`InstanceType::GameObject`].
    pub fn decode(raw: i16, var_type: ReferenceType) -> Result<Self> {
        if raw >= 0 {
            return Ok(if var_type == ReferenceType::Instance {
                Self::Instance(InstanceID(raw as i32))
            } else {
                Self::GameObject(GMRef::new(raw as i32))
            });
        }

        Ok(match raw {
            -1 => Self::Self_,
            -2 => Self::Other,
            -3 => Self::All,
            -4 => Self::NoOne,
            -5 => Self::Global,
            -6 => Self::Builtin,
            -7 => Self::Local,
            -9 => Self::StackTop,
            -15 => Self::Argument,
            -16 => Self::Static,
            _ => bail!("Invalid instance type {raw} (0x{raw:04X})"),
        })
    }

    /// Decodes an instance type from the given raw value,
    /// assuming that this is not a `Instance` instance type.
    pub fn decode_normal(raw: i16) -> Result<Self> {
        Self::decode(raw, ReferenceType::Normal)
    }

    /// Encodes this instance type into an [`i16`].
    ///
    /// If the game object reference is erroneously higher
    /// than [`i16::MAX`], you will be boiled (truncated).
    #[must_use]
    pub const fn encode(self, ref_type: ReferenceType) -> i16 {
        if !matches!(ref_type, ReferenceType::Normal | ReferenceType::Instance) {
            // yeah the instance type is lost with non-standard reference types.
            // why? idfk.
            return 0;
        }
        match self {
            Self::GameObject(game_object_ref) => game_object_ref.index as i16,
            Self::Instance(instance_id) => instance_id.0 as i16,
            Self::Self_ => -1,
            Self::Other => -2,
            Self::All => -3,
            Self::NoOne => -4,
            Self::Global => -5,
            Self::Builtin => -6,
            Self::Local => -7,
            Self::StackTop => -9,
            Self::Argument => -15,
            Self::Static => -16,
        }
    }

    /// Encodes this instance type into an [`i16`], assuming that the variable reference type is normal.
    #[inline]
    #[must_use]
    pub const fn encode_normal(self) -> i16 {
        self.encode(ReferenceType::Normal)
    }

    /// Converts an instance type to a variable type.
    ///
    /// Global, Local and Static stay unchanged.
    /// Argument becomes Builtin.
    /// Everything else becomes Self.
    #[must_use]
    pub const fn as_variable_type(self) -> VariableType {
        match self {
            Self::Global => VariableType::Global,
            Self::Local => VariableType::Local,
            Self::Static => VariableType::Static,
            Self::Argument => VariableType::Builtin,
            _ => VariableType::Self_,
        }
    }
}
