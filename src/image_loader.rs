use egui::{ColorImage, Context, TextureHandle, TextureOptions};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct ImageCache {
    textures: HashMap<PathBuf, Option<TextureHandle>>,
}

impl Default for ImageCache {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageCache {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.textures.clear();
    }

    pub fn resolve_path(base_dir: Option<&Path>, url: &str) -> PathBuf {
        let p = Path::new(url);
        if p.is_absolute() {
            p.to_path_buf()
        } else if let Some(base) = base_dir {
            base.join(p)
        } else {
            p.to_path_buf()
        }
    }

    pub fn get_or_load(
        &mut self,
        ctx: &Context,
        base_dir: Option<&Path>,
        url: &str,
    ) -> Option<&TextureHandle> {
        let path = Self::resolve_path(base_dir, url);

        if !self.textures.contains_key(&path) {
            let handle = Self::load_texture_from_path(ctx, &path);
            self.textures.insert(path.clone(), handle);
        }

        self.textures.get(&path).and_then(|opt| opt.as_ref())
    }

    fn load_texture_from_path(ctx: &Context, path: &Path) -> Option<TextureHandle> {
        let bytes = std::fs::read(path).ok()?;
        let img = image::load_from_memory(&bytes).ok()?;
        let size = [img.width() as _, img.height() as _];
        let image_buffer = img.to_rgba8();
        let pixels = image_buffer.as_flat_samples();
        let color_image = ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
        let name = path.to_string_lossy().to_string();
        Some(ctx.load_texture(name, color_image, TextureOptions::LINEAR))
    }

    pub fn get_embedded_logo(&mut self, ctx: &Context) -> Option<&TextureHandle> {
        let key = PathBuf::from("__embedded_logo__");
        if !self.textures.contains_key(&key) {
            let bytes = include_bytes!("../assets/icon.png");
            let handle = if let Ok(img) = image::load_from_memory(bytes) {
                let size = [img.width() as _, img.height() as _];
                let image_buffer = img.to_rgba8();
                let pixels = image_buffer.as_flat_samples();
                let color_image = ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                Some(ctx.load_texture("embedded_logo", color_image, TextureOptions::LINEAR))
            } else {
                None
            };
            self.textures.insert(key.clone(), handle);
        }
        self.textures.get(&key).and_then(|opt| opt.as_ref())
    }
}
