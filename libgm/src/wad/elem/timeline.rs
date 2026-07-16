// SPDX-License-Identifier: GPL-3.0-only

use crate::prelude::*;
use crate::util::init::vec_with_capacity;
use crate::wad::build::builder::DataBuilder;
use crate::wad::chunk::gm_named_list_chunk;
use crate::wad::elem::GMElement;
use crate::wad::elem::game_object;
use crate::wad::parse::reader::DataReader;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Timelines {
    pub elems: Vec<Option<Timeline>>,
}

// probably nullable?
gm_named_list_chunk!(TMLN, Timelines, Timeline, nullable);

impl GMElement for Timelines {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let elems: Vec<Option<Timeline>> = reader.read_pointer_list_opt()?;
        Ok(Self { elems })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<()> {
        builder.write_pointer_list_opt(&self.elems)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Timeline {
    pub name: GMRef<String>,
    pub moments: Vec<Moment>,
}

impl GMElement for Timeline {
    fn deserialize(reader: &mut DataReader) -> Result<Self> {
        let name: GMRef<String> = reader.read_gm_string()?;
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
        builder.write_gm_string(self.name)?;
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
