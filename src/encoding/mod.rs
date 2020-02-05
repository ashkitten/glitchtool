use failure::Error;
use image::RgbaImage;

pub mod bmp;
pub mod jpeg;
pub mod png;

type Image = RgbaImage;

pub trait Encoder {
    fn encode(&self, img: &Image) -> Result<Vec<u8>, Error>;
    fn decode(&self, buf: &[u8]) -> Result<Image, Error>;
}

impl<E: Encoder + ?Sized + Send> Encoder for Box<E> {
    fn encode(&self, img: &Image) -> Result<Vec<u8>, Error> {
        (**self).encode(img)
    }

    fn decode(&self, buf: &[u8]) -> Result<Image, Error> {
        (**self).decode(buf)
    }
}

impl<E: Encoder + ?Sized + Send> Encoder for &E {
    fn encode(&self, img: &Image) -> Result<Vec<u8>, Error> {
        (*self).encode(img)
    }

    fn decode(&self, buf: &[u8]) -> Result<Image, Error> {
        (*self).decode(buf)
    }
}

pub struct EncodedImage<E>(Vec<u8>, E);

impl<E: Encoder> EncodedImage<E> {
    pub fn encode(img: &Image, enc: E) -> Result<Self, Error> {
        Ok(Self(enc.encode(img)?, enc))
    }

    pub fn decode(&self) -> Result<Image, Error> {
        self.1.decode(&self.0)
    }

    pub fn bytes(&self) -> &Vec<u8> {
        &self.0
    }

    pub fn bytes_mut(&mut self) -> &mut Vec<u8> {
        &mut self.0
    }
}
