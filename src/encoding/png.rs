use super::{Encoder, Image};
use failure::Error;
use image::{png::PNGEncoder, ImageFormat, RGBA};

pub struct PNG;

impl Encoder for PNG {
    fn encode(&self, img: &Image) -> Result<Vec<u8>, Error> {
        let mut buffer = Vec::new();

        let encoder = PNGEncoder::new(&mut buffer);
        let (width, height) = img.dimensions();
        encoder.encode(&img.clone().into_raw(), width, height, RGBA(8))?;

        Ok(buffer)
    }

    fn decode(&self, buf: &[u8]) -> Result<Image, Error> {
        Ok(image::load_from_memory_with_format(buf, ImageFormat::PNG)?.to_rgba())
    }
}
