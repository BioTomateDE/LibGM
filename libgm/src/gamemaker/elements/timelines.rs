use crate::gamemaker::deserialize::reader::DataReader;
use crate::gamemaker::deserialize::resources::GMRef;
use crate::gamemaker::elements::game_objects::GMGameObjectEvent;
use crate::gamemaker::elements::{GMChunkElement, GMElement};
use crate::gamemaker::serialize::builder::DataBuilder;
use crate::prelude::*;
use crate::util::init::vec_with_capacity;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Default)]
pub struct GMTimelines {
    pub timelines: Vec<GMTimeline>,
    pub exists: bool,
}

impl Deref for GMTimelines {
    type Target = Vec<GMTimeline>;
    fn deref(&self) -> &Self::Target {
        &self.timelines
    }
}

impl DerefMut for GMTimelines {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.timelines
    }
}

impl GMChunkElement for GMTimelines {
    const NAME: &'static str = "TMLN";
    fn exists(&self) -> bool {
        self.exists
    }
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
    pub moments: Vec<GMTimelineMoment>,
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

        let mut moments: Vec<GMTimelineMoment> = vec_with_capacity(moment_count)?;
        for (i, time_point) in time_points.into_iter().enumerate() {
            reader.assert_pos(event_pointers[i], "Timeline Event")?;
            let time_event = GMGameObjectEvent::deserialize(reader)?;
            moments.push(GMTimelineMoment { step: time_point, event: time_event });
        }

        Ok(Self { name, moments })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_gm_string(&self.name);
        builder.write_usize(self.moments.len())?;
        for moment in &self.moments {
            builder.write_u32(moment.step);
            builder.write_pointer(&moment.event)?;
        }
        for moment in &self.moments {
            builder.resolve_pointer(&moment.event)?;
            moment.event.serialize(builder)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMTimelineMoment {
    pub step: u32,
    pub event: GMGameObjectEvent,
}
