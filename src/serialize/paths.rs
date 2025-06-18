use crate::deserialize::all::GMData;
use crate::deserialize::paths::GMPathPoint;
use crate::serialize::chunk_writing::{DataBuilder, GMPointer};

pub fn build_chunk_path(builder: &mut DataBuilder, gm_data: &GMData) -> Result<(), String> {
    builder.start_chunk("PATH")?;

    let path_count: usize = gm_data.paths.paths.len();
    builder.write_usize(path_count);

    for i in 0..path_count {
        builder.write_placeholder(GMPointer::Path(i))?;
    }

    for (i, path) in gm_data.paths.paths.iter().enumerate() {
        builder.resolve_pointer(GMPointer::Path(i))?;
        builder.write_gm_string(&path.name)?;
        builder.write_bool32(path.is_smooth);
        builder.write_bool32(path.is_closed);
        builder.write_u32(path.precision);
        build_path_points(builder, &path.points);
    }


    builder.finish_chunk(&gm_data.general_info)?;
    Ok(())
}

fn build_path_points(builder: &mut DataBuilder, points: &Vec<GMPathPoint>) {
    builder.write_usize(points.len());

    for point in points {
        builder.write_f32(point.x);
        builder.write_f32(point.y);
        builder.write_f32(point.speed);
    }
}

