mod action;
pub mod subtype;

pub use self::action::Action;
use subtype::{Alarm, Draw, Gesture, Key, Mouse, Other, Step};

use macros::num_enum;

use crate::{
    prelude::*,
    util::{assert, fmt::typename},
    wad::{
        deserialize::reader::DataReader,
        elements::{GMElement, game_object::GMGameObject},
        serialize::builder::DataBuilder,
    },
};

/// How many event types existed in the oldest GameMaker version.
///
/// It is guaranteed that at least this many event types exist for any data file.
/// Modern data files may have more, up to [`EVENT_COUNT_MAX`].
///
/// (I guessed the count, decrease if issues arise)
pub const EVENT_COUNT_MIN: u32 = 12;

/// How many event types currently exist in GameMaker.
///
/// The actual event count may be lower for an older data file.
pub const EVENT_COUNT_MAX: u32 = 15;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Events {
    /// Triggered when the game object instance is created.
    pub create_events: Vec<SubEvent<()>>,

    /// Triggered when the game object instance is destroyed.
    pub destroy_events: Vec<SubEvent<()>>,

    /// Triggered when a user-set alarm reaches 0.
    ///
    /// An alarm event type which can be triggered modifying
    /// the builtin `alarm` variable (array) in other scripts.
    ///
    /// This does not have an associated enum; it is simply an alarm array index `0..12`.
    pub alarm_events: Vec<SubEvent<Alarm>>,

    /// Triggered on every game step (aka. frame).
    ///
    /// See [`EventSubtypeStep`].
    pub step_events: Vec<SubEvent<Step>>,

    /// Triggered when this game object instance collides with another game object (any instance).
    ///
    /// The subtype is the ID of the other game object (to check collision against).
    pub collision_events: Vec<SubEvent<GMRef<GMGameObject>>>,

    /// Triggered on every step/frame a specified key is held down.
    ///
    /// The key is specified in [`EventSubtypeKey`].
    pub keyboard_events: Vec<SubEvent<Key>>,

    /// Triggered on a mouse event (like holding, pressing down, releasing, mouse wheel, etc.).
    ///
    /// See [`EventSubtypeMouse`].
    pub mouse_events: Vec<SubEvent<Mouse>>,

    /// Some event that was too irrelevan to be included into the main list.
    /// Also includes user-defined events.
    ///
    /// See [`EventSubtypeOther`].
    pub other_events: Vec<SubEvent<Other>>,

    /// Triggered when the game loop is in the rendering/drawing stage.
    ///
    /// This occurs every step/frame, but is called with different
    /// timing and with a different purpose than [`Event::Step`].
    pub draw_events: Vec<SubEvent<Draw>>,

    /// Triggered on the first step/frame a specified key is pressed down.
    ///
    /// The key is specified in [`EventSubtypeKey`].
    pub key_press_events: Vec<SubEvent<Key>>,

    /// Triggered on the step/frame a specified key is released (no longer held down).
    ///
    /// The key is specified in [`EventSubtypeKey`].
    pub key_release_events: Vec<SubEvent<Key>>,

    /// A trigger event type. Only used in Pre-GameMaker Studio.
    pub trigger_events: Vec<SubEvent<()>>,

    /// Triggered when this game object instance is cleaned up, which can happen when:
    /// * The instance is destroyed
    /// * The room gets switched
    /// * The game ends
    pub cleanup_events: Vec<SubEvent<()>>,

    /// Triggered when the user performs some touchscreen event.
    ///
    /// See [`EventSubtypeGesture`].
    pub gesture_events: Vec<SubEvent<Gesture>>,

    /// A pre-create event type.
    /// TODO(doc): what is this? why does it exist? is it gm1 only?
    pub pre_create_events: Vec<SubEvent<()>>,
}

impl GMElement for Events {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let pointers: Vec<u32> = reader.read_simple_list()?;
        let count = pointers.len() as u32;
        if count < EVENT_COUNT_MIN {
            reader.warn_invalid_const(format!(
                "Expected a minimum of {EVENT_COUNT_MIN} event types; actually got {count}"
            ))?;
        }
        if count > EVENT_COUNT_MAX {
            reader.warn_invalid_const(format!(
                "Expected a maximum of {EVENT_COUNT_MAX} event types; actually got {count}"
            ))?;
        }

        // assert_pos are missing

        let create_events: Vec<SubEvent<()>> = reader.read_pointer_list()?;
        let destroy_events: Vec<SubEvent<()>> = reader.read_pointer_list()?;
        let alarm_events: Vec<SubEvent<Alarm>> = reader.read_pointer_list()?;
        let step_events: Vec<SubEvent<Step>> = reader.read_pointer_list()?;
        let collision_events: Vec<SubEvent<GMRef<GMGameObject>>> = reader.read_pointer_list()?;
        let keyboard_events: Vec<SubEvent<Key>> = reader.read_pointer_list()?;
        let mouse_events: Vec<SubEvent<Mouse>> = reader.read_pointer_list()?;
        let other_events: Vec<SubEvent<Other>> = reader.read_pointer_list()?;
        let draw_events: Vec<SubEvent<Draw>> = reader.read_pointer_list()?;
        let key_press_events: Vec<SubEvent<Key>> = reader.read_pointer_list()?;
        let key_release_events: Vec<SubEvent<Key>> = reader.read_pointer_list()?;
        let trigger_events: Vec<SubEvent<()>> = reader.read_pointer_list()?;
        let cleanup_events: Vec<SubEvent<()>> = reader.read_pointer_list()?;
        let gesture_events: Vec<SubEvent<Gesture>> = reader.read_pointer_list()?;
        let pre_create_events: Vec<SubEvent<()>> = reader.read_pointer_list()?;

        Ok(Self {
            create_events,
            destroy_events,
            alarm_events,
            step_events,
            collision_events,
            keyboard_events,
            mouse_events,
            other_events,
            draw_events,
            key_press_events,
            key_release_events,
            trigger_events,
            cleanup_events,
            gesture_events,
            pre_create_events,
        })
    }

    // ugly code, refactors are welcome
    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        // manual pointer list
        let count = EVENT_COUNT_MAX; // TODO: is this fine for older versions?

        builder.write_u32(count);
        let pointer_list_pos = builder.len();
        for _ in 0..count {
            builder.write_u32(0xDEAD_C0DE);
        }

        macro_rules! overwrite_pointer {
            ($offset:literal) => {
                builder.overwrite_usize(builder.len(), pointer_list_pos + 4 * $offset)?;
            };
        }

        overwrite_pointer!(0);
        builder.write_pointer_list(&self.create_events)?;

        overwrite_pointer!(1);
        builder.write_pointer_list(&self.destroy_events)?;

        overwrite_pointer!(2);
        builder.write_pointer_list(&self.alarm_events)?;

        overwrite_pointer!(3);
        builder.write_pointer_list(&self.step_events)?;

        overwrite_pointer!(4);
        builder.write_pointer_list(&self.collision_events)?;

        overwrite_pointer!(5);
        builder.write_pointer_list(&self.keyboard_events)?;

        overwrite_pointer!(6);
        builder.write_pointer_list(&self.mouse_events)?;

        overwrite_pointer!(7);
        builder.write_pointer_list(&self.other_events)?;

        overwrite_pointer!(8);
        builder.write_pointer_list(&self.draw_events)?;

        overwrite_pointer!(9);
        builder.write_pointer_list(&self.key_press_events)?;

        overwrite_pointer!(10);
        builder.write_pointer_list(&self.key_release_events)?;

        overwrite_pointer!(11);
        builder.write_pointer_list(&self.trigger_events)?;

        overwrite_pointer!(12);
        builder.write_pointer_list(&self.cleanup_events)?;

        overwrite_pointer!(13);
        builder.write_pointer_list(&self.gesture_events)?;

        overwrite_pointer!(14);
        builder.write_pointer_list(&self.pre_create_events)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(private_bounds)] // fhejhfjkhjkfhdskfhkjs
pub struct SubEvent<T: EventSubtype> {
    pub subtype: T,
    pub actions: Vec<Action>,
}

impl<T: EventSubtype> GMElement for SubEvent<T> {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let subtype: u32 = reader.read_u32()?;
        let subtype: T = T::parse(subtype)
            .with_context(|| format!("parsing Event subtype {}", typename::<T>()))?;
        let actions: Vec<Action> = reader.read_pointer_list()?;
        Ok(Self { subtype, actions })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        let subtype: u32 = self.subtype.build();
        builder.write_u32(subtype);
        builder.write_pointer_list(&self.actions)?;
        Ok(())
    }
}

trait EventSubtype: Copy {
    fn parse(subtype: u32) -> Result<Self>;
    fn build(self) -> u32;
}

impl EventSubtype for () {
    fn parse(subtype: u32) -> Result<Self> {
        // can't use `reader.options.verify_constants` here :c
        assert::int(subtype, 0, "Event Subtype for event with no data")?;
        Ok(())
    }

    fn build(self) -> u32 {
        0
    }
}

impl<T> EventSubtype for GMRef<T> {
    fn parse(subtype: u32) -> Result<Self> {
        Ok(Self::new(subtype))
    }

    fn build(self) -> u32 {
        self.index
    }
}

/// The type an [`Event`] can be.
///
/// Some event types have a subtype as well, which is denoted
/// by a number after in the code entry name.
///
/// All events with a subtype state their subtype enum in their docstring.
/// Events with no "real" subtype always have their subtype number set to zero.
///
/// For more information on events, see <https://manual.gamemaker.io/lts/en/The_Asset_Editors/Object_Properties/Object_Events.htm>.
#[num_enum(i32)]
pub enum EventType {
    /// Triggered when the game object instance is created.
    Create = 0,

    /// Triggered when the game object instance is destroyed.
    Destroy = 1,

    /// Triggered when a user-set alarm reaches 0.
    ///
    /// An alarm event type which can be triggered modifying
    /// the builtin `alarm` variable (array) in other scripts.
    ///
    /// This does not have an associated enum; it is simply an alarm array index `0..12`.
    Alarm = 2,

    /// Triggered on every game step (aka. frame).
    ///
    /// See [`EventSubtypeStep`].
    Step = 3,

    /// Triggered when this game object instance collides with another game object (any instance).
    ///
    /// The subtype is the ID of the other game object (to check collision against).
    Collision = 4,

    /// Triggered on every step/frame a specified key is held down.
    ///
    /// The key is specified in [`EventSubtypeKey`].
    Keyboard = 5,

    /// Triggered on a mouse event (like holding, pressing down, releasing, mouse wheel, etc.).
    ///
    /// See [`EventSubtypeMouse`].
    Mouse = 6,

    /// Some event that was too irrelevan to be included into the main list.
    /// Also includes user-defined events.
    ///
    /// See [`EventSubtypeOther`].
    Other = 7,

    /// Triggered when the game loop is in the rendering/drawing stage.
    ///
    /// This occurs every step/frame, but is called with different
    /// timing and with a different purpose than [`Event::Step`].
    Draw = 8,

    /// Triggered on the first step/frame a specified key is pressed down.
    ///
    /// The key is specified in [`EventSubtypeKey`].
    KeyPress = 9,

    /// Triggered on the step/frame a specified key is released (no longer held down).
    ///
    /// The key is specified in [`EventSubtypeKey`].
    KeyRelease = 10,

    /// A trigger event type. Only used in Pre-GameMaker Studio.
    Trigger = 11,

    /// Triggered when this game object instance is cleaned up, which can happen when:
    /// * The instance is destroyed
    /// * The room gets switched
    /// * The game ends
    CleanUp = 12,

    /// Triggered when the user performs some touchscreen event.
    ///
    /// See [`EventSubtypeGesture`].
    Gesture = 13,

    /// A pre-create event type.
    /// TODO(doc): what is this? why does it exist? is it gm1 only?
    PreCreate = 14,
}
