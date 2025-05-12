use crate::deserialize::all::GMData;
use crate::deserialize::paths::GMPathPoint;
use crate::serialize::all::DataBuilder;
use crate::serialize::chunk_writing::{ChunkBuilder, GMPointer};

pub fn build_chunk_path(data_builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    let mut builder = ChunkBuilder::new(data_builder, "PATH");

    let path_count: usize = gm_data.paths.paths_by_index.len();
    builder.write_usize(path_count);

    for i in 0..path_count {
        data_builder.push_pointer_placeholder(&mut builder, GMPointer::path(i))?;
    }

    for (i, path) in gm_data.paths.paths_by_index.iter().enumerate() {
        data_builder.push_pointer_resolve(&mut builder, GMPointer::path(i))?;
        builder.write_gm_string(data_builder, &path.name)?;
        builder.write_bool32(path.is_smooth);
        builder.write_bool32(path.is_closed);
        builder.write_u32(path.precision);
        build_path_points(&mut builder, &path.points);
    }


    builder.finish(data_builder)?;
    Ok(())
}

fn build_path_points(builder: &mut ChunkBuilder, points: &Vec<GMPathPoint>) {
    builder.write_usize(points.len());

    for point in points {
        builder.write_f32(point.x);
        builder.write_f32(point.y);
        builder.write_f32(point.speed);
    }
}

