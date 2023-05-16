use actix_web::{
    delete, get, post, put,
    web::{self, Bytes},
    App, HttpResponse, HttpServer,
};
use db::models::{NewReview, NewReviewPart};
use diesel::{r2d2::ConnectionManager, PgConnection};
use dotenvy::dotenv;
use r2d2::Pool;
use speedy::{Readable, Writable};
use std::{env, io, time::SystemTime};

type DbPool = Pool<ConnectionManager<PgConnection>>;

#[post("/reviews")]
async fn post_review(pool: web::Data<DbPool>, body: Bytes) -> HttpResponse {
    let mut conn = pool.get().unwrap();
    let created_at = SystemTime::now();
    let NewReviewPart {
        isbn,
        username,
        rating,
        description,
    } = NewReviewPart::read_from_buffer(&body).unwrap();
    let review = NewReview {
        isbn,
        username,
        rating,
        description,
        created_at,
        updated_at: created_at,
    };
    db::create_review(&mut conn, &review).unwrap();
    HttpResponse::Ok().into()
}

#[get("/reviews/book/{isbn}")]
async fn get_reviews_by_book(pool: web::Data<DbPool>, isbn: web::Path<i64>) -> Vec<u8> {
    let isbn = isbn.into_inner();
    let mut conn = pool.get().unwrap();
    let reviews = db::get_reviews_by_book(&mut conn, isbn).unwrap();
    reviews.write_to_vec().unwrap()
}

#[get("/reviews/user/{username}")]
async fn get_reviews_by_username(pool: web::Data<DbPool>, username: web::Path<String>) -> Vec<u8> {
    let username = username.into_inner();
    let mut conn = pool.get().unwrap();
    let reviews = db::get_reviews_by_username(&mut conn, &username).unwrap();
    reviews.write_to_vec().unwrap()
}

#[put("/reviews")]
async fn update_review(pool: web::Data<DbPool>, body: Bytes) -> HttpResponse {
    let mut conn = pool.get().unwrap();
    let updated_at = SystemTime::now();
    let NewReviewPart {
        isbn,
        username,
        rating,
        description,
    } = NewReviewPart::read_from_buffer(&body).unwrap();
    db::update_review(&mut conn, isbn, username, description, rating, updated_at).unwrap();
    HttpResponse::Ok().into()
}

#[delete("/reviews/{isbn}/{username}")]
async fn delete_review<'a>(
    pool: web::Data<DbPool>,
    path: web::Path<(i64, String)>,
) -> HttpResponse {
    let mut conn = pool.get().unwrap();
    let (isbn, username) = path.into_inner();
    db::delete_review(&mut conn, isbn, &username).unwrap();
    HttpResponse::Ok().into()
}

fn config(cfg: &mut web::ServiceConfig) {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = DbPool::new(ConnectionManager::new(db_url)).expect("Failed to create db pool");
    cfg.app_data(web::Data::new(pool.clone()))
        .service(post_review)
        .service(get_reviews_by_book)
        .service(get_reviews_by_username)
        .service(update_review)
        .service(delete_review);
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    HttpServer::new(|| App::new().configure(config))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{
        test::{self, call_and_read_body, TestRequest},
        App,
    };
    use db::models::{Rating, Review};
    use std::time::Duration;

    #[actix_web::test]
    async fn api_test() {
        let mut app = test::init_service(App::new().configure(config)).await;
        let (isbn, username, rating, description) =
            (9_780_747_542_155, "anon", Rating::One, "really good book");

        let resp = TestRequest::post()
            .uri("/reviews")
            .set_payload(
                NewReviewPart {
                    isbn,
                    username,
                    rating,
                    description,
                }
                .write_to_vec()
                .unwrap(),
            )
            .send_request(&mut app)
            .await;
        assert!(resp.status().is_success());

        let req = TestRequest::get()
            .uri(&format!("/reviews/book/{isbn}"))
            .to_request();
        let resp = call_and_read_body(&app, req).await;
        let reviews = Vec::<Review>::read_from_buffer(&resp).unwrap();
        assert_eq!(reviews.len(), 1);
        assert_eq!(reviews[0].isbn, isbn);
        assert_eq!(reviews[0].username, username);
        assert_eq!(reviews[0].rating, rating);
        assert_eq!(reviews[0].description, description);
        let now = SystemTime::now();
        assert!(now.duration_since(reviews[0].created_at).unwrap() < Duration::from_secs(1));
        assert!(now.duration_since(reviews[0].updated_at).unwrap() < Duration::from_secs(1));

        let rating = Rating::Five;
        let resp = TestRequest::put()
            .uri("/reviews")
            .set_payload(
                NewReviewPart {
                    isbn,
                    username,
                    rating,
                    description,
                }
                .write_to_vec()
                .unwrap(),
            )
            .send_request(&mut app)
            .await;
        assert!(resp.status().is_success());

        let req = TestRequest::get()
            .uri(&format!("/reviews/user/{username}"))
            .to_request();
        let resp = call_and_read_body(&app, req).await;
        let reviews = Vec::<Review>::read_from_buffer(&resp).unwrap();
        assert_eq!(reviews.len(), 1);
        assert_eq!(reviews[0].isbn, isbn);
        assert_eq!(reviews[0].username, username);
        assert_eq!(reviews[0].rating, rating);
        assert_eq!(reviews[0].description, description);

        let resp = TestRequest::delete()
            .uri(&format!("/reviews/{isbn}/{username}"))
            .send_request(&mut app)
            .await;
        assert!(resp.status().is_success());
    }
}
