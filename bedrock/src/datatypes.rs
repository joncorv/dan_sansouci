use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

trait ImageProcessing {
    fn sizes_needed(&self) -> (bool, bool, bool, bool);
    fn resizes_needed(&self) -> (bool, bool, bool, bool);
}

// NOTE: Image Size Reference
// md: 700 px max width
// lg: 1600 px max width
// xl: 2500 px longest edge
// vt: 1200 x 2,400 for mobile

#[derive(Debug)]
pub enum Size {
    Md,
    Lg,
    Xl,
    Vt,
}

#[derive(Debug, Default)]
pub enum ImageType {
    #[default]
    Category,
    Slideshow,
    Painting,
}

#[derive(Debug, Default)]
pub struct ImageMetadata {
    pub image_type: ImageType,

    pub id: Uuid,
    pub idx: i32,
    pub name: String,
    pub medium: String,
    pub dimensions: String,

    pub img_md: String,
    pub img_md_width: i32,
    pub img_md_height: i32,

    pub img_lg: Option<String>,
    pub img_lg_width: Option<i32>,
    pub img_lg_height: Option<i32>,

    pub img_xl: Option<String>,
    pub img_xl_width: Option<i32>,
    pub img_xl_height: Option<i32>,

    pub img_vt: String,
    pub img_vt_width: i32,
    pub img_vt_height: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SlideshowSchema {
    pub id: Uuid,
    pub idx: i32,

    pub img_md: String,
    pub img_md_width: i32,
    pub img_md_height: i32,

    pub img_lg: Option<String>,
    pub img_lg_width: Option<i32>,
    pub img_lg_height: Option<i32>,

    pub img_xl: Option<String>,
    pub img_xl_width: Option<i32>,
    pub img_xl_height: Option<i32>,

    pub img_vt: String,
    pub img_vt_width: i32,
    pub img_vt_height: i32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct CategorySchema {
    pub category_id: Uuid,
    pub idx: i32,
    pub name: String,

    pub img_md: String,
    pub img_md_width: i32,
    pub img_md_height: i32,

    pub img_lg: Option<String>,
    pub img_lg_width: Option<i32>,
    pub img_lg_height: Option<i32>,

    pub img_xl: Option<String>,
    pub img_xl_width: Option<i32>,
    pub img_xl_height: Option<i32>,

    pub img_vt: String,
    pub img_vt_width: i32,
    pub img_vt_height: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PaintingSchema {
    pub id: Uuid,
    pub category_id: Uuid,
    pub idx: i32,
    pub name: String,
    pub medium: String,
    pub dimensions: String,

    pub img_md: String,
    pub img_md_width: i32,
    pub img_md_height: i32,

    pub img_lg: Option<String>,
    pub img_lg_width: Option<i32>,
    pub img_lg_height: Option<i32>,

    pub img_xl: Option<String>,
    pub img_xl_width: Option<i32>,
    pub img_xl_height: Option<i32>,
}
