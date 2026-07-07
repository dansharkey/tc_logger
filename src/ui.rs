use crate::capture::ChannelStats;
use eframe::egui::{self, Color32, FontId, RichText, Stroke};

// ── Palette ───────────────────────────────────────────────────────────────────

pub const CHANNEL_COLORS: [Color32; 4] = [
    Color32::from_rgb(192, 57,  43),  // T1 brick red
    Color32::from_rgb(212, 122, 31),  // T2 burnt amber
    Color32::from_rgb(139, 158, 42),  // T3 olive green
    Color32::from_rgb(46,  125, 107), // T4 dark teal
];

pub const BG:        Color32 = Color32::from_rgb(245, 243, 240);
pub const BG_PANEL:  Color32 = Color32::from_rgb(238, 235, 230);
pub const BG_CARD:   Color32 = Color32::from_rgb(245, 243, 240);
pub const FG:        Color32 = Color32::from_rgb(60,  55,  50);
pub const FG_MUTED:  Color32 = Color32::from_rgb(140, 135, 128);
pub const FG_LABEL:  Color32 = Color32::from_rgb(100, 95,  90);
pub const GREEN:     Color32 = Color32::from_rgb(46,  125, 107);
pub const RED:       Color32 = Color32::from_rgb(192, 57,  43);

// ── Theme ─────────────────────────────────────────────────────────────────────

pub fn apply_theme(ctx: &egui::Context) {
    let mut v = egui::Visuals::light();
    v.panel_fill                          = BG;
    v.window_fill                         = BG;
    v.widgets.noninteractive.bg_fill      = BG_PANEL;
    v.widgets.inactive.bg_fill            = Color32::from_rgb(228, 224, 219);
    v.widgets.hovered.bg_fill             = Color32::from_rgb(215, 210, 204);
    v.widgets.active.bg_fill              = Color32::from_rgb(200, 194, 187);
    v.widgets.noninteractive.fg_stroke    = Stroke::new(1.0, FG);
    v.widgets.inactive.fg_stroke          = Stroke::new(1.0, FG);
    ctx.set_visuals(v);
}

pub fn panel_frame() -> egui::Frame {
    egui::Frame::none()
        .fill(BG_PANEL)
        .inner_margin(egui::Margin::symmetric(16.0, 10.0))
}

pub fn central_frame() -> egui::Frame {
    egui::Frame::none()
        .fill(BG)
        .inner_margin(egui::Margin::symmetric(16.0, 12.0))
}

// ── Reusable widgets ──────────────────────────────────────────────────────────

/// Flat coloured action button (Start, Stop etc.)
pub fn action_button(ui: &mut egui::Ui, label: &str, color: Color32) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).color(Color32::WHITE))
            .fill(color)
            .min_size(egui::vec2(90.0, 28.0)),
    )
}

/// Neutral button with consistent sizing.
pub fn plain_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(egui::Button::new(label).min_size(egui::vec2(90.0, 28.0)))
}

/// Muted section label (e.g. "Port", "Interval").
pub fn field_label(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text).color(FG_LABEL));
}

/// Single channel readout card.
pub fn channel_card(ui: &mut egui::Ui, name: &str, color: Color32, stats: &ChannelStats) {
    egui::Frame::none()
        .fill(BG_CARD)
        .rounding(4.0)
        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
        .show(ui, |ui| {
            // Colour swatch + name
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(4.0, 16.0),
                    egui::Sense::hover(),
                );
                ui.painter().rect_filled(rect, 0.0, color);
                ui.add_space(6.0);
                ui.label(RichText::new(name).strong().color(FG));
            });

            ui.add_space(4.0);

            match stats.latest {
                Some(v) => {
                    ui.label(
                        RichText::new(format!("{:.1} °C", v))
                            .font(FontId::proportional(26.0))
                            .color(color),
                    );
                    if let (Some(mn), Some(mx)) = (stats.min, stats.max) {
                        ui.label(
                            RichText::new(format!("↓{:.1}  ↑{:.1}", mn, mx))
                                .font(FontId::monospace(11.0))
                                .color(FG_MUTED),
                        );
                    }
                }
                None => {
                    ui.label(
                        RichText::new("—")
                            .font(FontId::proportional(26.0))
                            .color(FG_MUTED),
                    );
                }
            }
        });
}

/// Elapsed time display: MM:SS
pub fn elapsed_label(ui: &mut egui::Ui, elapsed: f64) {
    let mins = (elapsed / 60.0) as u64;
    let secs = (elapsed % 60.0) as u64;
    ui.label(
        RichText::new(format!("{:02}:{:02}", mins, secs))
            .font(FontId::monospace(14.0))
            .color(FG_LABEL),
    );
}
