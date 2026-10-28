mod app;
mod db_backend;
mod views;

use app::App;
use iced::Theme;

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title("Rust Recipe Project")
        .theme(Theme::TokyoNight)
        .run()
}