
mod app;
mod database;
mod config;

use eframe::egui;
use std::sync::Arc;

fn main() -> Result<(), eframe::Error> {
    // 初始化日志
    env_logger::init();

    // 加载配置
    let config = config::Config::load("config.toml").unwrap_or_default();

    // 初始化数据库
    let db = Arc::new(database::Database::new(&config.database.path).unwrap());

    // 创建应用状态
    let app_state = app::AppState::new(db, config);

    // 创建窗口选项
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([app_state.config.ui.window_width as f32, app_state.config.ui.window_height as f32])
            .with_icon(
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon.png")[..])
                    .unwrap_or_default()
            ),
        ..Default::default()
    };

    // 运行应用
    eframe::run_native(
        &app_state.config.ui.window_title,
        options,
        Box::new(|cc| {
            // 设置字体
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert(
                "my_font".to_owned(),
                egui::FontData::from_static(include_bytes!("../assets/fonts/NotoSansSC-Regular.ttf")),
            );
            fonts.families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "my_font".to_owned());
            cc.egui_ctx.set_fonts(fonts);

            Box::new(app::SteelPipeApp::new(cc, app_state))
        }),
    )
}
