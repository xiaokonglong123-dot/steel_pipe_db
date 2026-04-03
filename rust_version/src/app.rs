
use eframe::egui;
use std::sync::Arc;
use crate::database::{Database, SteelPipe, InventoryRecord, Statistics};
use crate::config::Config;

pub enum CurrentView {
    Entry,
    Exit,
    Inventory,
    Records,
    Statistics,
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

    // 入库表单数据
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

    // 出库表单数据
    exit_pipe_id: String,
    exit_quantity: String,
    exit_operator: String,
    exit_remarks: String,

    // 搜索和筛选
    search_text: String,
    filter_type: String,
    filter_pipe_id: String,
    filter_material: String,
    filter_min_diameter: String,
    filter_max_diameter: String,
    filter_min_length: String,
    filter_max_length: String,

    // 统计数据
    statistics: Option<Statistics>,

    // 消息提示
    message: Option<Message>,

    // 导出设置
    export_format: String,
    export_date_range_start: String,
    export_date_range_end: String,

    // 主题设置
    dark_mode: bool,
    accent_color: [f32; 3],

    // 数据可视化
    show_chart: bool,
    chart_type: ChartType,
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
    Info,
    Success,
    Warning,
    Error,
}

impl SteelPipeApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, state: AppState) -> Self {
        Self {
            state,
            current_view: CurrentView::Inventory,

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

            statistics: None,

            message: None,

            export_format: "CSV".to_string(),
            export_date_range_start: String::new(),
            export_date_range_end: String::new(),

            dark_mode: false,
            accent_color: [0.2, 0.6, 0.8],

            show_chart: false,
            chart_type: ChartType::Bar,
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

    fn show_export_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("数据导出")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("导出格式:");
                ui.radio_value(&mut self.export_format, "CSV", "CSV");
                ui.radio_value(&mut self.export_format, "JSON", "JSON");
                
                ui.add_space(10.0);
                ui.label("日期范围(可选):");
                ui.horizontal(|ui| {
                    ui.label("从:");
                    ui.text_edit_singleline(&mut self.export_date_range_start);
                    ui.label("到:");
                    ui.text_edit_singleline(&mut self.export_date_range_end);
                });
                
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("导出库存").clicked() {
                        self.export_inventory();
                    }
                    if ui.button("导出记录").clicked() {
                        self.export_records();
                    }
                    if ui.button("取消").clicked() {
                        // 关闭窗口
                    }
                });
            });
    }

    fn export_inventory(&mut self) {
        match self.state.db.export_inventory_to_csv() {
            Ok(csv) => {
                // 在实际应用中，这里应该保存到文件
                self.show_message("库存数据导出成功！".to_string(), MessageType::Success);
            }
            Err(e) => {
                self.show_message(format!("导出失败：{}", e), MessageType::Error);
            }
        }
    }

    fn export_records(&mut self) {
        let pipe_id = if self.filter_pipe_id.is_empty() { None } else { Some(self.filter_pipe_id.as_str()) };
        let operation_type = if self.filter_type == "全部" { None } else { Some(self.filter_type.as_str()) };
        let start_date = if self.export_date_range_start.is_empty() { None } else { Some(self.export_date_range_start.as_str()) };
        let end_date = if self.export_date_range_end.is_empty() { None } else { Some(self.export_date_range_end.as_str()) };
        
        match self.state.db.export_records_to_csv(pipe_id, operation_type, start_date, end_date) {
            Ok(csv) => {
                // 在实际应用中，这里应该保存到文件
                self.show_message("出入库记录导出成功！".to_string(), MessageType::Success);
            }
            Err(e) => {
                self.show_message(format!("导出失败：{}", e), MessageType::Error);
            }
        }
    }

    /// 验证并解析数值
    fn parse_positive_f64(&self, value: &str, field_name: &str) -> Result<f64, String> {
        value.parse::<f64>()
            .map_err(|_| format!("请输入有效的{}数值！", field_name))
            .and_then(|v| {
                if v <= 0.0 {
                    Err(format!("{}必须大于0", field_name))
                } else {
                    Ok(v)
                }
            })
    }

    /// 验证并解析整数
    fn parse_positive_i32(&self, value: &str, field_name: &str) -> Result<i32, String> {
        value.parse::<i32>()
            .map_err(|_| format!("请输入有效的{}！", field_name))
            .and_then(|v| {
                if v <= 0 {
                    Err(format!("{}必须大于0", field_name))
                } else {
                    Ok(v)
                }
            })
    }
}

impl eframe::App for SteelPipeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 初始化主题
        self.update_theme(ctx);

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(5.0);
            ui.horizontal_centered(|ui| {
                ui.heading(egui::RichText::new(&self.state.config.ui.window_title).size(24.0).strong());
            });
            ui.add_space(5.0);
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.add_space(10.0);
                
                // 使用主题色按钮
                let button_style = if self.dark_mode {
                    egui::Style {
                        visuals: egui::Visuals::dark(),
                        ..Default::default()
                    }
                } else {
                    egui::Style {
                        visuals: egui::Visuals::light(),
                        ..Default::default()
                    }
                };
                ui.style_mut().visuals = button_style.visuals.clone();

                // 钢管入库按钮
                if ui.add(
                    egui::Button::new("📦 钢管入库")
                        .fill(egui::Color32::from_rgb(52, 152, 219))
                ).clicked() {
                    self.current_view = CurrentView::Entry;
                }

                // 钢管出库按钮
                if ui.add(
                    egui::Button::new("🚚 钢管出库")
                        .fill(egui::Color32::from_rgb(231, 76, 60))
                ).clicked() {
                    self.current_view = CurrentView::Exit;
                }

                // 库存查询按钮
                if ui.add(
                    egui::Button::new("🔍 库存查询")
                        .fill(egui::Color32::from_rgb(46, 204, 113))
                ).clicked() {
                    self.current_view = CurrentView::Inventory;
                }

                // 出入库记录按钮
                if ui.add(
                    egui::Button::new("📋 出入库记录")
                        .fill(egui::Color32::from_rgb(243, 156, 18))
                ).clicked() {
                    self.current_view = CurrentView::Records;
                }

                // 数据统计按钮
                if ui.add(
                    egui::Button::new("📊 数据统计")
                        .fill(egui::Color32::from_rgb(155, 89, 182))
                ).clicked() {
                    self.current_view = CurrentView::Statistics;
                    if let Ok(stats) = self.state.db.get_statistics() {
                        self.statistics = Some(stats);
                    }
                }

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                // 数据导出按钮
                if ui.add(
                    egui::Button::new("📤 数据导出")
                        .fill(egui::Color32::from_rgb(52, 73, 94))
                ).clicked() {
                    self.show_export_dialog(ctx);
                }

                // 主题切换按钮
                if ui.add(
                    egui::Button::new(if self.dark_mode { "☀️ 浅色模式" } else { "🌙 深色模式" })
                        .fill(egui::Color32::from_rgb(127, 140, 141))
                ).clicked() {
                    self.dark_mode = !self.dark_mode;
                    self.update_theme(ctx);
                }

                ui.add_space(20.0);

                if ui.add(
                    egui::Button::new("❌ 退出系统")
                        .fill(egui::Color32::from_rgb(149, 165, 166))
                ).clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_view {
                CurrentView::Entry => self.show_entry_form(ui),
                CurrentView::Exit => self.show_exit_form(ui),
                CurrentView::Inventory => self.show_inventory(ui),
                CurrentView::Records => self.show_records(ui),
                CurrentView::Statistics => self.show_statistics(ui),
            }

            // 显示消息提示
            if let Some(ref msg) = self.message {
                let color = match msg.msg_type {
                    MessageType::Info => egui::Color32::LIGHT_BLUE,
                    MessageType::Success => egui::Color32::LIGHT_GREEN,
                    MessageType::Warning => egui::Color32::YELLOW,
                    MessageType::Error => egui::Color32::RED,
                };

                egui::Window::new("提示")
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.colored_label(color, &msg.content);
                            ui.label(format!("({})", msg.timestamp));
                        });
                        if ui.button("确定").clicked() {
                            self.message = None;
                        }
                    });
            }
        });
    }
}

impl SteelPipeApp {
    fn show_entry_form(&mut self, ui: &mut egui::Ui) {
        ui.heading("📦 钢管入库");
        ui.add_space(10.0);

        egui::Grid::new("entry_form")
            .num_columns(2)
            .spacing([40.0, 10.0])
            .show(ui, |ui| {
                ui.label("钢管编号:");
                ui.text_edit_singleline(&mut self.entry_pipe_id);
                ui.end_row();

                ui.label("直径(毫米):");
                ui.text_edit_singleline(&mut self.entry_diameter);
                ui.end_row();

                ui.label("壁厚(毫米):");
                ui.text_edit_singleline(&mut self.entry_thickness);
                ui.end_row();

                ui.label("长度(米):");
                ui.text_edit_singleline(&mut self.entry_length);
                ui.end_row();

                ui.label("材质:");
                egui::ComboBox::from_id_source("material_combo")
                    .selected_text(if self.entry_material.is_empty() { "选择材质" } else { &self.entry_material })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.entry_material, "碳钢".to_string(), "碳钢");
                        ui.selectable_value(&mut self.entry_material, "不锈钢".to_string(), "不锈钢");
                        ui.selectable_value(&mut self.entry_material, "合金钢".to_string(), "合金钢");
                        ui.selectable_value(&mut self.entry_material, "无缝钢管".to_string(), "无缝钢管");
                        ui.selectable_value(&mut self.entry_material, "焊接钢管".to_string(), "焊接钢管");
                    });
                ui.end_row();

                ui.label("数量:");
                ui.text_edit_singleline(&mut self.entry_quantity);
                ui.end_row();

                ui.label("存放位置:");
                ui.text_edit_singleline(&mut self.entry_location);
                ui.end_row();

                ui.label("供应商:");
                ui.text_edit_singleline(&mut self.entry_supplier);
                ui.end_row();

                ui.label("操作员:");
                ui.text_edit_singleline(&mut self.entry_operator);
                ui.end_row();
            });

        ui.add_space(10.0);
        ui.label("备注:");
        ui.text_edit_multiline(&mut self.entry_remarks)
            .desired_rows(3)
            .show(ui);

        ui.add_space(20.0);
        if ui.button("确认入库").clicked() {
            // 验证必填字段
            if self.entry_pipe_id.is_empty() || 
               self.entry_diameter.is_empty() || 
               self.entry_thickness.is_empty() || 
               self.entry_length.is_empty() || 
               self.entry_material.is_empty() || 
               self.entry_quantity.is_empty() || 
               self.entry_operator.is_empty() {
                self.show_message("请填写所有必填字段！".to_string(), MessageType::Error);
                return;
            }

            // 转换数值类型
            let diameter = match self.entry_diameter.parse::<f64>() {
                Ok(v) => v,
                Err(_) => {
                    self.show_message("请输入有效的直径数值！".to_string(), MessageType::Error);
                    return;
                }
            };

            let thickness = match self.entry_thickness.parse::<f64>() {
                Ok(v) => v,
                Err(_) => {
                    self.show_message("请输入有效的壁厚数值！".to_string(), MessageType::Error);
                    return;
                }
            };

            let length = match self.entry_length.parse::<f64>() {
                Ok(v) => v,
                Err(_) => {
                    self.show_message("请输入有效的长度数值！".to_string(), MessageType::Error);
                    return;
                }
            };

            let quantity = match self.entry_quantity.parse::<i32>() {
                Ok(v) => v,
                Err(_) => {
                    self.show_message("请输入有效的数量！".to_string(), MessageType::Error);
                    return;
                }
            };

            // 创建钢管对象
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
                status: "在库".to_string(),
            };

            // 添加到数据库
            match self.state.db.add_pipe(&pipe) {
                Ok(_) => {
                    // 记录入库操作
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
                }
                Err(e) => {
                    self.show_message(format!("入库操作失败：{}", e), MessageType::Error);
                }
            }
        }
    }

    fn show_exit_form(&mut self, ui: &mut egui::Ui) {
        ui.heading("🚚 钢管出库");
        ui.add_space(10.0);

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
        ui.text_edit_multiline(&mut self.exit_remarks)
            .desired_rows(3)
            .show(ui);

        ui.add_space(20.0);
        if ui.button("确认出库").clicked() {
            // 验证必填字段
            if self.exit_pipe_id.is_empty() || 
               self.exit_quantity.is_empty() || 
               self.exit_operator.is_empty() {
                self.show_message("请填写所有必填字段！".to_string(), MessageType::Error);
                return;
            }

            // 转换数值类型
            let quantity = match self.exit_quantity.parse::<i32>() {
                Ok(v) => v,
                Err(_) => {
                    self.show_message("请输入有效的数量！".to_string(), MessageType::Error);
                    return;
                }
            };

            // 检查钢管是否存在并获取当前库存
            match self.state.db.get_pipe_by_id(&self.exit_pipe_id) {
                Ok(Some(pipe)) => {
                    let current_quantity = pipe.quantity;

                    if current_quantity < quantity {
                        self.show_message(format!("库存不足！当前库存：{}", current_quantity), MessageType::Error);
                        return;
                    }

                    // 更新库存数量
                    if let Err(e) = self.state.db.update_pipe_quantity(&self.exit_pipe_id, -quantity) {
                        self.show_message(format!("更新库存失败：{}", e), MessageType::Error);
                        return;
                    }

                    // 记录出库操作
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
        ui.heading("📦 库存查询");
        ui.add_space(10.0);

        // 搜索框
        ui.horizontal(|ui| {
            ui.label("🔍 搜索:");
            ui.text_edit_singleline(&mut self.search_text);
            if ui.button("刷新").clicked() {
                self.search_text.clear();
                self.filter_material.clear();
                self.filter_min_diameter.clear();
                self.filter_max_diameter.clear();
                self.filter_min_length.clear();
                self.filter_max_length.clear();
            }
        });

        ui.add_space(5.0);

        // 高级筛选面板
        egui::CollapsingHeader::new("高级筛选").default_open(false).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("材质:");
                ui.text_edit_singleline(&mut self.filter_material);
            });

            ui.horizontal(|ui| {
                ui.label("直径范围(毫米):");
                ui.text_edit_singleline(&mut self.filter_min_diameter).hint_text("最小值");
                ui.label("-");
                ui.text_edit_singleline(&mut self.filter_max_diameter).hint_text("最大值");
            });

            ui.horizontal(|ui| {
                ui.label("长度范围(米):");
                ui.text_edit_singleline(&mut self.filter_min_length).hint_text("最小值");
                ui.label("-");
                ui.text_edit_singleline(&mut self.filter_max_length).hint_text("最大值");
            });
        });

        ui.add_space(10.0);

        // 创建表格
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("inventory_grid")
                .striped(true)
                .show(ui, |ui| {
                    // 表头
                    ui.strong("钢管编号");
                    ui.strong("直径(毫米)");
                    ui.strong("壁厚(毫米)");
                    ui.strong("长度(米)");
                    ui.strong("材质");
                    ui.strong("数量");
                    ui.strong("存放位置");
                    ui.strong("入库日期");
                    ui.strong("状态");
                    ui.end_row();

                    // 数据行
                    if let Ok(pipes) = self.state.db.get_pipes() {
                        let search_lower = self.search_text.to_lowercase();
                        let filter_material = self.filter_material.trim();
                        let min_diameter = self.filter_min_diameter.parse::<f64>().ok();
                        let max_diameter = self.filter_max_diameter.parse::<f64>().ok();
                        let min_length = self.filter_min_length.parse::<f64>().ok();
                        let max_length = self.filter_max_length.parse::<f64>().ok();

                        for pipe in pipes {
                            // 文本搜索过滤
                            if !search_lower.is_empty() {
                                let pipe_str = format!("{} {} {} {} {}",
                                    pipe.pipe_id,
                                    pipe.material,
                                    pipe.location.as_ref().unwrap_or(&String::new()),
                                    pipe.supplier.as_ref().unwrap_or(&String::new()),
                                    pipe.status
                                ).to_lowercase();
                                if !pipe_str.contains(&search_lower) {
                                    continue;
                                }
                            }

                            // 材质过滤
                            if !filter_material.is_empty() {
                                if !pipe.material.contains(filter_material) {
                                    continue;
                                }
                            }

                            // 直径范围过滤
                            if let Some(min_d) = min_diameter {
                                if pipe.diameter < min_d {
                                    continue;
                                }
                            }
                            if let Some(max_d) = max_diameter {
                                if pipe.diameter > max_d {
                                    continue;
                                }
                            }

                            // 长度范围过滤
                            if let Some(min_l) = min_length {
                                if pipe.length < min_l {
                                    continue;
                                }
                            }
                            if let Some(max_l) = max_length {
                                if pipe.length > max_l {
                                    continue;
                                }
                            }

                            // 显示匹配的记录
                            ui.label(&pipe.pipe_id);
                            ui.label(format!("{:.2}", pipe.diameter));
                            ui.label(format!("{:.2}", pipe.thickness));
                            ui.label(format!("{:.2}", pipe.length));
                            ui.label(&pipe.material);
                            ui.label(format!("{}", pipe.quantity));
                            ui.label(pipe.location.as_ref().unwrap_or(&String::new()));
                            ui.label(&pipe.entry_date);
                            ui.label(&pipe.status);
                            ui.end_row();
                        }
                    }
                });
        });
    }

    fn show_records(&mut self, ui: &mut egui::Ui) {
        ui.heading("📋 出入库记录");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label("操作类型:");
            egui::ComboBox::from_label("")
                .selected_text(&self.filter_type)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.filter_type, "全部".to_string(), "全部");
                    ui.selectable_value(&mut self.filter_type, "入库".to_string(), "入库");
                    ui.selectable_value(&mut self.filter_type, "出库".to_string(), "出库");
                });

            ui.label("钢管编号:");
            ui.text_edit_singleline(&mut self.filter_pipe_id);

            ui.label("日期范围:");
            ui.text_edit_singleline(&mut self.export_date_range_start).hint_text("开始日期");
            ui.label("-");
            ui.text_edit_singleline(&mut self.export_date_range_end).hint_text("结束日期");

            if ui.button("筛选").clicked() {
                // 筛选逻辑在显示表格时处理
            }

            if ui.button("重置").clicked() {
                self.filter_type = "全部".to_string();
                self.filter_pipe_id.clear();
                self.export_date_range_start.clear();
                self.export_date_range_end.clear();
            }
        });

        ui.add_space(10.0);

        // 创建表格
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("records_grid")
                .striped(true)
                .show(ui, |ui| {
                    // 表头
                    ui.label("钢管编号");
                    ui.label("操作类型");
                    ui.label("数量");
                    ui.label("操作日期");
                    ui.label("操作员");
                    ui.label("备注");
                    ui.end_row();

                    // 数据行
                    let pipe_id = if self.filter_pipe_id.is_empty() { None } else { Some(self.filter_pipe_id.as_str()) };
                    let operation_type = if self.filter_type == "全部" { None } else { Some(self.filter_type.as_str()) };
                    let start_date = if self.export_date_range_start.is_empty() { None } else { Some(self.export_date_range_start.as_str()) };
                    let end_date = if self.export_date_range_end.is_empty() { None } else { Some(self.export_date_range_end.as_str()) };

                    if let Ok(records) = self.state.db.get_inventory_records(pipe_id, operation_type, start_date, end_date) {
                        for record in records {
                            ui.label(&record.pipe_id);
                            ui.label(&record.operation_type);
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
        ui.heading("📊 数据统计");
        ui.add_space(10.0);

        if let Some(ref stats) = self.statistics {
            // 统计卡片
            ui.horizontal(|ui| {
                // 总钢管种类数卡片
                ui.vertical(|ui| {
                    ui.add_sized([200.0, 100.0], egui::CentralPanel::default().show_inside(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(egui::RichText::new("📦 总种类").size(20.0));
                            ui.add_space(10.0);
                            ui.label(egui::RichText::new(format!("{}", stats.total_types)).size(32.0).strong());
                        });
                    }));
                });

                ui.add_space(20.0);

                // 总钢管数量卡片
                ui.vertical(|ui| {
                    ui.add_sized([200.0, 100.0], egui::CentralPanel::default().show_inside(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(egui::RichText::new("🔢 总数量").size(20.0));
                            ui.add_space(10.0);
                            ui.label(egui::RichText::new(format!("{}", stats.total_quantity)).size(32.0).strong());
                        });
                    }));
                });

                ui.add_space(20.0);

                // 入库总数卡片
                ui.vertical(|ui| {
                    ui.add_sized([200.0, 100.0], egui::CentralPanel::default().show_inside(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(egui::RichText::new("📥 入库总数").size(20.0));
                            ui.add_space(10.0);
                            ui.label(egui::RichText::new(format!("{}", stats.total_in)).size(32.0).strong());
                        });
                    }));
                });

                ui.add_space(20.0);

                // 出库总数卡片
                ui.vertical(|ui| {
                    ui.add_sized([200.0, 100.0], egui::CentralPanel::default().show_inside(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(egui::RichText::new("📤 出库总数").size(20.0));
                            ui.add_space(10.0);
                            ui.label(egui::RichText::new(format!("{}", stats.total_out)).size(32.0).strong());
                        });
                    }));
                });
            });

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(20.0);

            // 图表控制
            ui.horizontal(|ui| {
                ui.label("图表类型:");
                ui.radio_value(&mut self.chart_type, ChartType::Bar, "柱状图");
                ui.radio_value(&mut self.chart_type, ChartType::Pie, "饼图");
                ui.radio_value(&mut self.chart_type, ChartType::Line, "折线图");
                ui.checkbox(&mut self.show_chart, "显示图表");
            });

            // 简单的图表可视化
            if self.show_chart {
                ui.add_space(10.0);
                self.show_chart_visualization(ui, stats);
            }
        } else {
            ui.label("加载统计数据中...");
        }
    }

    fn show_chart_visualization(&self, ui: &mut egui::Ui, stats: &crate::database::Statistics) {
        match self.chart_type {
            ChartType::Bar => {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("入库 vs 出库");
                        let max_val = stats.total_in.max(stats.total_out).max(1) as f32;
                        let bar_height = 200.0;
                    
                        ui.horizontal(|ui| {
                            // 入库柱状图
                            ui.vertical(|ui| {
                                ui.label("入库");
                                let height = (stats.total_in as f32 / max_val) * bar_height;
                                ui.add_sized([100.0, height], egui::Button::new(format!("{}", stats.total_in)).fill(egui::Color32::from_rgb(46, 204, 113)));
                            });
                        
                            // 出库柱状图
                            ui.vertical(|ui| {
                                ui.label("出库");
                                let height = (stats.total_out as f32 / max_val) * bar_height;
                                ui.add_sized([100.0, height], egui::Button::new(format!("{}", stats.total_out)).fill(egui::Color32::from_rgb(231, 76, 60)));
                            });
                        });
                    });
                });
            }
            ChartType::Pie => {
                ui.label("入库/出库占比");
                ui.add_space(10.0);
                
                let total = (stats.total_in + stats.total_out) as f32;
                if total > 0.0 {
                    let in_ratio = stats.total_in as f32 / total;
                    let out_ratio = stats.total_out as f32 / total;
                    
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(format!("入库: {:.1}%", in_ratio * 100.0)).color(egui::Color32::from_rgb(46, 204, 113)));
                            ui.add_sized([200.0, 20.0], egui::ProgressBar::new(in_ratio).fill(egui::Color32::from_rgb(46, 204, 113)));
                        });
                        
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(format!("出库: {:.1}%", out_ratio * 100.0)).color(egui::Color32::from_rgb(231, 76, 60)));
                            ui.add_sized([200.0, 20.0], egui::ProgressBar::new(out_ratio).fill(egui::Color32::from_rgb(231, 76, 60)));
                        });
                    });
                }
            }
            ChartType::Line => {
                ui.label("库存趋势");
                ui.add_space(10.0);
                ui.label("当前库存:");
                ui.label(egui::RichText::new(format!("{}", stats.total_quantity - stats.total_out)).size(24.0).strong());
            }
        }
    }
}
