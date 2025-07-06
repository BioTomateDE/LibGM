use crate::gm_deserialize::{GMChunkElement, GMElement, DataReader, GMRef};
use crate::gm_serialize::DataBuilder;
use crate::gamemaker::game_objects::GMGameObjectEvent;
use crate::utility::vec_with_capacity;


#[derive(Debug, Clone)]
pub struct GMTimelines {
	pub timelines: Vec<GMTimeline>,
	pub exists: bool,
}

impl GMChunkElement for GMTimelines {
	fn empty() -> Self {
		Self { timelines: vec![], exists: false }
	}
	fn exists(&self) -> bool {
		self.exists
	}
}

impl GMElement for GMTimelines {
	fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
		let timelines: Vec<GMTimeline> = reader.read_pointer_list()?;
		Ok(Self { timelines, exists: true })
	}

	fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
		if !self.exists { return Ok(()) }
		builder.write_pointer_list(&self.timelines)?;
		Ok(())
	}
}



#[derive(Debug, Clone)]
pub struct GMTimeline {
	pub name: GMRef<String>,
	pub moments: Vec<GMTimelineMoment>,
}
impl GMElement for GMTimeline {
	fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
		let name: GMRef<String> = reader.read_gm_string()?;
		let moment_count: usize = reader.read_usize()?;
		// TODO verify pointers {~~}
		let mut time_points: Vec<u32> = vec_with_capacity(moment_count)?;
		for _ in 0..moment_count {
			time_points.push(reader.read_u32()?);
			let _event_ptr = reader.read_usize()?;
		}
		let mut moments: Vec<GMTimelineMoment> = vec_with_capacity(moment_count)?;
		for time_point in time_points {
			let time_event = GMGameObjectEvent::deserialize(reader)?;
			moments.push(GMTimelineMoment { step: time_point, event: time_event });
		}
		Ok(Self { name, moments })
	}

	fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
		builder.write_gm_string(&self.name)?;
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


#[derive(Debug, Clone)]
pub struct GMTimelineMoment {
	pub step: u32,
	pub event: GMGameObjectEvent,
}

