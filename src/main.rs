mod app;
mod capture;
mod export;
mod serial;
mod ui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Thermocouple Logger")
            .with_inner_size([1100.0, 680.0])
            .with_min_inner_size([800.0, 500.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Thermocouple Logger",
        options,
        Box::new(|cc| Box::new(app::App::new(cc))),
    )
}
