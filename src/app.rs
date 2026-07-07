use crate::capture::{Capture, ChannelStats, State, CHANNEL_NAMES};
use crate::{export, serial, ui};
use eframe::egui;
use egui::Color32;
use egui_plot::{Legend, Line, Plot, PlotPoints, VLine, Corner, LineStyle};
use std::time::Duration;

pub struct App {
    capture: Capture,
    port_list: Vec<String>,
    selected_port: usize,
    interval_ms: u64,
    marker_label: String,
    view_window: f64,
    last_export: Option<String>,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        Self {
            capture: Capture::new(),
            port_list: serial::list_ports(),
            selected_port: 0,
            interval_ms: 1000,
            marker_label: "contact".to_string(),
            view_window: 0.0,
            last_export: None,
        }
    }

    fn selected_port_name(&self) -> Option<&str> {
        self.port_list.get(self.selected_port).map(String::as_str)
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ui::apply_theme(ctx);

        if self.capture.state() == State::Running {
            ctx.request_repaint_after(Duration::from_millis(250));
        }

        // Stop and surface error if serial thread died
        if self.capture.error().is_some() && self.capture.state() != State::Idle {
            self.capture.stop();
        }

        let (samples, markers) = self.capture.snapshot();
        let state = self.capture.state();

        // Pre-compute stats once per frame
        let stats: Vec<ChannelStats> = (0..4)
            .map(|i| ChannelStats::compute(&samples, i))
            .collect();

        // ── Top panel ─────────────────────────────────────────────────────────
        egui::TopBottomPanel::top("controls")
            .frame(ui::panel_frame())
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 12.0;

                    // Port
                    ui::field_label(ui, "Port");
                    if state == State::Idle {
                        egui::ComboBox::from_id_source("port")
                            .width(160.0)
                            .show_index(ui, &mut self.selected_port, self.port_list.len(), |i| {
                                self.port_list.get(i).cloned().unwrap_or_default()
                            });
                        if ui.small_button("↻").clicked() {
                            self.port_list = serial::list_ports();
                            self.selected_port = 0;
                        }
                    } else {
                        ui.strong(self.selected_port_name().unwrap_or("—"));
                    }

                    ui.separator();

                    // Interval
                    ui::field_label(ui, "Interval");
                    egui::ComboBox::from_id_source("interval")
                        .width(70.0)
                        .selected_text(format!("{}ms", self.interval_ms))
                        .show_ui(ui, |ui| {
                            for &ms in &[500u64, 1000, 2000, 5000] {
                                ui.selectable_value(&mut self.interval_ms, ms, format!("{}ms", ms));
                            }
                        });

                    ui.separator();

                    // Transport controls
                    match state {
                        State::Idle => {
                            if ui::action_button(ui, "▶  Start", ui::GREEN).clicked() {
                                if let Some(port) = self.selected_port_name() {
                                    self.capture.start(port, self.interval_ms);
                                    self.last_export = None;
                                }
                            }
                        }
                        State::Running => {
                            if ui::plain_button(ui, "⏸  Pause").clicked() { self.capture.pause(); }
                            if ui::action_button(ui, "■  Stop", ui::RED).clicked() { self.capture.stop(); }
                        }
                        State::Paused => {
                            if ui::action_button(ui, "▶  Resume", ui::GREEN).clicked() { self.capture.resume(); }
                            if ui::action_button(ui, "■  Stop", ui::RED).clicked() { self.capture.stop(); }
                        }
                    }

                    ui.separator();

                    // Marker
                    ui::field_label(ui, "Mark");
                    ui.add(egui::TextEdit::singleline(&mut self.marker_label).desired_width(100.0));
                    if ui.add_enabled(
                        state == State::Running,
                        egui::Button::new("⚑ Mark").min_size(egui::vec2(70.0, 28.0)),
                    ).clicked() {
                        self.capture.add_marker(self.marker_label.clone());
                    }

                    ui.separator();

                    // View window
                    ui::field_label(ui, "View");
                    egui::ComboBox::from_id_source("view")
                        .width(70.0)
                        .selected_text(if self.view_window == 0.0 { "All".into() } else { format!("{}s", self.view_window as u64) })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.view_window, 0.0, "All");
                            for &w in &[30.0f64, 60.0, 120.0, 300.0] {
                                ui.selectable_value(&mut self.view_window, w, format!("{}s", w as u64));
                            }
                        });

                    // Right side
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add_enabled(
                            !samples.is_empty(),
                            egui::Button::new("↓ Export CSV").min_size(egui::vec2(100.0, 28.0)),
                        ).clicked() {
                            self.last_export = export::write(&samples, &markers).ok();
                        }

                        if let Some(ref name) = self.last_export {
                            ui.label(egui::RichText::new(format!("✓ {}", name)).color(ui::GREEN));
                        }

                        if state != State::Idle {
                            ui::elapsed_label(ui, self.capture.elapsed());
                        }
                    });
                });

                if let Some(ref err) = self.capture.error() {
                    ui.colored_label(ui::RED, format!("⚠ {}", err));
                }
            });

        // ── Right panel: channel readouts ─────────────────────────────────────
        egui::SidePanel::right("readouts")
            .min_width(160.0)
            .max_width(180.0)
            .frame(ui::panel_frame())
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 16.0;

                for (i, name) in CHANNEL_NAMES.iter().enumerate() {
                    ui::channel_card(ui, name, ui::CHANNEL_COLORS[i], &stats[i]);
                }

                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.label(
                        egui::RichText::new(format!("{} samples", samples.len()))
                            .font(egui::FontId::monospace(10.0))
                            .color(ui::FG_MUTED),
                    );
                });
            });

        // ── Central panel: plot ───────────────────────────────────────────────
        egui::CentralPanel::default()
            .frame(ui::central_frame())
            .show(ctx, |ui| {
                let mut plot = Plot::new("temp_plot")
                    .label_formatter(|name, value| {
                        if name.is_empty() { return String::new(); }
                        format!("{}: {:.1} °C @ {:.1}s", name, value.y, value.x)
                    })
                    .x_axis_label("Elapsed (s)")
                    .y_axis_label("Temperature (°C)")
                    .legend(Legend::default().position(Corner::LeftTop))
                    .allow_drag(true)
                    .allow_zoom(true)
                    .allow_scroll(true);

                if self.view_window > 0.0 {
                    if let Some(last) = samples.last() {
                        let x_min = (last.elapsed - self.view_window).max(0.0);
                        plot = plot.include_x(x_min).include_x(last.elapsed);
                    }
                }

                plot.show(ui, |plot_ui| {
                    for (i, name) in CHANNEL_NAMES.iter().enumerate() {
                        let points: PlotPoints = samples.iter()
                            .filter(|s| !s.temps[i].is_nan())
                            .map(|s| [s.elapsed, s.temps[i] as f64])
                            .collect();

                        plot_ui.line(
                            Line::new(points)
                                .name(*name)
                                .color(ui::CHANNEL_COLORS[i])
                                .width(1.8),
                        );
                    }

                    for marker in &markers {
                        plot_ui.vline(
                            VLine::new(marker.elapsed)
                                .color(Color32::from_rgba_premultiplied(80, 75, 70, 180))
                                .width(1.0)
                                .style(LineStyle::Dashed { length: 6.0 })
                                .name(&marker.label),
                        );
                    }
                });
            });
    }
}
