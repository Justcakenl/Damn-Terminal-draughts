mod app;
mod game;
mod ui;
mod ai;
mod persistence;

use app::App;

fn main() -> anyhow::Result<()> {
    let mut app = App::new();
    app.run()
}
