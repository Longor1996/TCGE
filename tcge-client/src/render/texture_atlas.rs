use rustc_hash::FxHashMap;
use image::ImageBuffer;
use image::DynamicImage;
use image::GenericImage;
use image::GenericImageView;
use super::TextureError;
use super::TextureObject;
use super::TextureObjectBuilder;

pub struct TextureAtlasBuilder {
    texture: DynamicImage,
    width: u32,
    height: u32,
    empty: Vec<RawTextureAtlasSprite>,
    sprites: FxHashMap<String, RawTextureAtlasSprite>,
}

#[derive(Debug, Copy, Clone)]
struct RawTextureAtlasSprite {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

impl TextureAtlasBuilder {
    pub fn new(initial_size: u32) -> Self {
        
        // Ensure it's a POT-size...
        let initial_size = initial_size.next_power_of_two();
        
        let empty = RawTextureAtlasSprite {
            x: 0, y: 0, w: initial_size, h: initial_size
        };
        
        Self {
            texture: DynamicImage::new_rgba8(initial_size, initial_size),
            width: initial_size,
            height: initial_size,
            empty: vec![empty],
            sprites: Default::default()
        }
    }
    
    pub fn contains(&self, str: &str) -> bool {
        self.sprites.contains_key(str)
    }
    
    pub fn grow(&mut self) {
        
        let s = if self.width > self.height {
            (self.width, self.height*2)
        } else {
            (self.width*2, self.height)
        };
        
        let (new_width, new_height) = s;
        let mut new_texture = DynamicImage::new_rgba8(new_width, new_height);
        
        // Blit textures and replace...
        new_texture.copy_from(&self.texture, 0, 0);
        self.texture = new_texture;
        
        // Add new horizontal empty space to queue...
        if new_width > self.width {
            self.empty.insert(0, RawTextureAtlasSprite {
                x: self.width,
                y: 0,
                w: self.width,
                h: self.height
            });
        }
        
        // Add new vertical empty space to queue...
        if new_height > self.height {
            self.empty.insert(0, RawTextureAtlasSprite {
                x: 0,
                y: self.height,
                w: self.width,
                h: self.height
            });
        }
        
        self.width = new_width;
        self.height = new_height;
    }
    
    pub fn insert<S: Into<String>>(&mut self, str: S, img: &DynamicImage) {
        let (width, height) = img.dimensions();
        
        let pot_width = width.next_power_of_two();
        let pot_height = height.next_power_of_two();
        
        let space = self.allocate_space(pot_width, pot_height);
        
        self.texture.copy_from(img, space.x, space.y);
        self.sprites.insert(str.into(), space);
    }
    
    fn allocate_space(&mut self, width: u32, height: u32) -> RawTextureAtlasSprite {
        
        let space = self.empty.iter()
            .enumerate()
            .filter(|(_, i)| i.w >= width && i.h >= height)
            .map(|(i, _)| i)
            .last();
        
        // No empty space left?
        let space: usize = match space {
            None => {
                // Grow and try again!
                self.grow();
                return self.allocate_space(width, height);
            },
            Some(i) => i
        };
        
        let space = self.empty.remove(space);
        
        if space.h > height {
            // Generate new vertical rect
            self.empty.insert(0, RawTextureAtlasSprite {
                x: space.x,
                y: space.y + height,
                w: space.w, // cover entire width
                h: space.h - height
            });
        }
        
        if space.w > width {
            // Generate new horizontal rect
            self.empty.insert(0, RawTextureAtlasSprite {
                x: space.x + width,
                y: space.y,
                w: space.w - width,
                h: height
            });
        }
        
        RawTextureAtlasSprite {
            x: space.x,
            y: space.y,
            w: width,
            h: height,
        }
    }
}

impl TextureAtlasBuilder {
    pub fn finish(mut self, gl: &gl::Gl) -> Result<TextureAtlas, TextureError> {
        
        self.texture.save("./atlas.png")
            .expect("Failed to save atlas for debugging.");
        
        let (width, height) = self.texture.dimensions();
        
        let texture = TextureObjectBuilder::new()
            .build_from_dynamic_image(gl, &self.texture)?;
        
        let sprites = self.sprites
            .drain()
            .map(|(name, raw_sprite)| {
                trace!("Sprite '{}' -> {:?}", name, raw_sprite);
                (name, TextureAtlasSprite {
                    x: raw_sprite.x,
                    y: raw_sprite.y,
                    w: raw_sprite.w as u16,
                    h: raw_sprite.h as u16,
                    umin: ((raw_sprite.x) / width) as f32,
                    umax: ((raw_sprite.x + raw_sprite.w) / width) as f32,
                    vmin: ((raw_sprite.y) / height) as f32,
                    vmax: ((raw_sprite.y + raw_sprite.h) / height) as f32,
                })
            })
            .collect();
        
        Ok(TextureAtlas {
            texture,
            sprites,
        })
    }
}

pub struct TextureAtlas {
    pub texture: TextureObject,
    pub sprites: FxHashMap<String, TextureAtlasSprite>,
}

pub struct TextureAtlasSprite {
    x: u32,
    y: u32,
    w: u16,
    h: u16,
    umin: f32,
    umax: f32,
    vmin: f32,
    vmax: f32,
}
