#[derive(serde::Serialize)]
struct AppInfo {
    generated_by: &'static str,
    platform: &'static str,
    family: &'static str,
}

#[tauri::command]
fn app_info() -> AppInfo {
    AppInfo {
        generated_by: "OpenAI Codex agents",
        platform: std::env::consts::OS,
        family: std::env::consts::FAMILY,
    }
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![app_info])
        .run(tauri::generate_context!())
        .expect("failed to run Tank Battle desktop app");
}
