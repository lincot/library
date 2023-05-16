use db::{
    create_book, delete_book, establish_connection, get_book, load_books,
    models::{Book, NewBook},
};
use diesel::pg::PgConnection;
use eframe::{
    egui::{
        CentralPanel, Color32, ColorImage, Context, Grid, Label, ScrollArea, TextureHandle, Ui,
        Widget,
    },
    App, Frame,
};
use std::{
    fs::copy,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

fn load_image<P: AsRef<Path>>(path: P) -> Result<ColorImage, image::ImageError> {
    let image = image::io::Reader::open(path)?
        .with_guessed_format()?
        .decode()?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()))
}

fn parse_isbn(s: &str) -> Option<i64> {
    let s: [u8; 13] = s.as_bytes().try_into().ok()?;
    (s.iter().all(|b| b.is_ascii_digit())
        && s.iter()
            .enumerate()
            .map(|(i, b)| if i % 2 == 0 { 1 } else { 3 } * (b - b'0'))
            .sum::<u8>()
            % 10
            == 0)
        .then(|| {
            (0..13)
                .map(|i| 10i64.pow(12 - i as u32) * (s[i] - b'0') as i64)
                .sum()
        })
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Create,
    Read,
    Update,
    Delete,
}

pub struct Library {
    connection: PgConnection,
    tab: Tab,
    isbn: String,
    title: String,
    author: String,
    description: String,
    language: String,
    issue_year: String,
    cover_path: Option<PathBuf>,
    book_path: Option<PathBuf>,
    book_created_label_end: Instant,
    book_deleted_label_end: Instant,
    book_creation_failed_error: Option<diesel::result::Error>,
    book_find_failed_error: Option<diesel::result::Error>,
    book_deletion_failed_error: Option<diesel::result::Error>,
    update_instead_of_create: bool,
    books: Option<(Vec<Book>, Vec<TextureHandle>)>,
}

impl Default for Library {
    fn default() -> Self {
        Self {
            connection: establish_connection(),
            tab: Tab::Create,
            isbn: String::with_capacity(13),
            title: String::with_capacity(64),
            author: String::with_capacity(64),
            description: String::with_capacity(1024),
            language: String::with_capacity(16),
            issue_year: String::with_capacity(4),
            cover_path: None,
            book_path: None,
            book_created_label_end: Instant::now(),
            book_deleted_label_end: Instant::now(),
            book_creation_failed_error: None,
            book_find_failed_error: None,
            book_deletion_failed_error: None,
            update_instead_of_create: false,
            books: None,
        }
    }
}

impl Library {
    fn create_tab(&mut self, ui: &mut Ui) {
        let now = Instant::now();
        let isbn = parse_isbn(&self.isbn);
        let lang = self.language.parse();
        let year = self.issue_year.parse();
        let mut button_enabled = true;
        let checks = [
            isbn.is_some(),
            !self.title.is_empty(),
            !self.author.is_empty(),
            lang.is_ok(),
            year.is_ok(),
            !self.description.is_empty(),
        ];
        Grid::new("grid_of_inputs").show(ui, |ui| {
            for ((label, var, singleline), check) in [
                ("ISBN-13", &mut self.isbn, true),
                ("title", &mut self.title, true),
                ("author", &mut self.author, true),
                ("language", &mut self.language, true),
                ("issue year", &mut self.issue_year, true),
                ("description", &mut self.description, false),
            ]
            .into_iter()
            .zip(checks)
            {
                let label = if check {
                    ui.label(label)
                } else {
                    button_enabled = false;
                    ui.colored_label(ui.visuals().error_fg_color, label)
                };
                if singleline {
                    ui.text_edit_singleline(var)
                } else {
                    ui.text_edit_multiline(var)
                }
                .labelled_by(label.id);
                ui.end_row();
            }
            for (label, path_var) in [
                ("cover", &mut self.cover_path),
                ("book file", &mut self.book_path),
            ] {
                let label = if path_var.is_some() {
                    ui.label(label)
                } else {
                    button_enabled = false;
                    ui.colored_label(ui.visuals().error_fg_color, label)
                };
                ui.horizontal(|ui| {
                    if ui.button("open file...").labelled_by(label.id).clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            *path_var = Some(path);
                        }
                    }
                    if let Some(path) = path_var {
                        ui.label(path.to_str().unwrap_or("???"));
                    }
                });
                ui.end_row();
            }
            ui.label("");
            ui.label("                                                                                        ");
            ui.end_row();
        });
        ui.horizontal(|ui| {
            ui.scope(|ui| {
                ui.set_enabled(button_enabled);
                if ui
                    .button(if self.update_instead_of_create {
                        "update book"
                    } else {
                        "create book"
                    })
                    .clicked()
                {
                    let isbn = isbn.unwrap();
                    let mut failed_to_delete = false;
                    if self.update_instead_of_create {
                        match delete_book(&mut self.connection, isbn) {
                            Err(e) => {
                                self.book_created_label_end = now;
                                self.book_creation_failed_error = Some(e);
                                failed_to_delete = true;
                            }
                            Ok(0) => {
                                self.book_created_label_end = now;
                                self.book_creation_failed_error =
                                    Some(diesel::result::Error::NotFound);
                                failed_to_delete = true;
                            }
                            _ => {
                                self.book_created_label_end = now + Duration::from_secs(3);
                                self.book_creation_failed_error = None;
                            }
                        }
                        self.update_instead_of_create = false;
                    }

                    if !failed_to_delete {
                        let from = self.cover_path.as_ref().unwrap();
                        let to: PathBuf = format!("covers/{}", self.isbn).into();
                        if from != &to {
                            copy(from, to).unwrap();
                        }
                        let from = self.book_path.as_ref().unwrap();
                        let to: PathBuf = format!("books/{}", self.isbn).into();
                        if from != &to {
                            copy(from, to).unwrap();
                        }
                        if let Err(e) = create_book(
                            &mut self.connection,
                            &NewBook {
                                isbn,
                                title: &self.title,
                                author: &self.author,
                                description: &self.description,
                                language: lang.unwrap(),
                                issue_year: year.unwrap(),
                            },
                        ) {
                            self.book_created_label_end = now;
                            self.book_creation_failed_error = Some(e);
                        } else {
                            self.book_created_label_end = now + Duration::from_secs(3);
                            self.book_creation_failed_error = None;
                        }
                    }
                }
            });
            if self.book_created_label_end > now {
                ui.colored_label(Color32::from_rgb(119, 221, 119), "book created!");
            }
            if let Some(e) = &self.book_creation_failed_error {
                ui.colored_label(
                    ui.visuals().error_fg_color,
                    format!("book failed to create: {e}"),
                );
            }
        });
    }

    fn read_tab(&mut self, ui: &mut Ui) {
        if self.books.is_none() {
            match load_books(&mut self.connection) {
                Ok(books) => {
                    let mut texture_handles = Vec::with_capacity(books.capacity());
                    for book in &books {
                        texture_handles.push(ui.ctx().load_texture(
                            "cover",
                            load_image(&format!("covers/{}", book.isbn)).unwrap(),
                            Default::default(),
                        ));
                    }
                    self.books = Some((books, texture_handles));
                }
                Err(e) => {
                    ui.colored_label(
                        ui.visuals().error_fg_color,
                        format!("failed to load books: {e}"),
                    );
                    return;
                }
            }
        };
        let (books, texture_handles) = self.books.as_ref().unwrap();
        ScrollArea::vertical().show(ui, |ui| {
            for ((id, book), texture) in (1337..).zip(books).zip(texture_handles) {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.image(texture, texture.size_vec2());
                        Grid::new(id).show(ui, |ui| {
                            for (label, val) in [
                                ("ISBN-13", &book.isbn.to_string() as &str),
                                ("title", &book.title),
                                ("author", &book.author),
                                ("language", book.language.to_str()),
                                ("issue year", &book.issue_year.to_string()),
                            ] {
                                ui.label(label);
                                ui.label(val);
                                ui.end_row();
                            }
                            ui.label("description");
                            Label::new(&book.description).wrap(true).ui(ui);
                            ui.end_row();
                            ui.label("book file");
                            if ui.button("save file...").clicked() {
                                if let Some(path) = rfd::FileDialog::new().save_file() {
                                    copy(format!("books/{}", book.isbn), path).unwrap();
                                }
                            }
                        });
                    });
                });
            }
        });
    }

    fn update_tab(&mut self, ui: &mut Ui) {
        let isbn = parse_isbn(&self.isbn);
        ui.horizontal(|ui| {
            let label = if isbn.is_some() {
                ui.label("ISBN-13")
            } else {
                ui.colored_label(ui.visuals().error_fg_color, "ISBN-13")
            };
            ui.text_edit_singleline(&mut self.isbn)
                .labelled_by(label.id);
        });
        ui.horizontal(|ui| {
            ui.scope(|ui| {
                ui.set_enabled(isbn.is_some());
                if ui.button("update book").clicked() {
                    let isbn = isbn.unwrap();
                    match get_book(&mut self.connection, isbn) {
                        Ok(book) => {
                            self.book_find_failed_error = None;
                            self.tab = Tab::Create;
                            self.update_instead_of_create = true;
                            self.title = book.title;
                            self.author = book.author;
                            self.language = book.language.to_str().into();
                            self.issue_year = book.issue_year.to_string();
                            self.description = book.description;
                            self.cover_path = Some(format!("covers/{}", self.isbn).into());
                            self.book_path = Some(format!("books/{}", self.isbn).into());
                        }
                        Err(e) => {
                            self.book_find_failed_error = Some(e);
                        }
                    }
                }
            });
            if let Some(e) = &self.book_find_failed_error {
                ui.colored_label(
                    ui.visuals().error_fg_color,
                    format!("failed to load book: {e}"),
                );
            }
        });
    }

    fn delete_tab(&mut self, ui: &mut Ui) {
        let now = Instant::now();
        let isbn = parse_isbn(&self.isbn);
        ui.horizontal(|ui| {
            let label = if isbn.is_some() {
                ui.label("ISBN-13")
            } else {
                ui.colored_label(ui.visuals().error_fg_color, "ISBN-13")
            };
            ui.text_edit_singleline(&mut self.isbn)
                .labelled_by(label.id);
        });
        ui.horizontal(|ui| {
            ui.scope(|ui| {
                ui.set_enabled(isbn.is_some());
                if ui.button("delete book").clicked() {
                    let isbn = isbn.unwrap();
                    match delete_book(&mut self.connection, isbn) {
                        Err(e) => {
                            self.book_deleted_label_end = now;
                            self.book_deletion_failed_error = Some(e);
                        }
                        Ok(0) => {
                            self.book_deleted_label_end = now;
                            self.book_deletion_failed_error = Some(diesel::result::Error::NotFound);
                        }
                        _ => {
                            self.book_deleted_label_end = now + Duration::from_secs(3);
                            self.book_deletion_failed_error = None;
                        }
                    }
                }
            });
            if self.book_deleted_label_end > now {
                ui.colored_label(Color32::from_rgb(119, 221, 119), "book deleted!");
            }
            if let Some(e) = &self.book_deletion_failed_error {
                ui.colored_label(
                    ui.visuals().error_fg_color,
                    format!("failed to delete book: {e}"),
                );
            }
        });
    }
}

impl App for Library {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        ctx.set_pixels_per_point(2.);
        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.tab, Tab::Create, "create");
                ui.selectable_value(&mut self.tab, Tab::Read, "read");
                ui.selectable_value(&mut self.tab, Tab::Update, "update");
                ui.selectable_value(&mut self.tab, Tab::Delete, "delete");
            });

            if self.tab != Tab::Read {
                self.books = None;
            }
            match self.tab {
                Tab::Create => self.create_tab(ui),
                Tab::Read => self.read_tab(ui),
                Tab::Update => self.update_tab(ui),
                Tab::Delete => self.delete_tab(ui),
            }
        });
    }
}
