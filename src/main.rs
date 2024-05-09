mod app;
mod fitch;
mod parser;
mod state;
mod ui;

use app::App;

fn main() {
    let mut app = App::new().unwrap();
    app.listen();
}
