mod action;
pub mod subtype;

use subtype::{Alarm, Collision, Draw, Gesture, Key, Mouse, Other, Step};

pub use self::action::Action;
use crate::{
    prelude::*,
    util::{assert, fmt::typename},
    wad::{deserialize::reader::DataReader, elements::GMElement, serialize::builder::DataBuilder},
};

/// Reference: <https://manual.gamemaker.io/lts/en/The_Asset_Editors/Object_Properties/Object_Events.htm>
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Events {
    /// Triggered when the game object instance is created.
    pub create: Vec<SubEvent<()>>,

    /// Triggered when the game object instance is destroyed.
    pub destroy: Vec<SubEvent<()>>,

    /// Triggered when a user-set alarm reaches 0.
    ///
    /// An alarm event type which can be triggered modifying
    /// the builtin `alarm` variable (array) in other scripts.
    ///
    /// This is simply an alarm array index `0..12`.
    /// See [`Alarm`].
    pub alarm: Vec<SubEvent<Alarm>>,

    /// Triggered on every game step (aka. frame).
    ///
    /// See [`Step`].
    pub step: Vec<SubEvent<Step>>,

    /// Triggered when this game object instance collides with another game object (any instance).
    ///
    /// The subtype is the ID of the other game object (to check collision against).
    pub collision: Vec<SubEvent<Collision>>,

    /// Triggered on every step/frame a specified key is held down.
    ///
    /// The key is specified in [`Key`].
    pub keyboard: Vec<SubEvent<Key>>,

    /// Triggered on a mouse event (like holding, pressing down, releasing, mouse wheel, etc.).
    ///
    /// See [`Mouse`].
    pub mouse: Vec<SubEvent<Mouse>>,

    /// Some event that was too irrelevan to be included into the main list.
    /// Also includes user-defined events.
    ///
    /// See [`Other`].
    pub other: Vec<SubEvent<Other>>,

    /// Triggered when the game loop is in the rendering/drawing stage.
    ///
    /// This occurs every step/frame, but is called with different
    /// timing and with a different purpose than [`Step`] events.
    pub draw: Vec<SubEvent<Draw>>,

    /// Triggered on the first step/frame a specified key is pressed down.
    ///
    /// The key is specified in [`Key`].
    pub key_press: Vec<SubEvent<Key>>,

    /// Triggered on the step/frame a specified key is released (no longer held down).
    ///
    /// The key is specified in [`Key`].
    pub key_release: Vec<SubEvent<Key>>,

    /// A trigger event type. Only used in Pre-GameMaker Studio.
    pub trigger: Vec<SubEvent<()>>,

    /// Triggered when this game object instance is cleaned up, which can happen when:
    /// * The instance is destroyed
    /// * The room gets switched
    /// * The game ends
    pub cleanup: Vec<SubEvent<()>>,

    /// Triggered when the user performs some touchscreen event.
    ///
    /// See [`Gesture`].
    pub gesture: Vec<SubEvent<Gesture>>,

    /// A pre-create event type.
    /// TODO(doc): what is this? why does it exist? is it gms2 only?
    pub pre_create: Vec<SubEvent<()>>,
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

        let create: Vec<SubEvent<()>> = reader.read_pointer_list()?;
        let destroy: Vec<SubEvent<()>> = reader.read_pointer_list()?;
        let alarm: Vec<SubEvent<Alarm>> = reader.read_pointer_list()?;
        let step: Vec<SubEvent<Step>> = reader.read_pointer_list()?;
        let collision: Vec<SubEvent<Collision>> = reader.read_pointer_list()?;
        let keyboard: Vec<SubEvent<Key>> = reader.read_pointer_list()?;
        let mouse: Vec<SubEvent<Mouse>> = reader.read_pointer_list()?;
        let other: Vec<SubEvent<Other>> = reader.read_pointer_list()?;
        let draw: Vec<SubEvent<Draw>> = reader.read_pointer_list()?;
        let key_press: Vec<SubEvent<Key>> = reader.read_pointer_list()?;
        let key_release: Vec<SubEvent<Key>> = reader.read_pointer_list()?;
        let trigger: Vec<SubEvent<()>> = reader.read_pointer_list()?;
        let cleanup: Vec<SubEvent<()>> = reader.read_pointer_list()?;

        let gesture: Vec<SubEvent<Gesture>>;
        let pre_create: Vec<SubEvent<()>>;
        if gms2 {
            gesture = reader.read_pointer_list()?;
            pre_create = reader.read_pointer_list()?;
        } else {
            gesture = Vec::new();
            pre_create = Vec::new();
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
        builder.write_pointer_list(&self.key_release)?;

        builder.overwrite_pointer_with_cur_pos(pointer_list_pos, 12)?;
        builder.write_pointer_list(&self.cleanup)?;

        if gms2 {
            builder.overwrite_pointer_with_cur_pos(pointer_list_pos, 13)?;
            builder.write_pointer_list(&self.gesture)?;

            builder.overwrite_pointer_with_cur_pos(pointer_list_pos, 14)?;
            builder.write_pointer_list(&self.pre_create)?;
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
        changed |= dedup_events(&mut self.create);
        changed |= dedup_events(&mut self.destroy);
        changed |= dedup_events(&mut self.alarm);
        changed |= dedup_events(&mut self.step);
        changed |= dedup_events(&mut self.collision);
        changed |= dedup_events(&mut self.keyboard);
        changed |= dedup_events(&mut self.mouse);
        changed |= dedup_events(&mut self.other);
        changed |= dedup_events(&mut self.draw);
        changed |= dedup_events(&mut self.key_press);
        changed |= dedup_events(&mut self.key_release);
        changed |= dedup_events(&mut self.key_release);
        changed |= dedup_events(&mut self.cleanup);
        changed |= dedup_events(&mut self.gesture);
        changed |= dedup_events(&mut self.pre_create);
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
        handler(&mut self.create, ())
    }

    pub fn destroy_handler(&mut self) -> &mut SubEvent<()> {
        handler(&mut self.destroy, ())
    }

    pub fn alarm_handler(&mut self, alarm: Alarm) -> &mut SubEvent<Alarm> {
        handler(&mut self.alarm, alarm)
    }

    pub fn step_handler(&mut self, step: Step) -> &mut SubEvent<Step> {
        handler(&mut self.step, step)
    }

    pub fn collision_handler(&mut self, coll: impl Into<Collision>) -> &mut SubEvent<Collision> {
        handler(&mut self.collision, coll.into())
    }

    pub fn key_hold_handler(&mut self, key: Key) -> &mut SubEvent<Key> {
        handler(&mut self.keyboard, key)
    }

    pub fn mouse_handler(&mut self, mouse: Mouse) -> &mut SubEvent<Mouse> {
        handler(&mut self.mouse, mouse)
    }

    pub fn other_handler(&mut self, other: Other) -> &mut SubEvent<Other> {
        handler(&mut self.other, other)
    }

    pub fn draw_handler(&mut self, draw: Draw) -> &mut SubEvent<Draw> {
        handler(&mut self.draw, draw)
    }

    pub fn key_press_handler(&mut self, key: Key) -> &mut SubEvent<Key> {
        handler(&mut self.key_press, key)
    }

    pub fn key_release_handler(&mut self, key: Key) -> &mut SubEvent<Key> {
        handler(&mut self.key_release, key)
    }

    pub fn trigger_handler(&mut self) -> &mut SubEvent<()> {
        handler(&mut self.trigger, ())
    }

    pub fn cleanup_handler(&mut self) -> &mut SubEvent<()> {
        handler(&mut self.cleanup, ())
    }

    pub fn gesture_handler(&mut self, gesture: Gesture) -> &mut SubEvent<Gesture> {
        handler(&mut self.gesture, gesture)
    }

    pub fn pre_create_handler(&mut self) -> &mut SubEvent<()> {
        handler(&mut self.pre_create, ())
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
pub trait EventSubtype: std::fmt::Debug + Copy + PartialEq {
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
