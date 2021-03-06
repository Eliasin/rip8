pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
pub const MAX_SPRITE_SIZE: usize = 15;

pub type IsPixelOverwritten = bool;

#[derive(Debug)]
pub struct InvalidSpriteSizeError {
    size: usize,
}

impl InvalidSpriteSizeError {
    pub fn new(size: usize) -> InvalidSpriteSizeError {
        InvalidSpriteSizeError{
            size: size
        }
    }
}

impl std::fmt::Display for InvalidSpriteSizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Draw attempted using invalid sprite size {}", self.size)
    }
}

impl std::error::Error for InvalidSpriteSizeError {}

pub struct Screen {
    screen: [[bool; SCREEN_WIDTH]; SCREEN_HEIGHT],
    has_changed: bool,
    last_drawn_sprite: Option<Vec<u8>>,
    last_draw_result: Option<Vec<u8>>,
    last_draw_area: Option<Vec<u8>>,
}

impl Screen {
    pub fn new() -> Screen {
        Screen{
            screen: [[false; SCREEN_WIDTH]; SCREEN_HEIGHT],
            has_changed: false,
            last_drawn_sprite: None,
            last_draw_result: None,
            last_draw_area: None,
        }
    }

    pub fn inspect_last_drawn_sprite(&self) -> Option<Vec<u8>> {
        self.last_drawn_sprite.clone()
    }

    pub fn inspect_last_draw_area(&self) -> Option<Vec<u8>> {
        self.last_draw_area.clone()
    }

    pub fn inspect_last_draw_result(&self) -> Option<Vec<u8>> {
        self.last_draw_result.clone()
    }

    pub fn clear(&mut self) {
        self.screen = [[false; SCREEN_WIDTH]; SCREEN_HEIGHT];
        self.has_changed = true;
    }

    fn draw_sprite_line(&mut self, x: u8, y: u8, sprite_line: u8) -> bool {
        let mut is_pixel_overwritten = false;
        let wrapped_y = (y as usize) % SCREEN_HEIGHT;

        let mut draw_line = 0;
        let mut draw_area = 0;

        for i in 0..8 {
            let wrapped_x = ((x as usize) + (i as usize)) % SCREEN_WIDTH;
            let sprite_pixel = (sprite_line & (0x80 >> i)) != 0;
            let final_value = self.screen[wrapped_y][wrapped_x] ^ sprite_pixel;
            draw_area = draw_area | ((self.screen[wrapped_y][wrapped_x] as u8) << (7 - i));
            self.screen[wrapped_y][wrapped_x] = final_value;
            draw_line = draw_line | ((final_value as u8) << (7 - i));

            is_pixel_overwritten = is_pixel_overwritten || (sprite_pixel && self.screen[wrapped_y][wrapped_x]);
        }

        self.last_draw_result.as_mut().unwrap().push(draw_line);
        self.last_draw_area.as_mut().unwrap().push(draw_area);

        self.has_changed = true;

        is_pixel_overwritten
    }

    pub fn draw(&mut self, x: u8, y: u8, sprite: Vec<u8>) -> Result<IsPixelOverwritten, Box<dyn std::error::Error>> {
        let mut is_pixel_overwritten = false;
        if sprite.len() > MAX_SPRITE_SIZE {
            return Err(Box::new(InvalidSpriteSizeError::new(sprite.len())));
        }

        self.last_drawn_sprite = Some(sprite.clone());
        self.last_draw_area = Some(vec![]);
        self.last_draw_result = Some(vec![]);

        for (i, sprite_line) in sprite.iter().enumerate() {
            let does_line_overwrite_pixel = self.draw_sprite_line(x, y + (i as u8), *sprite_line);
            is_pixel_overwritten = is_pixel_overwritten || does_line_overwrite_pixel;
        }

        Ok(is_pixel_overwritten)
    }

    pub fn inspect_screen<'a>(&'a self) -> &'a [[bool; SCREEN_WIDTH]; SCREEN_HEIGHT] {
        &self.screen
    }

    pub fn has_changed(&self) -> bool {
        self.has_changed
    }

    pub fn reset_changed(&mut self) {
        self.has_changed = false;
    }
}
