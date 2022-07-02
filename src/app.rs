use std::fs::File;
use std::io::Read;

use egui::vec2;
use egui::Color32;
use egui::DroppedFile;
use egui::InputState;
use egui::Key;
use egui::Rect;
use egui::Sense;
use egui::Stroke;

use crate::chip8::Chip8;

pub struct EmulatorApp {
    emulator: Chip8,
    pixels: [[u8; 64]; 32],
}

impl EmulatorApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        EmulatorApp {
            emulator: Chip8::with_program(include_bytes!("ibm_logo.ch8")),
            pixels: [[0u8; 64]; 32],
        }
    }

    pub fn render_display(&mut self, ui: &mut egui::Ui) {
        let desired_size = ui.available_size();
        let (whole_rect, _) =
            ui.allocate_exact_size(desired_size, Sense::focusable_noninteractive());

        // Create fading effect by adding/substracting from internal buffer of
        // pixel brightness.
        let new_pixels = &self.emulator.display.pixels;
        for (y, row) in new_pixels.iter().enumerate() {
            for (x, pixel) in row.iter().enumerate() {
                if *pixel {
                    self.pixels[y][x] = self.pixels[y][x].saturating_add(128);
                } else {
                    self.pixels[y][x] = self.pixels[y][x].saturating_sub(16);
                }
            }
        }

        // Draw pixels as rects
        let stroke = Stroke::new(0.5, Color32::from_gray(80));
        let pixel_width = whole_rect.width() / 64.0;
        let pixel_height = whole_rect.height() / 32.0;
        if ui.is_rect_visible(whole_rect) {
            for (y, row) in self.pixels.iter().enumerate() {
                for (x, pixel) in row.iter().enumerate() {
                    let min =
                        whole_rect.min + vec2(x as f32 * pixel_width, y as f32 * pixel_height);
                    let pixel_rect = Rect::from_min_size(min, vec2(pixel_width, pixel_height));
                    let color = Color32::from_gray(*pixel);
                    ui.painter().rect(pixel_rect, 0.0, color, stroke)
                }
            }
        }
    }

    fn load_dropped_file(&mut self, drop: &DroppedFile) {
        if let Some(path) = &drop.path {
            let mut data: Vec<u8> = Vec::new();
            File::open(path).unwrap().read_to_end(&mut data).unwrap();
            self.emulator = Chip8::with_program(&data);
        } else if let Some(bytes) = &drop.bytes {
            self.emulator = Chip8::with_program(&*bytes);
        }
    }

    fn update_keys(&mut self, input: &InputState) {
        let keys = &mut self.emulator.keys;
        keys[0x0] = input.key_down(Key::Num0);
        keys[0x1] = input.key_down(Key::Num1);
        keys[0x2] = input.key_down(Key::Num2);
        keys[0x3] = input.key_down(Key::Num3);

        keys[0x4] = input.key_down(Key::Num4);
        keys[0x5] = input.key_down(Key::Num5);
        keys[0x6] = input.key_down(Key::Num6);
        keys[0x7] = input.key_down(Key::Num7);

        keys[0x8] = input.key_down(Key::Num8);
        keys[0x9] = input.key_down(Key::Num9);
        keys[0xA] = input.key_down(Key::A);
        keys[0xB] = input.key_down(Key::B);

        keys[0xC] = input.key_down(Key::C);
        keys[0xD] = input.key_down(Key::D);
        keys[0xE] = input.key_down(Key::E);
        keys[0xF] = input.key_down(Key::F);
    }
}

impl eframe::App for EmulatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Load new program if a file is dropped on the app
        if !ctx.input().raw.dropped_files.is_empty() {
            self.load_dropped_file(&ctx.input().raw.dropped_files[0]);
        }
        self.update_keys(&ctx.input());

        // egui is rendering at 60Hz, Chip8 runs at 500Hz, so we need to run
        // 8-ish cpu cycles for each frame.
        for _ in 0..8 {
            self.emulator.emulate_tick().unwrap();
        }

        // Render emulator display
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_display(ui);
        });

        // Always repaint to keep rendering at 60Hz.
        ctx.request_repaint()
    }
}