use crate::encoding::{bmp::BMP, jpeg::JPEG, png::PNG, EncodedImage, Encoder};
use failure::Error;
use fuse::{
    BackgroundSession, FileAttr, FileType, Filesystem, ReplyAttr, ReplyData, ReplyDirectory,
    ReplyEntry, ReplyOpen, ReplyWrite, Request,
};
use image::RgbaImage;
use log::error;
use std::{
    cmp,
    collections::BTreeMap,
    ffi::OsStr,
    time::{Duration, SystemTime},
};

type BoxEncoder = Box<dyn Encoder + Send>;

macro_rules! encodings {
    ( $( $ext:expr => $enc:expr ),+ $(,)? ) => {
        const ENCODINGS: &[&str] = &[ $($ext),* ];

        fn get_encoder(extension: &str) -> Option<BoxEncoder> {
            match extension {
                $( $ext => Some(Box::new($enc)) ),+,
                _ => None,
            }
        }
    }
}

encodings! {
    "bmp" => BMP,
    "png" => PNG,
    "jpg" => JPEG,
}

struct EncodingFS {
    image: RgbaImage,
    changed: SystemTime,
    inodes: BTreeMap<u64, EncodedImage<BoxEncoder>>,
}

impl EncodingFS {
    fn new(image: RgbaImage) -> Self {
        Self {
            image,
            changed: SystemTime::now(),
            inodes: BTreeMap::new(),
        }
    }

    fn inode(&mut self, ino: u64) -> Result<&EncodedImage<BoxEncoder>, Error> {
        if !self.inodes.contains_key(&ino) {
            let data = EncodedImage::encode(
                &self.image,
                get_encoder(ENCODINGS[ino as usize - 2]).unwrap(),
            )?;
            self.inodes.insert(ino, data);
        }
        Ok(self.inodes.get(&ino).unwrap())
    }

    fn inode_mut(&mut self, ino: u64) -> Result<&mut EncodedImage<BoxEncoder>, Error> {
        if !self.inodes.contains_key(&ino) {
            let data = EncodedImage::encode(
                &self.image,
                get_encoder(ENCODINGS[ino as usize - 2]).unwrap(),
            )?;
            self.inodes.insert(ino, data);
        }
        Ok(self.inodes.get_mut(&ino).unwrap())
    }
}

impl Filesystem for EncodingFS {
    fn lookup(&mut self, _req: &Request, _parent: u64, name: &OsStr, reply: ReplyEntry) {
        let name = name.to_str().unwrap();
        let ino = {
            match ENCODINGS
                .iter()
                .position(|ext| name == format!("image.{}", ext))
            {
                Some(i) => (i + 2) as u64,
                None => {
                    reply.error(libc::ENOENT);
                    return;
                }
            }
        };

        let data = self.inode(ino).unwrap();
        let buf = data.bytes();

        reply.entry(
            &Duration::from_secs(0),
            &FileAttr {
                ino,
                size: buf.len() as u64,
                blocks: 0,
                atime: self.changed,
                mtime: self.changed,
                ctime: self.changed,
                ftype: FileType::RegularFile,
                perm: 0o666,
                nlink: 0,
                uid: 0,
                gid: 0,
                rdev: 0,
            },
            0,
        );
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        if ino == 1 {
            reply.attr(
                &Duration::from_secs(0),
                &FileAttr {
                    ino,
                    size: ENCODINGS.len() as u64 + 2, // ., .., etc
                    blocks: 0,
                    atime: self.changed,
                    mtime: self.changed,
                    ctime: self.changed,
                    ftype: FileType::Directory,
                    perm: 0o777,
                    nlink: 0,
                    uid: 0,
                    gid: 0,
                    rdev: 0,
                },
            );
        } else {
            let data = self.inode(ino).unwrap();
            let buf = data.bytes();

            reply.attr(
                &Duration::from_secs(0),
                &FileAttr {
                    ino,
                    size: buf.len() as u64,
                    blocks: 0,
                    atime: self.changed,
                    mtime: self.changed,
                    ctime: self.changed,
                    ftype: FileType::RegularFile,
                    perm: 0o666,
                    nlink: 0,
                    uid: 0,
                    gid: 0,
                    rdev: 0,
                },
            );
        }
    }

    fn setattr(
        &mut self,
        req: &Request,
        ino: u64,
        _mode: Option<u32>,
        _uid: Option<u32>,
        _gid: Option<u32>,
        size: Option<u64>,
        _atime: Option<SystemTime>,
        _mtime: Option<SystemTime>,
        _fh: Option<u64>,
        _crtime: Option<SystemTime>,
        _chgtime: Option<SystemTime>,
        _bkuptime: Option<SystemTime>,
        _flags: Option<u32>,
        reply: ReplyAttr,
    ) {
        match self.inode_mut(ino) {
            Ok(encoded) => {
                if let Some(size) = size {
                    encoded.bytes_mut().resize(size as usize, 0);
                }
            }
            Err(_) => {
                reply.error(libc::ENOENT);
                return;
            }
        }

        self.getattr(req, ino, reply);
    }

    fn open(&mut self, _req: &Request, _ino: u64, _flags: u32, reply: ReplyOpen) {
        reply.opened(0, 0);
    }

    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        size: u32,
        reply: ReplyData,
    ) {
        match self.inode(ino) {
            Ok(encoded) => {
                let buf = encoded.bytes();
                let end = cmp::min(offset as usize + size as usize, buf.len());
                reply.data(&buf[offset as usize..end]);
            }
            Err(_) => reply.error(libc::ENOENT),
        }
    }

    fn write(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        data: &[u8],
        _flags: u32,
        reply: ReplyWrite,
    ) {
        match self.inode_mut(ino) {
            Ok(encoded) => {
                let buf = encoded.bytes_mut();
                let offset = offset as usize;
                let len = data.len();
                let end = offset + len;
                buf.resize(cmp::max(buf.len(), end), 0);
                buf.splice(offset..end, data.to_owned());

                match encoded.decode() {
                    Ok(img) => {
                        self.image = img;
                        self.inodes = BTreeMap::new();
                    }
                    Err(e) => error!("{:?}", e),
                }

                reply.written(len as u32);
            }
            Err(_) => reply.error(libc::ENOENT),
        }
    }

    fn readdir(
        &mut self,
        _req: &Request,
        _ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        let entries = {
            let mut e = vec![
                (1, FileType::Directory, ".".to_string()),
                (1, FileType::Directory, "..".to_string()),
            ];

            for (i, ext) in ENCODINGS.iter().enumerate() {
                let name = format!("image.{}", ext);
                e.push((i as u64 + 2, FileType::RegularFile, name));
            }

            e
        };

        for (i, entry) in entries.into_iter().enumerate().skip(offset as usize) {
            reply.add(entry.0, (i + 1) as i64, entry.1, entry.2);
        }
        reply.ok();
    }
}

pub fn start<'a>(image: RgbaImage, mountpoint: String) -> BackgroundSession<'a> {
    // TODO: take command line options
    let options = ["auto_unmount", "default_permissions"]
        .iter()
        .map(OsStr::new)
        .flat_map(|option| vec![OsStr::new("-o"), option])
        .collect::<Vec<_>>();

    let fs = EncodingFS::new(image);
    unsafe { fuse::spawn_mount(fs, &mountpoint, &options).unwrap() }
}
