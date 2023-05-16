// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "lang"))]
    pub struct Lang;

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "rating"))]
    pub struct Rating;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Lang;

    books (isbn) {
        isbn -> Int8,
        title -> Text,
        author -> Text,
        description -> Text,
        language -> Lang,
        issue_year -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Rating;

    reviews (isbn, username) {
        isbn -> Int8,
        username -> Varchar,
        rating -> Rating,
        description -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(reviews -> books (isbn));

diesel::allow_tables_to_appear_in_same_query!(
    books,
    reviews,
);
