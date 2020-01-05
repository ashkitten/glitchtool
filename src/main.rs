use azul::prelude::*;

struct GlitchApp {}

impl Layout for GlitchApp {
    fn layout(&self, _: LayoutInfo<Self>) -> Dom<Self> {
        Dom::div().with_child(Dom::label("Hello, World!"))
    }
}

fn main() {
    let mut app = App::new(GlitchApp {}, AppConfig::default()).unwrap();
    let window = app
        .create_window(WindowCreateOptions::default(), css::native())
        .unwrap();
    app.run(window).unwrap();
}
