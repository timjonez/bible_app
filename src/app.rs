use rusqlite::Connection;
use std::vec::Vec;

pub struct Database {
    conn: Connection
}

impl Database {
    pub fn new() -> Self {
        let conn = Connection::open("./db.sqlite").unwrap();
        Self {conn}
    }

    pub fn get_chapters(&mut self, id: &i64) -> i64 {
        let mut query = self.conn.prepare(
            "SELECT book, MAX(chapter) FROM verses WHERE book = ?1").unwrap();
        query.query_row([id], |row| {
            Ok(row.get_unwrap(1))
        }).unwrap()
    }

    pub fn get_book(&mut self, id: &i64) -> Book {
        let mut query = self.conn.prepare("SELECT * FROM books WHERE id = ?1").unwrap();
        let book = query.query_row([id], |row| {
            let book = Book {
                id: row.get_unwrap(0),
                name: row.get_unwrap(1),
                num_chapters: 0
            };
            Ok(book)
        }).unwrap();

        book
    }

    pub fn get_books(&mut self) -> Vec<Book> {
        let mut books = std::vec::Vec::new();
        let mut books_query = self.conn.prepare("SELECT * FROM books").unwrap();
        let mut rows = books_query.query([]).unwrap();

        while let Some(row) = rows.next().unwrap() {
            let book = Book {
                id: row.get_unwrap(0),
                name: row.get_unwrap(1),
                num_chapters: 0
            };
            books.push(book);
        };

        books
    }

    pub fn get_verses(&mut self, chapter: i64, book: &Book) -> Vec<Verse> {
        let mut verses = std::vec::Vec::new();
        let mut verse_query = self.conn.prepare("SELECT * FROM verses WHERE book = ?1 AND chapter = ?2").unwrap();
        let mut rows = verse_query.query((&book.id, &chapter)).unwrap();

        while let Some(row) = rows.next().unwrap() {
            let verse = Verse {
                number: row.get_unwrap(3),
                chapter: row.get_unwrap(2),
                book: row.get_unwrap(1),
                text: row.get_unwrap(4),
            };
            verses.push(verse);
        };
        verses
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Clone)]
pub struct Book {
    pub id: i64,
    pub name: String,
    pub num_chapters: i64,
}



impl PartialEq for Book {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Clone)]
pub struct Verse {
    number: i8,
    chapter: i8,
    book: i8,
    text: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Selection {
    book: Book,
    chapter: i64,
    verses: Vec<Verse>
}

impl Selection {
    pub fn new(book: Book, chapter: i64, verses: Vec<Verse>) -> Self {
        Self {
            book,
            chapter,
            verses
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct BibleApp {
    current_book: Selection,

    #[serde(skip)]
    books: std::vec::Vec<Book>,
    #[serde(skip)]
    db: Database
}

impl Default for BibleApp {
    fn default() -> Self {
        let mut db = Database::new();
        let books = Database::get_books(&mut db);
        let mut default_book = Database::get_book(&mut db, &1);
        default_book.num_chapters = Database::get_chapters(&mut db, &default_book.id);
        let default_chapter: i64 = 1;
        let verses = Database::get_verses(&mut db, default_chapter, &default_book);

        let selection = Selection::new(default_book, default_chapter, verses);

        Self {
            current_book: selection,
            books,
            db
        }
    }
}

impl BibleApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            let data = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            return data
        }

        Default::default()
    }
}

impl eframe::App for BibleApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {current_book, books, db} = self;

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");
            egui::ComboBox::from_id_source(1)
                .selected_text(format!("{}", current_book.book.name))
                .show_ui(ui, |ui| {
                    for book in books {
                        if ui.add(egui::SelectableLabel::new(current_book.book.name == book.name, book.name.clone())).clicked() {
                            current_book.chapter = 1;
                            current_book.book = Database::get_book(db, &book.id);
                            current_book.book.num_chapters = Database::get_chapters(db, &book.id);
                            current_book.verses = Database::get_verses(db, current_book.chapter, book);
                        }
                    }
                });

            egui::ComboBox::from_id_source(2)
                .selected_text(format!("{}", current_book.chapter))
                .show_ui(ui, |ui| {
                    for chapter in 1..current_book.book.num_chapters {
                        if ui.add(egui::SelectableLabel::new(current_book.chapter == chapter, format!("{}",chapter))).clicked() {
                            current_book.chapter = chapter;
                            current_book.verses = Database::get_verses(db, chapter, &current_book.book);
                        }
                    }
                })
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let Self {current_book, books: _, db: _} = self;
            ui.heading(format!("{} {}", &current_book.book.name, &current_book.chapter));
            let mut formatted_string = String::new();
            for verse in &current_book.verses {
                let f_str = format!("{}. {}\n", verse.number, verse.text);
                formatted_string.push_str(&f_str)
            }
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.label(formatted_string);
            });

        });

    }
}