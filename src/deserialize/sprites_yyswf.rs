

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
    pub used_items: UTSpriteYYSWFItem,
    pub frames: UTSpriteYYSWFTimelineFrame,
    pub collision_masks: UTSpriteYYSWFCollisionMask,
}
#[derive(Debug, Clone)]
pub struct UTSpriteYYSWFItem {
    pub id: i32,
    pub item_type: UTSpriteYYSWFItemType,
    pub shape_data: UTSpriteYYSWFShapeData,
    pub bitmap_data: UTSpriteYYSWFBitmapData,
}
#[derive(Debug, Clone)]
pub enum UTSpriteYYSWFItemType {
    ItemInvalid,
    ItemShape,
    ItemBitmap,
    ItemFont,
    ItemTextField,
    ItemSprite,
}
#[derive(Debug, Clone)]
pub enum UTSpriteYYSWFFillType {
    FillInvalid,
    FillSolid,
    FillGradient,
    FillBitmap,
}
#[derive(Debug, Clone)]
pub enum UTSpriteYYSWFBitmapFillType {
    FillRepeat,
    FillClamp,
    FillRepeatPoint,
    FillClampPoint,
}
#[derive(Debug, Clone)]
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
    pub fill_styles: UTSpriteYYSWFFillData,
    pub line_styles: UTSpriteYYSWFLineStyleData,
    pub subshapes: UTSpriteYYSWFSubshapeData,
}

#[derive(Debug, Clone)]
pub struct UTSpriteYYSWFFillData {
    pub type_: UTSpriteYYSWFFillType,
    pub bitmap_fill_data: UTSpriteYYSWFBitmapFillData,
    pub gradient_fill_data: UTSpriteYYSWFGradientFillData,
    pub solid_fill_data: UTSpriteYYSWFSolidFillData,
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
    type_: UTSpriteYYSWFBitmapType,
    width: i32,
    height: i32,
    tpe_index: Option<i32>,
    image_data: Vec<u8>,
    alpha_data: Vec<u8>,
    color_palette_data: Vec<u8>,
}

#[derive(Debug, Clone)]
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

