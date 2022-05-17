use eframe::egui;

#[derive(Clone)]
pub struct ImageTextures {
    pub forge: egui::TextureHandle,
    pub fabric: egui::TextureHandle,
    pub none: egui::TextureHandle,
    pub local: egui::TextureHandle,
    pub curseforge: egui::TextureHandle,
    pub modrinth: egui::TextureHandle,
    pub bin: egui::TextureHandle,
    pub mod_status_ok: egui::TextureHandle,
    pub mod_status_outdated: egui::TextureHandle,
    pub mod_status_invalid: egui::TextureHandle,
    pub settings: egui::TextureHandle,
}

impl ImageTextures {
    /// This function should not be called in the update method
    pub fn new(ctx: &egui::Context) -> Self {
        let forge = ctx.load_texture(
            "forge-icon",
            load_image_from_memory(include_bytes!("../../res/forge.png")).unwrap(),
        );

        let fabric = ctx.load_texture(
            "fabric-icon",
            load_image_from_memory(include_bytes!("../../res/fabric.png")).unwrap(),
        );

        let none = ctx.load_texture(
            "source-local-icon",
            load_image_from_memory(include_bytes!("../../res/none.png")).unwrap(),
        );

        let local = ctx.load_texture(
            "source-local-icon",
            load_image_from_memory(include_bytes!("../../res/local.png")).unwrap(),
        );

        let curseforge = ctx.load_texture(
            "source-curseforge-icon",
            load_image_from_memory(include_bytes!("../../res/curseforge.png")).unwrap(),
        );

        let modrinth = ctx.load_texture(
            "source-modrinth-icon",
            load_image_from_memory(include_bytes!("../../res/modrinth.png")).unwrap(),
        );

        let bin = ctx.load_texture(
            "bin-icon",
            load_image_from_memory(include_bytes!("../../res/bin.png")).unwrap(),
        );

        let mod_status_ok = ctx.load_texture(
            "mod-status-ok",
            load_image_from_memory(include_bytes!("../../res/status_ok.png")).unwrap(),
        );

        let mod_status_outdated = ctx.load_texture(
            "mod-status-outdated",
            load_image_from_memory(include_bytes!("../../res/status_outdated.png")).unwrap(),
        );

        let mod_status_invalid = ctx.load_texture(
            "mod-status-invalid",
            load_image_from_memory(include_bytes!("../../res/status_invalid.png")).unwrap(),
        );

        let settings = ctx.load_texture(
            "settings-icon",
            load_image_from_memory(include_bytes!("../../res/settings.png")).unwrap(),
        );

        Self {
            forge,
            fabric,
            none,
            local,
            curseforge,
            modrinth,
            bin,
            mod_status_ok,
            mod_status_outdated,
            mod_status_invalid,
            settings,
        }
    }
}

fn load_image_from_memory(image_data: &[u8]) -> Result<egui::ColorImage, image::ImageError> {
    let image = image::load_from_memory(image_data)?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()))
}
