use num_enum::TryFromPrimitive;
use crate::deserialize::chunk_reading::UTChunk;
use crate::deserialize::general_info::UTGeneralInfo;
use crate::deserialize::sprites::align_reader;

#[derive(Debug, Clone)]
pub struct UTSpriteYYSWF {
    pub version: i32,
    pub jpeg_table: Vec<u8>,
    pub timeline: UTSpriteYYSWFTimeline,
}
#[derive(Debug, Clone)]
pub struct UTSpriteYYSWFTimeline {
    pub framerate: i32,
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
    pub mask_width: i32,
    pub mask_height: i32,
    pub used_items: Vec<UTSpriteYYSWFItem>,
    pub frames: Vec<UTSpriteYYSWFTimelineFrame>,
    pub collision_masks: Vec<UTSpriteYYSWFCollisionMask>,
}
#[derive(Debug, Clone)]
pub struct UTSpriteYYSWFItem {
    pub id: i32,
    pub item_type: UTSpriteYYSWFItemType,
    pub shape_data: Option<UTSpriteYYSWFShapeData>,
    pub bitmap_data: Option<UTSpriteYYSWFBitmapData>,
}
#[derive(Debug, Clone, TryFromPrimitive)]
#[repr(i32)]
pub enum UTSpriteYYSWFItemType {
    ItemInvalid,
    ItemShape,
    ItemBitmap,
    ItemFont,
    ItemTextField,
    ItemSprite,
}
#[derive(Debug, Clone)]
pub enum UTSpriteYYSWFFillData {
    FillInvalid,
    FillSolid(UTSpriteYYSWFSolidFillData),
    FillGradient(UTSpriteYYSWFGradientFillData),
    FillBitmap(UTSpriteYYSWFBitmapFillData),
}
#[derive(Debug, Clone, TryFromPrimitive)]
#[repr(i32)]
pub enum UTSpriteYYSWFBitmapFillType {
    FillRepeat,
    FillClamp,
    FillRepeatPoint,
    FillClampPoint,
}
#[derive(Debug, Clone, TryFromPrimitive)]
#[repr(i32)]
pub enum UTSpriteYYSWFGradientFillType {
    FillLinear,
    FillRadial,
}

#[derive(Debug, Clone)]
pub struct UTSpriteYYSWFShapeData {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
    pub style_groups: Vec<UTSpriteYYSWFStyleGroup>
}
#[derive(Debug, Clone)]
pub struct UTSpriteYYSWFStyleGroup {
    pub fill_styles: Vec<UTSpriteYYSWFFillData>,
    pub line_styles: Vec<UTSpriteYYSWFLineStyleData>,
    pub subshapes: Vec<UTSpriteYYSWFSubshapeData>,
}

#[derive(Debug, Clone)]
pub struct UTSpriteYYSWFBitmapFillData {
    pub bitmap_fill_type: UTSpriteYYSWFBitmapFillType,
    pub char_id: i32,
    transformation_matrix: UTSpriteYYSWFMatrix33,
}

pub static YYSWF_MATRIX33_MATRIX_SIZE: usize = 9;
#[derive(Debug, Clone)]
pub struct UTSpriteYYSWFMatrix33 {
    pub values: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct UTSpriteYYSWFGradientFillData {
    pub tpe_index: Option<usize>,
    pub gradient_fill_type: UTSpriteYYSWFGradientFillType,
    pub transformation_matrix: UTSpriteYYSWFMatrix33,
    pub records: Vec<UTSpriteYYSWFGradientRecord>,
}

#[derive(Debug, Clone)]
pub struct UTSpriteYYSWFGradientRecord {
    pub ratio: i32,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

#[derive(Debug, Clone)]
pub struct UTSpriteYYSWFSolidFillData {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

#[derive(Debug, Clone)]
pub struct UTSpriteYYSWFLineStyleData {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

#[derive(Debug, Clone)]
pub struct UTSpriteYYSWFSubshapeData {
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

#[derive(Debug, Clone)]
pub struct UTSpriteYYSWFBitmapData {
    bitmap_type: UTSpriteYYSWFBitmapType,
    width: usize,
    height: usize,
    tpe_index: Option<i32>,
    /// will be empty if tpe index is Some
    image_data: Vec<u8>,
    /// will be empty if tpe index is Some
    alpha_data: Vec<u8>,
    /// will be empty if tpe index is Some
    color_palette_data: Vec<u8>,
}

#[derive(Debug, Clone, TryFromPrimitive)]
#[repr(i32)]
pub enum UTSpriteYYSWFBitmapType {
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

#[derive(Debug, Clone)]
pub struct UTSpriteYYSWFTimelineFrame {
    pub frame_objects: Vec<UTSpriteYYSWFTimelineObject>,
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
}

#[derive(Debug, Clone)]
pub struct UTSpriteYYSWFTimelineObject {
    pub char_id: i32,
    pub char_index: i32,
    pub depth: i32,
    pub clipping_depth: i32,
    pub transformation_matrix: UTSpriteYYSWFMatrix33,
    pub color_matrix: UTSpriteYYSWFColorMatrix,
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
}

pub static YYSWF_COLOR_MATRIX_SIZE: usize = 4;
#[derive(Debug, Clone)]
pub struct UTSpriteYYSWFColorMatrix {
    pub additive: Vec<i32>,
    pub multiply: Vec<i32>,
}

#[derive(Debug, Clone)]
pub struct UTSpriteYYSWFCollisionMask {
    pub rle_data: Vec<u8>,
}




pub fn parse_yyswf_timeline(chunk: &mut UTChunk, general_info: &UTGeneralInfo) -> Result<UTSpriteYYSWFTimeline, String> {
    let used_items_count = chunk.read_usize()?;
    let mut used_items: Vec<UTSpriteYYSWFItem> = Vec::with_capacity(used_items_count);
    for _ in 0..used_items_count {
        used_items.push(parse_yyswf_item(chunk, general_info)?);
    }

    let framerate: i32 = chunk.read_i32()?;
    let frames_count: usize = chunk.read_usize()?;
    let min_x: f32 = chunk.read_f32()?;
    let max_x: f32 = chunk.read_f32()?;
    let min_y: f32 = chunk.read_f32()?;
    let max_y: f32 = chunk.read_f32()?;
    let collision_masks_count: usize = chunk.read_usize()?;
    let mask_width: i32 = chunk.read_i32()?;
    let mask_height: i32 = chunk.read_i32()?;

    let mut frames: Vec<UTSpriteYYSWFTimelineFrame> = Vec::with_capacity(frames_count);
    for _ in 0..frames_count {
        frames.push(parse_yyswf_timeline_frame(chunk)?);
    }

    let mut collision_masks = Vec::with_capacity(collision_masks_count);
    for _ in 0..collision_masks_count {
        let rle_length: usize = chunk.read_usize()?;        // could be -1 idk

        let rle_data: Vec<u8> = match chunk.data.get(chunk.file_index .. chunk.file_index+rle_length) {
            Some(bytes) => bytes.to_vec(),
            None => return Err(format!(
                "Trying to read RLE Data of Timeline out of bounds while parsing \
                Sprite YYSWF at position {} in chunk '{}': {} > {}.",
                chunk.name, chunk.file_index, chunk.file_index + rle_length, chunk.data.len(),
            )),
        };
        chunk.file_index += rle_length;
        align_reader(chunk, 4, 0x00)?;      // [From UndertaleModTool] "why it's not aligned before the data is beyond my brain."

        collision_masks.push(UTSpriteYYSWFCollisionMask {rle_data});
    }

    Ok(UTSpriteYYSWFTimeline {
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


fn parse_yyswf_item(chunk: &mut UTChunk, general_info: &UTGeneralInfo) -> Result<UTSpriteYYSWFItem, String> {
    let item_type: i32 = chunk.read_i32()?;
    let item_type: UTSpriteYYSWFItemType = match item_type.try_into() {
        Ok(ok) => ok,
        Err(_) => return Err(format!(
            "Invalid YYSWF Item Type 0x{:08X} at position {} while parsing Sprite YYSWF in chunk '{}'.",
            item_type, chunk.file_index, chunk.name,
        )),
    };
    let id: i32 = chunk.read_i32()?;
    let mut shape_data: Option<UTSpriteYYSWFShapeData> = None;
    let mut bitmap_data: Option<UTSpriteYYSWFBitmapData> = None;

    match item_type {
        UTSpriteYYSWFItemType::ItemShape => shape_data = Some(parse_yyswf_shape_data(chunk, general_info)?),
        UTSpriteYYSWFItemType::ItemBitmap => bitmap_data = Some(parse_yyswf_bitmap_data(chunk, general_info)?),
        UTSpriteYYSWFItemType::ItemFont | UTSpriteYYSWFItemType::ItemInvalid | UTSpriteYYSWFItemType::ItemTextField | UTSpriteYYSWFItemType::ItemSprite => {},
    }

    Ok(UTSpriteYYSWFItem {
        id,
        item_type,
        shape_data,
        bitmap_data,
    })
}


fn parse_yyswf_shape_data(chunk: &mut UTChunk, general_info: &UTGeneralInfo) -> Result<UTSpriteYYSWFShapeData, String> {
    let min_x: f32 = chunk.read_f32()?;
    let max_x: f32 = chunk.read_f32()?;
    let min_y: f32 = chunk.read_f32()?;
    let max_y: f32 = chunk.read_f32()?;

    let style_group_count: usize = chunk.read_usize()?;     // could be -1 maybe
    let mut style_groups: Vec<UTSpriteYYSWFStyleGroup> = Vec::with_capacity(style_group_count);
    for _ in 0..style_group_count {
        style_groups.push(parse_yyswf_style_group(chunk, general_info)?);
    }

    Ok(UTSpriteYYSWFShapeData {
        min_x,
        max_x,
        min_y,
        max_y,
        style_groups,
    })
}


fn parse_yyswf_style_group(chunk: &mut UTChunk, general_info: &UTGeneralInfo) -> Result<UTSpriteYYSWFStyleGroup, String> {
    let fill_data_count: usize = chunk.read_usize()?;               // could be -1 maybe
    let line_style_count: usize = chunk.read_usize()?;         // could be -1 maybe
    let subshape_count: usize = chunk.read_usize()?;                // could be -1 maybe

    let mut fill_styles: Vec<UTSpriteYYSWFFillData> = Vec::with_capacity(fill_data_count);
    for _ in 0..fill_data_count {
        fill_styles.push(parse_yyswf_fill_data(chunk, general_info)?);
    }

    let mut line_styles: Vec<UTSpriteYYSWFLineStyleData> = Vec::with_capacity(line_style_count);
    for _ in 0..line_style_count {
        let red: u8 = chunk.read_u8()?;
        let green: u8 = chunk.read_u8()?;
        let blue: u8 = chunk.read_u8()?;
        let alpha: u8 = chunk.read_u8()?;

        line_styles.push(UTSpriteYYSWFLineStyleData {
            red,
            green,
            blue,
            alpha,
        });
    }

    let mut subshapes: Vec<UTSpriteYYSWFSubshapeData> = Vec::with_capacity(subshape_count);
    for _ in 0..subshape_count {
        subshapes.push(parse_yyswf_subshapes(chunk)?);
    }

    Ok(UTSpriteYYSWFStyleGroup {
        fill_styles,
        line_styles,
        subshapes,
    })
}

fn parse_yyswf_fill_data(chunk: &mut UTChunk, general_info: &UTGeneralInfo) -> Result<UTSpriteYYSWFFillData, String> {
    let fill_type: i32 = chunk.read_i32()?;
    match fill_type {
        1 => {  // Solid Fill
            let red: u8 = chunk.read_u8()?;
            let green: u8 = chunk.read_u8()?;
            let blue: u8 = chunk.read_u8()?;
            let alpha: u8 = chunk.read_u8()?;
            Ok(UTSpriteYYSWFFillData::FillSolid(UTSpriteYYSWFSolidFillData {
                red,
                green,
                blue,
                alpha,
            }))
        },

        2 => {  // Gradient Fill
            let gradient_fill_type: i32 = chunk.read_i32()?;
            let gradient_fill_type: UTSpriteYYSWFGradientFillType = match gradient_fill_type.try_into() {
                Ok(ok) => ok,
                Err(_) => return Err(format!(
                    "Invalid YYSWF Fill Gradient Type 0x{:08X} at position {} while parsing Sprite YYSWF in chunk '{}'.",
                    gradient_fill_type, chunk.file_index, chunk.name,
                )),
            };

            let mut tpe_index: Option<usize> = None;
            if general_info.is_version_at_least(2022, 1, 0, 0) {
                tpe_index = Some(chunk.read_usize()?);      // maybe -1 idk
            }

            let transformation_matrix: UTSpriteYYSWFMatrix33 = parse_yyswf_transformation_matrix(chunk)?;

            let record_count: usize = chunk.read_usize()?;
            let mut records: Vec<UTSpriteYYSWFGradientRecord> = Vec::with_capacity(record_count);
            for _ in 0..record_count {
                let ratio: i32 = chunk.read_i32()?;
                let red: u8 = chunk.read_u8()?;
                let green: u8 = chunk.read_u8()?;
                let blue: u8 = chunk.read_u8()?;
                let alpha: u8 = chunk.read_u8()?;
                records.push(UTSpriteYYSWFGradientRecord {
                    ratio,
                    red,
                    green,
                    blue,
                    alpha,
                });
            }

            Ok(UTSpriteYYSWFFillData::FillGradient(UTSpriteYYSWFGradientFillData {
                tpe_index,
                gradient_fill_type,
                transformation_matrix,
                records,
            }))
        },

        3 => {      // Fill Bitmap
            let bitmap_fill_type: i32 = chunk.read_i32()?;
            let bitmap_fill_type: UTSpriteYYSWFBitmapFillType = match bitmap_fill_type.try_into() {
                Ok(ok) => ok,
                Err(_) => return Err(format!(
                    "Invalid YYSWF Bitmap Fill Type 0x{:08X} at position {} while parsing Sprite YYSWF in chunk '{}'.",
                    bitmap_fill_type, chunk.file_index, chunk.name,
                ))
            };
            let char_id: i32 = chunk.read_i32()?;
            let transformation_matrix: UTSpriteYYSWFMatrix33 = parse_yyswf_transformation_matrix(chunk)?;

            Ok(UTSpriteYYSWFFillData::FillBitmap(UTSpriteYYSWFBitmapFillData {
                bitmap_fill_type,
                char_id,
                transformation_matrix,
            }))
        },

        _ => Err(format!(
            "Invalid YYSWF Fill Type 0x{:08X} at position {} while parsing Sprite YYSWF in chunk '{}'.",
            fill_type, chunk.file_index, chunk.name,
        )),
    }
}

fn parse_yyswf_transformation_matrix(chunk: &mut UTChunk) -> Result<UTSpriteYYSWFMatrix33, String> {
    let mut transformation_matrix_values: Vec<f32> = Vec::with_capacity(YYSWF_MATRIX33_MATRIX_SIZE);
    for _ in 0..YYSWF_MATRIX33_MATRIX_SIZE {
        transformation_matrix_values.push(chunk.read_f32()?);
    }
    Ok(UTSpriteYYSWFMatrix33 { values: transformation_matrix_values })
}


fn parse_yyswf_subshapes(chunk: &mut UTChunk) -> Result<UTSpriteYYSWFSubshapeData, String> {
    let fill_style1: i32 = chunk.read_i32()?;
    let fill_style2: i32 = chunk.read_i32()?;
    let line_style: i32 = chunk.read_i32()?;
    let point_count: usize = chunk.read_usize()?;
    let line_count: usize = chunk.read_usize()?;
    let triangle_count: usize = chunk.read_usize()? * 3;
    let aa_line_count: usize = chunk.read_usize()?;
    let aa_vector_count: usize = chunk.read_usize()? * 3;
    let line_aa_line_count: usize = chunk.read_usize()?;
    let line_aa_vector_count: usize = chunk.read_usize()?;

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

    Ok(UTSpriteYYSWFSubshapeData {
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


fn parse_yyswf_bitmap_data(chunk: &mut UTChunk, general_info: &UTGeneralInfo) -> Result<UTSpriteYYSWFBitmapData, String> {
    let bitmap_type: i32 = chunk.read_i32()?;
    let bitmap_type: UTSpriteYYSWFBitmapType = match bitmap_type.try_into() {
        Ok(ok) => ok,
        Err(_) => return Err(format!(
            "Invalid YYSWF Bitmap Type 0x{:08X} at position {} while parsing Sprite YYSWF in chunk '{}'.",
            bitmap_type, chunk.file_index, chunk.name,
        )),
    };

    let width: usize = chunk.read_usize()?;             // could be -1 idk
    let height: usize = chunk.read_usize()?;            // could be -1 idk
    let mut tpe_index: Option<i32> = None;
    let mut image_data: Vec<u8> = Vec::with_capacity(0);
    let mut alpha_data: Vec<u8> = Vec::with_capacity(0);
    let mut color_palette_data: Vec<u8> = Vec::with_capacity(0);


    if general_info.is_version_at_least(2022, 1, 0, 0) {
        tpe_index = Some(chunk.read_i32()?);
    } else {
        let image_data_length: usize = chunk.read_usize()?;
        let alpha_data_length: usize = chunk.read_usize()?;
        let color_palette_data_length: usize = chunk.read_usize()?;

        image_data = match chunk.data.get(chunk.file_index .. chunk.file_index+image_data_length) {
            Some(bytes) => bytes.to_vec(),
            None => return Err(format!(
                "Trying to read Image Data of Bitmap Data out of bounds while parsing \
                Sprite YYSWF at position {} in chunk '{}': {} > {}.",
                chunk.name, chunk.file_index, chunk.file_index + image_data_length, chunk.data.len(),
            )),
        };
        chunk.file_index += image_data_length;

        alpha_data = match chunk.data.get(chunk.file_index .. chunk.file_index+alpha_data_length) {
            Some(bytes) => bytes.to_vec(),
            None => return Err(format!(
                "Trying to read Alpha Data of Bitmap Data out of bounds while parsing \
                Sprite YYSWF at position {} in chunk '{}': {} > {}.",
                chunk.name, chunk.file_index, chunk.file_index + alpha_data_length, chunk.data.len(),
            )),
        };
        chunk.file_index += alpha_data_length;

        color_palette_data = match chunk.data.get(chunk.file_index .. chunk.file_index+color_palette_data_length) {
            Some(bytes) => bytes.to_vec(),
            None => return Err(format!(
                "Trying to read Color Palette Data of Bitmap Data out of bounds while parsing \
                Sprite YYSWF at position {} in chunk '{}': {} > {}.",
                chunk.name, chunk.file_index, chunk.file_index + color_palette_data_length, chunk.data.len(),
            )),
        };
        chunk.file_index += color_palette_data_length;

        align_reader(chunk, 4, 0x00)?;

    }

    Ok(UTSpriteYYSWFBitmapData {
        bitmap_type,
        width,
        height,
        tpe_index,
        image_data,
        alpha_data,
        color_palette_data,
    })
}


fn parse_yyswf_timeline_frame(chunk: &mut UTChunk) -> Result<UTSpriteYYSWFTimelineFrame, String> {
    let frame_object_count: usize = chunk.read_usize()?;     // could be -1
    let min_x: f32 = chunk.read_f32()?;
    let max_x: f32 = chunk.read_f32()?;
    let min_y: f32 = chunk.read_f32()?;
    let max_y: f32 = chunk.read_f32()?;

    let mut frame_objects: Vec<UTSpriteYYSWFTimelineObject> = Vec::with_capacity(frame_object_count);
    for _ in 0..frame_object_count {
        let char_id: i32 = chunk.read_i32()?;
        let char_index: i32 = chunk.read_i32()?;
        let depth: i32 = chunk.read_i32()?;
        let clipping_depth: i32 = chunk.read_i32()?;
        let color_matrix: UTSpriteYYSWFColorMatrix = parse_yyswf_color_matrix(chunk)?;
        let min_x: f32 = chunk.read_f32()?;
        let max_x: f32 = chunk.read_f32()?;
        let min_y: f32 = chunk.read_f32()?;
        let max_y: f32 = chunk.read_f32()?;
        let transformation_matrix: UTSpriteYYSWFMatrix33 = parse_yyswf_transformation_matrix(chunk)?;

        frame_objects.push(UTSpriteYYSWFTimelineObject {
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

    Ok(UTSpriteYYSWFTimelineFrame {
        frame_objects,
        min_x,
        max_x,
        min_y,
        max_y,
    })
}


fn parse_yyswf_color_matrix(chunk: &mut UTChunk) -> Result<UTSpriteYYSWFColorMatrix, String> {
    let mut additive: Vec<i32> = Vec::with_capacity(YYSWF_COLOR_MATRIX_SIZE);
    let mut multiply: Vec<i32> = Vec::with_capacity(YYSWF_COLOR_MATRIX_SIZE);

    for _ in 0..YYSWF_COLOR_MATRIX_SIZE {
        additive.push(chunk.read_i32()?);
    }

    for _ in 0..YYSWF_COLOR_MATRIX_SIZE {
        multiply.push(chunk.read_i32()?);
    }

    Ok(UTSpriteYYSWFColorMatrix { additive, multiply })
}

