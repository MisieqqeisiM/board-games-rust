use crate::mouse::MouseCommand;

pub struct Camera {
    x: f64,
    y: f64,
    scale: f64,
    last_mouse_x: f64,
    last_mouse_y: f64,
    shifting: bool,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            scale: 1.0,
            last_mouse_x: 0.0,
            last_mouse_y: 0.0,
            shifting: false,
        }
    }

    pub fn get_x(&self) -> f64 {
        self.x as f64
    }
    pub fn get_y(&self) -> f64 {
        self.y as f64
    }
    pub fn get_scale(&self) -> f64 {
        self.scale as f64
    }

    pub fn get_mouse_position(&self) -> (f64, f64) {
        self.get_world_position(self.last_mouse_x, self.last_mouse_y)
    }

    pub fn get_world_position(&self, screen_x: f64, screen_y: f64) -> (f64, f64) {
        let world_x = self.x + screen_x / self.scale;
        let world_y = self.y + screen_y / self.scale;
        (world_x, world_y)
    }

    pub fn update(&mut self, mouse: MouseCommand) -> bool {
        match mouse {
            MouseCommand::Scroll {
                delta_x: _,
                delta_y,
            } => {
                let (pivot_x, pivot_y) =
                    self.get_world_position(self.last_mouse_x, self.last_mouse_y);

                if delta_y > 0.0 {
                    self.scale /= 1.2;
                } else if delta_y < 0.0 {
                    self.scale *= 1.2;
                }

                self.x = pivot_x - self.last_mouse_x / self.scale;
                self.y = pivot_y - self.last_mouse_y / self.scale;
                true
            }
            MouseCommand::Down { button, x, y } => {
                if button == 2 {
                    self.shifting = true;
                }
                self.last_mouse_x = x as f64;
                self.last_mouse_y = y as f64;
                false
            }
            MouseCommand::Up { button, x, y } => {
                if button == 2 {
                    self.shifting = false;
                }
                self.last_mouse_x = x as f64;
                self.last_mouse_y = y as f64;
                false
            }
            MouseCommand::Move { x, y } => {
                if self.shifting {
                    self.x -= (x as f64 - self.last_mouse_x) / self.scale;
                    self.y -= (y as f64 - self.last_mouse_y) / self.scale;
                }
                self.last_mouse_x = x as f64;
                self.last_mouse_y = y as f64;
                self.shifting
            }
        }
    }
}
