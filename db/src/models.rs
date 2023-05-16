use crate::schema::{books, reviews};
use diesel::prelude::*;
use speedy::{Readable, Writable};
use std::{str::FromStr, time::SystemTime};

#[derive(Clone, Copy, Debug, diesel_derive_enum::DbEnum, Readable, Writable, PartialEq, Eq)]
#[ExistingTypePath = "crate::schema::sql_types::Lang"]
pub enum Lang {
    English,
    Russian,
    Ukrainian,
    German,
    Chinese,
    Japanese,
}

impl Lang {
    pub fn to_str(self) -> &'static str {
        match self {
            Self::English => "English",
            Self::Russian => "Russian",
            Self::Ukrainian => "Ukrainian",
            Self::German => "German",
            Self::Chinese => "Chinese",
            Self::Japanese => "Japanese",
        }
    }
}

impl FromStr for Lang {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "English" => Ok(Self::English),
            "Russian" => Ok(Self::Russian),
            "Ukrainian" => Ok(Self::Ukrainian),
            "German" => Ok(Self::German),
            "Chinese" => Ok(Self::Chinese),
            "Japanese" => Ok(Self::Japanese),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Queryable, Readable, Writable)]
pub struct Book {
    pub isbn: i64,
    pub title: String,
    pub author: String,
    pub description: String,
    pub language: Lang,
    pub issue_year: i32,
}

#[derive(Insertable)]
#[diesel(table_name = books)]
pub struct NewBook<'a> {
    pub isbn: i64,
    pub title: &'a str,
    pub author: &'a str,
    pub description: &'a str,
    pub language: Lang,
    pub issue_year: i32,
}

#[derive(Clone, Copy, Debug, diesel_derive_enum::DbEnum, Readable, Writable, PartialEq, Eq)]
#[ExistingTypePath = "crate::schema::sql_types::Rating"]
pub enum Rating {
    One,
    Two,
    Three,
    Four,
    Five,
}

#[derive(Debug, Queryable, Readable, Writable)]
pub struct Review {
    pub isbn: i64,
    pub username: String,
    pub rating: Rating,
    pub description: String,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

#[derive(Insertable)]
#[diesel(table_name = reviews)]
pub struct NewReview<'a> {
    pub isbn: i64,
    pub username: &'a str,
    pub rating: Rating,
    pub description: &'a str,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

#[derive(Readable, Writable)]
pub struct NewReviewPart<'a> {
    pub isbn: i64,
    pub username: &'a str,
    pub rating: Rating,
    pub description: &'a str,
}
