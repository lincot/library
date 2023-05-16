use diesel::{pg::PgConnection, prelude::*};
use models::{Book, NewBook, NewReview, Rating, Review};
use schema::{books, reviews};
use std::{env, time::SystemTime};

pub mod models;
pub mod schema;

pub fn establish_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn create_book(
    conn: &mut PgConnection,
    book: &NewBook,
) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(books::table).values(book).execute(conn)
}

pub fn delete_book(conn: &mut PgConnection, isbn: i64) -> Result<usize, diesel::result::Error> {
    diesel::delete(books::table.filter(books::isbn.eq(isbn))).execute(conn)
}

pub fn load_books(conn: &mut PgConnection) -> Result<Vec<Book>, diesel::result::Error> {
    books::table.load::<Book>(conn)
}

pub fn get_book(conn: &mut PgConnection, isbn: i64) -> Result<Book, diesel::result::Error> {
    books::table
        .filter(books::isbn.eq(isbn))
        .first::<Book>(conn)
}

pub fn create_review(
    conn: &mut PgConnection,
    review: &NewReview,
) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(reviews::table)
        .values(review)
        .execute(conn)
}

pub fn get_reviews_by_book(
    conn: &mut PgConnection,
    isbn: i64,
) -> Result<Vec<Review>, diesel::result::Error> {
    reviews::table
        .filter(reviews::isbn.eq(isbn))
        .load::<Review>(conn)
}

pub fn get_reviews_by_username(
    conn: &mut PgConnection,
    username: &str,
) -> Result<Vec<Review>, diesel::result::Error> {
    reviews::table
        .filter(reviews::username.eq(username))
        .load::<Review>(conn)
}

pub fn update_review(
    conn: &mut PgConnection,
    isbn: i64,
    username: &str,
    description: &str,
    rating: Rating,
    updated_at: SystemTime,
) -> Result<usize, diesel::result::Error> {
    diesel::update(reviews::table.find((isbn, username)))
        .set((
            reviews::description.eq(description),
            reviews::rating.eq(rating),
            reviews::updated_at.eq(updated_at),
        ))
        .execute(conn)
}

pub fn delete_review(
    conn: &mut PgConnection,
    isbn: i64,
    username: &str,
) -> Result<usize, diesel::result::Error> {
    diesel::delete(reviews::table.find((isbn, username))).execute(conn)
}
