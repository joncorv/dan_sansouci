
-----------------------------------------------------------
------------------------ Create Tables --------------------
-----------------------------------------------------------
CREATE TABLE categories (
    category_id integer PRIMARY KEY UNIQUE,
    idx integer UNIQUE,
    name text,
    src_img text
);
CREATE TABLE paintings (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    category_id integer REFERENCES categories(category_id) ON DELETE CASCADE,
    idx integer,
    name text NOT NULL,
    medium text,
    src_thumb text,
    src_thumb_width integer,
    src_thumb_height integer,
    src_img text,
    src_img_width integer,
    src_img_height integer
);
CREATE TABLE slideshow (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    idx integer,
    src_img text,
    width integer,
    height integer
);

-----------------------------------------------------------
------------------------ Create Data --------------------
-----------------------------------------------------------
INSERT INTO categories (category_id, idx, name) 
VALUES
  (0, 0, 'Wildlife'),
  (1, 1, 'Day of the Dead'),
  (2, 2, 'Landscapes & Cityscapes'),
  (3, 3, 'Figurative & Portraits'),
  (4, 4, 'Illustrations'),
  (5, 5, 'Book Art')
ON CONFLICT (category_id) 
DO UPDATE set
  category_id = EXCLUDED.category_id,
  idx = EXCLUDED.idx,
  name = EXCLUDED.name;

INSERT INTO slideshow (idx, src_img)
VALUES 
    (0, '/public/slideshow/slideshow_01.jpg'),
    (1, '/public/slideshow/slideshow_02.jpg'),
    (2, '/public/slideshow/slideshow_03.jpg');
-- ON CONFLICT (idx)
-- DO UPDATE set 
--   idx = EXCLUDED.idx,
--   src_img = EXCLUDED.src_img;

-- INSERT INTO paintings (idx, name, src_thumb, src_img, category_id)
-- VALUES
--     (0, 'Photo 1', 'public/dummy_thumb.jpg', '/public/dummy_image.jpg', 0),
--     (1, 'Photo 2', 'public/dummy_thumb.jpg', '/public/dummy_image.jpg', 0),
--     (2, 'Photo 3', 'public/dummy_thumb.jpg', '/public/dummy_image.jpg', 0),
--     (3, 'Photo 1', 'public/dummy_thumb.jpg', '/public/dummy_image.jpg', 1),
--     (4, 'Photo 2', 'public/dummy_thumb.jpg', '/public/dummy_image.jpg', 1),
--     (5, 'Photo 3', 'public/dummy_thumb.jpg', '/public/dummy_image.jpg', 1);
-- ON CONFLICT (idx)
-- DO UPDATE set
--   idx = EXCLUDED.idx,
--   name = EXCLUDED.name,
--   src_thumb = EXCLUDED.src_thumb,
--   src_img = EXCLUDED.src_img,
--   category_id = EXCLUDED.category_id;


