use crate::config::Config;
use crate::database::{Database, InventoryRecord, MaterialStats, OperationLog, Statistics, SteelPipe};
use eframe::egui;
use std::sync::Arc;

pub enum CurrentView {
    Dashboard,
    Entry,
    Exit,
    Inventory,
    Records,
    Statistics,
    LowStock,
}

pub struct AppState {
    pub db: Arc<Database>,
    pub config: Config,
}

impl AppState {
    pub fn new(db: Arc<Database>, config: Config) -> Self {
        Self { db, config }
    }
}

pub struct SteelPipeApp {
    state: AppState,
    current_view: CurrentView,

    entry_pipe_id: String,
    entry_diameter: String,
    entry_thickness: String,
    entry_length: String,
    entry_material: String,
    entry_quantity: String,
    entry_location: String,
    entry_supplier: String,
    entry_operator: String,
    entry_remarks: String,

    exit_pipe_id: String,
    exit_quantity: String,
    exit_operator: String,
    exit_remarks: String,

    search_text: String,
    filter_type: String,
    filter_pipe_id: String,
    filter_material: String,
    filter_min_diameter: String,
    filter_max_diameter: String,
    filter_min_length: String,
    filter_max_length: String,
    filter_status: String,

    statistics: Option<Statistics>,
    material_stats: Vec<MaterialStats>,
    low_stock_items: Vec<SteelPipe>,
    recent_records: Vec<InventoryRecord>,

    message: Option<Message>,
    confirm_dialog: Option<ConfirmDialog>,

    export_format: String,
    export_date_range_start: String,
    export_date_range_end: String,
    show_export_dialog: bool,

    dark_mode: bool,

    show_chart: bool,
    chart_type: ChartType,

    inventory_page: i64,
    inventory_page_size: i64,
    inventory_total: i64,

    editing_pipe: Option<SteelPipe>,
    show_edit_dialog: bool,

    low_stock_threshold: i32,

    import_csv_content: String,
    import_file_path: String,
    import_operator: String,
    import_format: String,
    show_import_dialog: bool,
    import_result: Option<String>,

    operation_logs: Vec<OperationLog>,
    show_undo_dialog: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ChartType {
    Bar,
    Pie,
    Line,
}

#[derive(Debug, Clone)]
struct Message {
    content: String,
    msg_type: MessageType,
    timestamp: String,
}

#[derive(Debug, Clone, Copy)]
enum MessageType {
    Success,
    Error,
}

#[derive(Debug, Clone)]
struct ConfirmDialog {
    title: String,
    content: String,
    action: ConfirmAction,
}

#[derive(Debug, Clone)]
enum ConfirmAction {
    DeletePipe(String),
}

const ITEMS_PER_PAGE: i64 = 20;
const LOW_STOCK_DEFAULT_THRESHOLD: i32 = 10;

impl SteelPipeApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, state: AppState) -> Self {
        let mut app = Self {
            state,
            current_view: CurrentView::Dashboard,

            entry_pipe_id: String::new(),
            entry_diameter: String::new(),
            entry_thickness: String::new(),
            entry_length: String::new(),
            entry_material: String::new(),
            entry_quantity: String::new(),
            entry_location: String::new(),
            entry_supplier: String::new(),
            entry_operator: String::new(),
            entry_remarks: String::new(),

            exit_pipe_id: String::new(),
            exit_quantity: String::new(),
            exit_operator: String::new(),
            exit_remarks: String::new(),

            search_text: String::new(),
            filter_type: "全部".to_string(),
            filter_pipe_id: String::new(),
            filter_material: String::new(),
            filter_min_diameter: String::new(),
            filter_max_diameter: String::new(),
            filter_min_length: String::new(),
            filter_max_length: String::new(),
            filter_status: "全部".to_string(),

            statistics: None,
            material_stats: Vec::new(),
            low_stock_items: Vec::new(),
            recent_records: Vec::new(),

            message: None,
            confirm_dialog: None,

            export_format: "CSV".to_string(),
            export_date_range_start: String::new(),
            export_date_range_end: String::new(),
            show_export_dialog: false,

            dark_mode: false,

            show_chart: true,
            chart_type: ChartType::Bar,

            inventory_page: 0,
            inventory_page_size: ITEMS_PER_PAGE,
            inventory_total: 0,

            editing_pipe: None,
            show_edit_dialog: false,

            low_stock_threshold: LOW_STOCK_DEFAULT_THRESHOLD,

            import_csv_content: String::new(),
            import_file_path: String::new(),
            import_operator: String::new(),
            import_format: "CSV".to_string(),
            show_import_dialog: false,
            import_result: None,

            operation_logs: Vec::new(),
            show_undo_dialog: false,
        };
        app.refresh_dashboard_data();
        app
    }

    fn refresh_dashboard_data(&mut self) {
        if let Ok(stats) = self.state.db.get_statistics() {
            self.statistics = Some(stats);
        }
        if let Ok(records) = self.state.db.get_recent_records(10) {
            self.recent_records = records;
        }
        if let Ok(low_stock) = self.state.db.get_low_stock_pipes(LOW_STOCK_DEFAULT_THRESHOLD) {
            self.low_stock_items = low_stock;
        }
        if let Ok(count) = self.state.db.get_pipes_count() {
            self.inventory_total = count;
        }
        if let Ok(mat_stats) = self.state.db.get_statistics_by_material() {
            self.material_stats = mat_stats;
        }
    }

    fn show_message(&mut self, msg: String, msg_type: MessageType) {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        self.message = Some(Message {
            content: msg,
            msg_type,
            timestamp,
        });
    }

    fn clear_entry_form(&mut self) {
        self.entry_pipe_id.clear();
        self.entry_diameter.clear();
        self.entry_thickness.clear();
        self.entry_length.clear();
        self.entry_material.clear();
        self.entry_quantity.clear();
        self.entry_location.clear();
        self.entry_supplier.clear();
        self.entry_operator.clear();
        self.entry_remarks.clear();
    }

    fn clear_exit_form(&mut self) {
        self.exit_pipe_id.clear();
        self.exit_quantity.clear();
        self.exit_operator.clear();
        self.exit_remarks.clear();
    }

    fn update_theme(&self, ctx: &egui::Context) {
        if self.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }
    }

    fn card_frame(&self, ui: &mut egui::Ui, title: &str, value: &str, color: egui::Color32) {
        egui::Frame::group(ui.style())
            .fill(color.linear_multiply(0.1))
            .stroke(egui::Stroke::new(1.0, color.linear_multiply(0.3)))
            .rounding(8.0)
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new(title).size(14.0).color(color));
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new(value).size(28.0).strong().color(color));
                });
            });
    }

    fn nav_button(ui: &mut egui::Ui, text: &str, color: egui::Color32, active: bool) -> bool {
        let fill = if active {
            egui::Color32::WHITE.linear_multiply(0.15)
        } else {
            color.linear_multiply(0.8)
        };
        let stroke = if active {
            egui::Stroke::new(2.0, egui::Color32::WHITE.linear_multiply(0.5))
        } else {
            egui::Stroke::NONE
        };
        ui.add_space(4.0);
        let btn = egui::Button::new(text)
            .fill(fill)
            .stroke(stroke)
            .rounding(6.0)
            .min_size(egui::vec2(ui.available_width(), 36.0));
        let response = ui.add(btn);
        ui.add_space(4.0);
        response.clicked()
    }

    fn show_export_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_export_dialog {
            return;
        }
        let mut should_close = false;
        egui::Window::new("数据导出")
            .collapsible(false)
            .resizable(true)
            .show(ctx, |ui| {
                ui.label("导出格式:");
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.export_format, "CSV".to_string(), "CSV");
                    ui.radio_value(&mut self.export_format, "Excel".to_string(), "Excel (.xlsx)");
                });

                ui.add_space(10.0);
                ui.label("日期范围(可选):");
                ui.horizontal(|ui| {
                    ui.label("从:");
                    ui.text_edit_singleline(&mut self.export_date_range_start);
                    ui.label("到:");
                    ui.text_edit_singleline(&mut self.export_date_range_end);
                });

                ui.add_space(10.0);
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("导出库存").clicked() {
                        self.export_inventory();
                        should_close = true;
                    }
                    if ui.button("导出记录").clicked() {
                        self.export_records();
                        should_close = true;
                    }
                    ui.separator();
                    if ui.button("关闭").clicked() {
                        should_close = true;
                    }
                });
            });
        if should_close {
            self.show_export_dialog = false;
        }
    }

    fn export_inventory(&mut self) {
        let ext = if self.export_format == "Excel" { "xlsx" } else { "csv" };
        let path = format!(
            "inventory_export_{}.{}",
            chrono::Local::now().format("%Y%m%d_%H%M%S"),
            ext
        );
        let res = if self.export_format == "Excel" {
            self.state.db.export_inventory_to_excel(&path)
        } else {
            self.state.db.export_inventory_to_file(&path)
        };
        match res {
            Ok(()) => {
                self.show_message(
                    format!("库存数据已导出到: {}", path),
                    MessageType::Success,
                );
            }
            Err(e) => {
                self.show_message(format!("导出失败：{}", e), MessageType::Error);
            }
        }
    }

    fn export_records(&mut self) {
        let ext = if self.export_format == "Excel" { "xlsx" } else { "csv" };
        let path = format!(
            "records_export_{}.{}",
            chrono::Local::now().format("%Y%m%d_%H%M%S"),
            ext
        );
        let pipe_id = if self.filter_pipe_id.is_empty() {
            None
        } else {
            Some(self.filter_pipe_id.as_str())
        };
        let operation_type = if self.filter_type == "全部" {
            None
        } else {
            Some(self.filter_type.as_str())
        };
        let start_date = if self.export_date_range_start.is_empty() {
            None
        } else {
            Some(self.export_date_range_start.as_str())
        };
        let end_date = if self.export_date_range_end.is_empty() {
            None
        } else {
            Some(self.export_date_range_end.as_str())
        };

        let res = if self.export_format == "Excel" {
            self.state.db.export_records_to_excel(&path, pipe_id, operation_type, start_date, end_date)
        } else {
            self.state.db.export_records_to_file(&path, pipe_id, operation_type, start_date, end_date)
        };
        match res {
            Ok(()) => {
                self.show_message(
                    format!("出入库记录已导出到: {}", path),
                    MessageType::Success,
                );
            }
            Err(e) => {
                self.show_message(format!("导出失败：{}", e), MessageType::Error);
            }
        }
    }

    fn open_confirm_delete(&mut self, pipe_id: String) {
        self.confirm_dialog = Some(ConfirmDialog {
            title: "确认删除".to_string(),
            content: format!("确定要删除钢管 '{}' 吗？此操作将同时删除相关的出入库记录，且不可撤销！", pipe_id),
            action: ConfirmAction::DeletePipe(pipe_id),
        });
    }

    fn open_edit_dialog(&mut self, pipe: SteelPipe) {
        self.editing_pipe = Some(pipe);
        self.show_edit_dialog = true;
    }

    fn save_edit(&mut self) {
        if let Some(ref pipe) = self.editing_pipe {
            match self.state.db.update_pipe(pipe) {
                Ok(()) => {
                    let before_json = serde_json::to_string(pipe).unwrap_or_default();
                    let _ = self.state.db.log_operation(
                        "update_pipe", "pipe", &pipe.pipe_id,
                        &before_json, "", "system", "编辑更新",
                    );
                    self.show_message("钢管信息更新成功！".to_string(), MessageType::Success);
                    self.show_edit_dialog = false;
                    self.editing_pipe = None;
                    self.refresh_dashboard_data();
                }
                Err(e) => {
                    self.show_message(format!("更新失败：{}", e), MessageType::Error);
                }
            }
        }
    }
}

impl eframe::App for SteelPipeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_theme(ctx);

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                ui.heading(
                    egui::RichText::new(&self.state.config.ui.window_title)
                        .size(22.0)
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                    ui.label(egui::RichText::new(&now).size(13.0).weak());
                });
            });
            ui.add_space(4.0);
            ui.separator();
        });

        egui::SidePanel::left("side_panel")
            .default_width(180.0)
            .show(ctx, |ui| {
                ui.add_space(12.0);
                ui.vertical_centered_justified(|ui| {
                    if Self::nav_button(ui, "📊 首页概览", egui::Color32::from_rgb(52, 152, 219), matches!(self.current_view, CurrentView::Dashboard)) {
                        self.current_view = CurrentView::Dashboard;
                        self.refresh_dashboard_data();
                    }
                    if Self::nav_button(ui, "📦 钢管入库", egui::Color32::from_rgb(46, 204, 113), matches!(self.current_view, CurrentView::Entry)) {
                        self.current_view = CurrentView::Entry;
                    }
                    if Self::nav_button(ui, "🚚 钢管出库", egui::Color32::from_rgb(231, 76, 60), matches!(self.current_view, CurrentView::Exit)) {
                        self.current_view = CurrentView::Exit;
                    }
                    if Self::nav_button(ui, "🔍 库存查询", egui::Color32::from_rgb(52, 73, 94), matches!(self.current_view, CurrentView::Inventory)) {
                        self.current_view = CurrentView::Inventory;
                        if let Ok(count) = self.state.db.get_pipes_count() {
                            self.inventory_total = count;
                        }
                    }
                    if Self::nav_button(ui, "📋 出入库记录", egui::Color32::from_rgb(243, 156, 18), matches!(self.current_view, CurrentView::Records)) {
                        self.current_view = CurrentView::Records;
                    }
                    if Self::nav_button(ui, "📈 数据统计", egui::Color32::from_rgb(155, 89, 182), matches!(self.current_view, CurrentView::Statistics)) {
                        self.current_view = CurrentView::Statistics;
                        self.refresh_dashboard_data();
                    }
                    if Self::nav_button(ui, "⚠️ 库存预警", egui::Color32::from_rgb(230, 126, 34), matches!(self.current_view, CurrentView::LowStock)) {
                        self.current_view = CurrentView::LowStock;
                        if let Ok(items) = self.state.db.get_low_stock_pipes(self.low_stock_threshold) {
                            self.low_stock_items = items;
                        }
                    }

                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(12.0);

                    if ui.button("📤 数据导出").clicked() {
                        self.show_export_dialog = true;
                    }
                    ui.add_space(6.0);
                    if ui.button("📥 数据导入").clicked() {
                        self.show_import_dialog = true;
                        self.import_csv_content.clear();
                        self.import_result = None;
                    }
                    ui.add_space(6.0);
                    if ui.button("↩ 撤回操作").clicked() {
                        self.show_undo_dialog = true;
                        if let Ok(logs) = self.state.db.get_operation_logs(50) {
                            self.operation_logs = logs;
                        }
                    }
                    ui.add_space(6.0);
                    if ui.button(if self.dark_mode { "☀ 浅色模式" } else { "🌙 深色模式" }).clicked() {
                        self.dark_mode = !self.dark_mode;
                        self.update_theme(ctx);
                    }

                    ui.add_space(20.0);
                    ui.separator();
                    ui.add_space(12.0);
                    if ui.button("❌ 退出系统").clicked() {
                        std::process::exit(0);
                    }
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_view {
                CurrentView::Dashboard => self.show_dashboard(ui),
                CurrentView::Entry => self.show_entry_form(ui),
                CurrentView::Exit => self.show_exit_form(ui),
                CurrentView::Inventory => self.show_inventory(ui),
                CurrentView::Records => self.show_records(ui),
                CurrentView::Statistics => self.show_statistics(ui),
                CurrentView::LowStock => self.show_low_stock(ui),
            }

            if let Some(ref msg) = self.message {
                let color = match msg.msg_type {
                    MessageType::Success => egui::Color32::LIGHT_GREEN,
                    MessageType::Error => egui::Color32::RED,
                };
                let content = msg.content.clone();
                let timestamp = msg.timestamp.clone();
                let mut should_close = false;

                egui::Window::new("提示")
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.colored_label(color, &content);
                            ui.label(format!("({})", timestamp));
                        });
                        ui.add_space(8.0);
                        if ui.button("确定").clicked() {
                            should_close = true;
                        }
                    });
                if should_close {
                    self.message = None;
                }
            }

            if let Some(ref confirm) = self.confirm_dialog {
                let mut confirmed = false;
                let mut cancelled = false;
                egui::Window::new(&confirm.title)
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.label(&confirm.content);
                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            if ui.button("确认").clicked() {
                                confirmed = true;
                            }
                            if ui.button("取消").clicked() {
                                cancelled = true;
                            }
                        });
                    });
                if confirmed {
                    let ConfirmAction::DeletePipe(ref pipe_id) = confirm.action;
                    if let Ok(Some(pipe)) = self.state.db.get_pipe_by_id(pipe_id) {
                        let snapshot = serde_json::to_string(&pipe).unwrap_or_default();
                        let _ = self.state.db.log_operation(
                            "delete_pipe", "pipe", pipe_id,
                            &snapshot, "", "system", "删除钢管",
                        );
                    }
                    match self.state.db.delete_pipe(pipe_id) {
                        Ok(()) => {
                            self.show_message(
                                format!("钢管 '{}' 已删除", pipe_id),
                                MessageType::Success,
                            );
                            self.refresh_dashboard_data();
                        }
                        Err(e) => {
                            self.show_message(format!("删除失败：{}", e), MessageType::Error);
                        }
                    }
                    self.confirm_dialog = None;
                }
                if cancelled {
                    self.confirm_dialog = None;
                }
            }

            if self.show_edit_dialog {
                self.show_edit_window(ctx);
            }

            self.show_export_dialog(ctx);
            self.show_import_dialog(ctx);
            self.show_undo_dialog(ctx);
        });
    }
}

impl SteelPipeApp {
    fn show_dashboard(&mut self, ui: &mut egui::Ui) {
        ui.heading("首页概览");
        ui.add_space(12.0);

        if let Some(ref stats) = self.statistics {
            ui.horizontal(|ui| {
                self.card_frame(
                    ui,
                    "钢管种类",
                    &stats.total_types.to_string(),
                    egui::Color32::from_rgb(52, 152, 219),
                );
                ui.add_space(12.0);
                self.card_frame(
                    ui,
                    "库存总量",
                    &stats.total_quantity.to_string(),
                    egui::Color32::from_rgb(46, 204, 113),
                );
                ui.add_space(12.0);
                self.card_frame(
                    ui,
                    "入库总数",
                    &stats.total_in.to_string(),
                    egui::Color32::from_rgb(52, 73, 94),
                );
                ui.add_space(12.0);
                self.card_frame(
                    ui,
                    "出库总数",
                    &stats.total_out.to_string(),
                    egui::Color32::from_rgb(231, 76, 60),
                );
            });

            ui.add_space(16.0);

            if !self.low_stock_items.is_empty() {
                egui::Frame::group(ui.style())
                    .fill(egui::Color32::from_rgb(230, 126, 34).linear_multiply(0.08))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(230, 126, 34).linear_multiply(0.3)))
                    .rounding(8.0)
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("⚠ 库存预警").color(egui::Color32::from_rgb(230, 126, 34)).strong());
                            ui.label(format!("({} 项低于阈值)", self.low_stock_items.len()));
                            if ui.button("查看详情").clicked() {
                                self.current_view = CurrentView::LowStock;
                            }
                        });
                    });
                ui.add_space(12.0);
            }

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("最近操作记录").size(16.0).strong());
                    ui.add_space(8.0);
                    egui::Frame::group(ui.style())
                        .rounding(8.0)
                        .inner_margin(12.0)
                        .show(ui, |ui| {
                            for record in &self.recent_records {
                                let op_color = if record.operation_type == "入库" {
                                    egui::Color32::from_rgb(46, 204, 113)
                                } else {
                                    egui::Color32::from_rgb(231, 76, 60)
                                };
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(&record.operation_type).color(op_color).strong());
                                    ui.label(format!("| {} | 数量: {}", record.pipe_id, record.quantity));
                                    ui.label(egui::RichText::new(&record.operation_date).weak());
                                });
                            }
                            if self.recent_records.is_empty() {
                                ui.label(egui::RichText::new("暂无操作记录").weak());
                            }
                        });
                });

                ui.add_space(20.0);

                if !self.material_stats.is_empty() {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("按材质统计").size(16.0).strong());
                        ui.add_space(8.0);
                        egui::Frame::group(ui.style())
                            .rounding(8.0)
                            .inner_margin(12.0)
                            .show(ui, |ui| {
                                for ms in &self.material_stats {
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new(&ms.material).strong());
                                        ui.label(format!("| 种类: {} | 数量: {}", ms.type_count, ms.total_quantity));
                                    });
                                }
                            });
                    });
                }
            });
        } else {
            ui.label("加载中...");
        }
    }

    fn show_entry_form(&mut self, ui: &mut egui::Ui) {
        ui.heading("钢管入库");
        ui.add_space(12.0);

        egui::Frame::group(ui.style())
            .rounding(8.0)
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.label(egui::RichText::new("基本信息").strong());
                ui.add_space(8.0);
                egui::Grid::new("entry_form")
                    .num_columns(4)
                    .spacing([20.0, 10.0])
                    .show(ui, |ui| {
                        ui.label("钢管编号:");
                        ui.text_edit_singleline(&mut self.entry_pipe_id);
                        ui.label("材质:");
                        egui::ComboBox::from_id_source("material_combo")
                            .selected_text(if self.entry_material.is_empty() {
                                "选择材质"
                            } else {
                                &self.entry_material
                            })
                            .show_ui(ui, |ui| {
                                for mat in &["碳钢", "不锈钢", "合金钢", "无缝钢管", "焊接钢管"] {
                                    ui.selectable_value(&mut self.entry_material, mat.to_string(), *mat);
                                }
                            });
                        ui.end_row();

                        ui.label("直径(mm):");
                        ui.text_edit_singleline(&mut self.entry_diameter);
                        ui.label("壁厚(mm):");
                        ui.text_edit_singleline(&mut self.entry_thickness);
                        ui.end_row();

                        ui.label("长度(m):");
                        ui.text_edit_singleline(&mut self.entry_length);
                        ui.label("数量:");
                        ui.text_edit_singleline(&mut self.entry_quantity);
                        ui.end_row();
                    });

                ui.add_space(8.0);
                ui.label(egui::RichText::new("附加信息").strong());
                ui.add_space(8.0);
                egui::Grid::new("entry_form_extra")
                    .num_columns(4)
                    .spacing([20.0, 10.0])
                    .show(ui, |ui| {
                        ui.label("存放位置:");
                        ui.text_edit_singleline(&mut self.entry_location);
                        ui.label("供应商:");
                        ui.text_edit_singleline(&mut self.entry_supplier);
                        ui.end_row();

                        ui.label("操作员:");
                        ui.text_edit_singleline(&mut self.entry_operator);
                        ui.end_row();
                    });

                ui.add_space(8.0);
                ui.label("备注:");
                ui.add(egui::TextEdit::multiline(&mut self.entry_remarks).min_size(egui::vec2(ui.available_width(), 50.0)));
            });

        ui.add_space(16.0);
        if ui
            .add_sized(
                [120.0, 36.0],
                egui::Button::new("确认入库")
                    .fill(egui::Color32::from_rgb(46, 204, 113))
                    .rounding(6.0),
            )
            .clicked()
        {
            if self.entry_pipe_id.is_empty()
                || self.entry_diameter.is_empty()
                || self.entry_thickness.is_empty()
                || self.entry_length.is_empty()
                || self.entry_material.is_empty()
                || self.entry_quantity.is_empty()
                || self.entry_operator.is_empty()
            {
                self.show_message("请填写所有必填字段（钢管编号、直径、壁厚、长度、材质、数量、操作员）！".to_string(), MessageType::Error);
                return;
            }

            let diameter: f64 = match self.entry_diameter.parse() {
                Ok(v) if v > 0.0 => v,
                _ => {
                    self.show_message("请输入有效的直径数值！".to_string(), MessageType::Error);
                    return;
                }
            };
            let thickness: f64 = match self.entry_thickness.parse() {
                Ok(v) if v > 0.0 => v,
                _ => {
                    self.show_message("请输入有效的壁厚数值！".to_string(), MessageType::Error);
                    return;
                }
            };
            let length: f64 = match self.entry_length.parse() {
                Ok(v) if v > 0.0 => v,
                _ => {
                    self.show_message("请输入有效的长度数值！".to_string(), MessageType::Error);
                    return;
                }
            };
            let quantity: i32 = match self.entry_quantity.parse() {
                Ok(v) if v > 0 => v,
                _ => {
                    self.show_message("请输入有效的数量！".to_string(), MessageType::Error);
                    return;
                }
            };

            let pipe = SteelPipe {
                id: None,
                pipe_id: self.entry_pipe_id.clone(),
                diameter,
                thickness,
                length,
                material: self.entry_material.clone(),
                quantity,
                location: if self.entry_location.is_empty() { None } else { Some(self.entry_location.clone()) },
                supplier: if self.entry_supplier.is_empty() { None } else { Some(self.entry_supplier.clone()) },
                entry_date: String::new(),
                last_update: None,
                status: "在库".to_string(),
            };

            match self.state.db.add_pipe(&pipe) {
                Ok(()) => {
                    let _ = self.state.db.log_operation(
                        "add_pipe", "pipe", &self.entry_pipe_id,
                        "", &format!("{{\"qty\": {}}}", quantity),
                        &self.entry_operator, &self.entry_remarks,
                    );
                    let record = InventoryRecord {
                        id: None,
                        pipe_id: self.entry_pipe_id.clone(),
                        operation_type: "入库".to_string(),
                        quantity,
                        operation_date: String::new(),
                        operator: self.entry_operator.clone(),
                        remarks: if self.entry_remarks.is_empty() { None } else { Some(self.entry_remarks.clone()) },
                    };
                    if let Err(e) = self.state.db.add_inventory_record(&record) {
                        self.show_message(format!("记录入库操作失败：{}", e), MessageType::Error);
                        return;
                    }
                    self.clear_entry_form();
                    self.show_message("钢管入库操作成功！".to_string(), MessageType::Success);
                    self.refresh_dashboard_data();
                }
                Err(e) => {
                    self.show_message(format!("入库操作失败：{}", e), MessageType::Error);
                }
            }
        }
    }

    fn show_exit_form(&mut self, ui: &mut egui::Ui) {
        ui.heading("钢管出库");
        ui.add_space(12.0);

        egui::Frame::group(ui.style())
            .rounding(8.0)
            .inner_margin(16.0)
            .show(ui, |ui| {
                egui::Grid::new("exit_form")
                    .num_columns(2)
                    .spacing([40.0, 10.0])
                    .show(ui, |ui| {
                        ui.label("钢管编号:");
                        ui.text_edit_singleline(&mut self.exit_pipe_id);
                        ui.end_row();

                        ui.label("出库数量:");
                        ui.text_edit_singleline(&mut self.exit_quantity);
                        ui.end_row();

                        ui.label("操作员:");
                        ui.text_edit_singleline(&mut self.exit_operator);
                        ui.end_row();
                    });

                ui.add_space(10.0);
                ui.label("备注:");
                ui.add(egui::TextEdit::multiline(&mut self.exit_remarks).min_size(egui::vec2(ui.available_width(), 50.0)));
            });

        ui.add_space(16.0);
        if ui
            .add_sized(
                [120.0, 36.0],
                egui::Button::new("确认出库")
                    .fill(egui::Color32::from_rgb(231, 76, 60))
                    .rounding(6.0),
            )
            .clicked()
        {
            if self.exit_pipe_id.is_empty() || self.exit_quantity.is_empty() || self.exit_operator.is_empty() {
                self.show_message("请填写所有必填字段！".to_string(), MessageType::Error);
                return;
            }

            let quantity: i32 = match self.exit_quantity.parse() {
                Ok(v) if v > 0 => v,
                _ => {
                    self.show_message("请输入有效的数量！".to_string(), MessageType::Error);
                    return;
                }
            };

            match self.state.db.get_pipe_by_id(&self.exit_pipe_id) {
                Ok(Some(pipe)) => {
                    if pipe.quantity < quantity {
                        self.show_message(
                            format!("库存不足！当前库存：{}", pipe.quantity),
                            MessageType::Error,
                        );
                        return;
                    }

                    let before_qty = pipe.quantity;
                    if let Err(e) = self.state.db.update_pipe_quantity(&self.exit_pipe_id, -quantity) {
                        self.show_message(format!("更新库存失败：{}", e), MessageType::Error);
                        return;
                    }

                    let _ = self.state.db.log_operation(
                        "exit_pipe", "pipe", &self.exit_pipe_id,
                        &format!("{{\"qty\": {}}}", before_qty),
                        &format!("{{\"qty\": {}}}", before_qty - quantity),
                        &self.exit_operator, &self.exit_remarks,
                    );

                    let record = InventoryRecord {
                        id: None,
                        pipe_id: self.exit_pipe_id.clone(),
                        operation_type: "出库".to_string(),
                        quantity,
                        operation_date: String::new(),
                        operator: self.exit_operator.clone(),
                        remarks: if self.exit_remarks.is_empty() { None } else { Some(self.exit_remarks.clone()) },
                    };

                    if let Err(e) = self.state.db.add_inventory_record(&record) {
                        self.show_message(format!("记录出库操作失败：{}", e), MessageType::Error);
                        return;
                    }

                    self.clear_exit_form();
                    self.show_message("钢管出库操作成功！".to_string(), MessageType::Success);
                    self.refresh_dashboard_data();
                }
                Ok(None) => {
                    self.show_message("未找到该钢管编号！".to_string(), MessageType::Error);
                }
                Err(e) => {
                    self.show_message(format!("查询钢管失败：{}", e), MessageType::Error);
                }
            }
        }
    }

    fn show_inventory(&mut self, ui: &mut egui::Ui) {
        ui.heading("库存查询");
        ui.add_space(12.0);

        ui.horizontal(|ui| {
            ui.label("搜索:");
            ui.add(egui::TextEdit::singleline(&mut self.search_text).hint_text("输入关键词搜索..."));
            if ui.button("刷新").clicked() {
                self.search_text.clear();
                self.filter_material.clear();
                self.filter_min_diameter.clear();
                self.filter_max_diameter.clear();
                self.filter_min_length.clear();
                self.filter_max_length.clear();
                self.filter_status = "全部".to_string();
            }
        });

        ui.add_space(8.0);

        egui::CollapsingHeader::new("高级筛选")
            .default_open(false)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("材质:");
                    ui.text_edit_singleline(&mut self.filter_material);
                    ui.label("状态:");
                    egui::ComboBox::from_id_source("status_combo")
                        .selected_text(&self.filter_status)
                        .show_ui(ui, |ui| {
                            for s in &["全部", "在库", "已出库"] {
                                ui.selectable_value(&mut self.filter_status, s.to_string(), *s);
                            }
                        });
                });
                ui.horizontal(|ui| {
                    ui.label("直径范围(mm):");
                    ui.add(egui::TextEdit::singleline(&mut self.filter_min_diameter).hint_text("最小"));
                    ui.label("-");
                    ui.add(egui::TextEdit::singleline(&mut self.filter_max_diameter).hint_text("最大"));
                    ui.label("长度范围(m):");
                    ui.add(egui::TextEdit::singleline(&mut self.filter_min_length).hint_text("最小"));
                    ui.label("-");
                    ui.add(egui::TextEdit::singleline(&mut self.filter_max_length).hint_text("最大"));
                });
            });

        ui.add_space(10.0);

        let offset = self.inventory_page * self.inventory_page_size;
        if let Ok(pipes) = self.state.db.get_pipes_paginated(offset, self.inventory_page_size) {
            let search_lower = self.search_text.to_lowercase();
            let filter_material = self.filter_material.trim();
            let min_diameter = self.filter_min_diameter.parse::<f64>().ok();
            let max_diameter = self.filter_max_diameter.parse::<f64>().ok();
            let min_length = self.filter_min_length.parse::<f64>().ok();
            let max_length = self.filter_max_length.parse::<f64>().ok();

            let filtered: Vec<SteelPipe> = pipes
                .into_iter()
                .filter(|pipe| {
                    if !search_lower.is_empty() {
                        let pipe_str = format!(
                            "{} {} {} {} {}",
                            pipe.pipe_id,
                            pipe.material,
                            pipe.location.as_ref().unwrap_or(&String::new()),
                            pipe.supplier.as_ref().unwrap_or(&String::new()),
                            pipe.status
                        )
                        .to_lowercase();
                        if !pipe_str.contains(&search_lower) {
                            return false;
                        }
                    }
                    if !filter_material.is_empty() && !pipe.material.contains(filter_material) {
                        return false;
                    }
                    if let Some(min_d) = min_diameter {
                        if pipe.diameter < min_d { return false; }
                    }
                    if let Some(max_d) = max_diameter {
                        if pipe.diameter > max_d { return false; }
                    }
                    if let Some(min_l) = min_length {
                        if pipe.length < min_l { return false; }
                    }
                    if let Some(max_l) = max_length {
                        if pipe.length > max_l { return false; }
                    }
                    if self.filter_status != "全部" && pipe.status != self.filter_status {
                        return false;
                    }
                    true
                })
                .collect();

            egui::ScrollArea::horizontal().show(ui, |ui| {
                egui::Grid::new("inventory_grid")
                    .striped(true)
                    .show(ui, |ui| {
                        ui.strong("操作");
                        ui.strong("钢管编号");
                        ui.strong("直径(mm)");
                        ui.strong("壁厚(mm)");
                        ui.strong("长度(m)");
                        ui.strong("材质");
                        ui.strong("数量");
                        ui.strong("存放位置");
                        ui.strong("供应商");
                        ui.strong("入库日期");
                        ui.strong("状态");
                        ui.end_row();

                        for pipe in &filtered {
                            let status_color = if pipe.status == "在库" {
                                egui::Color32::from_rgb(46, 204, 113)
                            } else {
                                egui::Color32::from_rgb(231, 76, 60)
                            };
                            let qty_color = if pipe.quantity <= LOW_STOCK_DEFAULT_THRESHOLD {
                                egui::Color32::from_rgb(230, 126, 34)
                            } else {
                                ui.style().visuals.text_color()
                            };

                            ui.horizontal(|ui| {
                                if ui.small_button("编辑").clicked() {
                                    self.open_edit_dialog(pipe.clone());
                                }
                                if ui.small_button("删除").clicked() {
                                    self.open_confirm_delete(pipe.pipe_id.clone());
                                }
                            });
                            ui.label(&pipe.pipe_id);
                            ui.label(format!("{:.2}", pipe.diameter));
                            ui.label(format!("{:.2}", pipe.thickness));
                            ui.label(format!("{:.2}", pipe.length));
                            ui.label(&pipe.material);
                            ui.label(egui::RichText::new(format!("{}", pipe.quantity)).color(qty_color));
                            ui.label(pipe.location.as_ref().unwrap_or(&String::new()));
                            ui.label(pipe.supplier.as_ref().unwrap_or(&String::new()));
                            ui.label(&pipe.entry_date);
                            ui.label(egui::RichText::new(&pipe.status).color(status_color));
                            ui.end_row();
                        }
                    });
            });

            ui.add_space(12.0);
            ui.horizontal(|ui| {
                ui.label(format!("共 {} 条记录", self.inventory_total));
                ui.add_space(20.0);
                let total_pages = (self.inventory_total as f64 / self.inventory_page_size as f64).ceil() as i64;
                if total_pages > 0 {
                    if self.inventory_page > 0 && ui.button("◀ 上一页").clicked() {
                        self.inventory_page -= 1;
                    }
                    ui.label(format!("第 {} / {} 页", self.inventory_page + 1, total_pages));
                    if self.inventory_page < total_pages - 1 && ui.button("下一页 ▶").clicked() {
                        self.inventory_page += 1;
                    }
                }
            });
        }
    }

    fn show_records(&mut self, ui: &mut egui::Ui) {
        ui.heading("出入库记录");
        ui.add_space(12.0);

        ui.horizontal(|ui| {
            ui.label("操作类型:");
            egui::ComboBox::from_id_source("record_type_combo")
                .selected_text(&self.filter_type)
                .show_ui(ui, |ui| {
                    for t in &["全部", "入库", "出库"] {
                        ui.selectable_value(&mut self.filter_type, t.to_string(), *t);
                    }
                });

            ui.label("钢管编号:");
            ui.text_edit_singleline(&mut self.filter_pipe_id);

            ui.label("日期范围:");
            ui.add(egui::TextEdit::singleline(&mut self.export_date_range_start).hint_text("开始日期"));
            ui.label("-");
            ui.add(egui::TextEdit::singleline(&mut self.export_date_range_end).hint_text("结束日期"));

            let _ = ui.button("筛选");
            if ui.button("重置").clicked() {
                self.filter_type = "全部".to_string();
                self.filter_pipe_id.clear();
                self.export_date_range_start.clear();
                self.export_date_range_end.clear();
            }
        });

        ui.add_space(10.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("records_grid")
                .striped(true)
                .show(ui, |ui| {
                    ui.strong("钢管编号");
                    ui.strong("操作类型");
                    ui.strong("数量");
                    ui.strong("操作日期");
                    ui.strong("操作员");
                    ui.strong("备注");
                    ui.end_row();

                    let pipe_id = if self.filter_pipe_id.is_empty() { None } else { Some(self.filter_pipe_id.as_str()) };
                    let operation_type = if self.filter_type == "全部" { None } else { Some(self.filter_type.as_str()) };
                    let start_date = if self.export_date_range_start.is_empty() { None } else { Some(self.export_date_range_start.as_str()) };
                    let end_date = if self.export_date_range_end.is_empty() { None } else { Some(self.export_date_range_end.as_str()) };

                    if let Ok(records) = self.state.db.get_inventory_records(pipe_id, operation_type, start_date, end_date) {
                        for record in &records {
                            let op_color = if record.operation_type == "入库" {
                                egui::Color32::from_rgb(46, 204, 113)
                            } else {
                                egui::Color32::from_rgb(231, 76, 60)
                            };
                            ui.label(&record.pipe_id);
                            ui.label(egui::RichText::new(&record.operation_type).color(op_color));
                            ui.label(format!("{}", record.quantity));
                            ui.label(&record.operation_date);
                            ui.label(&record.operator);
                            ui.label(record.remarks.as_ref().unwrap_or(&String::new()));
                            ui.end_row();
                        }
                    }
                });
        });
    }

    fn show_statistics(&mut self, ui: &mut egui::Ui) {
        ui.heading("数据统计");
        ui.add_space(12.0);

        if let Some(ref stats) = self.statistics {
            ui.horizontal(|ui| {
                self.card_frame(ui, "总种类", &stats.total_types.to_string(), egui::Color32::from_rgb(52, 152, 219));
                ui.add_space(12.0);
                self.card_frame(ui, "总数量", &stats.total_quantity.to_string(), egui::Color32::from_rgb(46, 204, 113));
                ui.add_space(12.0);
                self.card_frame(ui, "入库总数", &stats.total_in.to_string(), egui::Color32::from_rgb(52, 73, 94));
                ui.add_space(12.0);
                self.card_frame(ui, "出库总数", &stats.total_out.to_string(), egui::Color32::from_rgb(231, 76, 60));
            });

            ui.add_space(16.0);

            if !self.material_stats.is_empty() {
                ui.label(egui::RichText::new("按材质分类统计").size(16.0).strong());
                ui.add_space(8.0);
                egui::Frame::group(ui.style())
                    .rounding(8.0)
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        egui::Grid::new("material_stats_grid")
                            .striped(true)
                            .show(ui, |ui| {
                                ui.strong("材质");
                                ui.strong("种类数");
                                ui.strong("总数量");
                                ui.strong("占比");
                                ui.end_row();
                                let total_qty = stats.total_quantity.max(1) as f32;
                                for ms in &self.material_stats {
                                    ui.label(&ms.material);
                                    ui.label(format!("{}", ms.type_count));
                                    ui.label(format!("{}", ms.total_quantity));
                                    let pct = ms.total_quantity as f32 / total_qty * 100.0;
                                    ui.add(egui::ProgressBar::new(pct / 100.0).text(format!("{:.1}%", pct)));
                                    ui.end_row();
                                }
                            });
                    });
                ui.add_space(12.0);
            }

            ui.horizontal(|ui| {
                ui.label("图表类型:");
                ui.radio_value(&mut self.chart_type, ChartType::Bar, "柱状图");
                ui.radio_value(&mut self.chart_type, ChartType::Pie, "饼图");
                ui.radio_value(&mut self.chart_type, ChartType::Line, "折线图");
                ui.checkbox(&mut self.show_chart, "显示图表");
            });

            if self.show_chart {
                ui.add_space(10.0);
                self.show_chart_visualization(ui, stats);
            }
        } else {
            ui.label("加载统计数据中...");
        }
    }

    fn show_low_stock(&mut self, ui: &mut egui::Ui) {
        ui.heading("库存预警");
        ui.add_space(12.0);

        ui.horizontal(|ui| {
            ui.label("预警阈值:");
            ui.add(egui::DragValue::new(&mut self.low_stock_threshold).clamp_range(1..=1000));
            if ui.button("查询").clicked() {
                if let Ok(items) = self.state.db.get_low_stock_pipes(self.low_stock_threshold) {
                    self.low_stock_items = items;
                }
            }
        });

        ui.add_space(10.0);

        if self.low_stock_items.is_empty() {
            ui.label("所有库存充足，暂无预警项目。");
        } else {
            egui::Frame::group(ui.style())
                .fill(egui::Color32::from_rgb(230, 126, 34).linear_multiply(0.08))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(230, 126, 34).linear_multiply(0.3)))
                .rounding(8.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new(format!("共 {} 项低于预警阈值", self.low_stock_items.len())).strong());
                    ui.add_space(8.0);
                    egui::Grid::new("low_stock_grid")
                        .striped(true)
                        .show(ui, |ui| {
                            ui.strong("钢管编号");
                            ui.strong("材质");
                            ui.strong("当前库存");
                            ui.strong("存放位置");
                            ui.strong("状态");
                            ui.end_row();
                            for pipe in &self.low_stock_items {
                                ui.label(&pipe.pipe_id);
                                ui.label(&pipe.material);
                                ui.label(egui::RichText::new(format!("{}", pipe.quantity)).color(egui::Color32::RED).strong());
                                ui.label(pipe.location.as_ref().unwrap_or(&String::new()));
                                ui.label(&pipe.status);
                                ui.end_row();
                            }
                        });
                });
        }
    }

    fn show_chart_visualization(&self, ui: &mut egui::Ui, stats: &Statistics) {
        match self.chart_type {
            ChartType::Bar => {
                ui.label("入库 vs 出库");
                let max_val = stats.total_in.max(stats.total_out).max(1) as f32;
                let bar_height = 150.0;
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("入库");
                        let height = (stats.total_in as f32 / max_val) * bar_height;
                        ui.add_sized(
                            [80.0, height.max(20.0)],
                            egui::Button::new(format!("{}", stats.total_in))
                                .fill(egui::Color32::from_rgb(46, 204, 113)),
                        );
                    });
                    ui.add_space(20.0);
                    ui.vertical(|ui| {
                        ui.label("出库");
                        let height = (stats.total_out as f32 / max_val) * bar_height;
                        ui.add_sized(
                            [80.0, height.max(20.0)],
                            egui::Button::new(format!("{}", stats.total_out))
                                .fill(egui::Color32::from_rgb(231, 76, 60)),
                        );
                    });
                });
            }
            ChartType::Pie => {
                let total = (stats.total_in + stats.total_out) as f32;
                if total > 0.0 {
                    let in_ratio = stats.total_in as f32 / total;
                    let out_ratio = stats.total_out as f32 / total;
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(format!("入库: {:.1}%", in_ratio * 100.0))
                                .color(egui::Color32::from_rgb(46, 204, 113)));
                            ui.add_sized(
                                [200.0, 20.0],
                                egui::ProgressBar::new(in_ratio)
                                    .fill(egui::Color32::from_rgb(46, 204, 113)),
                            );
                        });
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(format!("出库: {:.1}%", out_ratio * 100.0))
                                .color(egui::Color32::from_rgb(231, 76, 60)));
                            ui.add_sized(
                                [200.0, 20.0],
                                egui::ProgressBar::new(out_ratio)
                                    .fill(egui::Color32::from_rgb(231, 76, 60)),
                            );
                        });
                    });
                }
            }
            ChartType::Line => {
                ui.label("库存趋势");
                ui.add_space(10.0);
                ui.label("当前库存:");
                ui.label(
                    egui::RichText::new(format!("{}", stats.total_quantity))
                        .size(24.0)
                        .strong(),
                );
            }
        }
    }

    fn show_edit_window(&mut self, ctx: &egui::Context) {
        let mut should_close = false;
        let mut should_save = false;

        egui::Window::new("编辑钢管信息")
            .collapsible(false)
            .resizable(true)
            .show(ctx, |ui| {
                if let Some(ref mut pipe) = self.editing_pipe {
                    egui::Grid::new("edit_form")
                        .num_columns(2)
                        .spacing([20.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("钢管编号:");
                            ui.label(&pipe.pipe_id);
                            ui.end_row();

                            ui.label("直径(mm):");
                            ui.add(egui::DragValue::new(&mut pipe.diameter).speed(0.1));
                            ui.end_row();

                            ui.label("壁厚(mm):");
                            ui.add(egui::DragValue::new(&mut pipe.thickness).speed(0.1));
                            ui.end_row();

                            ui.label("长度(m):");
                            ui.add(egui::DragValue::new(&mut pipe.length).speed(0.1));
                            ui.end_row();

                            ui.label("材质:");
                            egui::ComboBox::from_id_source("edit_material_combo")
                                .selected_text(&pipe.material)
                                .show_ui(ui, |ui| {
                                    for mat in &["碳钢", "不锈钢", "合金钢", "无缝钢管", "焊接钢管"] {
                                        ui.selectable_value(&mut pipe.material, mat.to_string(), *mat);
                                    }
                                });
                            ui.end_row();

                            ui.label("数量:");
                            ui.add(egui::DragValue::new(&mut pipe.quantity).speed(1.0).clamp_range(0..=100000));
                            ui.end_row();

                            ui.label("存放位置:");
                            ui.text_edit_singleline(pipe.location.get_or_insert_with(String::new));
                            ui.end_row();

                            ui.label("供应商:");
                            ui.text_edit_singleline(pipe.supplier.get_or_insert_with(String::new));
                            ui.end_row();

                            ui.label("状态:");
                            egui::ComboBox::from_id_source("edit_status_combo")
                                .selected_text(&pipe.status)
                                .show_ui(ui, |ui| {
                                    for s in &["在库", "已出库"] {
                                        ui.selectable_value(&mut pipe.status, s.to_string(), *s);
                                    }
                                });
                            ui.end_row();
                        });

                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if ui.button("保存").clicked() {
                            should_save = true;
                        }
                        if ui.button("取消").clicked() {
                            should_close = true;
                        }
                    });
                }
            });

        if should_save {
            self.save_edit();
        }
        if should_close {
            self.show_edit_dialog = false;
            self.editing_pipe = None;
        }
    }

    fn show_import_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_import_dialog {
            return;
        }
        let mut should_close = false;
        egui::Window::new("数据导入")
            .collapsible(false)
            .resizable(true)
            .default_width(600.0)
            .default_height(500.0)
            .show(ctx, |ui| {
                ui.label("导入方式:");
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.import_format, "CSV".to_string(), "CSV粘贴导入");
                    ui.radio_value(&mut self.import_format, "Excel".to_string(), "Excel文件导入");
                });
                ui.add_space(8.0);

                match self.import_format.as_str() {
                    "CSV" => {
                        ui.label("CSV格式: 钢管编号,直径(mm),壁厚(mm),长度(m),材质,数量,存放位置(可选),供应商(可选)");
                        ui.add_space(8.0);
                        ui.label("操作员:");
                        ui.text_edit_singleline(&mut self.import_operator);
                        ui.add_space(8.0);
                        ui.label("粘贴CSV内容:");
                        ui.add(egui::TextEdit::multiline(&mut self.import_csv_content)
                            .min_size(egui::vec2(ui.available_width(), 250.0)));
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            if ui.button("开始导入").clicked() {
                                self.do_csv_import();
                            }
                            if ui.button("清空").clicked() {
                                self.import_csv_content.clear();
                                self.import_result = None;
                            }
                        });
                    }
                    "Excel" => {
                        ui.label("Excel格式: 第一行为表头，列顺序同CSV格式");
                        ui.label("支持 .xlsx 和 .xls 文件");
                        ui.add_space(8.0);
                        ui.label("操作员:");
                        ui.text_edit_singleline(&mut self.import_operator);
                        ui.add_space(8.0);
                        ui.label("Excel文件路径:");
                        ui.text_edit_singleline(&mut self.import_file_path);
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            if ui.button("开始导入").clicked() {
                                self.do_excel_import();
                            }
                            ui.separator();
                        });
                    }
                    _ => {}
                }

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("关闭").clicked() {
                        should_close = true;
                    }
                });

                if let Some(ref result) = self.import_result {
                    ui.add_space(8.0);
                    egui::Frame::group(ui.style())
                        .fill(egui::Color32::LIGHT_BLUE.linear_multiply(0.1))
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            ui.label(result);
                        });
                }
            });
        if should_close {
            self.show_import_dialog = false;
        }
    }

    fn do_csv_import(&mut self) {
        if self.import_operator.is_empty() {
            self.show_message("请填写操作员！".to_string(), MessageType::Error);
            return;
        }
        if self.import_csv_content.is_empty() {
            self.show_message("请粘贴CSV内容！".to_string(), MessageType::Error);
            return;
        }
        match self.state.db.import_pipes_from_csv(&self.import_csv_content, &self.import_operator) {
            Ok((success, fail)) => {
                self.import_result = Some(format!("导入完成！成功: {} 条，失败: {} 条", success, fail));
                self.show_message(self.import_result.clone().unwrap(), MessageType::Success);
                self.refresh_dashboard_data();
            }
            Err(e) => {
                self.import_result = Some(format!("{}", e));
                self.show_message(format!("{}", e), MessageType::Error);
            }
        }
    }

    fn do_excel_import(&mut self) {
        if self.import_operator.is_empty() {
            self.show_message("请填写操作员！".to_string(), MessageType::Error);
            return;
        }
        if self.import_file_path.is_empty() {
            self.show_message("请填写Excel文件路径！".to_string(), MessageType::Error);
            return;
        }
        match self.state.db.import_pipes_from_excel(&self.import_file_path, &self.import_operator) {
            Ok((success, fail)) => {
                self.import_result = Some(format!("导入完成！成功: {} 条，失败: {} 条", success, fail));
                self.show_message(self.import_result.clone().unwrap(), MessageType::Success);
                self.refresh_dashboard_data();
            }
            Err(e) => {
                self.import_result = Some(format!("{}", e));
                self.show_message(format!("{}", e), MessageType::Error);
            }
        }
    }

    fn show_undo_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_undo_dialog {
            return;
        }
        let mut should_close = false;
        egui::Window::new("撤回操作")
            .collapsible(false)
            .resizable(true)
            .default_width(700.0)
            .default_height(400.0)
            .show(ctx, |ui| {
                ui.label("最近操作记录（点击撤回可撤销对应操作）:");
                ui.add_space(8.0);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("undo_grid")
                        .striped(true)
                        .show(ui, |ui| {
                            ui.strong("操作");
                            ui.strong("目标类型");
                            ui.strong("目标ID");
                            ui.strong("操作员");
                            ui.strong("时间");
                            ui.strong("备注");
                            ui.strong("操作");
                            ui.end_row();

                            let logs_copy: Vec<_> = self.operation_logs.iter().map(|l| (l.id, l.operation_type.clone(), l.target_type.clone(), l.target_id.clone(), l.operator.clone(), l.timestamp.clone(), l.remarks.clone())).collect();
                            for (id, op_type, target_type, target_id, operator, timestamp, remarks) in &logs_copy {
                                let op_color = match op_type.as_str() {
                                    "add_pipe" | "入库" => egui::Color32::from_rgb(46, 204, 113),
                                    "delete_pipe" | "出库" => egui::Color32::from_rgb(231, 76, 60),
                                    "update_pipe" => egui::Color32::from_rgb(52, 152, 219),
                                    _ => ui.style().visuals.text_color(),
                                };
                                ui.label(egui::RichText::new(op_type).color(op_color));
                                ui.label(target_type);
                                ui.label(target_id);
                                ui.label(operator);
                                ui.label(timestamp);
                                ui.label(remarks);
                                if ui.small_button("撤回").clicked() {
                                    match self.state.db.undo_operation(*id) {
                                        Ok(msg) => {
                                            self.show_message(msg, MessageType::Success);
                                            if let Ok(logs) = self.state.db.get_operation_logs(50) {
                                                self.operation_logs = logs;
                                            }
                                            self.refresh_dashboard_data();
                                        }
                                        Err(e) => {
                                            self.show_message(format!("撤回失败：{}", e), MessageType::Error);
                                        }
                                    }
                                }
                                ui.end_row();
                            }

                            if self.operation_logs.is_empty() {
                                ui.label(egui::RichText::new("暂无操作记录").weak());
                                ui.end_row();
                            }
                        });
                });
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("撤回最近一次操作").clicked() {
                        match self.state.db.undo_last_operation() {
                            Ok(msg) => {
                                self.show_message(msg, MessageType::Success);
                                if let Ok(logs) = self.state.db.get_operation_logs(50) {
                                    self.operation_logs = logs;
                                }
                                self.refresh_dashboard_data();
                            }
                            Err(e) => {
                                self.show_message(format!("撤回失败：{}", e), MessageType::Error);
                            }
                        }
                    }
                    if ui.button("刷新").clicked() {
                        if let Ok(logs) = self.state.db.get_operation_logs(50) {
                            self.operation_logs = logs;
                        }
                    }
                    ui.separator();
                    if ui.button("关闭").clicked() {
                        should_close = true;
                    }
                });
            });
        if should_close {
            self.show_undo_dialog = false;
        }
    }
}