use image::{png::PNGEncoder, ColorType::RGBA, ImageFormat, RgbaImage};

type Image = RgbaImage;

pub trait Encoder {
    fn encode(&self, img: &Image) -> Vec<u8>;
    fn decode(&self, buf: &[u8]) -> Image;
}

impl<E: Encoder + ?Sized + Send> Encoder for Box<E> {
    fn encode(&self, img: &Image) -> Vec<u8> {
        (**self).encode(img)
    }

    fn decode(&self, buf: &[u8]) -> Image {
        (**self).decode(buf)
    }
}

pub struct EncodedImage<E>(Vec<u8>, E);

impl<E: Encoder> EncodedImage<E> {
    pub fn encode(img: &Image, enc: E) -> Self {
        Self(enc.encode(img), enc)
    }

    pub fn decode(&self) -> Image {
        self.1.decode(&self.0)
    }

    pub fn bytes(&self) -> &Vec<u8> {
        &self.0
    }
}

pub struct PNG;

impl Encoder for PNG {
    fn encode(&self, img: &Image) -> Vec<u8> {
        let mut buffer = Vec::new();

        let encoder = PNGEncoder::new(&mut buffer);
        let (width, height) = img.dimensions();
        encoder
            .encode(&img.clone().into_raw(), width, height, RGBA(8))
            .unwrap();

        buffer
    }

    fn decode(&self, buf: &[u8]) -> Image {
        image::load_from_memory_with_format(buf, ImageFormat::PNG)
            .unwrap()
            .to_rgba()
    }
}
