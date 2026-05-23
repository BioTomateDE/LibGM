// SPDX-License-Identifier: GPL-3.0-only
// TODO: This code is unreadable garbage.
// If you want to waste your time, you can try to refactor this <3

pub mod action;
pub mod subtype;

pub use self::action::Action;
pub use self::action::ExeType;
pub use self::action::Kind;
pub use self::action::LibId;
pub use self::action::Who;
use self::subtype::Alarm;
use self::subtype::Collision;
use self::subtype::Draw;
use self::subtype::Gesture;
use self::subtype::Key;
use self::subtype::Mouse;
use self::subtype::Other;
use self::subtype::Step;
use crate::gml::GMCode;
use crate::prelude::*;
use crate::util::assert;
use crate::util::fmt::typename;
use crate::wad::build::builder::DataBuilder;
use crate::wad::elem::GMElement;
use crate::wad::parse::reader::DataReader;

/// Reference: <https://manual.gamemaker.io/lts/en/The_Asset_Editors/Object_Properties/Object_Events.htm>
#[derive(Debug, Clone, PartialEq)]
pub struct EventGroups {
    /// Triggered when the game object instance is created.
    pub create_handlers: Vec<Action>,

    /// Triggered when the game object instance is destroyed.
    pub destroy_handlers: Vec<Action>,

    /// Triggered when a user-set alarm reaches 0.
    ///
    /// An alarm event type which can be triggered modifying
    /// the builtin `alarm` variable (array) in other scripts.
    ///
    /// This is simply an alarm array index `0..12`.
    /// See [`Alarm`].
    pub alarm: EventGroup<Alarm>,

    /// Triggered on every game step (aka. frame).
    ///
    /// See [`Step`].
    pub step: EventGroup<Step>,

    /// Triggered when this game object instance collides
    /// with another game object (any instance).
    ///
    /// The subtype is the ID of the other game object (to check collision against).
    pub collision: EventGroup<Collision>,

    /// Triggered on every step/frame a specified key is held down.
    ///
    /// The key is specified in [`Key`].
    pub keyboard: EventGroup<Key>,

    /// Triggered on a mouse event (like holding, pressing down, releasing,
    /// mouse wheel, etc.).
    ///
    /// See [`Mouse`].
    pub mouse: EventGroup<Mouse>,

    /// Some event that was too irrelevan to be included into the main list.
    /// Also includes user-defined events.
    ///
    /// See [`Other`].
    pub other: EventGroup<Other>,

    /// Triggered when the game loop is in the rendering/drawing stage.
    ///
    /// This occurs every step/frame, but is called with different
    /// timing and with a different purpose than [`Step`] events.
    pub draw: EventGroup<Draw>,

    /// Triggered on the first step/frame a specified key is pressed down.
    ///
    /// The key is specified in [`Key`].
    pub key_press: EventGroup<Key>,

    /// Triggered on the step/frame a specified key is released (no longer held down).
    ///
    /// The key is specified in [`Key`].
    pub key_release: EventGroup<Key>,

    /// A trigger event type. Only used in Pre-GameMaker Studio.
    pub trigger_handlers: Vec<Action>,

    /// Triggered when this game object instance is cleaned up, which can happen
    /// when:
    /// * The instance is destroyed
    /// * The room gets switched
    /// * The game ends
    pub cleanup_handlers: Vec<Action>,

    /// Triggered when the user performs some touchscreen event.
    ///
    /// See [`Gesture`].
    pub gesture: EventGroup<Gesture>,

    /// A pre-create event type.
    /// TODO(doc): what is this? why does it exist? is it gms2 only?
    pub pre_create_handlers: Vec<Action>,
}

impl GMElement for EventGroups {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let pointers: Vec<u32> = reader
            .read_simple_list()
            .context("reading outer event pointer list")?;
        let count = pointers.len() as u32;

        let expected_count = type_count_by_wad(reader.general_info.wad_version);
        let ctx = || {
            format!(
                "validating event type count for game with GM Version {} and WAD Version {}",
                reader.general_info.version, reader.general_info.wad_version,
            )
        };
        reader
            .assert_int(count, expected_count, "event type count")
            .with_context(ctx)?;
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
            create_handlers: dedup_events_no_subtype(create),
            destroy_handlers: dedup_events_no_subtype(destroy),
            alarm: EventGroup::new(alarm),
            step: EventGroup::new(step),
            collision: EventGroup::new(collision),
            keyboard: EventGroup::new(keyboard),
            mouse: EventGroup::new(mouse),
            other: EventGroup::new(other),
            draw: EventGroup::new(draw),
            key_press: EventGroup::new(key_press),
            key_release: EventGroup::new(key_release),
            trigger_handlers: dedup_events_no_subtype(trigger),
            cleanup_handlers: dedup_events_no_subtype(cleanup),
            gesture: EventGroup::new(gesture),
            pre_create_handlers: dedup_events_no_subtype(pre_create),
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        // manual pointer list
        let count: u32 = type_count_by_wad(builder.wad_version());

        builder.write_u32(count);
        let pointer_list_pos = builder.len();
        for _ in 0..count {
            builder.write_u32(0xDEAD_C0DE);
        }

        // TODO: You have to make sure that there are no
        // empty [`SubEvent`]s, otherwise (old) runners segfault.
        let create = Event::unit(self.create_handlers.clone());
        let destroy = Event::unit(self.destroy_handlers.clone());
        let alarm = &self.alarm.events;
        let step = &self.step.events;
        let collision = &self.collision.events;
        let keyboard = &self.keyboard.events;
        let mouse = &self.mouse.events;
        let other = &self.other.events;
        let draw = &self.draw.events;
        let key_press = &self.key_press.events;
        let key_release = &self.key_release.events;
        let trigger = Event::unit(self.trigger_handlers.clone());

        builder.overwrite_pointer_with_cur_pos(pointer_list_pos, 0)?;
        builder.write_pointer_list(&create)?;

        builder.overwrite_pointer_with_cur_pos(pointer_list_pos, 1)?;
        builder.write_pointer_list(&destroy)?;

        builder.overwrite_pointer_with_cur_pos(pointer_list_pos, 2)?;
        builder.write_pointer_list(alarm)?;

        builder.overwrite_pointer_with_cur_pos(pointer_list_pos, 3)?;
        builder.write_pointer_list(step)?;

        builder.overwrite_pointer_with_cur_pos(pointer_list_pos, 4)?;
        builder.write_pointer_list(collision)?;

        builder.overwrite_pointer_with_cur_pos(pointer_list_pos, 5)?;
        builder.write_pointer_list(keyboard)?;

        builder.overwrite_pointer_with_cur_pos(pointer_list_pos, 6)?;
        builder.write_pointer_list(mouse)?;

        builder.overwrite_pointer_with_cur_pos(pointer_list_pos, 7)?;
        builder.write_pointer_list(other)?;

        builder.overwrite_pointer_with_cur_pos(pointer_list_pos, 8)?;
        builder.write_pointer_list(draw)?;

        builder.overwrite_pointer_with_cur_pos(pointer_list_pos, 9)?;
        builder.write_pointer_list(key_press)?;

        builder.overwrite_pointer_with_cur_pos(pointer_list_pos, 10)?;
        builder.write_pointer_list(key_release)?;

        builder.overwrite_pointer_with_cur_pos(pointer_list_pos, 11)?;
        builder.write_pointer_list(&trigger)?;

        if count > 12 {
            let cleanup = Event::unit(self.cleanup_handlers.clone());
            builder.overwrite_pointer_with_cur_pos(pointer_list_pos, 12)?;
            builder.write_pointer_list(&cleanup)?;
        }

        if count > 13 {
            let gesture = &self.gesture.events;
            builder.overwrite_pointer_with_cur_pos(pointer_list_pos, 13)?;
            builder.write_pointer_list(gesture)?;
        }
        if count > 14 {
            let pre_create = Event::unit(self.pre_create_handlers.clone());
            builder.overwrite_pointer_with_cur_pos(pointer_list_pos, 14)?;
            builder.write_pointer_list(&pre_create)?;
        }

        Ok(())
    }
}

#[must_use]
const fn type_count_by_wad(wad_version: u8) -> u32 {
    if wad_version < 16 {
        return 12;
    }
    if wad_version == 16 {
        return 13;
    }
    15
}

fn dedup_events<T: EventSubtype>(events: &mut Vec<Event<T>>) -> bool {
    let mut i: usize = 0;
    let mut changed: bool = false;

    while i < events.len() {
        let sub: T = events[i].subtype;
        // if there is another SubEvent with the same subtype, then merge it
        if let Some(j) = events[i + 1..].iter().position(|e| e.subtype == sub) {
            // TODO: Of all GM48 datafiles + UT/DR, this never happened.
            let dupe = events.remove(i + 1 + j);
            events[i].actions.extend(dupe.actions);
            changed = true;
        } else {
            i += 1;
        }
    }

    changed
}

#[must_use]
fn dedup_events_no_subtype(events: Vec<Event<()>>) -> Vec<Action> {
    // TODO: Usually, this does "nothing", as in, the events each have one action.
    // Sometimes (33 times in all events of all objects of all GM48 datafiles),
    // there is an event with no actions gets flattened.
    // These empty events were previously thought to be invalid and cause segfaults.
    // What's happening here? I've seen this happen in modern postlts, maybe it's
    // only a modern thing.
    events.into_iter().flat_map(|e| e.actions).collect()
}

/// Not meant to be implemented outside the crate.
pub trait EventSubtype: std::fmt::Debug + Copy + PartialEq {
    #[doc(hidden)]
    fn parse(subtype: u32) -> Result<Self>;

    #[doc(hidden)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct Event<T: EventSubtype> {
    pub subtype: T,
    pub actions: Vec<Action>,
}

impl<T: EventSubtype> Event<T> {
    /// Creates a new `SubEvent` with the given subtype and no actions.
    #[must_use]
    const fn new(subtype: T) -> Self {
        Self { subtype, actions: Vec::new() }
    }
}

impl Event<()> {
    #[must_use]
    fn unit(actions: Vec<Action>) -> Vec<Self> {
        if actions.is_empty() {
            Vec::new()
        } else {
            vec![Self { subtype: (), actions }]
        }
    }
}

impl<T: EventSubtype> GMElement for Event<T> {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let subtype: u32 = reader.read_u32()?;
        let subtype: T = T::parse(subtype)
            .with_context(|| format!("parsing Event subtype {}", typename::<T>()))?;
        let mut actions: Vec<Action> = reader.read_pointer_list()?;
        actions.retain(|x| x.__exists);
        Ok(Self { subtype, actions })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        let subtype: u32 = self.subtype.build();
        builder.write_u32(subtype);
        builder.write_pointer_list(&self.actions)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EventGroup<T: EventSubtype> {
    pub events: Vec<Event<T>>,
}

impl<T: EventSubtype> EventGroup<T> {
    #[must_use]
    fn new(mut events: Vec<Event<T>>) -> Self {
        dedup_events(&mut events);
        Self { events }
    }

    /// Gets all event handlers ([`Action`]s) for the given event subtype.
    ///
    /// This function will fail if there is no event handler for the given
    /// subtype. If you want to simplify the process by automatically
    /// creating a new empty handler if it does not exist, use
    /// [`EventGroup::handlers_for`] instead.
    pub fn get_handlers_for(&self, subtype: T) -> Result<&Vec<Action>> {
        if let Some(event) = self.events.iter().find(|e| e.subtype == subtype) {
            return Ok(&event.actions);
        }
        Err(err!(
            "Could not find any event handlers for subtype {subtype:?}"
        ))
    }

    /// Gets all event handlers ([`Action`]s) for the given event subtype.
    ///
    /// This automatically creates a new empty handler if it does not exist.
    /// If you do not want this behavior or cannot borrow this struct mutably,
    /// use [`EventGroup::get_handlers_for`] instead.
    #[must_use = "if you only want to make sure a handler exists, use `make_handler_for()`"]
    pub fn handlers_for(&mut self, subtype: T) -> &mut Vec<Action> {
        for (idx, event) in self.events.iter().enumerate() {
            if event.subtype == subtype {
                // Reborrow needed because of borrow checker incompetence
                let event = &mut self.events[idx];
                return &mut event.actions;
            }
        }

        // No event handler found for the given subtype; create a new one.
        let new = self.events.push_mut(Event::new(subtype));
        &mut new.actions
    }

    /// Ensures an event handler exists for the given subtype.
    ///
    /// This is a no-op if there is already a `SubEvent` with this subtype.
    /// Otherwise, new empty `SubEvent` will be pushed to the list.
    pub fn make_handler_for(
        &mut self,
        subtype: T,
        lib_id: LibId,
        kind: Kind,
        exe_type: ExeType,
        who: Who,
        code: GMRef<GMCode>,
    ) {
        let actions = self.handlers_for(subtype);
        actions.push(Action::new(lib_id, kind, exe_type, who, code));
    }

    /// An iterator that yields all actions of this event; no matter the
    /// subtype.
    pub fn all_actions(&self) -> impl Iterator<Item = &Action> {
        self.events.iter().flat_map(|e| &e.actions)
    }

    /// An iterator that yields all actions of this event; no matter the
    /// subtype.
    pub fn all_actions_mut(&mut self) -> impl Iterator<Item = &mut Action> {
        self.events.iter_mut().flat_map(|e| &mut e.actions)
    }

    pub fn iter(&self) -> core::slice::Iter<'_, Event<T>> {
        self.events.iter()
    }

    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, Event<T>> {
        self.events.iter_mut()
    }
}

impl<T: EventSubtype> IntoIterator for EventGroup<T> {
    type IntoIter = std::vec::IntoIter<Self::Item>;
    type Item = Event<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.events.into_iter()
    }
}

impl<'a, T: EventSubtype> IntoIterator for &'a EventGroup<T> {
    type IntoIter = core::slice::Iter<'a, Event<T>>;
    type Item = &'a Event<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.events.iter()
    }
}

impl<'a, T: EventSubtype> IntoIterator for &'a mut EventGroup<T> {
    type IntoIter = core::slice::IterMut<'a, Event<T>>;
    type Item = &'a mut Event<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.events.iter_mut()
    }
}
