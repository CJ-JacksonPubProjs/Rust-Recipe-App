use iced::widget::{container, text};
use iced::{Element, Length};
 
fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .run()
}
 
#[derive(Default)]  
struct App;
 
#[derive(Debug, Clone)]
enum Message {}
 
impl App {
    fn update(&mut self, _message: Message) {}
 
    fn view(&self) -> Element<'_, Message> {
        container(text("example gui").size(24))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}
