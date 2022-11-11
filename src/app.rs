use crate::Database;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Book {
    pub id: i64,
    pub name: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Verse {
    number: i8,
    chapter: i8,
    book: Book,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct BibleApp {
    current_book: String,

    #[serde(skip)]
    db: Database,
    #[serde(skip)]
    books: std::vec::Vec<Book>
}

impl Default for BibleApp {
    fn default() -> Self {
        let app = match Database::new() {
            Ok(db) => {
                let cursor = db.connection.prepare("SELECT * FROM books")
                    .unwrap()
                    .into_cursor();
                let mut books = std::vec::Vec::new();
                for row in cursor.map(|row| row.unwrap()) {
                    let new_book = Book {
                        id: row.get::<i64, _>("id"),
                        name: row.get::<String, _>("name")
                    };
                    books.push(new_book);
                }
                
                Self {
                    db,
                    books,
                    current_book: "".to_owned()
                }
            },
            Err(e) => panic!("{:?}", e)
        };

        app
    }
}

impl BibleApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for BibleApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {current_book, db, books} = self;

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");
            egui::ComboBox::from_label("Book")
                .selected_text(format!("{}", current_book))
                .show_ui(ui, |ui| {
                    for book in books {
                        ui.selectable_value(current_book, book.name.clone(), book.name.clone());
                    }
                });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let Self {current_book, db, books} = self;
            ui.heading(current_book);
            ui.label(format!(
                "\
                1. In the beginning God created the heaven and the earth \n\
                2. In the beginning God created the heaven and the earth in the beginning God
                "
            ));
        });

    }
}