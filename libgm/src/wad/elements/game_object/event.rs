mod action;
pub mod subtype;

pub use self::action::Action;
use subtype::{Alarm, Collision, Draw, Gesture, Key, Mouse, Other, Step};

use crate::{
    prelude::*,
    util::{assert, fmt::typename},
    wad::{deserialize::reader::DataReader, elements::GMElement, serialize::builder::DataBuilder},
};

/// Reference: <https://manual.gamemaker.io/lts/en/The_Asset_Editors/Object_Properties/Object_Events.htm>
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
    /// This is simply an alarm array index `0..12`.
    /// See [`Alarm`].
    pub alarm_events: Vec<SubEvent<Alarm>>,

    /// Triggered on every game step (aka. frame).
    ///
    /// See [`Step`].
    pub step_events: Vec<SubEvent<Step>>,

    /// Triggered when this game object instance collides with another game object (any instance).
    ///
    /// The subtype is the ID of the other game object (to check collision against).
    pub collision_events: Vec<SubEvent<Collision>>,

    /// Triggered on every step/frame a specified key is held down.
    ///
    /// The key is specified in [`Key`].
    pub keyboard_events: Vec<SubEvent<Key>>,

    /// Triggered on a mouse event (like holding, pressing down, releasing, mouse wheel, etc.).
    ///
    /// See [`Mouse`].
    pub mouse_events: Vec<SubEvent<Mouse>>,

    /// Some event that was too irrelevan to be included into the main list.
    /// Also includes user-defined events.
    ///
    /// See [`Other`].
    pub other_events: Vec<SubEvent<Other>>,

    /// Triggered when the game loop is in the rendering/drawing stage.
    ///
    /// This occurs every step/frame, but is called with different
    /// timing and with a different purpose than [`Step`] events.
    pub draw_events: Vec<SubEvent<Draw>>,

    /// Triggered on the first step/frame a specified key is pressed down.
    ///
    /// The key is specified in [`Key`].
    pub key_press_events: Vec<SubEvent<Key>>,

    /// Triggered on the step/frame a specified key is released (no longer held down).
    ///
    /// The key is specified in [`Key`].
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
    /// See [`Gesture`].
    pub gesture_events: Vec<SubEvent<Gesture>>,

    /// A pre-create event type.
    /// TODO(doc): what is this? why does it exist? is it gm1 only?
    pub pre_create_events: Vec<SubEvent<()>>,
}

impl GMElement for Events {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let pointers: Vec<u32> = reader
            .read_simple_list()
            .context("reading outer event pointer list")?;
        let count = pointers.len() as u32;
        let gms2 = reader.general_info.is_version_at_least((2, 0));

        // i dont know if this is exactly correct
        if gms2 && count != 15 {
            reader.warn_invalid_const(format!(
                "Expected 15 event types in GMS2; actually got {count}"
            ))?;
        }
        if !gms2 && count != 13 {
            reader.warn_invalid_const(format!(
                "Expected 14 event types in GMS1; actually got {count}"
            ))?;
        }

        // assert_pos are missing

        let create_events: Vec<SubEvent<()>> = reader.read_pointer_list()?;
        let destroy_events: Vec<SubEvent<()>> = reader.read_pointer_list()?;
        let alarm_events: Vec<SubEvent<Alarm>> = reader.read_pointer_list()?;
        let step_events: Vec<SubEvent<Step>> = reader.read_pointer_list()?;
        let collision_events: Vec<SubEvent<Collision>> = reader.read_pointer_list()?;
        let keyboard_events: Vec<SubEvent<Key>> = reader.read_pointer_list()?;
        let mouse_events: Vec<SubEvent<Mouse>> = reader.read_pointer_list()?;
        let other_events: Vec<SubEvent<Other>> = reader.read_pointer_list()?;
        let draw_events: Vec<SubEvent<Draw>> = reader.read_pointer_list()?;
        let key_press_events: Vec<SubEvent<Key>> = reader.read_pointer_list()?;
        let key_release_events: Vec<SubEvent<Key>> = reader.read_pointer_list()?;
        let trigger_events: Vec<SubEvent<()>> = reader.read_pointer_list()?;
        let cleanup_events: Vec<SubEvent<()>> = reader.read_pointer_list()?;

        let gesture_events: Vec<SubEvent<Gesture>>;
        let pre_create_events: Vec<SubEvent<()>>;
        if gms2 {
            gesture_events = reader.read_pointer_list()?;
            pre_create_events = reader.read_pointer_list()?;
        } else {
            gesture_events = Vec::new();
            pre_create_events = Vec::new();
        }

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
        let gms2 = builder.is_version_at_least((2, 0));
        let count: u32 = if gms2 { 15 } else { 13 };

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

        if gms2 {
            overwrite_pointer!(13);
            builder.write_pointer_list(&self.gesture_events)?;

            overwrite_pointer!(14);
            builder.write_pointer_list(&self.pre_create_events)?;
        }
        Ok(())
    }
}

impl Events {
    /// Deduplicates [`SubEvent`]s with the same `subtype` by merging their actions.
    ///
    /// This function should ideally be called once, some time after deserializing
    /// but before getting event handlers using [`Self::key_hold_handler`] and friends.
    ///
    /// Returns `true` if some events were collapsed
    /// or `false` if there were no duplicate subtypes.
    ///
    /// ## Explanation
    /// For example, if there are two `SubEvent`s in `self.key_press_events`
    /// that are both `subtype: Key::LeftControl`, then the second `SubEvent`
    /// is removed and its actions are appended to the first subevent's actions.
    ///
    /// The same applies if there are even more subevents with the same subtype.
    ///
    /// Some event groups do not have a subtype (it is the unit type `()`).
    /// For those event groups, *all* subevents will be merged into one.
    pub fn collapse(&mut self) -> bool {
        let mut changed: bool = false;
        changed |= dedup_events(&mut self.create_events);
        changed |= dedup_events(&mut self.destroy_events);
        changed |= dedup_events(&mut self.alarm_events);
        changed |= dedup_events(&mut self.step_events);
        changed |= dedup_events(&mut self.collision_events);
        changed |= dedup_events(&mut self.keyboard_events);
        changed |= dedup_events(&mut self.mouse_events);
        changed |= dedup_events(&mut self.other_events);
        changed |= dedup_events(&mut self.draw_events);
        changed |= dedup_events(&mut self.key_press_events);
        changed |= dedup_events(&mut self.key_release_events);
        changed |= dedup_events(&mut self.trigger_events);
        changed |= dedup_events(&mut self.cleanup_events);
        changed |= dedup_events(&mut self.gesture_events);
        changed |= dedup_events(&mut self.pre_create_events);
        changed
    }
}

fn dedup_events<T: EventSubtype>(events: &mut Vec<SubEvent<T>>) -> bool {
    let mut i: usize = 0;
    let mut changed: bool = false;

    while i < events.len() {
        let sub: T = events[i].subtype;
        // if there is another SubEvent with the same subtype, then merge it
        if let Some(j) = events[i + 1..].iter().position(|e| e.subtype == sub) {
            let dupe = events.remove(i + 1 + j);
            events[i].actions.extend(dupe.actions);
            changed = true;
        } else {
            i += 1;
        }
    }

    changed
}

impl Events {
    pub fn create_handler(&mut self) -> &mut SubEvent<()> {
        handler(&mut self.create_events, ())
    }

    pub fn destroy_handler(&mut self) -> &mut SubEvent<()> {
        handler(&mut self.destroy_events, ())
    }

    pub fn alarm_handler(&mut self, alarm: Alarm) -> &mut SubEvent<Alarm> {
        handler(&mut self.alarm_events, alarm)
    }

    pub fn step_handler(&mut self, step: Step) -> &mut SubEvent<Step> {
        handler(&mut self.step_events, step)
    }

    pub fn collision_handler(&mut self, coll: impl Into<Collision>) -> &mut SubEvent<Collision> {
        handler(&mut self.collision_events, coll.into())
    }

    pub fn key_hold_handler(&mut self, key: Key) -> &mut SubEvent<Key> {
        handler(&mut self.keyboard_events, key)
    }

    pub fn mouse_handler(&mut self, mouse: Mouse) -> &mut SubEvent<Mouse> {
        handler(&mut self.mouse_events, mouse)
    }

    pub fn other_handler(&mut self, other: Other) -> &mut SubEvent<Other> {
        handler(&mut self.other_events, other)
    }

    pub fn draw_handler(&mut self, draw: Draw) -> &mut SubEvent<Draw> {
        handler(&mut self.draw_events, draw)
    }

    pub fn key_press_handler(&mut self, key: Key) -> &mut SubEvent<Key> {
        handler(&mut self.key_press_events, key)
    }

    pub fn key_release_handler(&mut self, key: Key) -> &mut SubEvent<Key> {
        handler(&mut self.key_release_events, key)
    }

    pub fn trigger_handler(&mut self) -> &mut SubEvent<()> {
        handler(&mut self.trigger_events, ())
    }

    pub fn cleanup_handler(&mut self) -> &mut SubEvent<()> {
        handler(&mut self.cleanup_events, ())
    }

    pub fn gesture_handler(&mut self, gesture: Gesture) -> &mut SubEvent<Gesture> {
        handler(&mut self.gesture_events, gesture)
    }

    pub fn pre_create_handler(&mut self) -> &mut SubEvent<()> {
        handler(&mut self.pre_create_events, ())
    }
}

fn handler<T: EventSubtype>(vector: &mut Vec<SubEvent<T>>, subtype: T) -> &mut SubEvent<T> {
    for (idx, event) in vector.iter().enumerate() {
        if event.subtype == subtype {
            // Reborrow needed because of borrow checker incompetence
            let event = &mut vector[idx];
            return event;
        }
    }

    // No event handler found for the given subtype; create a new one.
    vector.push(SubEvent::new(subtype));
    let last = vector.len() - 1; // also needed to satisfy borrow checker
    &mut vector[last]
}

#[derive(Debug, Clone, PartialEq)]
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

impl<T: EventSubtype> SubEvent<T> {
    /// Creates a new `SubEvent` with the given subtype and no actions.
    #[must_use]
    pub const fn new(subtype: T) -> Self {
        Self { subtype, actions: Vec::new() }
    }
}

/// Not meant to be implemented from outside the crate.
pub trait EventSubtype: Copy + Eq {
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
