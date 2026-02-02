use std::{cmp::max, collections::HashMap};

#[derive(Clone, Copy)]
pub struct BoundingBox {
    pub atlas_id: u32,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub rotated: bool,
}

const ATLAS_SIZE: u32 = 2048;

#[derive(Debug)]
struct SingleAtlas {
    id: u32,
    width: u32,
    row_size: u32,
    row_usage: Vec<u32>,
}

impl SingleAtlas {
    pub fn for_image(id: u32, sample_width: u32, sample_height: u32) -> Self {
        let mut row_height = 32;
        while row_height < max(sample_width, sample_height) {
            row_height *= 2;
        }
        let rows = ATLAS_SIZE / row_height;
        Self::new(id, ATLAS_SIZE, rows)
    }

    pub fn new(id: u32, width: u32, rows: u32) -> Self {
        Self {
            id,
            width,
            row_size: width / rows,
            row_usage: vec![0; rows as usize],
        }
    }

    pub fn add_image(&mut self, width: u32, height: u32) -> Option<BoundingBox> {
        let rotated = width > height;

        let (width, height) = if rotated {
            (height, width)
        } else {
            (width, height)
        };

        if height > self.row_size {
            return None;
        };

        if height > 16 && height <= self.row_size / 2 {
            return None;
        };

        for (row_index, usage) in self.row_usage.iter_mut().enumerate() {
            if *usage + width <= self.width {
                let x = *usage;
                let y = row_index as u32 * self.row_size;
                *usage += width;
                return Some(BoundingBox {
                    atlas_id: self.id,
                    x,
                    y,
                    width,
                    height,
                    rotated,
                });
            }
        }
        return None;
    }
}

pub struct ImageAtlas {
    images: HashMap<u64, BoundingBox>,
    atlases: Vec<SingleAtlas>,
}

impl ImageAtlas {
    pub fn new() -> Self {
        Self {
            images: HashMap::new(),
            atlases: Vec::new(),
        }
    }

    pub fn add_image(&mut self, id: u64, width: u32, height: u32) -> (BoundingBox, bool) {
        for atlas in self.atlases.iter_mut() {
            if let Some(bounding_box) = atlas.add_image(width, height) {
                self.images.insert(id, bounding_box);
                return (bounding_box, false);
            }
        }
        let mut new_atlas = SingleAtlas::for_image(self.atlases.len() as u32, width, height);
        let result = new_atlas
            .add_image(width, height)
            .expect("Newly created atlas must accept the image");
        self.images.insert(id, result);
        self.atlases.push(new_atlas);
        (result, true)
    }

    pub fn get_image_bounds(&self, id: &u64) -> Option<&BoundingBox> {
        self.images.get(id)
    }
}
