use super::{Encoder, Image};
use failure::Error;
use image::{bmp::BMPEncoder, ImageFormat, RGBA};

pub struct BMP;

impl Encoder for BMP {
    fn encode(&self, img: &Image) -> Result<Vec<u8>, Error> {
        let mut buffer = Vec::new();

        let mut encoder = BMPEncoder::new(&mut buffer);
        let (width, height) = img.dimensions();
        encoder.encode(&img.clone().into_raw(), width, height, RGBA(8))?;

        Ok(buffer)
    }

    fn decode(&self, buf: &[u8]) -> Result<Image, Error> {
        Ok(image::load_from_memory_with_format(buf, ImageFormat::BMP)?.to_rgba())
    }
}
