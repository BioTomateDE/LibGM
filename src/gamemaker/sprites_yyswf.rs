use num_enum::TryFromPrimitive;
use crate::gamemaker::chunk_reading::{vec_with_capacity, DataReader, GMElement};

#[derive(Debug, Clone, PartialEq)]
pub struct GMSpriteTypeSWF {
    pub swf_version: i32,
    pub yyswf_version: i32,
    pub jpeg_table: Vec<u8>,
    pub timeline: GMSpriteYYSWFTimeline,
}


#[derive(Debug, Clone, PartialEq)]
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
impl GMElement for GMSpriteYYSWFTimeline {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let used_items: Vec<GMSpriteYYSWFItem> = reader.read_simple_list()?;
        let framerate: i32 = reader.read_i32()?;
        let frames_count: usize = reader.read_usize()?;
        let min_x: f32 = reader.read_f32()?;
        let max_x: f32 = reader.read_f32()?;
        let min_y: f32 = reader.read_f32()?;
        let max_y: f32 = reader.read_f32()?;
        let collision_masks_count: usize = reader.read_usize()?;
        let mask_width: i32 = reader.read_i32()?;
        let mask_height: i32 = reader.read_i32()?;

        let mut frames: Vec<GMSpriteYYSWFTimelineFrame> = vec_with_capacity(frames_count)?;
        for _ in 0..frames_count {
            frames.push(GMSpriteYYSWFTimelineFrame::deserialize(reader)?);
        }

        let mut collision_masks: Vec<GMSpriteYYSWFCollisionMask> = vec_with_capacity(collision_masks_count)?;
        for _ in 0..collision_masks_count {
            let rle_length: usize = reader.read_usize()?;      // could be -1 idk

            let rle_data: Vec<u8> = reader.read_bytes_dyn(rle_length).map_err(|e| format!("Trying to read RLE Data of Timeline {e}"))?.to_vec();
            reader.align(4)?;    // [From UndertaleModTool] "why it's not aligned before the data is beyond my brain"

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
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMSpriteYYSWFItem {
    pub id: i32,
    pub item_type: GMSpriteYYSWFItemType,
    pub shape_data: Option<GMSpriteYYSWFShapeData>,
    pub bitmap_data: Option<GMSpriteYYSWFBitmapData>,
}
impl GMElement for GMSpriteYYSWFItem {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let item_type: i32 = reader.read_i32()?;
        let item_type: GMSpriteYYSWFItemType = item_type.try_into().map_err(|_| format!(
            "Invalid YYSWF Item Type 0x{:08X} at position {} while parsing Sprite YYSWF",
            item_type, reader.cur_pos,
        ))?;
        let id: i32 = reader.read_i32()?;
        let mut shape_data: Option<GMSpriteYYSWFShapeData> = None;
        let mut bitmap_data: Option<GMSpriteYYSWFBitmapData> = None;

        match item_type {
            GMSpriteYYSWFItemType::ItemShape => shape_data = Some(GMSpriteYYSWFShapeData::deserialize(reader)?),
            GMSpriteYYSWFItemType::ItemBitmap => bitmap_data = Some(GMSpriteYYSWFBitmapData::deserialize(reader)?),
            GMSpriteYYSWFItemType::ItemFont |
            GMSpriteYYSWFItemType::ItemInvalid |
            GMSpriteYYSWFItemType::ItemTextField |
            GMSpriteYYSWFItemType::ItemSprite => {},
        }

        Ok(GMSpriteYYSWFItem {
            id,
            item_type,
            shape_data,
            bitmap_data,
        })
    }
}


#[derive(Debug, Clone, PartialEq, TryFromPrimitive)]
#[repr(i32)]
pub enum GMSpriteYYSWFItemType {
    ItemInvalid,
    ItemShape,
    ItemBitmap,
    ItemFont,
    ItemTextField,
    ItemSprite,
}


#[derive(Debug, Clone, PartialEq)]
pub enum GMSpriteYYSWFFillData {
    FillInvalid,
    FillSolid(GMSpriteYYSWFSolidFillData),
    FillGradient(GMSpriteYYSWFGradientFillData),
    FillBitmap(GMSpriteYYSWFBitmapFillData),
}
impl GMElement for GMSpriteYYSWFFillData {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let fill_type: i32 = reader.read_i32()?;
        match fill_type {
            1 => {  // Solid Fill
                let red: u8 = reader.read_u8()?;
                let green: u8 = reader.read_u8()?;
                let blue: u8 = reader.read_u8()?;
                let alpha: u8 = reader.read_u8()?;
                Ok(GMSpriteYYSWFFillData::FillSolid(GMSpriteYYSWFSolidFillData {
                    red,
                    green,
                    blue,
                    alpha,
                }))
            },

            2 => {  // Gradient Fill
                let gradient_fill_type: i32 = reader.read_i32()?;
                let gradient_fill_type: GMSpriteYYSWFGradientFillType = gradient_fill_type.try_into().map_err(|_|format!(
                    "Invalid YYSWF Fill Gradient Type 0x{:08X} at position {} while parsing Sprite YYSWF",
                    gradient_fill_type, reader.cur_pos,
                ))?;

                let mut tpe_index: Option<usize> = None;
                if reader.general_info.is_version_at_least((2022, 1, 0, 0)) {
                    tpe_index = Some(reader.read_usize()?);      // maybe -1 idk
                }

                let transformation_matrix = GMSpriteYYSWFMatrix33 { values: reader.read_simple_list::<f32>()? };
                let records: Vec<GMSpriteYYSWFGradientRecord> = reader.read_simple_list()?;

                Ok(GMSpriteYYSWFFillData::FillGradient(GMSpriteYYSWFGradientFillData {
                    tpe_index,
                    gradient_fill_type,
                    transformation_matrix,
                    records,
                }))
            },

            3 => {      // Fill Bitmap
                let bitmap_fill_type: i32 = reader.read_i32()?;
                let bitmap_fill_type: GMSpriteYYSWFBitmapFillType = bitmap_fill_type.try_into().map_err(|_| format!(
                    "Invalid YYSWF Bitmap Fill Type 0x{:08X} at position {} while parsing Sprite YYSWF",
                    bitmap_fill_type, reader.cur_pos,
                ))?;
                let char_id: i32 = reader.read_i32()?;
                let transformation_matrix = GMSpriteYYSWFMatrix33 { values: reader.read_simple_list::<f32>()? };

                Ok(GMSpriteYYSWFFillData::FillBitmap(GMSpriteYYSWFBitmapFillData {
                    bitmap_fill_type,
                    char_id,
                    transformation_matrix,
                }))
            },

            _ => Err(format!(
                "Invalid YYSWF Fill Type 0x{:08X} at position {} while parsing Sprite YYSWF",
                fill_type, reader.cur_pos,
            )),
        }
    }
}


#[derive(Debug, Clone, PartialEq, TryFromPrimitive)]
#[repr(i32)]
pub enum GMSpriteYYSWFBitmapFillType {
    FillRepeat,
    FillClamp,
    FillRepeatPoint,
    FillClampPoint,
}
#[derive(Debug, Clone, PartialEq, TryFromPrimitive)]
#[repr(i32)]
pub enum GMSpriteYYSWFGradientFillType {
    FillLinear,
    FillRadial,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMSpriteYYSWFShapeData {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
    pub style_groups: Vec<GMSpriteYYSWFStyleGroup>
}
impl GMElement for GMSpriteYYSWFShapeData {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let min_x: f32 = reader.read_f32()?;
        let max_x: f32 = reader.read_f32()?;
        let min_y: f32 = reader.read_f32()?;
        let max_y: f32 = reader.read_f32()?;
        let style_groups: Vec<GMSpriteYYSWFStyleGroup> = reader.read_simple_list()?;
        Ok(GMSpriteYYSWFShapeData { min_x, max_x, min_y, max_y, style_groups })
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMSpriteYYSWFStyleGroup {
    pub fill_styles: Vec<GMSpriteYYSWFFillData>,
    pub line_styles: Vec<GMSpriteYYSWFLineStyleData>,
    pub subshapes: Vec<GMSpriteYYSWFSubshapeData>,
}
impl GMElement for GMSpriteYYSWFStyleGroup {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let fill_data_count: usize = reader.read_usize()?;       // could be -1 maybe
        let line_style_count: usize = reader.read_usize()?;      // could be -1 maybe
        let subshape_count: usize = reader.read_usize()?;        // could be -1 maybe

        let mut fill_styles: Vec<GMSpriteYYSWFFillData> = vec_with_capacity(fill_data_count)?;
        for _ in 0..fill_data_count {
            fill_styles.push(GMSpriteYYSWFFillData::deserialize(reader)?);
        }

        let mut line_styles: Vec<GMSpriteYYSWFLineStyleData> = vec_with_capacity(line_style_count)?;
        for _ in 0..line_style_count {
            line_styles.push(GMSpriteYYSWFLineStyleData::deserialize(reader)?);
        }

        let mut subshapes: Vec<GMSpriteYYSWFSubshapeData> = vec_with_capacity(subshape_count)?;
        for _ in 0..subshape_count {
            subshapes.push(GMSpriteYYSWFSubshapeData::deserialize(reader)?);
        }

        Ok(GMSpriteYYSWFStyleGroup {
            fill_styles,
            line_styles,
            subshapes,
        })
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMSpriteYYSWFBitmapFillData {
    pub bitmap_fill_type: GMSpriteYYSWFBitmapFillType,
    pub char_id: i32,
    transformation_matrix: GMSpriteYYSWFMatrix33,
}

pub static YYSWF_MATRIX33_MATRIX_SIZE: usize = 9;
#[derive(Debug, Clone, PartialEq)]
pub struct GMSpriteYYSWFMatrix33 {
    pub values: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMSpriteYYSWFGradientFillData {
    pub tpe_index: Option<usize>,
    pub gradient_fill_type: GMSpriteYYSWFGradientFillType,
    pub transformation_matrix: GMSpriteYYSWFMatrix33,
    pub records: Vec<GMSpriteYYSWFGradientRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMSpriteYYSWFGradientRecord {
    pub ratio: i32,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}
impl GMElement for GMSpriteYYSWFGradientRecord {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let ratio: i32 = reader.read_i32()?;
        let red: u8 = reader.read_u8()?;
        let green: u8 = reader.read_u8()?;
        let blue: u8 = reader.read_u8()?;
        let alpha: u8 = reader.read_u8()?;
        Ok(GMSpriteYYSWFGradientRecord {
            ratio,
            red,
            green,
            blue,
            alpha,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GMSpriteYYSWFSolidFillData {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}
impl GMElement for GMSpriteYYSWFSolidFillData {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let red: u8 = reader.read_u8()?;
        let green: u8 = reader.read_u8()?;
        let blue: u8 = reader.read_u8()?;
        let alpha: u8 = reader.read_u8()?;
        Ok(GMSpriteYYSWFSolidFillData { red, green, blue, alpha })
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMSpriteYYSWFLineStyleData {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}
impl GMElement for GMSpriteYYSWFLineStyleData {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let red: u8 = reader.read_u8()?;
        let green: u8 = reader.read_u8()?;
        let blue: u8 = reader.read_u8()?;
        let alpha: u8 = reader.read_u8()?;
        Ok(GMSpriteYYSWFLineStyleData { red, green, blue, alpha })
    }
}


#[derive(Debug, Clone, PartialEq)]
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
impl GMElement for GMSpriteYYSWFSubshapeData {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let fill_style1: i32 = reader.read_i32()?;
        let fill_style2: i32 = reader.read_i32()?;
        let line_style: i32 = reader.read_i32()?;
        let point_count: usize = reader.read_usize()?;
        let line_count: usize = reader.read_usize()?;
        let triangle_count: usize = reader.read_usize()? * 3;
        let aa_line_count: usize = reader.read_usize()?;
        let aa_vector_count: usize = reader.read_usize()? * 3;
        let line_aa_line_count: usize = reader.read_usize()?;
        let line_aa_vector_count: usize = reader.read_usize()?;

        let mut points: Vec<(f32, f32)> = vec_with_capacity(point_count)?;
        let mut lines: Vec<(i32, i32)> = vec_with_capacity(line_count)?;
        let mut triangles: Vec<i32> = vec_with_capacity(triangle_count)?;
        let mut aa_lines: Vec<(i32, i32)> = vec_with_capacity(aa_line_count)?;
        let mut aa_vectors: Vec<(f32, f32)> = vec_with_capacity(aa_vector_count)?;
        let mut line_aa_lines: Vec<(i32, i32)> = vec_with_capacity(line_aa_line_count)?;
        let mut line_aa_vectors: Vec<(f32, f32)> = vec_with_capacity(line_aa_vector_count)?;

        for _ in 0..point_count {
            points.push((reader.read_f32()?, reader.read_f32()?));
        }
        for _ in 0..line_count {
            lines.push((reader.read_i32()?, reader.read_i32()?));
        }
        for _ in 0..triangle_count {
            triangles.push(reader.read_i32()?);
        }
        for _ in 0..aa_line_count {
            aa_lines.push((reader.read_i32()?, reader.read_i32()?));
        }
        for _ in 0..aa_vector_count {
            aa_vectors.push((reader.read_f32()?, reader.read_f32()?));
        }
        for _ in 0..line_aa_line_count {
            line_aa_lines.push((reader.read_i32()?, reader.read_i32()?));
        }
        for _ in 0..line_aa_vector_count {
            line_aa_vectors.push((reader.read_f32()?, reader.read_f32()?));
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
}


#[derive(Debug, Clone, PartialEq)]
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
impl GMElement for GMSpriteYYSWFBitmapData {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let bitmap_type: i32 = reader.read_i32()?;
        let bitmap_type: GMSpriteYYSWFBitmapType = bitmap_type.try_into().map_err(|_| format!(
            "Invalid YYSWF Bitmap Type 0x{:08X} at position {} while parsing Sprite YYSWF",
            bitmap_type, reader.cur_pos,
        ))?;

        let width: u32 = reader.read_u32()?;             // could be -1 idk
        let height: u32 = reader.read_u32()?;            // could be -1 idk
        let mut tpe_index: Option<i32> = None;
        let mut image_data: Vec<u8> = Vec::with_capacity(0);
        let mut alpha_data: Vec<u8> = Vec::with_capacity(0);
        let mut color_palette_data: Vec<u8> = Vec::with_capacity(0);

        if reader.general_info.is_version_at_least((2022, 1, 0, 0)) {
            tpe_index = Some(reader.read_i32()?);
        } else {
            let image_data_length: usize = reader.read_usize()?;
            let alpha_data_length: usize = reader.read_usize()?;
            let color_palette_data_length: usize = reader.read_usize()?;

            image_data = reader.read_bytes_dyn(image_data_length)
                .map_err(|e| format!("Trying to read Image Data of Bitmap Data {e}"))?.to_vec();
            alpha_data = reader.read_bytes_dyn(alpha_data_length)
                .map_err(|e| format!("Trying to read Alpha Data of Bitmap Data {e}"))?.to_vec();
            color_palette_data = reader.read_bytes_dyn(color_palette_data_length)
                .map_err(|e| format!("Trying to read Color Palette Data of Bitmap Data {e}"))?.to_vec();

            reader.align(4)?;
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
}


#[derive(Debug, Clone, PartialEq, TryFromPrimitive)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct GMSpriteYYSWFTimelineFrame {
    pub frame_objects: Vec<GMSpriteYYSWFTimelineObject>,
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
}
impl GMElement for GMSpriteYYSWFTimelineFrame {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let frame_object_count: usize = reader.read_usize()?;
        let min_x: f32 = reader.read_f32()?;
        let max_x: f32 = reader.read_f32()?;
        let min_y: f32 = reader.read_f32()?;
        let max_y: f32 = reader.read_f32()?;
        let mut frame_objects: Vec<GMSpriteYYSWFTimelineObject> = vec_with_capacity(frame_object_count)?;
        for _ in 0..frame_object_count {
            frame_objects.push(GMSpriteYYSWFTimelineObject::deserialize(reader)?);
        }
        Ok(Self { frame_objects, min_x, max_x, min_y, max_y })
    }
}

#[derive(Debug, Clone, PartialEq)]
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
impl GMElement for GMSpriteYYSWFTimelineObject {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let char_id: i32 = reader.read_i32()?;
        let char_index: i32 = reader.read_i32()?;
        let depth: i32 = reader.read_i32()?;
        let clipping_depth: i32 = reader.read_i32()?;
        let color_matrix = GMSpriteYYSWFColorMatrix::deserialize(reader)?;
        let min_x: f32 = reader.read_f32()?;
        let max_x: f32 = reader.read_f32()?;
        let min_y: f32 = reader.read_f32()?;
        let max_y: f32 = reader.read_f32()?;
        let transformation_matrix = GMSpriteYYSWFMatrix33 { values: reader.read_simple_list::<f32>()? };

        Ok(GMSpriteYYSWFTimelineObject {
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
}


pub static YYSWF_COLOR_MATRIX_SIZE: usize = 4;
#[derive(Debug, Clone, PartialEq)]
pub struct GMSpriteYYSWFColorMatrix {
    pub additive: Vec<i32>,
    pub multiply: Vec<i32>,
}
impl GMElement for GMSpriteYYSWFColorMatrix {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let mut additive: Vec<i32> = Vec::with_capacity(YYSWF_COLOR_MATRIX_SIZE);
        let mut multiply: Vec<i32> = Vec::with_capacity(YYSWF_COLOR_MATRIX_SIZE);

        for _ in 0..YYSWF_COLOR_MATRIX_SIZE {
            additive.push(reader.read_i32()?);
        }

        for _ in 0..YYSWF_COLOR_MATRIX_SIZE {
            multiply.push(reader.read_i32()?);
        }

        Ok(GMSpriteYYSWFColorMatrix { additive, multiply })
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMSpriteYYSWFCollisionMask {
    pub rle_data: Vec<u8>,
}

