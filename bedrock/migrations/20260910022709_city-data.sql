CREATE TABLE categories (
    category_id integer PRIMARY KEY,
    idx integer UNIQUE,
    name text,
    src_img text
);

CREATE TABLE photos (
    id uuid PRIMARY KEY,
    idx integer,
    name text NOT NULL,
    src_thumb text,
    src_img text,
    category_id integer REFERENCES categories(category_id) ON DELETE CASCADE
);

CREATE TABLE slideshow (
    id uuid PRIMARY KEY,
    idx integer,
    src_img text
);

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
