// SPDX-License-Identifier: GPL-3.0-only

mod action;
pub mod subtype;

pub use self::action::Action;
use self::subtype::Alarm;
use self::subtype::Collision;
use self::subtype::Draw;
use self::subtype::Gesture;
use self::subtype::Key;
use self::subtype::Mouse;
use self::subtype::Other;
use self::subtype::Step;
use crate::prelude::*;
use crate::util::assert;
use crate::util::fmt::typename;
use crate::wad::build::builder::DataBuilder;
use crate::wad::elem::GMElement;
use crate::wad::elem::general_info::GeneralInfo;
use crate::wad::parse::reader::DataReader;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventGroup {
    Create,
    Destroy,
    Alarm,
    Step,
    Collision,
    Keyboard,
    Mouse,
    Other,
    Draw,
    KeyPress,
    KeyRelease,
    Trigger,
    Cleanup,
    Gesture,
    PreCreate,
}

/// Reference: <https://manual.gamemaker.io/lts/en/The_Asset_Editors/Object_Properties/Object_Events.htm>
#[derive(Debug, Clone, PartialEq)]
pub struct EventGroups {
    /// Triggered when the game object instance is created.
    pub create: Vec<Event<()>>,

    /// Triggered when the game object instance is destroyed.
    pub destroy: Vec<Event<()>>,

    /// Triggered when a user-set alarm reaches 0.
    ///
    /// An alarm event type which can be triggered modifying
    /// the builtin `alarm` variable (array) in other scripts.
    ///
    /// This is simply an alarm array index `0..12`.
    /// See [`Alarm`].
    pub alarm: Vec<Event<Alarm>>,

    /// Triggered on every game step (aka. frame).
    ///
    /// See [`Step`].
    pub step: Vec<Event<Step>>,

    /// Triggered when this game object instance collides
    /// with another game object (any instance).
    ///
    /// The subtype is the ID of the other game object (to check collision against).
    pub collision: Vec<Event<Collision>>,

    /// Triggered on every step/frame a specified key is held down.
    ///
    /// The key is specified in [`Key`].
    pub keyboard: Vec<Event<Key>>,

    /// Triggered on a mouse event (like holding, pressing down, releasing,
    /// mouse wheel, etc.).
    ///
    /// See [`Mouse`].
    pub mouse: Vec<Event<Mouse>>,

    /// Some event that was too irrelevan to be included into the main list.
    /// Also includes user-defined events.
    ///
    /// See [`Other`].
    pub other: Vec<Event<Other>>,

    /// Triggered when the game loop is in the rendering/drawing stage.
    ///
    /// This occurs every step/frame, but is called with different
    /// timing and with a different purpose than [`Step`] events.
    pub draw: Vec<Event<Draw>>,

    /// Triggered on the first step/frame a specified key is pressed down.
    ///
    /// The key is specified in [`Key`].
    pub key_press: Vec<Event<Key>>,

    /// Triggered on the step/frame a specified key is released (no longer held down).
    ///
    /// The key is specified in [`Key`].
    pub key_release: Vec<Event<Key>>,

    /// A trigger event type. Only used in Pre-GameMaker Studio.
    pub trigger: Vec<Event<()>>,

    /// Triggered when this game object instance is cleaned up, which can happen
    /// when:
    /// * The instance is destroyed
    /// * The room gets switched
    /// * The game ends
    pub cleanup: Vec<Event<()>>,

    /// Triggered when the user performs some touchscreen event.
    ///
    /// See [`Gesture`].
    pub gesture: Vec<Event<Gesture>>,

    /// A pre-create event type.
    /// DOCME: what is this? why does it exist? is it gms2 only?
    pub pre_create: Vec<Event<()>>,
}

impl GMElement for EventGroups {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let pointers: Vec<u32> = reader
            .read_simple_list()
            .ctx("reading outer event pointer list")?;
        let count = pointers.len() as u32;

        let expected_count = type_count_by_ver(&reader.general_info);
        let ctx = || {
            format!(
                "validating event type count for game with GM Version {} and WAD Version {}",
                reader.general_info.version, reader.general_info.wad_version,
            )
        };
        reader
            .assert_int(count, expected_count, "event type count")
            .ctx(ctx)?;
        // assert_pos are missing

        let create: Vec<Event<()>> = reader.read_pointer_list()?;
        let destroy: Vec<Event<()>> = reader.read_pointer_list()?;
        let alarm: Vec<Event<Alarm>> = reader.read_pointer_list()?;
        let step: Vec<Event<Step>> = reader.read_pointer_list()?;
        let collision: Vec<Event<Collision>> = reader.read_pointer_list()?;
        let keyboard: Vec<Event<Key>> = reader.read_pointer_list()?;
        let mouse: Vec<Event<Mouse>> = reader.read_pointer_list()?;
        let other: Vec<Event<Other>> = reader.read_pointer_list()?;
        let draw: Vec<Event<Draw>> = reader.read_pointer_list()?;
        let key_press: Vec<Event<Key>> = reader.read_pointer_list()?;
        let key_release: Vec<Event<Key>> = reader.read_pointer_list()?;
        let trigger: Vec<Event<()>> = reader.read_pointer_list()?;

        let mut cleanup: Vec<Event<()>> = Vec::new();
        let mut gesture: Vec<Event<Gesture>> = Vec::new();
        let mut pre_create: Vec<Event<()>> = Vec::new();
        if count > 12 {
            cleanup = reader.read_pointer_list()?;
        }
        if count > 13 {
            gesture = reader.read_pointer_list()?;
        }
        if count > 14 {
            pre_create = reader.read_pointer_list()?;
        }

        Ok(Self {
            create,
            destroy,
            alarm,
            step,
            collision,
            keyboard,
            mouse,
            other,
            draw,
            key_press,
            key_release,
            trigger,
            cleanup,
            gesture,
            pre_create,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        // manual pointer list
        let count: u32 = type_count_by_ver(&builder.gm_data.general_info);

        builder.write_u32(count);
        let pointer_list_pos = builder.pos();
        for _ in 0..count {
            builder.write_u32(0xDEAD_C0DE);
        }

        // TODO: You have to make sure that there are no
        // empty [`SubEvent`]s, otherwise (old) runners segfault.

        builder.overwrite_pointer_with_cur_pos(pointer_list_pos, 0)?;
        builder.write_pointer_list(&self.create)?;

        builder.overwrite_pointer_with_cur_pos(pointer_list_pos, 1)?;
        builder.write_pointer_list(&self.destroy)?;

        builder.overwrite_pointer_with_cur_pos(pointer_list_pos, 2)?;
        builder.write_pointer_list(&self.alarm)?;

        builder.overwrite_pointer_with_cur_pos(pointer_list_pos, 3)?;
        builder.write_pointer_list(&self.step)?;

        builder.overwrite_pointer_with_cur_pos(pointer_list_pos, 4)?;
        builder.write_pointer_list(&self.collision)?;

        builder.overwrite_pointer_with_cur_pos(pointer_list_pos, 5)?;
        builder.write_pointer_list(&self.keyboard)?;

        builder.overwrite_pointer_with_cur_pos(pointer_list_pos, 6)?;
        builder.write_pointer_list(&self.mouse)?;

        builder.overwrite_pointer_with_cur_pos(pointer_list_pos, 7)?;
        builder.write_pointer_list(&self.other)?;

        builder.overwrite_pointer_with_cur_pos(pointer_list_pos, 8)?;
        builder.write_pointer_list(&self.draw)?;

        builder.overwrite_pointer_with_cur_pos(pointer_list_pos, 9)?;
        builder.write_pointer_list(&self.key_press)?;

        builder.overwrite_pointer_with_cur_pos(pointer_list_pos, 10)?;
        builder.write_pointer_list(&self.key_release)?;

        builder.overwrite_pointer_with_cur_pos(pointer_list_pos, 11)?;
        builder.write_pointer_list(&self.trigger)?;

        if count > 12 {
            builder.overwrite_pointer_with_cur_pos(pointer_list_pos, 12)?;
            builder.write_pointer_list(&self.cleanup)?;
        }

        if count > 13 {
            builder.overwrite_pointer_with_cur_pos(pointer_list_pos, 13)?;
            builder.write_pointer_list(&self.gesture)?;
        }
        if count > 14 {
            builder.overwrite_pointer_with_cur_pos(pointer_list_pos, 14)?;
            builder.write_pointer_list(&self.pre_create)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Event<T: EventSubtype> {
    pub subtype: T,
    pub actions: Vec<Action>,
}

impl<T: EventSubtype> GMElement for Event<T> {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let subtype: i32 = reader.read_i32()?;
        let subtype: T =
            T::parse(subtype).ctx(|| format!("parsing Event subtype {}", typename::<T>()))?;
        let actions: Vec<Action> = reader.read_pointer_list()?;
        Ok(Self { subtype, actions })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        let subtype: i32 = self.subtype.build();
        builder.write_i32(subtype);
        builder.write_pointer_list(&self.actions)?;
        Ok(())
    }
}

/// Not meant to be implemented outside the crate.
pub trait EventSubtype: std::fmt::Debug + Copy + PartialEq {
    #[doc(hidden)]
    fn parse(subtype: i32) -> Result<Self>;

    #[doc(hidden)]
    fn build(self) -> i32;
}

impl EventSubtype for () {
    fn parse(subtype: i32) -> Result<Self> {
        // can't use `reader.options.verify_constants` here :c
        assert::int(subtype, 0, "Event Subtype for event with no data")?;
        Ok(())
    }

    fn build(self) -> i32 {
        0
    }
}

#[must_use]
fn type_count_by_ver(gen8: &GeneralInfo) -> u32 {
    if gen8.wad_version < 16 {
        return 12;
    }
    if gen8.wad_version == 16 && gen8.version < 2 {
        return 13;
    }
    15
}
