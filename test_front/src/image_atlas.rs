use std::{cmp::max, collections::HashMap};

#[derive(Clone, Copy)]
pub struct BoundingBox {
    pub atlas_id: u32,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

enum AtlasImageKind {
    WIDE,
    TALL,
}

struct SingleAtlas {
    id: u32,
    width: u32,
    row_size: u32,
    row_usage: Vec<u32>,
    image_kind: AtlasImageKind,
}

impl SingleAtlas {
    pub fn new(id: u32, width: u32, rows: u32, image_kind: AtlasImageKind) -> Self {
        Self {
            id,
            width,
            row_size: width / rows,
            row_usage: vec![0; rows as usize],
            image_kind,
        }
    }

    pub fn add_image(&mut self, width: u32, height: u32) -> Option<BoundingBox> {
        match self.image_kind {
            AtlasImageKind::WIDE => {
                if width < height {
                    return None;
                }
            }
            AtlasImageKind::TALL => {
                if height < width {
                    return None;
                }
            }
        };

        if max(width, height) > self.row_size {
            return None;
        };

        let size = match self.image_kind {
            AtlasImageKind::WIDE => height,
            AtlasImageKind::TALL => width,
        };

        for (row_index, usage) in self.row_usage.iter_mut().enumerate() {
            if *usage + size <= self.width {
                let x = match self.image_kind {
                    AtlasImageKind::TALL => *usage,
                    AtlasImageKind::WIDE => row_index as u32 * self.row_size,
                };
                let y = match self.image_kind {
                    AtlasImageKind::TALL => row_index as u32 * self.row_size,
                    AtlasImageKind::WIDE => *usage,
                };
                *usage += size;
                return Some(BoundingBox {
                    atlas_id: self.id,
                    x,
                    y,
                    width,
                    height,
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
            atlases: vec![
                SingleAtlas::new(0, 2048, 16, AtlasImageKind::TALL),
                SingleAtlas::new(1, 2048, 16, AtlasImageKind::WIDE),
                SingleAtlas::new(2, 2048, 8, AtlasImageKind::TALL),
                SingleAtlas::new(3, 2048, 8, AtlasImageKind::WIDE),
            ],
        }
    }

    pub fn add_image(&mut self, id: u64, width: u32, height: u32) -> Option<BoundingBox> {
        for atlas in self.atlases.iter_mut() {
            if let Some(bounding_box) = atlas.add_image(width, height) {
                self.images.insert(id, bounding_box);
                return Some(bounding_box);
            }
        }
        None
    }

    pub fn get_image_bounds(&self, id: &u64) -> Option<&BoundingBox> {
        self.images.get(id)
    }
}
