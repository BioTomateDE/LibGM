// TODO: please nuke this entire file, i hate all of this code.

mod action;
pub mod subtype;

use subtype::Alarm;
use subtype::Collision;
use subtype::Draw;
use subtype::Gesture;
use subtype::Key;
use subtype::Mouse;
use subtype::Other;
use subtype::Step;

pub use self::action::Action;
use crate::gml::GMCode;
use crate::prelude::*;
use crate::util::assert;
use crate::util::fmt::typename;
use crate::wad::deserialize::reader::DataReader;
use crate::wad::elements::GMElement;
use crate::wad::serialize::builder::DataBuilder;

/// Reference: <https://manual.gamemaker.io/lts/en/The_Asset_Editors/Object_Properties/Object_Events.htm>
#[derive(Debug, Clone, PartialEq)]
pub struct Events {
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
    pub alarm: Event<Alarm>,

    /// Triggered on every game step (aka. frame).
    ///
    /// See [`Step`].
    pub step: Event<Step>,

    /// Triggered when this game object instance collides with another game
    /// object (any instance).
    ///
    /// The subtype is the ID of the other game object (to check collision
    /// against).
    pub collision: Event<Collision>,

    /// Triggered on every step/frame a specified key is held down.
    ///
    /// The key is specified in [`Key`].
    pub keyboard: Event<Key>,

    /// Triggered on a mouse event (like holding, pressing down, releasing,
    /// mouse wheel, etc.).
    ///
    /// See [`Mouse`].
    pub mouse: Event<Mouse>,

    /// Some event that was too irrelevan to be included into the main list.
    /// Also includes user-defined events.
    ///
    /// See [`Other`].
    pub other: Event<Other>,

    /// Triggered when the game loop is in the rendering/drawing stage.
    ///
    /// This occurs every step/frame, but is called with different
    /// timing and with a different purpose than [`Step`] events.
    pub draw: Event<Draw>,

    /// Triggered on the first step/frame a specified key is pressed down.
    ///
    /// The key is specified in [`Key`].
    pub key_press: Event<Key>,

    /// Triggered on the step/frame a specified key is released (no longer held
    /// down).
    ///
    /// The key is specified in [`Key`].
    pub key_release: Event<Key>,

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
    pub gesture: Event<Gesture>,

    /// A pre-create event type.
    /// TODO(doc): what is this? why does it exist? is it gms2 only?
    pub pre_create_handlers: Vec<Action>,
}

impl GMElement for Events {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let pointers: Vec<u32> = reader
            .read_simple_list()
            .context("reading outer event pointer list")?;
        let count = pointers.len() as u32;
        let gms2 = reader.general_info.is_version_at_least((2, 0));

        // TODO: fix this for undertale 1.01 (only 12)
        if gms2 && count != 15 {
            reader.handle_invalid_const(format!(
                "Expected 15 event types in GMS2; actually got {count}"
            ))?;
        }
        if !gms2 && count != 13 {
            reader.handle_invalid_const(format!(
                "Expected 13 event types in GMS1; actually got {count}"
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
            create_handlers: dedup_events_no_subtype(create),
            destroy_handlers: dedup_events_no_subtype(destroy),
            alarm: Event::new(alarm),
            step: Event::new(step),
            collision: Event::new(collision),
            keyboard: Event::new(keyboard),
            mouse: Event::new(mouse),
            other: Event::new(other),
            draw: Event::new(draw),
            key_press: Event::new(key_press),
            key_release: Event::new(key_release),
            trigger_handlers: dedup_events_no_subtype(trigger),
            cleanup_handlers: dedup_events_no_subtype(cleanup),
            gesture: Event::new(gesture),
            pre_create_handlers: dedup_events_no_subtype(pre_create),
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        // manual pointer list
        let gms2 = builder.is_version_at_least((2, 0));
        let count: u32 = if gms2 { 15 } else { 13 };

        builder.write_u32(count);
        let pointer_list_pos = builder.len();
        for _ in 0..count {
            builder.write_u32(0xDEAD_C0DE);
        }

        let create = vec![SubEvent::unit(self.create_handlers.clone())];
        let destroy = vec![SubEvent::unit(self.destroy_handlers.clone())];
        let alarm = &self.alarm.0;
        let step = &self.step.0;
        let collision = &self.collision.0;
        let keyboard = &self.keyboard.0;
        let mouse = &self.mouse.0;
        let other = &self.other.0;
        let draw = &self.draw.0;
        let key_press = &self.key_press.0;
        let key_release = &self.key_release.0;
        let trigger = vec![SubEvent::unit(self.trigger_handlers.clone())];
        let cleanup = vec![SubEvent::unit(self.cleanup_handlers.clone())];

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

        builder.overwrite_pointer_with_cur_pos(pointer_list_pos, 12)?;
        builder.write_pointer_list(&cleanup)?;

        if gms2 {
            let gesture = &self.gesture.0;
            let pre_create = vec![SubEvent::unit(self.pre_create_handlers.clone())];

            builder.overwrite_pointer_with_cur_pos(pointer_list_pos, 13)?;
            builder.write_pointer_list(gesture)?;

            builder.overwrite_pointer_with_cur_pos(pointer_list_pos, 14)?;
            builder.write_pointer_list(&pre_create)?;
        }

        Ok(())
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

#[must_use]
fn dedup_events_no_subtype(events: Vec<SubEvent<()>>) -> Vec<Action> {
    events.into_iter().flat_map(|e| e.actions).collect()
}

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

#[derive(Debug, Clone, PartialEq)]
struct SubEvent<T: EventSubtype> {
    pub subtype: T,
    pub actions: Vec<Action>,
}

impl<T: EventSubtype> SubEvent<T> {
    /// Creates a new `SubEvent` with the given subtype and no actions.
    #[must_use]
    const fn new(subtype: T) -> Self {
        Self { subtype, actions: Vec::new() }
    }
}

impl SubEvent<()> {
    #[must_use]
    const fn unit(actions: Vec<Action>) -> Self {
        Self { subtype: (), actions }
    }
}

impl<T: EventSubtype> GMElement for SubEvent<T> {
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
pub struct Event<T: EventSubtype>(Vec<SubEvent<T>>);

impl<T: EventSubtype> Event<T> {
    #[must_use]
    fn new(mut subevents: Vec<SubEvent<T>>) -> Self {
        dedup_events(&mut subevents);
        Self(subevents)
    }

    /// Gets all event handlers ([`Action`]s) for the given event subtype.
    ///
    /// This function will fail if there is no event handler for the given
    /// subtype. If you want to simplify the process by automatically
    /// creating a new empty handler if it does not exist, use
    /// [`Event::handlers_for`] instead.
    pub fn get_handlers_for(&self, subtype: T) -> Result<&Vec<Action>> {
        if let Some(sub_event) = self.0.iter().find(|e| e.subtype == subtype) {
            return Ok(&sub_event.actions);
        }
        Err(err!(
            "Could not find any event handlers for subtype {subtype:?}"
        ))
    }

    /// Gets all event handlers ([`Action`]s) for the given event subtype.
    ///
    /// This automatically creates a new empty handler if it does not exist.
    /// If you do not want this behavior or cannot borrow this struct mutably,
    /// use [`Event::get_handlers_for`] instead.
    #[must_use = "if you only want to make sure a handler exists, use `make_handler_for()`"]
    pub fn handlers_for(&mut self, subtype: T) -> &mut Vec<Action> {
        for (idx, event) in self.0.iter().enumerate() {
            if event.subtype == subtype {
                // Reborrow needed because of borrow checker incompetence
                let event = &mut self.0[idx];
                return &mut event.actions;
            }
        }

        // No event handler found for the given subtype; create a new one.
        let idx: usize = self.0.len();
        self.0.push(SubEvent::new(subtype));
        &mut self.0[idx].actions
    }

    /// Ensures an event handler exists for the given subtype.
    ///
    /// This is a no-op if there is already a `SubEvent` with this subtype.
    /// Otherwise, new empty `SubEvent` will be pushed to the list.
    pub fn make_handler_for(&mut self, subtype: T, code: GMRef<GMCode>) {
        let actions = self.handlers_for(subtype);
        actions.push(Action::new(code));
    }

    /// An iterator that yields all actions of this event; no matter the
    /// subtype.
    pub fn all_actions(&self) -> impl Iterator<Item = &Action> {
        self.0.iter().flat_map(|e| &e.actions)
    }

    /// An iterator that yields all actions of this event; no matter the
    /// subtype.
    pub fn all_actions_mut(&mut self) -> impl Iterator<Item = &mut Action> {
        self.0.iter_mut().flat_map(|e| &mut e.actions)
    }
}
