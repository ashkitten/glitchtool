use azul::{prelude::*, widgets::button::Button};
use fuse::BackgroundSession;
use image::{Bgra, ImageBuffer};
use std::{path::Path, time::Duration};

mod encoding;
mod file_handle;
mod filesystem;

// needs to be a macro because `include_str!` wants a string literal
macro_rules! CSS_PATH {
    () => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/src/style.css")
    };
}

struct GlitchApp<'f> {
    // we'll make this an RgbaImage later, but azul doesn't currently work with RGBA8
    img: Option<ImageBuffer<Bgra<u8>, Vec<u8>>>,
    display_mode: DisplayMode,
    bytes_text_id: Option<TextId>,
    filesystem: Option<BackgroundSession<'f>>,
}

#[derive(PartialEq)]
enum DisplayMode {
    Image,
    Bytes,
}

impl<'f> Layout for GlitchApp<'f> {
    fn layout(&self, info: LayoutInfo) -> Dom<Self> {
        let mut dom = Dom::body();

        let mut button_bar = Dom::div().with_id("button_bar");

        let file_button = Button::with_label("Load image")
            .dom()
            .with_id("file_button")
            .with_callback(On::MouseUp, load_image);
        button_bar = button_bar.with_child(file_button);

        if self.img.is_some() {
            let toggle_button = Button::with_label("Toggle display mode")
                .dom()
                .with_id("toggle_button")
                .with_callback(On::MouseUp, toggle_display_mode);
            button_bar = button_bar.with_child(toggle_button);
        }

        dom = dom.with_child(button_bar);

        if self.img.is_some() {
            if self.display_mode == DisplayMode::Image {
                let image_id = info.resources.get_css_image_id("preview_image").unwrap();
                let image = Dom::image(*image_id).with_id("preview_image");

                dom = dom.with_child(image)
            }

            if self.display_mode == DisplayMode::Bytes {
                let text = Dom::text_id(self.bytes_text_id.unwrap()).with_id("image_bytes");
                dom = dom.with_child(text);
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

        let path = Path::new(&path);
        let fuse_path = path.parent().unwrap().join(path.file_stem().unwrap());
        std::fs::create_dir(&fuse_path).ok();

        info.state.filesystem = Some(filesystem::start(
            image::open(path).unwrap().to_rgba(),
            fuse_path.to_str().unwrap().to_string(),
        ));

        let img = image::open(path).unwrap().to_bgra();

        let raw_image = RawImage {
            pixels: img.clone().into_raw(),
            image_dimensions: (img.width() as usize, img.height() as usize),
            data_format: RawImageFormat::BGRA8,
        };

        let text = img
            .clone()
            .into_raw()
            .iter()
            .take(10000) // azul doesn't do any render culling currently
            .map(|byte| format!("{:02x}", byte))
            .collect::<Vec<String>>()
            .join(" ");

        let words = azul::text_layout::split_text_into_words(&text);
        info.state.bytes_text_id = Some(info.resources.add_text(words));

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
            bytes_text_id: None,
            filesystem: None,
        },
        AppConfig::default(),
    )
    .unwrap();

    #[cfg(debug_assertions)]
    {
        let hot_reloader = css::hot_reload_override_native(CSS_PATH!(), Duration::from_millis(500));
        app.run(WindowCreateOptions::new_hot_reload(hot_reloader));
    }

    #[cfg(not(debug_assertions))]
    {
        let css = css::override_native(include_str!(CSS_PATH!())).unwrap();
        app.run(WindowCreateOptions::new(css));
    }
}
