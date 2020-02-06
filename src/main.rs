use crate::filesystem::EncodingFS;
use fuse::BackgroundSession;
use iced::{
    button, executor, image::Handle, Application, Button, Column, Command, Element, Image,
    Settings, Subscription, Text,
};
use image::{bmp::BMPEncoder, RgbaImage, RGBA};
use std::{
    ffi::OsStr,
    path::Path,
    sync::mpsc::{channel, Receiver},
    time::Duration,
};
use tinyfiledialogs::open_file_dialog;

pub mod encoding;
pub mod filesystem;
pub mod time;

struct GlitchApp<'f> {
    image: Option<RgbaImage>,
    filesystem: Option<BackgroundSession<'f>>,
    file_button: button::State,
    receiver: Option<Receiver<Message>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    FileButtonPressed,
    ImageChanged(RgbaImage),
    Tick,
}

impl<'f> Application for GlitchApp<'f> {
    type Executor = executor::Default;
    type Message = Message;

    fn new() -> (Self, Command<Message>) {
        (
            Self {
                image: None,
                filesystem: None,
                file_button: button::State::default(),
                receiver: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("glitchtool")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::FileButtonPressed => {
                if let Some(path) = open_file_dialog("glitchapp", "", None) {
                    let path = Path::new(&path);
                    let mountpoint = path.parent().unwrap().join(path.file_stem().unwrap());
                    std::fs::create_dir(&mountpoint).ok();

                    let options = ["auto_unmount", "default_permissions"]
                        .iter()
                        .map(OsStr::new)
                        .flat_map(|option| vec![OsStr::new("-o"), option])
                        .collect::<Vec<_>>();

                    let image = image::open(path).unwrap().to_rgba();
                    self.image = Some(image.clone());

                    let (sender, receiver) = channel();
                    self.receiver = Some(receiver);

                    let fs = EncodingFS::new(image, sender);
                    unsafe {
                        let session = fuse::spawn_mount(fs, &mountpoint, &options).unwrap();
                        self.filesystem = Some(session);
                    }
                }
            }

            Message::ImageChanged(new_image) => {
                self.image = Some(new_image);
            }

            // hacky workaround to check receiver for messages, eventually this shouldn't be
            // necessary
            Message::Tick => {
                if let Some(ref receiver) = self.receiver {
                    if let Ok(message) = receiver.try_recv() {
                        return self.update(message);
                    }
                }
            }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        match self.receiver {
            Some(_) => time::every(Duration::from_millis(20)).map(|_| Message::Tick),
            None => Subscription::none(),
        }
    }

    fn view(&mut self) -> Element<Message> {
        let mut col = Column::new();
        col = col.push(
            Button::new(&mut self.file_button, Text::new("open file"))
                .on_press(Message::FileButtonPressed),
        );

        if let Some(ref img) = self.image {
            let mut buffer = Vec::new();
            let mut encoder = BMPEncoder::new(&mut buffer);
            let (width, height) = img.dimensions();
            encoder
                .encode(&img.clone().into_raw(), width, height, RGBA(8))
                .unwrap();
            col = col.push(Image::new(Handle::from_memory(buffer)));
        }

        col.into()
    }
}

fn main() {
    env_logger::builder().format_timestamp(None).init();

    GlitchApp::run(Settings::default());
}
