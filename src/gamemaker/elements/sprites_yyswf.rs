use num_enum::{IntoPrimitive, TryFromPrimitive};
use crate::gamemaker::deserialize::DataReader;
use crate::gamemaker::element::GMElement;
use crate::gamemaker::elements::sprites::GMSpriteShapeData;
use crate::gamemaker::serialize::DataBuilder;
use crate::gamemaker::serialize::traits::GMSerializeIfVersion;
use crate::utility::{num_enum_from, vec_with_capacity};

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
            collision_masks.push(GMSpriteYYSWFCollisionMask::deserialize(reader)?);
        }

        Ok(GMSpriteYYSWFTimeline { framerate, min_x, max_x, min_y, max_y, mask_width, mask_height, used_items, frames, collision_masks })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_simple_list(&self.used_items)?;
        builder.write_i32(self.framerate);
        builder.write_usize(self.frames.len())?;
        builder.write_f32(self.min_x);
        builder.write_f32(self.max_x);
        builder.write_f32(self.min_y);
        builder.write_f32(self.max_y);
        builder.write_usize(self.collision_masks.len())?;
        builder.write_i32(self.mask_width);
        builder.write_i32(self.mask_height);
        for frame in &self.frames {
            frame.serialize(builder)?;
        }
        for mask in &self.collision_masks {
            mask.serialize(builder)?;
        }
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMSpriteYYSWFItem {
    pub id: i32,
    pub item_data: GMSpriteYYSWFItemData,
}
impl GMElement for GMSpriteYYSWFItem {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let item_type: i32 = reader.read_i32()?;
        let id: i32 = reader.read_i32()?;
        let item_data: GMSpriteYYSWFItemData = match item_type {
            1 => GMSpriteYYSWFItemData::ItemShape(GMSpriteShapeData::deserialize(reader)?),
            2 => GMSpriteYYSWFItemData::ItemBitmap(GMSpriteYYSWFBitmapData::deserialize(reader)?),
            3 => GMSpriteYYSWFItemData::ItemFont,
            4 => GMSpriteYYSWFItemData::ItemTextField,
            5 => GMSpriteYYSWFItemData::ItemSprite,
            _ => return Err(format!(
                "Invalid YYSWF Item Type {0} 0x{0:08X} at position {1} while parsing Sprite YYSWF Item",
                item_type, reader.cur_pos,
            ))
        };
        Ok(GMSpriteYYSWFItem { id, item_data })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_i32(match &self.item_data {
            GMSpriteYYSWFItemData::ItemShape(_) => 1,
            GMSpriteYYSWFItemData::ItemBitmap(_) => 2,
            GMSpriteYYSWFItemData::ItemFont => 3,
            GMSpriteYYSWFItemData::ItemTextField => 4,
            GMSpriteYYSWFItemData::ItemSprite => 5,
        });
        builder.write_i32(self.id);
        match &self.item_data {
            GMSpriteYYSWFItemData::ItemShape(shape_data) => shape_data.serialize(builder)?,
            GMSpriteYYSWFItemData::ItemBitmap(bitmap_data) => bitmap_data.serialize(builder)?,
            GMSpriteYYSWFItemData::ItemFont => {}
            GMSpriteYYSWFItemData::ItemTextField => {}
            GMSpriteYYSWFItemData::ItemSprite => {}
        }
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum GMSpriteYYSWFItemData {
    ItemShape(GMSpriteShapeData<GMSpriteYYSWFSubshapeData>),
    ItemBitmap(GMSpriteYYSWFBitmapData),
    ItemFont,
    ItemTextField,
    ItemSprite,
}


#[derive(Debug, Clone, PartialEq)]
pub enum GMSpriteYYSWFFillData {
    FillSolid(GMSpriteYYSWFSolidFillData),
    FillGradient(GMSpriteYYSWFGradientFillData),
    FillBitmap(GMSpriteYYSWFBitmapFillData),
}
impl GMElement for GMSpriteYYSWFFillData {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let fill_type: i32 = reader.read_i32()?;
        let fill_data = match fill_type {
            1 => Self::FillSolid(GMSpriteYYSWFSolidFillData::deserialize(reader)?),
            2 => Self::FillGradient(GMSpriteYYSWFGradientFillData::deserialize(reader)?),
            3 => Self::FillBitmap(GMSpriteYYSWFBitmapFillData::deserialize(reader)?),
            _ => return Err(format!(
                "Invalid YYSWF Fill Type 0x{:08X} at position {} while parsing Sprite YYSWF Fill Data",
                fill_type, reader.cur_pos,
            )),
        };
        Ok(fill_data)
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_i32(match self {
            Self::FillSolid(_) => 1,
            Self::FillGradient(_) => 2,
            Self::FillBitmap(_) => 3,
        });
        match self {
            Self::FillSolid(solid_data) => solid_data.serialize(builder)?,
            Self::FillGradient(gradient_data) => gradient_data.serialize(builder)?,
            Self::FillBitmap(bitmap_data) => bitmap_data.serialize(builder)?,
        }
        Ok(())
    }
}


#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(i32)]
pub enum GMSpriteYYSWFBitmapFillType {
    FillRepeat,
    FillClamp,
    FillRepeatPoint,
    FillClampPoint,
}

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(i32)]
pub enum GMSpriteYYSWFGradientFillType {
    FillLinear,
    FillRadial,
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMSpriteYYSWFStyleGroup<T: GMElement> {
    pub fill_styles: Vec<GMSpriteYYSWFFillData>,
    pub line_styles: Vec<GMSpriteYYSWFLineStyleData>,
    pub subshapes: Vec<T>,
}
impl<T: GMElement> GMElement for GMSpriteYYSWFStyleGroup<T> {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let fill_data_count: usize = reader.read_i32()?.max(0) as usize;
        let line_style_count: usize = reader.read_i32()?.max(0) as usize;
        let subshape_count: usize = reader.read_i32()?.max(0) as usize;

        let mut fill_styles: Vec<GMSpriteYYSWFFillData> = vec_with_capacity(fill_data_count)?;
        for _ in 0..fill_data_count {
            fill_styles.push(GMSpriteYYSWFFillData::deserialize(reader)?);
        }

        let mut line_styles: Vec<GMSpriteYYSWFLineStyleData> = vec_with_capacity(line_style_count)?;
        for _ in 0..line_style_count {
            line_styles.push(GMSpriteYYSWFLineStyleData::deserialize(reader)?);
        }

        let mut subshapes: Vec<T> = vec_with_capacity(subshape_count)?;
        for _ in 0..subshape_count {
            subshapes.push(T::deserialize(reader)?);
        }

        Ok(GMSpriteYYSWFStyleGroup { fill_styles, line_styles, subshapes })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_usize(self.fill_styles.len())?;
        builder.write_usize(self.line_styles.len())?;
        builder.write_usize(self.subshapes.len())?;
        for fill_data in &self.fill_styles {
            fill_data.serialize(builder)?;
        }
        for line_data in &self.line_styles {
            line_data.serialize(builder)?;
        }
        for subshape in &self.subshapes {
            subshape.serialize(builder)?;
        }
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMSpriteYYSWFBitmapFillData {
    pub bitmap_fill_type: GMSpriteYYSWFBitmapFillType,
    pub char_id: i32,
    transformation_matrix: GMSpriteYYSWFMatrix33,
}
impl GMElement for GMSpriteYYSWFBitmapFillData {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String>{
        let bitmap_fill_type: GMSpriteYYSWFBitmapFillType = num_enum_from(reader.read_i32()?)?;
        let char_id: i32 = reader.read_i32()?;
        let transformation_matrix = GMSpriteYYSWFMatrix33::deserialize(reader)?;
        Ok(GMSpriteYYSWFBitmapFillData { bitmap_fill_type, char_id, transformation_matrix })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_i32(self.bitmap_fill_type.into());
        builder.write_i32(self.char_id);
        self.transformation_matrix.serialize(builder)?;
        Ok(())
    }
}


pub const YYSWF_MATRIX33_MATRIX_SIZE: usize = 9;
#[derive(Debug, Clone, PartialEq)]
pub struct GMSpriteYYSWFMatrix33 {
    pub values: [f32; YYSWF_MATRIX33_MATRIX_SIZE],
}
impl GMElement for GMSpriteYYSWFMatrix33 {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let mut values = [0f32; YYSWF_MATRIX33_MATRIX_SIZE];
        for item in &mut values {
            *item = reader.read_f32()?;
        }
        Ok(Self { values })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        for item in &self.values {
            builder.write_f32(*item);
        }
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMSpriteYYSWFGradientFillData {
    pub tpe_index: Option<i32>,
    pub gradient_fill_type: GMSpriteYYSWFGradientFillType,
    pub transformation_matrix: GMSpriteYYSWFMatrix33,
    pub records: Vec<GMSpriteYYSWFGradientRecord>,
}
impl GMElement for GMSpriteYYSWFGradientFillData {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let gradient_fill_type: GMSpriteYYSWFGradientFillType = num_enum_from(reader.read_i32()?)?;
        let tpe_index: Option<i32> = reader.deserialize_if_gm_version((2022, 1))?;
        let transformation_matrix = GMSpriteYYSWFMatrix33::deserialize(reader)?;
        let records: Vec<GMSpriteYYSWFGradientRecord> = reader.read_simple_list()?;
        Ok(GMSpriteYYSWFGradientFillData {
            tpe_index,
            gradient_fill_type,
            transformation_matrix,
            records,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_i32(self.gradient_fill_type.into());
        self.tpe_index.serialize_if_gm_ver(builder, "TPE Index", (2022, 1))?;
        self.transformation_matrix.serialize(builder)?;
        builder.write_simple_list(&self.records)?;
        Ok(())
    }
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
        Ok(GMSpriteYYSWFGradientRecord { ratio, red, green, blue, alpha })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_i32(self.ratio);
        builder.write_u8(self.red);
        builder.write_u8(self.green);
        builder.write_u8(self.blue);
        builder.write_u8(self.alpha);
        Ok(())
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

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_u8(self.red);
        builder.write_u8(self.green);
        builder.write_u8(self.blue);
        builder.write_u8(self.alpha);
        Ok(())
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

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_u8(self.red);
        builder.write_u8(self.green);
        builder.write_u8(self.blue);
        builder.write_u8(self.alpha);
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMSpriteYYSWFSubshapeData {
    pub fill_style1: i32,
    pub fill_style2: i32,
    pub line_style: i32,
    pub points: Vec<(f32, f32)>,
    pub lines: Vec<(i32, i32)>,
    pub triangles: Vec<i32>,
    pub line_points: Vec<(f32, f32)>,
    pub line_triangles: Vec<i32>,
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
        let line_point_count: usize = reader.read_usize()?;
        let line_triangle_count: usize = reader.read_usize()? * 3;
        let aa_line_count: usize = reader.read_usize()?;
        let aa_vector_count: usize = reader.read_usize()?;
        let line_aa_line_count: usize = reader.read_usize()?;
        let line_aa_vector_count: usize = reader.read_usize()?;

        let mut points: Vec<(f32, f32)> = vec_with_capacity(point_count)?;
        let mut lines: Vec<(i32, i32)> = vec_with_capacity(line_count)?;
        let mut triangles: Vec<i32> = vec_with_capacity(triangle_count)?;
        let mut line_points: Vec<(f32, f32)> = vec_with_capacity(line_point_count)?;
        let mut line_triangles: Vec<i32> = vec_with_capacity(line_triangle_count)?;
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
        for _ in 0..line_point_count {
            line_points.push((reader.read_f32()?, reader.read_f32()?));
        }
        for _ in 0..line_triangle_count {
            line_triangles.push(reader.read_i32()?);
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
            line_points,
            line_triangles,
            aa_lines,
            aa_vectors,
            line_aa_lines,
            line_aa_vectors,
        })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_i32(self.fill_style1);
        builder.write_i32(self.fill_style2);
        builder.write_i32(self.line_style);

        builder.write_usize(self.points.len())?;
        builder.write_usize(self.lines.len())?;
        builder.write_usize(self.triangles.len() / 3)?;
        builder.write_usize(self.line_points.len())?;
        builder.write_usize(self.line_triangles.len() / 3)?;
        builder.write_usize(self.aa_lines.len())?;
        builder.write_usize(self.aa_vectors.len())?;
        builder.write_usize(self.line_aa_lines.len())?;
        builder.write_usize(self.line_aa_vectors.len())?;

        for (x, y) in &self.points {
            builder.write_f32(*x);
            builder.write_f32(*y);
        }
        for (x, y) in &self.lines {
            builder.write_i32(*x);
            builder.write_i32(*y);
        }
        for i in &self.triangles {
            builder.write_i32(*i);
        }
        for (x, y) in &self.line_points {
            builder.write_f32(*x);
            builder.write_f32(*y);
        }
        for i in &self.line_triangles {
            builder.write_i32(*i);
        }
        for (x, y) in &self.aa_lines {
            builder.write_i32(*x);
            builder.write_i32(*y);
        }
        for (x, y) in &self.aa_vectors {
            builder.write_f32(*x);
            builder.write_f32(*y);
        }
        for (x, y) in &self.line_aa_lines {
            builder.write_i32(*x);
            builder.write_i32(*y);
        }
        for (x, y) in &self.line_aa_vectors {
            builder.write_f32(*x);
            builder.write_f32(*y);
        }
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMSpriteYYSWFBitmapData {
    pub bitmap_type: GMSpriteYYSWFBitmapType,
    pub width: i32,
    pub height: i32,
    pub ver_data: GMSpriteYYSWFBitmapDataVer,
}
impl GMElement for GMSpriteYYSWFBitmapData {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let bitmap_type: GMSpriteYYSWFBitmapType = num_enum_from(reader.read_i32()?)?;
        let width: i32 = reader.read_i32()?;
        let height: i32 = reader.read_i32()?;

        let ver_data = if reader.general_info.is_version_at_least((2022, 1)) {
            let tpe_index: i32 = reader.read_i32()?;
            GMSpriteYYSWFBitmapDataVer::Post2022_1(GMSpriteYYSWFBitmapDataPost2022_1 { tpe_index })
        }
        else {
            let image_data_length: usize = reader.read_i32()?.max(0) as usize;
            let alpha_data_length: usize = reader.read_i32()?.max(0) as usize;
            let color_palette_data_length: usize = reader.read_i32()?.max(0) as usize;

            let image_data: Vec<u8> = reader.read_bytes_dyn(image_data_length)
                .map_err(|e| format!("Trying to read Image Data of Bitmap Data {e}"))?.to_vec();
            let alpha_data: Vec<u8> = reader.read_bytes_dyn(alpha_data_length)
                .map_err(|e| format!("Trying to read Alpha Data of Bitmap Data {e}"))?.to_vec();
            let color_palette_data: Vec<u8> = reader.read_bytes_dyn(color_palette_data_length)
                .map_err(|e| format!("Trying to read Color Palette Data of Bitmap Data {e}"))?.to_vec();

            reader.align(4)?;
            GMSpriteYYSWFBitmapDataVer::Pre2022_1(GMSpriteYYSWFBitmapDataPre2022_1 { image_data, alpha_data, color_palette_data })
        };

        Ok(GMSpriteYYSWFBitmapData { bitmap_type, width, height, ver_data })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_i32(self.bitmap_type.into());
        builder.write_i32(self.width);
        builder.write_i32(self.height);
        if builder.is_gm_version_at_least((2022, 1)) {
            if let GMSpriteYYSWFBitmapDataVer::Post2022_1(ref data) = self.ver_data {
                builder.write_i32(data.tpe_index);
            } else {
                return Err("Sprite YYSWF Bitmap Data: TPE Index not set in Post 2022.1+".to_string())
            }
        } else {
            if let GMSpriteYYSWFBitmapDataVer::Pre2022_1(ref data) = self.ver_data {
                builder.write_usize(data.image_data.len())?;
                builder.write_usize(data.alpha_data.len())?;
                builder.write_usize(data.color_palette_data.len())?;
                builder.write_bytes(&data.image_data);
                builder.write_bytes(&data.alpha_data);
                builder.write_bytes(&data.color_palette_data);
                builder.align(4);
            } else {
                return Err("Sprite YYSWF Bitmap Data: version specific data not set in Pre 2022.1+".to_string())
            }
        }
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum GMSpriteYYSWFBitmapDataVer {
    Pre2022_1(GMSpriteYYSWFBitmapDataPre2022_1),
    Post2022_1(GMSpriteYYSWFBitmapDataPost2022_1),
}
#[derive(Debug, Clone, PartialEq)]
pub struct GMSpriteYYSWFBitmapDataPre2022_1 {
    pub image_data: Vec<u8>,
    pub alpha_data: Vec<u8>,
    pub color_palette_data: Vec<u8>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct GMSpriteYYSWFBitmapDataPost2022_1 {
    pub tpe_index: i32,
}


#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
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

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_usize(self.frame_objects.len())?;
        builder.write_f32(self.min_x);
        builder.write_f32(self.max_x);
        builder.write_f32(self.min_y);
        builder.write_f32(self.max_y);
        for frame_object in &self.frame_objects {
            frame_object.serialize(builder)?;
        }
        Ok(())
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
        let transformation_matrix = GMSpriteYYSWFMatrix33::deserialize(reader)?;

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

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        builder.write_i32(self.char_id);
        builder.write_i32(self.char_index);
        builder.write_i32(self.depth);
        builder.write_i32(self.clipping_depth);
        self.color_matrix.serialize(builder)?;
        builder.write_f32(self.min_x);
        builder.write_f32(self.max_x);
        builder.write_f32(self.min_y);
        builder.write_f32(self.max_y);
        self.transformation_matrix.serialize(builder)?;
        Ok(())
    }
}


pub const YYSWF_COLOR_MATRIX_SIZE: usize = 4;
#[derive(Debug, Clone, PartialEq)]
pub struct GMSpriteYYSWFColorMatrix {
    pub additive: [i32; YYSWF_COLOR_MATRIX_SIZE],
    pub multiply: [i32; YYSWF_COLOR_MATRIX_SIZE],
}
impl GMElement for GMSpriteYYSWFColorMatrix {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let mut additive = [0i32; YYSWF_COLOR_MATRIX_SIZE];
        for item in &mut additive {
            *item = reader.read_i32()?;
        }

        let mut multiply = [0i32; YYSWF_COLOR_MATRIX_SIZE];
        for item in &mut multiply {
            *item = reader.read_i32()?;
        }

        Ok(GMSpriteYYSWFColorMatrix { additive, multiply })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        for item in &self.additive {
            builder.write_i32(*item);
        }
        for item in &self.multiply {
            builder.write_i32(*item);
        }
        Ok(())
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct GMSpriteYYSWFCollisionMask {
    pub rle_data: Vec<u8>,
}
impl GMElement for GMSpriteYYSWFCollisionMask {
    fn deserialize(reader: &mut DataReader) -> Result<Self, String> {
        let rle_length: i32 = reader.read_i32()?;
        let rle_length: usize = if rle_length > 0 {rle_length as usize} else {0};
        let rle_data: Vec<u8> = reader.read_bytes_dyn(rle_length).map_err(|e| format!("Trying to read RLE Data of Timeline {e}"))?.to_vec();
        reader.align(4)?;    // [From UndertaleModTool] "why it's not aligned before the data is beyond my brain"
        Ok(Self { rle_data })
    }

    fn serialize(&self, builder: &mut DataBuilder) -> Result<(), String> {
        // From UTMT: writing zero for empty table would probably be smart but the padding handles it automatically?
        //            but you cant even have a yyswf sprite with a null rle data???
        if self.rle_data.len() != 0 {
            builder.write_usize(self.rle_data.len())?;
            builder.write_bytes(&self.rle_data);
        }
        builder.align(4);
        Ok(())
    }
}

