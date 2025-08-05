use eframe::egui;
use image::{ImageBuffer, Rgba};
use once_cell::sync::OnceCell;
use scap::{capturer::{Capturer, Options, Resolution, Area, Point, Size},
           frame::{FrameType, Frame},
};
use std::{process, sync::Mutex};

fn scap_checks() {
    if !scap::is_supported() {
        eprintln!("Platform not supported");
        process::exit(0);
    };
    if !scap::has_permission() {
        println!("Requesting permission");
        if !scap::request_permission() {
            eprintln!("Permission_denied");
            process::exit(0);
        }
    }
}

fn main() {
    scap_checks();
    let viewport = egui::viewport::ViewportBuilder::default()
        .with_transparent(true)
        .with_decorations(false)
        .with_window_level(egui::viewport::WindowLevel::AlwaysOnTop)
        .with_inner_size(egui::Vec2::new(100.0,100.0))
        .with_position(egui::Pos2::new(0.0,0.0));

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    match eframe::run_native(
        "My egui App",
        native_options,
        Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc))))
    ) {
        Ok(()) => {},
        Err(e) => eprintln!("Application Failed to start: {}", e),
    };
    std::thread::sleep(std::time::Duration::from_secs(10));
}

/*
impl Default for MyEguiApp {
    fn default() -> Self {
        Self {
            capturer: init_capturer()
        }
    }
}*/

fn init_capturer() -> Option<Capturer> {
    match Capturer::build(
        Options {
            fps: 1,
            target: None, //Primary display
            show_cursor: false,
            show_highlight: false,
            excluded_targets: None,
            output_type: FrameType::BGRAFrame,
            output_resolution: Resolution::_1080p,
            crop_area: None,
            ..Default::default()
        }
    ) {
        Ok(cap) => {
            println!("AAAAA");
            Some(cap)
        } Err(e) => {
            println!("AAAAA");
            eprintln!("Failed to build capturer: {:?}", e);
            None
        }
    }
    
}

#[derive(Default)]
struct MyEguiApp {
    capturer: Option<Mutex<Capturer>>,
}

impl MyEguiApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut visuals = egui::Visuals::dark();
        visuals.window_fill = egui::Color32::TRANSPARENT;
        visuals.widgets.noninteractive.bg_fill = egui::Color32::TRANSPARENT;
        cc.egui_ctx.set_visuals(visuals);

        Self {
            capturer: init_capturer().map(Mutex::new),
        }
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, uiframe: &mut eframe::Frame) {
        println!("UPDATE");
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                  ui.heading("Hello World!");
                if ui.button("Capture").clicked() {
                    println!("Screen captured");
                    if let Some(capturer_mutex) = &self.capturer {
                        let mut capturer = capturer_mutex.lock().unwrap();
                        capturer.start_capture();
                        if let Ok(frame) = capturer.get_next_frame() {
                            match frame {
                                Frame::BGRA(frame_data) => {
                                    let width: u32 = frame_data.width
                                        .try_into().unwrap();
                                    let height: u32 = frame_data.height
                                        .try_into().unwrap();
                                    let buffer = frame_data.data;
                                    let image: ImageBuffer<Rgba<u8>, _> =
                                        ImageBuffer::from_fn(width, height, |x, y| {
                                            let i = ((y * width + x) * 4) as usize;
                                            let b = buffer[i];
                                            let g = buffer[i+1];
                                            let r = buffer[i+2];
                                            let a = buffer[i+3];
                                            Rgba([r, g, b, a])
                                        });
                                    let _ = image.save("/app/workdir/image.png");
                                },
                                _ => println!("Not a BGRA frame"),
                            }
                        } else {eprintln!("Failed to receive frame")};
                        capturer.stop_capture();
                    }
                }
                if ui.button("Close").clicked() {
                    println!("Exiting");
                    let ctx = ctx.clone();
                    std::thread::spawn(move || {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    });
                }
            }
        );
    }
}
