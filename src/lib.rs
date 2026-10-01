use eframe::egui;
use std::fs;
use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Instant;
use walkdir::WalkDir;

#[derive(Clone)]
enum LogMessage {
    Info(String),
    Warning {
        path: PathBuf,
        line: usize,
        text: String,
    },
    Critical {
        path: PathBuf,
        line: usize,
        text: String,
    },
    Done {
        ms: f64,
        file_count: usize,
    },
}

#[derive(Clone, Copy, PartialEq)]
enum Language {
    Cs,
    En,
    Es,
}

fn t(lang: Language, key: &str) -> &'static str {
    match (lang, key) {
        (Language::Cs, "title") => "🔍 Hardcoded Hunter",
        (Language::En, "title") => "🔍 Hardcoded Hunter",
        (Language::Es, "title") => "🔍 Hardcoded Hunter",
        (Language::Cs, "folder_label") => "Cílová složka:",
        (Language::En, "folder_label") => "Target folder:",
        (Language::Es, "folder_label") => "Carpeta de destino:",
        (Language::Cs, "select_btn") => "📁 Vybrat složku...",
        (Language::En, "select_btn") => "📁 Select folder...",
        (Language::Es, "select_btn") => "📁 Seleccionar carpeta...",
        (Language::Cs, "start_btn") => "Spustit analýzu",
        (Language::En, "start_btn") => "Start analysis",
        (Language::Es, "start_btn") => "Iniciar análisis",
        (Language::Cs, "running_btn") => "Analýza...",
        (Language::En, "running_btn") => "Analyzing...",
        (Language::Es, "running_btn") => "Analizando...",
        (Language::Cs, "outputs_heading") => "Výstupy a nálezy:",
        (Language::En, "outputs_heading") => "Outputs and findings:",
        (Language::Es, "outputs_heading") => "Resultados y hallazgos:",
        (Language::Cs, "language_label") => "Jazyk / Language:",
        (Language::En, "language_label") => "Language:",
        (Language::Es, "language_label") => "Idioma:",
        _ => "Unknown text",
    }
}

struct HardcodedHunterApp {
    target_dir: String,
    is_running: bool,
    logs: Vec<(LogMessage, egui::Color32)>,
    rx: Option<mpsc::Receiver<LogMessage>>,
    current_lang: Language,
}

impl Default for HardcodedHunterApp {
    fn default() -> Self {
        Self {
            target_dir: "./".to_string(),
            is_running: false,
            logs: Vec::new(),
            rx: None,
            current_lang: Language::En,
        }
    }
}

impl eframe::App for HardcodedHunterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Kontrola, zda nám běží nějaké nové zprávy z vláken
        if let Some(ref rx) = self.rx {
            // Vybíráme zprávy bez blokování okna
            while let Ok(msg) = rx.try_recv() {
                let color = match &msg {
                    LogMessage::Info(_) => egui::Color32::WHITE,
                    LogMessage::Warning { .. } => egui::Color32::YELLOW,
                    LogMessage::Critical { .. } => egui::Color32::LIGHT_RED,
                    LogMessage::Done { .. } => egui::Color32::LIGHT_GREEN,
                };

                if let LogMessage::Done { .. } = msg {
                    self.is_running = false;
                }

                self.logs.push((msg, color));
            }
        }

        // Pokud analýza běží, vyžádáme si od UI pravidelný repaint, aby se hýbal výpis
        if self.is_running {
            ctx.request_repaint();
        }

        // --- HLAVNÍ GRAFICKÉ ROZHRANÍ ---
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(t(self.current_lang, "title"));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .selectable_label(self.current_lang == Language::Es, "🇪🇸 ES")
                        .clicked()
                    {
                        self.current_lang = Language::Es;
                    }
                    if ui
                        .selectable_label(self.current_lang == Language::En, "🇬🇧 EN")
                        .clicked()
                    {
                        self.current_lang = Language::En;
                    }
                    if ui
                        .selectable_label(self.current_lang == Language::Cs, "🇨🇿 CS")
                        .clicked()
                    {
                        self.current_lang = Language::Cs;
                    }
                    ui.label(t(self.current_lang, "language_label"));
                });
            });
            ui.add_space(10.0);

            // Výběr adresáře
            ui.horizontal(|ui| {
                ui.label(t(self.current_lang, "folder_label"));
                ui.text_edit_singleline(&mut self.target_dir);

                #[cfg(not(target_os = "android"))]
                if ui.button(t(self.current_lang, "select_btn")).clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.target_dir = path.display().to_string();
                    }
                }
            });

            ui.add_space(10.0);

            // Tlačítko pro spuštění
            let button_text = if self.is_running {
                t(self.current_lang, "running_btn")
            } else {
                t(self.current_lang, "start_btn")
            };
            if ui
                .add_enabled(!self.is_running, egui::Button::new(button_text))
                .clicked()
            {
                self.start_analysis();
            }

            ui.add_space(15.0);
            ui.separator();
            ui.heading(t(self.current_lang, "outputs_heading"));
            ui.add_space(5.0);

            // Okno s výpisy (Scrollable area)
            egui::ScrollArea::vertical()
                .max_height(ui.available_height() - 20.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for (msg, color) in &self.logs {
                        match msg {
                            LogMessage::Info(text) => {
                                ui.colored_label(*color, text);
                            }
                            LogMessage::Warning { path, line, text } => {
                                ui.colored_label(
                                    *color,
                                    format!("[WARN] {:?}:{} -> {}", path, line, text),
                                );
                            }
                            LogMessage::Critical { path, line, text } => {
                                ui.colored_label(
                                    *color,
                                    format!("[CRITICAL] {:?}:{} -> \"{}\"", path, line, text),
                                );
                            }
                            LogMessage::Done { ms, file_count } => {
                                ui.colored_label(
                                    *color,
                                    format!(
                                        "✔ Dokončeno v {:.2} ms (celkem prověřeno {} souborů)",
                                        ms, file_count
                                    ),
                                );
                            }
                        }
                    }
                });
        });
    }
}

impl HardcodedHunterApp {
    fn start_analysis(&mut self) {
        self.is_running = true;
        self.logs.clear();

        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);

        let target_dir = self.target_dir.clone();

        // Spustíme analýzu na pozadí, aby GUI nezamrzlo
        thread::spawn(move || {
            let start_time = Instant::now();
            let _ = tx.send(LogMessage::Info(format!(
                "Spouštím analýzu pro: {}",
                target_dir
            )));

            let (path_tx, path_rx) = mpsc::channel::<PathBuf>();
            let path_rx = Arc::new(Mutex::new(path_rx));

            let num_cpus_available = num_cpus::get();
            let num_workers = if num_cpus_available > 2 {
                num_cpus_available - 2
            } else {
                1
            };

            let _ = tx.send(LogMessage::Info(format!(
                "Aktivní worker pool: {} vláken (jader: {})",
                num_workers, num_cpus_available
            )));

            // Spuštění workerů
            let mut workers = vec![];
            for _ in 0..num_workers {
                let path_rx_clone = Arc::clone(&path_rx);
                let tx_clone = tx.clone();

                let handle = thread::spawn(move || loop {
                    let path = {
                        let locked = path_rx_clone.lock().unwrap();
                        match locked.recv() {
                            Ok(p) => p,
                            Err(_) => break,
                        }
                    };

                    process_file(&path, &tx_clone);
                });
                workers.push(handle);
            }

            // Produkce cest souborů (Jádro 0)
            let mut file_count = 0;
            for entry in WalkDir::new(&target_dir).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        if matches!(ext, "rs" | "cs" | "py" | "js" | "cpp" | "html" | "ts") {
                            file_count += 1;
                            let _ = path_tx.send(path.to_path_buf());
                        }
                    }
                }
            }

            drop(path_tx);

            for worker in workers {
                worker.join().unwrap();
            }

            let duration = start_time.elapsed();
            let ms = duration.as_secs_f64() * 1000.0;

            let _ = tx.send(LogMessage::Done { ms, file_count });
        });
    }
}

fn process_file(path: &PathBuf, tx: &mpsc::Sender<LogMessage>) {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };

    for (index, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') {
            continue;
        }

        // Ignorujeme sami sebe (pokud bychom skenovali tenhle nástroj), nebo filtrujeme řetězce
        if trimmed.contains('"') && !trimmed.ends_file_or_semicolon() {
            if trimmed.contains("assets/") || trimmed.contains('/') || trimmed.contains('.') {
                let _ = tx.send(LogMessage::Warning {
                    path: path.clone(),
                    line: index + 1,
                    text: trimmed.to_string(),
                });
            } else {
                let _ = tx.send(LogMessage::Critical {
                    path: path.clone(),
                    line: index + 1,
                    text: trimmed.to_string(),
                });
            }
        }
    }
}

// Malá pomocná metoda pro čistší kód nad řetězci
trait StringExt {
    fn ends_file_or_semicolon(&self) -> bool;
}

impl StringExt for str {
    fn ends_file_or_semicolon(&self) -> bool {
        self.ends_with(';') || self.ends_with(',')
    }
}

fn native_options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("Hardcoded Hunter"),
        ..Default::default()
    }
}

fn run_app(options: eframe::NativeOptions) -> eframe::Result<()> {
    eframe::run_native(
        "Hardcoded Hunter",
        options,
        Box::new(|_cc| Ok(Box::new(HardcodedHunterApp::default()))),
    )
}

#[cfg(not(target_os = "android"))]
pub fn run_desktop() -> eframe::Result<()> {
    run_app(native_options())
}

#[cfg(target_os = "android")]
#[no_mangle]
pub fn android_main(app: winit::platform::android::activity::AndroidApp) {
    let mut options = native_options();
    options.event_loop_builder = Some(Box::new(move |event_loop| {
        use winit::platform::android::EventLoopBuilderExtAndroid;
        event_loop.with_android_app(app);
    }));

    if let Err(error) = run_app(options) {
        panic!("failed to start Hardcoded Hunter: {error}");
    }
}
