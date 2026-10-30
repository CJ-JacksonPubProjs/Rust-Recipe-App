mod app;
mod db_backend;
mod views;

use app::App;

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title("Rust Recipe Project")
        .theme(App::theme)
        .run()
}