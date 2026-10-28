use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

// NOTE: Image Size Reference
// md: 700 px max width
// lg: 1,600 px max width
// xl: 3,200 px longest edge
// vt: 1,200 x 2,400 for mobile

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

    img_lg: String,
    img_lg_width: i32,
    img_lg_height: i32,

    img_xl: String,
    img_xl_width: i32,
    img_xl_height: i32,

    img_vt: String,
    img_vt_width: i32,
    img_vt_height: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Category {
    category_id: i32,
    idx: i32,
    name: String,

    img_md: String,
    img_md_width: i32,
    img_md_height: i32,

    img_lg: String,
    img_lg_width: i32,
    img_lg_height: i32,

    img_xl: String,
    img_xl_width: i32,
    img_xl_height: i32,

    img_vt: String,
    img_vt_width: i32,
    img_vt_height: i32,
}

pub struct Painting {
    id: Uuid,
    category_id: i32,
    idx: i32,
    name: String,
    medium: String,

    img_md: String,
    img_md_width: i32,
    img_md_height: i32,

    img_lg: String,
    img_lg_width: i32,
    img_lg_height: i32,

    img_xl: Option<String>,
    img_xl_width: Option<i32>,
    img_xl_height: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
struct PaintingDescription {
    path: String,
    idx: i32,
    name: String,
    medium: String,
    category_id: i32,
}
