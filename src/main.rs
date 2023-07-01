use std::{
    env,
    fs::{self},
    io::{self, Read},
    path::{Path, PathBuf},
};

use libheif_rs::{HeifContext, LibHeif};
use zip::ZipArchive;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <path>", args[0]);
        return;
    }
    let path = PathBuf::from(args[1].clone());
    let files = if path.is_file() {
        vec![path]
    } else {
        let walker = DirWalker::new(&path);
        walker.get_all_files()
    };

    for file in files {
        let tgt = file.with_extension("jpg");
        convert(&file, &tgt);
        println!("converted {:?} to {:?}", &file, &tgt);
    }
    println!("finish all tasks. see you next time~")
}

fn convert(src: &Path, tgt: &Path) {
    let file = fs::File::open(src).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };
        match outpath.extension() {
            Some(x) => {
                if x.to_ascii_lowercase() != "heic" {
                    continue;
                }
            }
            None => continue,
        }

        let mut buffer = vec![];
        file.read_to_end(&mut buffer).unwrap();

        let lib_heif = LibHeif::new();
        let ctx = HeifContext::read_from_bytes(&buffer).unwrap();
        let handle = ctx.primary_image_handle().unwrap();
        let width = handle.width();
        let height = handle.height();
        let image = lib_heif
            .decode(
                &handle,
                libheif_rs::ColorSpace::Rgb(libheif_rs::RgbChroma::Rgb),
                None,
            )
            .unwrap();
        let planes = image.planes().interleaved.unwrap();
        image::save_buffer(tgt, planes.data, width, height, image::ColorType::Rgb8).unwrap();
    }
}

#[derive(Debug)]
struct DirWalker<'a> {
    path: &'a Path,
    files: Vec<PathBuf>,
}

impl<'a> DirWalker<'a> {
    fn new(path: &'a Path) -> Self {
        Self {
            path: path,
            files: vec![],
        }
    }
    fn get_all_files(mut self) -> Vec<PathBuf> {
        self.visit_dirs(self.path).unwrap();
        self.files
    }
    fn visit_dirs(&mut self, dir: &Path) -> io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    self.visit_dirs(&path)?;
                } else {
                    self.files.push(entry.path());
                }
            }
        }
        Ok(())
    }
}
