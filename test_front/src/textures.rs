use std::{collections::HashMap, io::Cursor};

use image::{DynamicImage, GenericImage, GenericImageView, ImageReader};
use log::debug;

use crate::{
    components::canvas::Canvas,
    image_atlas::{BoundingBox, ImageAtlas},
};

pub struct Textures {
    atlas: ImageAtlas,
    cache: HashMap<Vec<u8>, u64>,
    current_id: u64,
}

impl Textures {
    pub fn new() -> Self {
        Self {
            atlas: ImageAtlas::new(),
            cache: HashMap::new(),
            current_id: 0,
        }
    }

    fn get_next_id(&mut self) -> u64 {
        let id = self.current_id;
        self.current_id += 1;
        id
    }

    pub fn insert_texture(&mut self, data: Vec<u8>, canvas: &mut Canvas) -> Option<u64> {
        if let Some(&id) = self.cache.get(&data) {
            return Some(id);
        }
        let img = ImageReader::new(Cursor::new(&data))
            .with_guessed_format()
            .ok()?
            .decode()
            .ok()?;
        let mut img = resize_image(img, 2048);
        let id = self.get_next_id();
        let (bounding_box, new_atlas) = self.atlas.add_image(id, img.width(), img.height());
        if bounding_box.rotated {
            img = img.rotate90()
        }
        self.cache.insert(data, id);
        if new_atlas {
            canvas.create_atlas();
        }
        canvas.update_atlas(img.to_rgba8().as_raw().to_vec(), bounding_box);
        Some(id)
    }

    pub fn get_bounds(&self, id: u64) -> BoundingBox {
        self.atlas.get_image_bounds(&id).unwrap().to_owned()
    }
}

fn resize_image(mut img: DynamicImage, max_size: u32) -> DynamicImage {
    while img.width() > max_size || img.height() > max_size {
        img = half_image(&img)
    }
    img
}

fn half_image(img: &DynamicImage) -> DynamicImage {
    let width = img.width() / 2;
    let height = img.height() / 2;
    let mut new_img = DynamicImage::new_rgba8(width, height);
    for x in 0..width {
        for y in 0..height {
            let mut r = 0;
            let mut g = 0;
            let mut b = 0;
            let mut a = 0;
            for dx in 0..2 {
                for dy in 0..2 {
                    let pixel = img.get_pixel(x * 2 + dx, y * 2 + dy);
                    r += pixel[0] as u32;
                    g += pixel[1] as u32;
                    b += pixel[2] as u32;
                    a += pixel[3] as u32;
                }
            }
            let new_pixel =
                image::Rgba([(r / 4) as u8, (g / 4) as u8, (b / 4) as u8, (a / 4) as u8]);
            new_img.put_pixel(x, y, new_pixel);
        }
    }
    new_img
}
