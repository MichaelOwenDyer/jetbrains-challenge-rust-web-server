CREATE TABLE IF NOT EXISTS blog_post
(
    id          INTEGER     PRIMARY KEY AUTOINCREMENT   NOT NULL,
    posted_on   DATE                                    NOT NULL,
    username    VARCHAR(15)                             NOT NULL,
    text        TEXT                                    NOT NULL,
    image_uuid  TEXT,
    avatar_uuid TEXT
);
