mod app;
mod fitch;
mod parser;
mod ui;

use app::App;

fn main() {
    let mut app = App::new().unwrap();
    app.listen();
}
