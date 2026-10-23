export interface Category {
  category_id: number;
  idx: number;
  name: string;
  src_img: string;
}

export interface Painting {
  id: string;
  category_id: number;
  idx: number;
  name: string;
  medium: string;
  src_thumb: string;
  src_thumb_width: number;
  src_thumb_height: number;
  src_img: string;
  src_img_width: number;
  src_img_height: number;
}

export interface Slideshow {
  id: string;
  idx: number;
  src_img: string;
  width: number;
  height: number;
}
