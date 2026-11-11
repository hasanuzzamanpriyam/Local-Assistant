mod config;

fn main() {
    let config = config::Config::load().expect("Failed to load config");
    println!("Config loaded: {}", config.model.default);
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}