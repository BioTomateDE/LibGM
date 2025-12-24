use macros::named_list_chunk;

use crate::{
    gamemaker::{
        deserialize::reader::DataReader,
        elements::{GMElement, game_object},
        serialize::builder::DataBuilder,
    },
    prelude::*,
    util::init::vec_with_capacity,
};

#[named_list_chunk("TMLN")]
pub struct GMTimelines {
    pub timelines: Vec<GMTimeline>,
    pub exists: bool,
}

impl GMElement for GMTimelines {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let timelines: Vec<GMTimeline> = reader.read_pointer_list()?;
        Ok(Self { timelines, exists: true })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_pointer_list(&self.timelines)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMTimeline {
    pub name: String,
    pub moments: Vec<Moment>,
}

impl GMElement for GMTimeline {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: String = reader.read_gm_string()?;
        let moment_count = reader.read_u32()?;

        let mut time_points: Vec<u32> = vec_with_capacity(moment_count)?;
        let mut event_pointers: Vec<u32> = vec_with_capacity(moment_count)?;

        for _ in 0..moment_count {
            time_points.push(reader.read_u32()?);
            event_pointers.push(reader.read_u32()?);
        }

        let mut moments: Vec<Moment> = vec_with_capacity(moment_count)?;
        for (i, time_point) in time_points.into_iter().enumerate() {
            reader.assert_pos(event_pointers[i], "Timeline Event")?;
            let actions = reader.read_pointer_list()?;
            moments.push(Moment { time_point, actions });
        }

        Ok(Self { name, moments })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.name);
        builder.write_usize(self.moments.len())?;
        for moment in &self.moments {
            builder.write_u32(moment.time_point);
            builder.write_pointer(&moment.actions);
        }
        for moment in &self.moments {
            builder.resolve_pointer(&moment.actions)?;
            builder.write_pointer_list(&moment.actions)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Moment {
    /// After how many steps this moment gets executed.
    pub time_point: u32,

    /// The actions that get executed at this moment (aka. event).
    pub actions: Vec<game_object::event::Action>,
}
