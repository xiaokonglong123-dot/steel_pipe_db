mod app;
mod config;
mod database;

use eframe::egui;
use std::sync::Arc;

fn main() -> Result<(), eframe::Error> {
    // 初始化日志
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // 加载配置
    let config = config::Config::load("config.toml").unwrap_or_else(|e| {
        eprintln!("配置文件加载失败: {}，使用默认配置", e);
        config::Config::default()
    });

    // 初始化数据库
    let db = match database::Database::new(&config.database.path) {
        Ok(db) => Arc::new(db),
        Err(e) => {
            eprintln!("数据库初始化失败: {}", e);
            std::process::exit(1);
        }
    };

    // 创建应用状态
    let app_state = app::AppState::new(db, config);

    // 提前提取窗口设置（解决借用问题）
    let window_title = app_state.config.ui.window_title.clone();
    let window_width = app_state.config.ui.window_width as f32;
    let window_height = app_state.config.ui.window_height as f32;

    // 创建窗口选项
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(window_width, window_height)),
        ..Default::default()
    };

    // 运行应用
    eframe::run_native(
        &window_title,
        options,
        Box::new(|cc| Box::new(app::SteelPipeApp::new(cc, app_state))),
    )
}
