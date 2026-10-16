CREATE TABLE cellar_data (
    id bigint PRIMARY KEY,
    population integer NOT NULL DEFAULT 500
);

INSERT INTO cellar_data (id, population)
VALUES (0, 500)
ON CONFLICT (id)
DO UPDATE SET population = EXCLUDED.population;
