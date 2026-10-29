export interface Slideshow {
  id: string;
  idx: number;

  img_md: string;
  img_md_width: number;
  img_md_height: number;

  img_lg: string | null;
  img_lg_width: number | null;
  img_lg_height: number | null;

  img_xl: string | null;
  img_xl_width: number | null;
  img_xl_height: number | null;

  img_vt: string;
  img_vt_width: number;
  img_vt_height: number;
}

export interface Category {
  category_id: number;
  idx: number;
  name: string;

  img_md: string;
  img_md_width: number;
  img_md_height: number;

  img_lg: string | null;
  img_lg_width: number | null;
  img_lg_height: number | null;

  img_xl: string | null;
  img_xl_width: number | null;
  img_xl_height: number | null;

  img_vt: string | null;
  img_vt_width: number | null;
  img_vt_height: number | null;
}

export interface Painting {
  id: string;
  category_id: number;
  idx: number;
  name: string;
  medium: string;
  dimensions: string;

  img_md: string;
  img_md_width: number;
  img_md_height: number;

  img_lg: string | null;
  img_lg_width: number | null;
  img_lg_height: number | null;

  img_xl: string | null;
  img_xl_width: number | null;
  img_xl_height: number | null;
}
