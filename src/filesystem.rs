use crate::{
    encoding::{EncodedImage, Encoder, PNG},
    file_handle::FileHandleMap,
};
use fuse::BackgroundSession;
use fuse_mt::{
    DirectoryEntry, FileAttr, FileType, FilesystemMT, FuseMT, RequestInfo, ResultEmpty,
    ResultEntry, ResultOpen, ResultReaddir,
};
use image::RgbaImage;
use libc::c_int;
use std::{
    cell::RefCell,
    cmp,
    ffi::OsStr,
    path::Path,
    sync::{Arc, Mutex},
};
use time::Timespec;

struct EncodingFS {
    image: RgbaImage,
    file_handles: Arc<Mutex<RefCell<FileHandleMap<EncodedImage<Box<dyn Encoder + Send>>>>>>,
}

impl EncodingFS {
    fn new(image: RgbaImage) -> Self {
        Self {
            image,
            file_handles: Arc::new(Mutex::new(RefCell::new(FileHandleMap::new()))),
        }
    }
}

impl FilesystemMT for EncodingFS {
    fn init(&self, _req: RequestInfo) -> ResultEmpty {
        Ok(())
    }

    fn getattr(&self, req: RequestInfo, path: &Path, fh: Option<u64>) -> ResultEntry {
        if path == Path::new("/") {
            Ok((
                Timespec::new(0, 0),
                // TODO: fill in fields better
                FileAttr {
                    size: 3, // ., .., image.png
                    blocks: 0,
                    atime: Timespec::new(0, 0),
                    mtime: Timespec::new(0, 0),
                    ctime: Timespec::new(0, 0),
                    crtime: Timespec::new(0, 0),
                    kind: FileType::Directory,
                    perm: 0o777,
                    nlink: 0,
                    uid: 0,
                    gid: 0,
                    rdev: 0,
                    flags: 0,
                },
            ))
        } else {
            let fh = if let Some(fh) = fh {
                fh
            } else {
                self.open(req, path, 0)?.0
            };

            if let Some(encoded) = (*self.file_handles.lock().unwrap())
                .borrow_mut()
                .get_handle(fh)
            {
                let buf = encoded.data.bytes();

                Ok((
                    Timespec::new(0, 0),
                    FileAttr {
                        size: buf.len() as u64, // ., .., image.png
                        blocks: 0,
                        atime: Timespec::new(0, 0),
                        mtime: Timespec::new(0, 0),
                        ctime: Timespec::new(0, 0),
                        crtime: Timespec::new(0, 0),
                        kind: FileType::RegularFile,
                        perm: 0o666,
                        nlink: 0,
                        uid: 0,
                        gid: 0,
                        rdev: 0,
                        flags: 0,
                    },
                ))
            } else {
                Err(libc::EBADF)
            }
        }
    }

    fn open(&self, _req: RequestInfo, path: &Path, flags: u32) -> ResultOpen {
        let extension = path
            .extension()
            .ok_or(libc::ENOENT)?
            .to_str()
            .ok_or(libc::ENOENT)?;
        let data = match extension {
            "png" => EncodedImage::encode(&self.image, Box::new(PNG) as Box<dyn Encoder + Send>),
            _ => return Err(libc::ENOENT),
        };

        Ok((
            (*self.file_handles.lock().unwrap())
                .borrow_mut()
                .new_handle(data, flags),
            0,
        ))
    }

    fn read(
        &self,
        _req: RequestInfo,
        path: &Path,
        fh: u64,
        offset: u64,
        size: u32,
        result: impl FnOnce(Result<&[u8], c_int>),
    ) {
        if let Some(encoded) = (*self.file_handles.lock().unwrap())
            .borrow_mut()
            .get_handle(fh)
        {
            let buf = encoded.data.bytes();
            let end = cmp::min(offset as usize + size as usize, buf.len());
            result(Ok(&buf[offset as usize..end]));
        } else {
            result(Err(libc::EBADF))
        }
    }

    fn opendir(&self, _req: RequestInfo, path: &Path, flags: u32) -> ResultOpen {
        Ok((0, 0))
    }

    fn readdir(&self, _req: RequestInfo, _path: &Path, _fh: u64) -> ResultReaddir {
        Ok((["png"])
            .iter()
            .map(|ext| DirectoryEntry {
                name: OsStr::new(&format!("image.{}", ext)).to_os_string(),
                kind: FileType::RegularFile,
            })
            .collect())
    }
}

pub fn start<'a>(image: RgbaImage, mountpoint: String) -> BackgroundSession<'a> {
    // TODO: take command line options
    let options = ["auto_unmount", "default_permissions"]
        .iter()
        .map(OsStr::new)
        .flat_map(|option| vec![OsStr::new("-o"), option])
        .collect::<Vec<_>>();

    let fs = FuseMT::new(EncodingFS::new(image), 0);
    unsafe { fuse_mt::spawn_mount(fs, &mountpoint, &options).unwrap() }
}
