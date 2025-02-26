use eframe::{egui, epi};
use egui::{panel::Side, Layout};
use serialport::SerialPort;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Preset {
    pub name: String,
    pub on: [bool; 6],
    pub brightness: [u8; 6],
    pub temperature: u8,
}

impl Default for Preset {
    fn default() -> Self {
        Self {
            name: String::from("Untitled"),
            on: [false; 6],
            brightness: [20; 6],
            temperature: 33,
        }
    }
}

pub struct ControllerApp {
    presets: Vec<Preset>,
    current_preset_idx: usize,

    port: Box<dyn SerialPort>,
    last_send_time: Option<f64>,
    latest_sent: bool,

    editing_idx: Option<usize>,
}

impl ControllerApp {
    pub fn new(port: Box<dyn SerialPort>) -> Self {
        Self {
            presets: vec![Preset::default()],
            current_preset_idx: 0,
            port,
            last_send_time: None,
            latest_sent: false,
            editing_idx: None,
        }
    }
}

impl epi::App for ControllerApp {
    fn name(&self) -> &str {
        "VL200T Controller"
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &mut epi::Frame<'_>,
        storage: Option<&dyn epi::Storage>,
    ) {
        if let Some(storage) = storage {
            let (presets, current_preset_idx): (Vec<Preset>, usize) = epi::get_value(storage, epi::APP_KEY).unwrap_or_default();
            if !presets.is_empty() {
                self.presets = presets;
            }
            self.current_preset_idx = current_preset_idx.min(self.presets.len() - 1);
        }
    }

    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, &(&self.presets, self.current_preset_idx));
    }

    fn on_exit(&mut self) {
        let _ = self.port.write("0..0..0..0..0..0.. ".as_bytes());
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        let Self {
            presets,
            current_preset_idx,
            port,
            last_send_time,
            latest_sent,
            editing_idx,
        } = self;

        if let Some(idx) = editing_idx {
            let idx = *idx;
            egui::Window::new(format!("Rename \"{}\"", presets[idx].name))
                .id(egui::Id::new("editing_window"))
                .collapsible(false)
                .show(ctx, |ui| {
                    let response = ui.text_edit_singleline(&mut presets[idx].name);

                    if response.lost_focus() {
                        editing_idx.take();
                    }
                    response.request_focus();
                });
        }

        egui::SidePanel::new(Side::Right, "main_controls")
            .resizable(false)
            .min_width(600.0)
            .show(ctx, |ui| {
                ui.style_mut().spacing.slider_width = ui.available_width() - 60.0;
                ui.add_space(5.0);
                ui.add(egui::Label::new("Light Temperature"));

                let current_preset = &mut presets[*current_preset_idx];
                let temperature_response = ui
                    .add(egui::Slider::new(&mut current_preset.temperature, 33..=56).suffix("00k"));
                ui.add_space(30.0);

                ui.add(egui::Separator::default());
                let (on_responses, brightness_responses): (Vec<_>, Vec<_>) = current_preset
                    .on
                    .iter_mut()
                    .zip(current_preset.brightness.iter_mut())
                    .enumerate()
                    .map(|(i, (on, brightness))| {
                        let on_response =
                            ui.checkbox(on, format!("Light {}", (i as u8 + 'A' as u8) as char));
                        let brightness_response =
                            ui.add(egui::Slider::new(brightness, 20..=100).suffix("%"));
                        ui.add(egui::Separator::default());

                        (on_response.changed(), brightness_response.changed())
                    })
                    .unzip();

                let power_state_changed = on_responses.iter().fold(false, |acc, &e| acc || e);
                let brightness_changed =
                    brightness_responses.iter().fold(false, |acc, &e| acc || e);
                let any_on = current_preset
                    .on
                    .iter()
                    .copied()
                    .any(std::convert::identity);

                if (temperature_response.changed() || power_state_changed || brightness_changed)
                    && any_on
                {
                    *latest_sent = false;
                }

                let time = ui.input().time;
                let waited_long_enough = last_send_time
                    .map(|last_send_time| time - last_send_time > 0.1)
                    .unwrap_or(true);
                let send_available = !*latest_sent && waited_long_enough;
                if send_available {
                    let message: Vec<u8> = current_preset
                        .on
                        .iter()
                        .zip(current_preset.brightness.iter())
                        .flat_map(|(on, brightness)| {
                            [
                                if *on { '1' as u8 } else { '0' as u8 },
                                '.' as u8 + (*brightness - 20),
                                '.' as u8 + (current_preset.temperature - 33),
                            ]
                            .into_iter()
                        })
                        .chain(std::iter::once(' ' as u8))
                        .collect();
                    let send_string = String::from_utf8(message).unwrap();
                    println!("Sending {}", send_string);
                    port.write(send_string.as_bytes()).unwrap();
                    *latest_sent = true;
                    *last_send_time = Some(time);
                }
                egui::warn_if_debug_build(ui);
            });

        egui::TopBottomPanel::new(egui::panel::TopBottomSide::Bottom, "preset_control").show(
            ctx,
            |ui| {
                ui.horizontal(|ui| {
                    ui.with_layout(Layout::left_to_right(), |ui| {
                        if ui.button("🗋 New Preset").clicked() {
                            presets.push(Default::default());
                        }
                        if ui.button("🗑 Delete").clicked() {
                            if presets.len() > 1 {
                                presets.remove(dbg!(*current_preset_idx));
                                *current_preset_idx = (*current_preset_idx).min(presets.len() - 1);
                                *latest_sent = false;
                            }
                        }
                    });
                });
            },
        );

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.set_min_width(135.0);
            let row_height = ui.fonts()[egui::TextStyle::Body].row_height();
            egui::ScrollArea::new([false, true])
                .auto_shrink([false, true])
                .show_rows(ui, row_height, presets.len(), |ui, row_range| {
                    for i in row_range {
                        let response =
                            ui.selectable_label(i == *current_preset_idx, &presets[i].name);
                        if response.clicked() {
                            if i != *current_preset_idx {
                                *current_preset_idx = i;
                                *latest_sent = false;
                            }
                        }

                        if response.double_clicked() {
                            editing_idx.replace(*current_preset_idx);
                        }
                    }
                });
        });

        let mut size = ctx.used_size();
        size.x = size.x.max(800.0);
        size.y = size.y.max(400.0);

        if ctx.used_size() != size {
            frame.set_window_size(size);
        }
    }
}
