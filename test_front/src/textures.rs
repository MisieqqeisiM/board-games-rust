use std::{collections::HashMap, hash::Hash, io::Cursor, vec};

use image::{DynamicImage, GenericImage, GenericImageView, ImageReader};
use log::debug;
use test_back::board::ObjectIdentifier;

use crate::{
    canvas::Canvas,
    image_atlas::{BoundingBox, ImageAtlas},
};

pub struct Textures {
    atlas: ImageAtlas,
    cache: HashMap<Vec<u8>, u64>,
    id_to_atlas: HashMap<ObjectIdentifier, u64>,
    atlas_to_id: HashMap<u64, ObjectIdentifier>,
    current_id: u64,
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

fn resize_image(mut img: DynamicImage, max_size: u32) -> DynamicImage {
    while img.width() > max_size || img.height() > max_size {
        img = half_image(&img)
    }
    img
}

impl Textures {
    pub fn new() -> Self {
        Self {
            atlas: ImageAtlas::new(),
            cache: HashMap::new(),
            atlas_to_id: HashMap::new(),
            id_to_atlas: HashMap::new(),
            current_id: 0,
        }
    }

    pub fn texture_exists(&self, data: &Vec<u8>) -> bool {
        self.cache.contains_key(data)
    }

    pub fn get_texture_id(&self, data: &Vec<u8>) -> Option<ObjectIdentifier> {
        self.cache
            .get(data)
            .and_then(|id| self.atlas_to_id.get(id))
            .cloned()
    }

    pub fn insert_texture(&mut self, obj_id: ObjectIdentifier, data: Vec<u8>, canvas: &mut Canvas) {
        let img = ImageReader::new(Cursor::new(&data))
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap();
        let img = resize_image(img, 256);
        let id = self.current_id;
        self.current_id += 1;
        self.atlas
            .add_image(id, img.width(), img.height())
            .inspect(|bounding_box| {
                self.cache.insert(data, id);
                self.atlas_to_id.insert(id, obj_id);
                self.id_to_atlas.insert(obj_id, id);
                canvas.update_atlas(img.to_rgba8().as_raw().to_vec(), *bounding_box);
            });
    }

    pub fn get_bounds(&self, obj_id: ObjectIdentifier) -> BoundingBox {
        debug!("ids: {:?}, {:?}", self.id_to_atlas, self.atlas_to_id);
        debug!("finding: {:?}", obj_id);
        let atlas_id = self.id_to_atlas.get(&obj_id).unwrap();
        self.atlas.get_image_bounds(atlas_id).unwrap().to_owned()
    }

    pub fn update_id(&mut self, old_id: ObjectIdentifier, new_id: ObjectIdentifier) {
        let Some(atlas_id) = self.id_to_atlas.remove(&old_id) else {
            return;
        };
        self.id_to_atlas.insert(new_id, atlas_id);
        self.atlas_to_id.insert(atlas_id, new_id);
    }
}
