use eframe::egui;

#[derive(Default, Clone)]
pub struct ImageTextures {
    pub forge: Option<egui::TextureHandle>,
    pub fabric: Option<egui::TextureHandle>,
    pub forge_and_fabric: Option<egui::TextureHandle>,
    pub none: Option<egui::TextureHandle>,
    pub local: Option<egui::TextureHandle>,
    pub curseforge: Option<egui::TextureHandle>,
    pub modrinth: Option<egui::TextureHandle>,
    pub bin: Option<egui::TextureHandle>,
    pub mod_status_ok: Option<egui::TextureHandle>,
    pub mod_status_outdated: Option<egui::TextureHandle>,
    pub mod_status_invalid: Option<egui::TextureHandle>,
}

impl ImageTextures {
    /// This function should not be called in the update method
    pub fn load_images(&mut self, ctx: &egui::Context) {
        self.forge = Some(ctx.load_texture(
            "forge-icon",
            load_image_from_memory(include_bytes!("../../res/forge.png")).unwrap(),
        ));

        self.fabric = Some(ctx.load_texture(
            "fabric-icon",
            load_image_from_memory(include_bytes!("../../res/fabric.png")).unwrap(),
        ));

        self.forge_and_fabric = Some(ctx.load_texture(
            "forge_and_fabric-icon",
            load_image_from_memory(include_bytes!("../../res/forge_and_fabric.png")).unwrap(),
        ));

        self.none = Some(ctx.load_texture(
            "source-local-icon",
            load_image_from_memory(include_bytes!("../../res/none.png")).unwrap(),
        ));

        self.local = Some(ctx.load_texture(
            "source-local-icon",
            load_image_from_memory(include_bytes!("../../res/local.png")).unwrap(),
        ));

        self.curseforge = Some(ctx.load_texture(
            "source-curseforge-icon",
            load_image_from_memory(include_bytes!("../../res/curseforge.png")).unwrap(),
        ));

        self.modrinth = Some(ctx.load_texture(
            "source-modrinth-icon",
            load_image_from_memory(include_bytes!("../../res/modrinth.png")).unwrap(),
        ));

        self.bin = Some(ctx.load_texture(
            "bin-icon",
            load_image_from_memory(include_bytes!("../../res/bin.png")).unwrap(),
        ));

        self.mod_status_ok = Some(ctx.load_texture(
            "mod-status-ok",
            load_image_from_memory(include_bytes!("../../res/status_ok.png")).unwrap(),
        ));

        self.mod_status_outdated = Some(ctx.load_texture(
            "mod-status-outdated",
            load_image_from_memory(include_bytes!("../../res/status_outdated.png")).unwrap(),
        ));

        self.mod_status_invalid = Some(ctx.load_texture(
            "mod-status-invalid",
            load_image_from_memory(include_bytes!("../../res/status_invalid.png")).unwrap(),
        ));
    }
}

fn load_image_from_memory(image_data: &[u8]) -> Result<egui::ColorImage, image::ImageError> {
    let image = image::load_from_memory(image_data)?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(egui::ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    ))
}
