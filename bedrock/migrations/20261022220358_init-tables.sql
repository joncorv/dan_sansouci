-----------------------------------------------------------
------------------------ Create Tables --------------------
-----------------------------------------------------------
CREATE TABLE slideshow (
    id UUID PRIMARY KEY,
    idx INTEGER NOT NULL,

    img_md TEXT NOT NULL,
    img_md_width INTEGER NOT NULL,
    img_md_height INTEGER NOT NULL,

    img_lg TEXT,
    img_lg_width INTEGER,
    img_lg_height INTEGER,

    img_xl TEXT,
    img_xl_width INTEGER,
    img_xl_height INTEGER,

    img_vt TEXT,
    img_vt_width INTEGER,
    img_vt_height INTEGER
);

CREATE TABLE categories (
    category_id INTEGER PRIMARY KEY UNIQUE,
    idx INTEGER UNIQUE NOT NULL,
    name TEXT NOT NULL,

    img_md TEXT NOT NULL,
    img_md_width INTEGER NOT NULL,
    img_md_height INTEGER NOT NULL,

    img_lg TEXT,
    img_lg_width INTEGER,
    img_lg_height INTEGER,

    img_xl TEXT,
    img_xl_width INTEGER,
    img_xl_height INTEGER,

    img_vt TEXT,
    img_vt_width INTEGER,
    img_vt_height INTEGER
);

CREATE TABLE paintings (
    id UUID PRIMARY KEY,
    category_id INTEGER NOT NULL REFERENCES categories(category_id) ON DELETE CASCADE,
    idx INTEGER NOT NULL,
    name TEXT NOT NULL,
    medium TEXT NOT NULL,
    dimensions TEXT NOT NULL,

    img_md TEXT NOT NULL,
    img_md_width INTEGER NOT NULL,
    img_md_height INTEGER NOT NULL,

    img_lg TEXT,
    img_lg_width INTEGER,
    img_lg_height INTEGER,

    img_xl TEXT,
    img_xl_width INTEGER,
    img_xl_height INTEGER 
);
