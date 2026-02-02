use crate::image_atlas::BoundingBox;

pub struct Canvas {
    canvas: ts::Canvas,
}

impl Canvas {
    pub fn new() -> Self {
        Self {
            canvas: ts::Canvas::new(),
        }
    }

    pub fn update_atlas(&mut self, data: Vec<u8>, bounding_box: BoundingBox) {
        self.canvas.updateAtlas(
            data,
            bounding_box.atlas_id,
            bounding_box.x,
            bounding_box.y,
            bounding_box.width,
            bounding_box.height,
        );
    }

    pub fn push(&mut self, data: Vec<f32>) {
        self.canvas.push(data);
    }

    pub fn draw(&self) {
        self.canvas.draw();
    }

    pub fn set_transform(&mut self, x: f32, y: f32, scale: f32) {
        self.canvas.setTransform(x, y, scale);
    }
}

mod ts {
    use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen(js_namespace = canvas)]
    unsafe extern "C" {
        pub type Canvas;

        #[wasm_bindgen(constructor)]
        pub fn new() -> Canvas;

        #[wasm_bindgen(method)]
        pub fn updateAtlas(
            this: &Canvas,
            data: Vec<u8>,
            atlas_id: u32,
            x: u32,
            y: u32,
            width: u32,
            height: u32,
        );

        #[wasm_bindgen(method)]
        pub fn draw(this: &Canvas);

        #[wasm_bindgen(method)]
        pub fn setTransform(this: &Canvas, x: f32, y: f32, scale: f32);

        #[wasm_bindgen(method)]
        pub fn push(this: &Canvas, data: Vec<f32>);
    }
}
