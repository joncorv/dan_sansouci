use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
struct PaintingCategory {
    id: Uuid,
    index: u32,
    name: String,
    thumbnail: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Painting {
    id: Uuid,
    index: u32,
    name: String,
    medium: String,
    src_hq: String,
    src_thumb: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct LandingImages {
    src_1: String,
    src_2: String,
    src_3: String,
}

async fn get_landing_images() {}

async fn get_painting_categories() {}

async fn get_paintings_by_index() {}
