use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use crate::deserialize::chunk_reading::GMChunk;
use crate::deserialize::general_info::GMGeneralInfo;
use crate::deserialize::sprites::align_reader;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GMSpriteTypeSWF {
    pub swf_version: i32,
    pub yyswf_version: i32,
    pub jpeg_table: Vec<u8>,
    pub timeline: GMSpriteYYSWFTimeline,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GMSpriteYYSWFTimeline {
    pub framerate: i32,
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
    pub mask_width: i32,
    pub mask_height: i32,
    pub used_items: Vec<GMSpriteYYSWFItem>,
    pub frames: Vec<GMSpriteYYSWFTimelineFrame>,
    pub collision_masks: Vec<GMSpriteYYSWFCollisionMask>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GMSpriteYYSWFItem {
    pub id: i32,
    pub item_type: GMSpriteYYSWFItemType,
    pub shape_data: Option<GMSpriteYYSWFShapeData>,
    pub bitmap_data: Option<GMSpriteYYSWFBitmapData>,
}
#[derive(Debug, Clone, PartialEq, TryFromPrimitive, Serialize, Deserialize)]
#[repr(i32)]
pub enum GMSpriteYYSWFItemType {
    ItemInvalid,
    ItemShape,
    ItemBitmap,
    ItemFont,
    ItemTextField,
    ItemSprite,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GMSpriteYYSWFFillData {
    FillInvalid,
    FillSolid(GMSpriteYYSWFSolidFillData),
    FillGradient(GMSpriteYYSWFGradientFillData),
    FillBitmap(GMSpriteYYSWFBitmapFillData),
}
#[derive(Debug, Clone, PartialEq, TryFromPrimitive, Serialize, Deserialize)]
#[repr(i32)]
pub enum GMSpriteYYSWFBitmapFillType {
    FillRepeat,
    FillClamp,
    FillRepeatPoint,
    FillClampPoint,
}
#[derive(Debug, Clone, PartialEq, TryFromPrimitive, Serialize, Deserialize)]
#[repr(i32)]
pub enum GMSpriteYYSWFGradientFillType {
    FillLinear,
    FillRadial,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GMSpriteYYSWFShapeData {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
    pub style_groups: Vec<GMSpriteYYSWFStyleGroup>
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GMSpriteYYSWFStyleGroup {
    pub fill_styles: Vec<GMSpriteYYSWFFillData>,
    pub line_styles: Vec<GMSpriteYYSWFLineStyleData>,
    pub subshapes: Vec<GMSpriteYYSWFSubshapeData>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GMSpriteYYSWFBitmapFillData {
    pub bitmap_fill_type: GMSpriteYYSWFBitmapFillType,
    pub char_id: i32,
    transformation_matrix: GMSpriteYYSWFMatrix33,
}

pub static YYSWF_MATRIX33_MATRIX_SIZE: usize = 9;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GMSpriteYYSWFMatrix33 {
    pub values: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GMSpriteYYSWFGradientFillData {
    pub tpe_index: Option<usize>,
    pub gradient_fill_type: GMSpriteYYSWFGradientFillType,
    pub transformation_matrix: GMSpriteYYSWFMatrix33,
    pub records: Vec<GMSpriteYYSWFGradientRecord>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GMSpriteYYSWFGradientRecord {
    pub ratio: i32,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GMSpriteYYSWFSolidFillData {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GMSpriteYYSWFLineStyleData {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GMSpriteYYSWFSubshapeData {
    pub fill_style1: i32,
    pub fill_style2: i32,
    pub line_style: i32,
    pub points: Vec<(f32, f32)>,
    pub lines: Vec<(i32, i32)>,
    pub triangles: Vec<i32>,        // 'ObservableCollection' in UndertaleModTool
    pub aa_lines: Vec<(i32, i32)>,
    pub aa_vectors: Vec<(f32, f32)>,
    pub line_aa_lines: Vec<(i32, i32)>,
    pub line_aa_vectors: Vec<(f32, f32)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GMSpriteYYSWFBitmapData {
    bitmap_type: GMSpriteYYSWFBitmapType,
    width: u32,
    height: u32,
    tpe_index: Option<i32>,
    /// will be empty if tpe index is Some
    image_data: Vec<u8>,
    /// will be empty if tpe index is Some
    alpha_data: Vec<u8>,
    /// will be empty if tpe index is Some
    color_palette_data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, TryFromPrimitive, Serialize, Deserialize)]
#[repr(i32)]
pub enum GMSpriteYYSWFBitmapType {
    TypeJPEGNoHeader,
    TypeJPEG,
    TypeJPEGWithAlpha,
    TypePNG,
    TypeGIF,
    TypeLossless8bit,
    TypeLossless15bit,
    TypeLossless24bit,
    TypeLossless8bitA,
    TypeLossless32bit,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GMSpriteYYSWFTimelineFrame {
    pub frame_objects: Vec<GMSpriteYYSWFTimelineObject>,
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GMSpriteYYSWFTimelineObject {
    pub char_id: i32,
    pub char_index: i32,
    pub depth: i32,
    pub clipping_depth: i32,
    pub transformation_matrix: GMSpriteYYSWFMatrix33,
    pub color_matrix: GMSpriteYYSWFColorMatrix,
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
}

pub static YYSWF_COLOR_MATRIX_SIZE: usize = 4;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GMSpriteYYSWFColorMatrix {
    pub additive: Vec<i32>,
    pub multiply: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GMSpriteYYSWFCollisionMask {
    pub rle_data: Vec<u8>,
}




pub fn parse_yyswf_timeline(chunk: &mut GMChunk, general_info: &GMGeneralInfo) -> Result<GMSpriteYYSWFTimeline, String> {
    let used_items_count: usize = chunk.read_usize_count()?;
    let mut used_items: Vec<GMSpriteYYSWFItem> = Vec::with_capacity(used_items_count);
    for _ in 0..used_items_count {
        used_items.push(parse_yyswf_item(chunk, general_info)?);
    }

    let framerate: i32 = chunk.read_i32()?;
    let frames_count: usize = chunk.read_usize_count()?;
    let min_x: f32 = chunk.read_f32()?;
    let max_x: f32 = chunk.read_f32()?;
    let min_y: f32 = chunk.read_f32()?;
    let max_y: f32 = chunk.read_f32()?;
    let collision_masks_count: usize = chunk.read_usize_count()?;
    let mask_width: i32 = chunk.read_i32()?;
    let mask_height: i32 = chunk.read_i32()?;

    let mut frames: Vec<GMSpriteYYSWFTimelineFrame> = Vec::with_capacity(frames_count);
    for _ in 0..frames_count {
        frames.push(parse_yyswf_timeline_frame(chunk)?);
    }

    let mut collision_masks = Vec::with_capacity(collision_masks_count);
    for _ in 0..collision_masks_count {
        let rle_length: usize = chunk.read_usize_pos()?;      // could be -1 idk

        let rle_data: Vec<u8> = chunk.data.get(chunk.cur_pos.. chunk.cur_pos+rle_length).ok_or_else(|| format!(
            "Trying to read RLE Data of Timeline out of bounds while parsing \
            Sprite YYSWF at position {} in chunk '{}': {} > {}",
            chunk.name, chunk.cur_pos, chunk.cur_pos + rle_length, chunk.data.len(),
        ))?.to_vec();
        chunk.cur_pos += rle_length;
        align_reader(chunk, 4, 0x00)?;    // [From UndertaleModTool] "why it's not aligned before the data is beyond my brain"

        collision_masks.push(GMSpriteYYSWFCollisionMask {rle_data});
    }

    Ok(GMSpriteYYSWFTimeline {
        framerate,
        min_x,
        max_x,
        min_y,
        max_y,
        mask_width,
        mask_height,
        used_items,
        frames,
        collision_masks,
    })
}


fn parse_yyswf_item(chunk: &mut GMChunk, general_info: &GMGeneralInfo) -> Result<GMSpriteYYSWFItem, String> {
    let item_type: i32 = chunk.read_i32()?;
    let item_type: GMSpriteYYSWFItemType = item_type.try_into().map_err(|_| format!(
        "Invalid YYSWF Item Type 0x{:08X} at position {} while parsing Sprite YYSWF in chunk '{}'",
        item_type, chunk.cur_pos, chunk.name,
    ))?;
    let id: i32 = chunk.read_i32()?;
    let mut shape_data: Option<GMSpriteYYSWFShapeData> = None;
    let mut bitmap_data: Option<GMSpriteYYSWFBitmapData> = None;

    match item_type {
        GMSpriteYYSWFItemType::ItemShape => shape_data = Some(parse_yyswf_shape_data(chunk, general_info)?),
        GMSpriteYYSWFItemType::ItemBitmap => bitmap_data = Some(parse_yyswf_bitmap_data(chunk, general_info)?),
        GMSpriteYYSWFItemType::ItemFont | GMSpriteYYSWFItemType::ItemInvalid | GMSpriteYYSWFItemType::ItemTextField | GMSpriteYYSWFItemType::ItemSprite => {},
    }

    Ok(GMSpriteYYSWFItem {
        id,
        item_type,
        shape_data,
        bitmap_data,
    })
}


fn parse_yyswf_shape_data(chunk: &mut GMChunk, general_info: &GMGeneralInfo) -> Result<GMSpriteYYSWFShapeData, String> {
    let min_x: f32 = chunk.read_f32()?;
    let max_x: f32 = chunk.read_f32()?;
    let min_y: f32 = chunk.read_f32()?;
    let max_y: f32 = chunk.read_f32()?;

    let style_group_count: usize = chunk.read_usize_count()?;     // could be -1 maybe
    let mut style_groups: Vec<GMSpriteYYSWFStyleGroup> = Vec::with_capacity(style_group_count);
    for _ in 0..style_group_count {
        style_groups.push(parse_yyswf_style_group(chunk, general_info)?);
    }

    Ok(GMSpriteYYSWFShapeData {
        min_x,
        max_x,
        min_y,
        max_y,
        style_groups,
    })
}


fn parse_yyswf_style_group(chunk: &mut GMChunk, general_info: &GMGeneralInfo) -> Result<GMSpriteYYSWFStyleGroup, String> {
    let fill_data_count: usize = chunk.read_usize_count()?;       // could be -1 maybe
    let line_style_count: usize = chunk.read_usize_count()?;      // could be -1 maybe
    let subshape_count: usize = chunk.read_usize_count()?;        // could be -1 maybe

    let mut fill_styles: Vec<GMSpriteYYSWFFillData> = Vec::with_capacity(fill_data_count);
    for _ in 0..fill_data_count {
        fill_styles.push(parse_yyswf_fill_data(chunk, general_info)?);
    }

    let mut line_styles: Vec<GMSpriteYYSWFLineStyleData> = Vec::with_capacity(line_style_count);
    for _ in 0..line_style_count {
        let red: u8 = chunk.read_u8()?;
        let green: u8 = chunk.read_u8()?;
        let blue: u8 = chunk.read_u8()?;
        let alpha: u8 = chunk.read_u8()?;

        line_styles.push(GMSpriteYYSWFLineStyleData {
            red,
            green,
            blue,
            alpha,
        });
    }

    let mut subshapes: Vec<GMSpriteYYSWFSubshapeData> = Vec::with_capacity(subshape_count);
    for _ in 0..subshape_count {
        subshapes.push(parse_yyswf_subshapes(chunk)?);
    }

    Ok(GMSpriteYYSWFStyleGroup {
        fill_styles,
        line_styles,
        subshapes,
    })
}

fn parse_yyswf_fill_data(chunk: &mut GMChunk, general_info: &GMGeneralInfo) -> Result<GMSpriteYYSWFFillData, String> {
    let fill_type: i32 = chunk.read_i32()?;
    match fill_type {
        1 => {  // Solid Fill
            let red: u8 = chunk.read_u8()?;
            let green: u8 = chunk.read_u8()?;
            let blue: u8 = chunk.read_u8()?;
            let alpha: u8 = chunk.read_u8()?;
            Ok(GMSpriteYYSWFFillData::FillSolid(GMSpriteYYSWFSolidFillData {
                red,
                green,
                blue,
                alpha,
            }))
        },

        2 => {  // Gradient Fill
            let gradient_fill_type: i32 = chunk.read_i32()?;
            let gradient_fill_type: GMSpriteYYSWFGradientFillType = gradient_fill_type.try_into().map_err(|_|format!(
                "Invalid YYSWF Fill Gradient Type 0x{:08X} at position {} while parsing Sprite YYSWF in chunk '{}'",
                gradient_fill_type, chunk.cur_pos, chunk.name,
            ))?;

            let mut tpe_index: Option<usize> = None;
            if general_info.is_version_at_least(2022, 1, 0, 0) {
                tpe_index = Some(chunk.read_usize_count()?);      // maybe -1 idk
            }

            let transformation_matrix: GMSpriteYYSWFMatrix33 = parse_yyswf_transformation_matrix(chunk)?;

            let record_count: usize = chunk.read_usize_count()?;
            let mut records: Vec<GMSpriteYYSWFGradientRecord> = Vec::with_capacity(record_count);
            for _ in 0..record_count {
                let ratio: i32 = chunk.read_i32()?;
                let red: u8 = chunk.read_u8()?;
                let green: u8 = chunk.read_u8()?;
                let blue: u8 = chunk.read_u8()?;
                let alpha: u8 = chunk.read_u8()?;
                records.push(GMSpriteYYSWFGradientRecord {
                    ratio,
                    red,
                    green,
                    blue,
                    alpha,
                });
            }

            Ok(GMSpriteYYSWFFillData::FillGradient(GMSpriteYYSWFGradientFillData {
                tpe_index,
                gradient_fill_type,
                transformation_matrix,
                records,
            }))
        },

        3 => {      // Fill Bitmap
            let bitmap_fill_type: i32 = chunk.read_i32()?;
            let bitmap_fill_type: GMSpriteYYSWFBitmapFillType = bitmap_fill_type.try_into().map_err(|_| format!(
                "Invalid YYSWF Bitmap Fill Type 0x{:08X} at position {} while parsing Sprite YYSWF in chunk '{}'",
                bitmap_fill_type, chunk.cur_pos, chunk.name,
            ))?;
            let char_id: i32 = chunk.read_i32()?;
            let transformation_matrix: GMSpriteYYSWFMatrix33 = parse_yyswf_transformation_matrix(chunk)?;

            Ok(GMSpriteYYSWFFillData::FillBitmap(GMSpriteYYSWFBitmapFillData {
                bitmap_fill_type,
                char_id,
                transformation_matrix,
            }))
        },

        _ => Err(format!(
            "Invalid YYSWF Fill Type 0x{:08X} at position {} while parsing Sprite YYSWF in chunk '{}'",
            fill_type, chunk.cur_pos, chunk.name,
        )),
    }
}

fn parse_yyswf_transformation_matrix(chunk: &mut GMChunk) -> Result<GMSpriteYYSWFMatrix33, String> {
    let mut transformation_matrix_values: Vec<f32> = Vec::with_capacity(YYSWF_MATRIX33_MATRIX_SIZE);
    for _ in 0..YYSWF_MATRIX33_MATRIX_SIZE {
        transformation_matrix_values.push(chunk.read_f32()?);
    }
    Ok(GMSpriteYYSWFMatrix33 { values: transformation_matrix_values })
}


fn parse_yyswf_subshapes(chunk: &mut GMChunk) -> Result<GMSpriteYYSWFSubshapeData, String> {
    let fill_style1: i32 = chunk.read_i32()?;
    let fill_style2: i32 = chunk.read_i32()?;
    let line_style: i32 = chunk.read_i32()?;
    let point_count: usize = chunk.read_usize_count()?;
    let line_count: usize = chunk.read_usize_count()?;
    let triangle_count: usize = chunk.read_usize_count()? * 3;
    let aa_line_count: usize = chunk.read_usize_count()?;
    let aa_vector_count: usize = chunk.read_usize_count()? * 3;
    let line_aa_line_count: usize = chunk.read_usize_count()?;
    let line_aa_vector_count: usize = chunk.read_usize_count()?;

    let mut points: Vec<(f32, f32)> = Vec::with_capacity(point_count);
    let mut lines: Vec<(i32, i32)> = Vec::with_capacity(line_count);
    let mut triangles: Vec<i32> = Vec::with_capacity(triangle_count);
    let mut aa_lines: Vec<(i32, i32)> = Vec::with_capacity(aa_line_count);
    let mut aa_vectors: Vec<(f32, f32)> = Vec::with_capacity(aa_vector_count);
    let mut line_aa_lines: Vec<(i32, i32)> = Vec::with_capacity(line_aa_line_count);
    let mut line_aa_vectors: Vec<(f32, f32)> = Vec::with_capacity(line_aa_vector_count);

    for _ in 0..point_count {
        points.push((chunk.read_f32()?, chunk.read_f32()?));
    }
    for _ in 0..line_count {
        lines.push((chunk.read_i32()?, chunk.read_i32()?));
    }
    for _ in 0..triangle_count {
        triangles.push(chunk.read_i32()?);
    }
    for _ in 0..aa_line_count {
        aa_lines.push((chunk.read_i32()?, chunk.read_i32()?));
    }
    for _ in 0..aa_vector_count {
        aa_vectors.push((chunk.read_f32()?, chunk.read_f32()?));
    }
    for _ in 0..line_aa_line_count {
        line_aa_lines.push((chunk.read_i32()?, chunk.read_i32()?));
    }
    for _ in 0..line_aa_vector_count {
        line_aa_vectors.push((chunk.read_f32()?, chunk.read_f32()?));
    }

    Ok(GMSpriteYYSWFSubshapeData {
        fill_style1,
        fill_style2,
        line_style,
        points,
        lines,
        triangles,
        aa_lines,
        aa_vectors,
        line_aa_lines,
        line_aa_vectors,
    })
}


fn parse_yyswf_bitmap_data(chunk: &mut GMChunk, general_info: &GMGeneralInfo) -> Result<GMSpriteYYSWFBitmapData, String> {
    let bitmap_type: i32 = chunk.read_i32()?;
    let bitmap_type: GMSpriteYYSWFBitmapType = bitmap_type.try_into().map_err(|_| format!(
        "Invalid YYSWF Bitmap Type 0x{:08X} at position {} while parsing Sprite YYSWF in chunk '{}'",
        bitmap_type, chunk.cur_pos, chunk.name,
    ))?;

    let width: u32 = chunk.read_u32()?;             // could be -1 idk
    let height: u32 = chunk.read_u32()?;            // could be -1 idk
    let mut tpe_index: Option<i32> = None;
    let mut image_data: Vec<u8> = Vec::with_capacity(0);
    let mut alpha_data: Vec<u8> = Vec::with_capacity(0);
    let mut color_palette_data: Vec<u8> = Vec::with_capacity(0);


    if general_info.is_version_at_least(2022, 1, 0, 0) {
        tpe_index = Some(chunk.read_i32()?);
    } else {
        let image_data_length: usize = chunk.read_usize_pos()?;
        let alpha_data_length: usize = chunk.read_usize_pos()?;
        let color_palette_data_length: usize = chunk.read_usize_pos()?;

        image_data = chunk.data.get(chunk.cur_pos.. chunk.cur_pos+image_data_length).ok_or_else(|| format!(
            "Trying to read Image Data of Bitmap Data out of bounds while parsing \
            Sprite YYSWF at position {} in chunk '{}': {} > {}",
            chunk.name, chunk.cur_pos, chunk.cur_pos + image_data_length, chunk.data.len(),
        ))?.to_vec();
        chunk.cur_pos += image_data_length;

        alpha_data = chunk.data.get(chunk.cur_pos.. chunk.cur_pos+alpha_data_length).ok_or_else(|| format!(
            "Trying to read Alpha Data of Bitmap Data out of bounds while parsing \
            Sprite YYSWF at position {} in chunk '{}': {} > {}",
            chunk.name, chunk.cur_pos, chunk.cur_pos + alpha_data_length, chunk.data.len(),
        ))?.to_vec();
        chunk.cur_pos += alpha_data_length;

        color_palette_data = chunk.data.get(chunk.cur_pos.. chunk.cur_pos+color_palette_data_length).ok_or_else(|| format!(
            "Trying to read Color Palette Data of Bitmap Data out of bounds while parsing \
            Sprite YYSWF at position {} in chunk '{}': {} > {}",
            chunk.name, chunk.cur_pos, chunk.cur_pos + color_palette_data_length, chunk.data.len(),
        ))?.to_vec();
        chunk.cur_pos += color_palette_data_length;

        align_reader(chunk, 4, 0x00)?;

    }

    Ok(GMSpriteYYSWFBitmapData {
        bitmap_type,
        width,
        height,
        tpe_index,
        image_data,
        alpha_data,
        color_palette_data,
    })
}


fn parse_yyswf_timeline_frame(chunk: &mut GMChunk) -> Result<GMSpriteYYSWFTimelineFrame, String> {
    let frame_object_count: usize = chunk.read_usize_count()?;     // could be -1
    let min_x: f32 = chunk.read_f32()?;
    let max_x: f32 = chunk.read_f32()?;
    let min_y: f32 = chunk.read_f32()?;
    let max_y: f32 = chunk.read_f32()?;

    let mut frame_objects: Vec<GMSpriteYYSWFTimelineObject> = Vec::with_capacity(frame_object_count);
    for _ in 0..frame_object_count {
        let char_id: i32 = chunk.read_i32()?;
        let char_index: i32 = chunk.read_i32()?;
        let depth: i32 = chunk.read_i32()?;
        let clipping_depth: i32 = chunk.read_i32()?;
        let color_matrix: GMSpriteYYSWFColorMatrix = parse_yyswf_color_matrix(chunk)?;
        let min_x: f32 = chunk.read_f32()?;
        let max_x: f32 = chunk.read_f32()?;
        let min_y: f32 = chunk.read_f32()?;
        let max_y: f32 = chunk.read_f32()?;
        let transformation_matrix: GMSpriteYYSWFMatrix33 = parse_yyswf_transformation_matrix(chunk)?;

        frame_objects.push(GMSpriteYYSWFTimelineObject {
            char_id,
            char_index,
            depth,
            clipping_depth,
            transformation_matrix,
            color_matrix,
            min_x,
            max_x,
            min_y,
            max_y,
        })
    }

    Ok(GMSpriteYYSWFTimelineFrame {
        frame_objects,
        min_x,
        max_x,
        min_y,
        max_y,
    })
}


fn parse_yyswf_color_matrix(chunk: &mut GMChunk) -> Result<GMSpriteYYSWFColorMatrix, String> {
    let mut additive: Vec<i32> = Vec::with_capacity(YYSWF_COLOR_MATRIX_SIZE);
    let mut multiply: Vec<i32> = Vec::with_capacity(YYSWF_COLOR_MATRIX_SIZE);

    for _ in 0..YYSWF_COLOR_MATRIX_SIZE {
        additive.push(chunk.read_i32()?);
    }

    for _ in 0..YYSWF_COLOR_MATRIX_SIZE {
        multiply.push(chunk.read_i32()?);
    }

    Ok(GMSpriteYYSWFColorMatrix { additive, multiply })
}

