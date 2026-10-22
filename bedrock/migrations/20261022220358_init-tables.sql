-----------------------------------------------------------
------------------------ Create Tables --------------------
-----------------------------------------------------------
CREATE TABLE categories (
    category_id integer PRIMARY KEY UNIQUE,
    idx integer UNIQUE NOT NULL,
    name text NOT NULL,
    src_img text NOT NULL
);
CREATE TABLE paintings (
    id uuid PRIMARY KEY,
    category_id integer NOT NULL REFERENCES categories(category_id) ON DELETE CASCADE,
    idx integer NOT NULL,
    name text NOT NULL,
    medium text NOT NULL,
    src_thumb text NOT NULL,
    src_thumb_width integer NOT NULL,
    src_thumb_height integer NOT NULL,
    src_img text NOT NULL,
    src_img_width integer NOT NULL,
    src_img_height integer NOT NULL
);
CREATE TABLE slideshow (
    id uuid PRIMARY KEY,
    idx integer NOT NULL,
    src_img text NOT NULL,
    width integer NOT NULL,
    height integer NOT NULL
);
