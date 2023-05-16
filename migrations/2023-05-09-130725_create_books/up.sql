CREATE TYPE lang AS ENUM ('english', 'russian', 'ukrainian', 'german', 'chinese', 'japanese');
CREATE TABLE books (
    isbn bigint primary key,
    title text not null,
    author text not null,
    description text not null,
    language lang not null,
    issue_year int not null
);
CREATE TYPE rating as ENUM ('one', 'two', 'three', 'four', 'five');
CREATE TABLE reviews (
    isbn bigint not null references books(isbn),
    username varchar(16) not null,
    primary key (isbn, username),
    rating rating not null,
    description text not null,
    created_at timestamp not null,
    updated_at timestamp not null
);
