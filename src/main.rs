mod app;
mod exporter;
mod manager;
mod model;
mod parser;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([900.0, 600.0])
            .with_title("Bookmark Studio"),
        ..Default::default()
    };

    eframe::run_native(
        "Bookmark Studio",
        options,
        Box::new(|cc| {
            // egui's built-in fonts do not contain Chinese glyphs. Bundle Noto Sans SC
            // so the UI works even on Windows systems without a Chinese font installed.
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert(
                "noto_sans_sc".to_owned(),
                egui::FontData::from_static(include_bytes!(
                    "../assets/NotoSansSC-Regular.ttf"
                )),
            );
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "noto_sans_sc".to_owned());
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .insert(0, "noto_sans_sc".to_owned());
            cc.egui_ctx.set_fonts(fonts);
            Ok(Box::new(app::BookmarkApp::default()))
        }),
    )
}
