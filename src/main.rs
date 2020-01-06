use azul::{
    prelude::*,
    widgets::{button::Button, label::Label},
};
use image::{Bgra, ImageBuffer};

struct GlitchApp {
    // we'll make this an RgbaImage later, but azul doesn't currently work with RGBA8
    img: Option<ImageBuffer<Bgra<u8>, Vec<u8>>>,
    display_mode: DisplayMode,
}

#[derive(PartialEq)]
enum DisplayMode {
    Image,
    Bytes,
}

impl Layout for GlitchApp {
    fn layout(&self, info: LayoutInfo) -> Dom<Self> {
        let button = Button::with_label("Load image")
            .dom()
            .with_callback(On::MouseUp, load_image);

        let mut dom = Dom::div().with_child(button);

        if let Some(ref img) = self.img {
            let toggle_button = Button::with_label("Toggle display mode")
                .dom()
                .with_callback(On::MouseUp, toggle_display_mode);
            dom = dom.with_child(toggle_button);

            if self.display_mode == DisplayMode::Image {
                let image_id = info.resources.get_css_image_id("preview_image").unwrap();
                let image = Dom::image(*image_id);

                dom = dom.with_child(image)
            }

            if self.display_mode == DisplayMode::Bytes {
                let text = img
                    .clone()
                    .into_raw()
                    .iter()
                    .take(1000) // currently lags like hell trying to display an entire image worth of hex bytes
                    .map(|byte| format!("{:02x}", byte))
                    .collect::<Vec<String>>()
                    .join(" ");
                let bytes = Label::new(text).dom();

                dom = dom.with_child(bytes)
            }
        }

        dom
    }
}

fn load_image(info: CallbackInfo<GlitchApp>) -> UpdateScreen {
    if let Some(path) = azul::dialogs::open_file_dialog(None, None) {
        // there doesn't seem to be a way to make the image update aside from deleting the image
        // id. should i also delete the image source first? unsure if potential memory leak
        info.resources.delete_css_image_id("preview_image");

        let img = image::open(path).unwrap().to_bgra();
        let raw_image = RawImage {
            pixels: img.clone().into_raw(),
            image_dimensions: (img.width() as usize, img.height() as usize),
            data_format: RawImageFormat::BGRA8,
        };
        info.state.img = Some(img);

        let image_id = info.resources.add_css_image_id("preview_image");
        info.resources
            .add_image_source(image_id, ImageSource::Raw(raw_image));
    }
    Redraw
}
fn toggle_display_mode(info: CallbackInfo<GlitchApp>) -> UpdateScreen {
    info.state.display_mode = match info.state.display_mode {
        DisplayMode::Image => DisplayMode::Bytes,
        DisplayMode::Bytes => DisplayMode::Image,
    };
    Redraw
}

fn main() {
    let app = App::new(
        GlitchApp {
            img: None,
            display_mode: DisplayMode::Image,
        },
        AppConfig::default(),
    )
    .unwrap();
    app.run(WindowCreateOptions::new(css::native()));
}
