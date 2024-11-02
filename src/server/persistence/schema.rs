// @generated automatically by Diesel CLI.

diesel::table! {
    blog_post (id) {
        id -> Integer,
        posted_on -> Date,
        username -> Text,
        text -> Text,
        image_uuid -> Nullable<Text>,
        avatar_uuid -> Nullable<Text>,
    }
}
