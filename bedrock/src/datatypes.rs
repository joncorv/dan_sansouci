use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

// NOTE: Image Size Reference
// md: 700 px max width
// lg: 1600 px max width
// xl: 2500 px longest edge
// vt: 1200 x 2,400 for mobile

pub enum Size {
    Md,
    Lg,
    Xl,
    Vt,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Slideshow {
    id: Uuid,
    idx: i32,

    img_md: String,
    img_md_width: i32,
    img_md_height: i32,

    img_lg: Option<String>,
    img_lg_width: Option<i32>,
    img_lg_height: Option<i32>,

    img_xl: Option<String>,
    img_xl_width: Option<i32>,
    img_xl_height: Option<i32>,

    img_vt: Option<String>,
    img_vt_width: Option<i32>,
    img_vt_height: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Category {
    pub category_id: i32,
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

    pub img_vt: Option<String>,
    pub img_vt_width: Option<i32>,
    pub img_vt_height: Option<i32>,
}

pub struct Painting {
    id: Uuid,
    category_id: i32,
    idx: i32,
    name: String,
    medium: String,
    dimensions: String,

    img_md: String,
    img_md_width: i32,
    img_md_height: i32,

    img_lg: Option<String>,
    img_lg_width: Option<i32>,
    img_lg_height: Option<i32>,

    img_xl: Option<String>,
    img_xl_width: Option<i32>,
    img_xl_height: Option<i32>,

    img_vt: Option<String>,
    img_vt_width: Option<i32>,
    img_vt_height: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
struct PaintingDescription {
    path: String,
    idx: i32,
    name: String,
    medium: String,
    category_id: i32,
}
