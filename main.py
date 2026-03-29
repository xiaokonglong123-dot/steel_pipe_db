import sqlite3
import tkinter as tk
from tkinter import ttk, messagebox
from datetime import datetime

class SteelPipeDB:
    def __init__(self, db_path="pipes.db"):
        self.conn = sqlite3.connect(db_path)
        self.cursor = self.conn.cursor()
        self.create_tables()

    def create_tables(self):
        # 创建钢管表
        self.cursor.execute('''
            CREATE TABLE IF NOT EXISTS pipes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                pipe_id TEXT UNIQUE NOT NULL,
                diameter REAL,
                thickness REAL,
                length REAL,
                material TEXT,
                quantity INTEGER,
                location TEXT,
                supplier TEXT,
                entry_date TEXT,
                status TEXT DEFAULT "在库"
            )
        ''')
        
        # 创建出入库记录表
        self.cursor.execute('''
            CREATE TABLE IF NOT EXISTS inventory_records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                pipe_id TEXT NOT NULL,
                operation_type TEXT NOT NULL,
                quantity INTEGER NOT NULL,
                operation_date TEXT NOT NULL,
                operator TEXT NOT NULL,
                remarks TEXT,
                FOREIGN KEY (pipe_id) REFERENCES pipes(pipe_id)
            )
        ''')
        self.conn.commit()

    def add_pipe(self, pipe_id, diameter, thickness, length, material, quantity, location, supplier):
        # 检查钢管ID是否已存在
        self.cursor.execute("SELECT id FROM pipes WHERE pipe_id = ?", (pipe_id,))
        if self.cursor.fetchone():
            # 如果存在，更新数量
            self.cursor.execute('''
                UPDATE pipes SET quantity = quantity + ?, location = ?, supplier = ?, last_update = ?
                WHERE pipe_id = ?
            ''', (quantity, location, supplier, datetime.now().strftime("%Y-%m-%d %H:%M:%S"), pipe_id))
        else:
            # 如果不存在，插入新记录
            self.cursor.execute('''
                INSERT INTO pipes (pipe_id, diameter, thickness, length, material, quantity, location, supplier, entry_date, status)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ''', (pipe_id, diameter, thickness, length, material, quantity, location, supplier, 
                   datetime.now().strftime("%Y-%m-%d %H:%M:%S"), "在库"))
        self.conn.commit()

    def get_pipes(self):
        self.cursor.execute("SELECT * FROM pipes")
        return self.cursor.fetchall()
    
    def get_pipe_by_id(self, pipe_id):
        self.cursor.execute("SELECT * FROM pipes WHERE pipe_id = ?", (pipe_id,))
        return self.cursor.fetchone()

    def delete_pipe(self, pipe_id):
        self.cursor.execute("DELETE FROM pipes WHERE id = ?", (pipe_id,))
        self.conn.commit()

    def update_pipe(self, pipe_id, diameter, thickness, length, material, quantity, location, supplier):
        self.cursor.execute('''
            UPDATE pipes SET diameter=?, thickness=?, length=?, material=?, quantity=?, location=?, supplier=?, last_update=?
            WHERE id=?
        ''', (diameter, thickness, length, material, quantity, location, supplier, 
               datetime.now().strftime("%Y-%m-%d %H:%M:%S"), pipe_id))
        self.conn.commit()
    
    def update_pipe_quantity(self, pipe_id, quantity_change):
        self.cursor.execute('''
            UPDATE pipes SET quantity = quantity + ?, last_update = ?
            WHERE pipe_id = ?
        ''', (quantity_change, datetime.now().strftime("%Y-%m-%d %H:%M:%S"), pipe_id))
        self.conn.commit()
    
    def add_inventory_record(self, pipe_id, operation_type, quantity, operator, remarks):
        self.cursor.execute('''
            INSERT INTO inventory_records (pipe_id, operation_type, quantity, operation_date, operator, remarks)
            VALUES (?, ?, ?, ?, ?, ?)
        ''', (pipe_id, operation_type, quantity, datetime.now().strftime("%Y-%m-%d %H:%M:%S"), operator, remarks))
        self.conn.commit()
    
    def get_inventory_records(self, pipe_id=None, operation_type=None, date_range=None):
        query = "SELECT * FROM inventory_records WHERE 1=1"
        params = []
        
        if pipe_id:
            query += " AND pipe_id = ?"
            params.append(pipe_id)
        
        if operation_type and operation_type != "全部":
            query += " AND operation_type = ?"
            params.append(operation_type)
        
        self.cursor.execute(query, params)
        return self.cursor.fetchall()
    
    def get_statistics(self):
        stats = {}
        
        # 总钢管种类数
        self.cursor.execute("SELECT COUNT(*) FROM pipes")
        stats["total_types"] = self.cursor.fetchone()[0]
        
        # 总钢管数量
        self.cursor.execute("SELECT SUM(quantity) FROM pipes")
        result = self.cursor.fetchone()[0]
        stats["total_quantity"] = result if result else 0
        
        # 入库总数
        self.cursor.execute("SELECT SUM(quantity) FROM inventory_records WHERE operation_type = '入库'")
        result = self.cursor.fetchone()[0]
        stats["total_in"] = result if result else 0
        
        # 出库总数
        self.cursor.execute("SELECT SUM(quantity) FROM inventory_records WHERE operation_type = '出库'")
        result = self.cursor.fetchone()[0]
        stats["total_out"] = result if result else 0
        
        return stats

    def close(self):
        self.conn.close()

class PipeApp:
    def __init__(self, root):
        self.root = root
        self.root.title("钢管原料进出入库管理系统")
        self.root.geometry("1200x800")
        self.db = SteelPipeDB()
        
        # 设置窗口图标（如果有的话）
        try:
            self.root.iconbitmap("steel_pipe.ico")
        except:
            pass

        self.create_widgets()
        self.load_data()

    def create_widgets(self):
        # 创建顶部导航栏
        nav_frame = tk.Frame(self.root, bg="#2c3e50", height=60)
        nav_frame.pack(fill=tk.X)
        
        # 添加标题
        title_label = tk.Label(nav_frame, text="钢管原料进出入库管理系统", 
                              font=("微软雅黑", 18, "bold"), bg="#2c3e50", fg="white")
        title_label.pack(pady=10)
        
        # 创建主内容区域
        content_frame = tk.Frame(self.root, bg="#ecf0f1")
        content_frame.pack(fill=tk.BOTH, expand=True, padx=20, pady=20)
        
        # 创建左侧功能按钮区
        button_frame = tk.Frame(content_frame, bg="#34495e", width=250)
        button_frame.pack(side=tk.LEFT, fill=tk.Y, padx=(0, 10))
        button_frame.pack_propagate(False)
        
        # 添加功能按钮
        buttons = [
            ("钢管入库", "#3498db", self.show_entry_form),
            ("钢管出库", "#e74c3c", self.show_exit_form),
            ("库存查询", "#2ecc71", self.show_inventory),
            ("出入库记录", "#f39c12", self.show_records),
            ("数据统计", "#9b59b6", self.show_statistics),
            ("退出系统", "#95a5a6", self.on_close)
        ]
        
        for text, color, command in buttons:
            btn = tk.Button(button_frame, text=text, bg=color, fg="white",
                          font=("微软雅黑", 12), relief=tk.FLAT,
                          activebackground=color, activeforeground="white",
                          command=command)
            btn.pack(fill=tk.X, padx=10, pady=10)
        
        # 创建右侧内容显示区
        self.content_area = tk.Frame(content_frame, bg="white", relief=tk.RAISED, bd=2)
        self.content_area.pack(side=tk.RIGHT, fill=tk.BOTH, expand=True)
        
        # 默认显示库存查询
        self.show_inventory()
    
    def clear_content_area(self):
        # 清空内容区域
        for widget in self.content_area.winfo_children():
            widget.destroy()
    
    def show_entry_form(self):
        self.clear_content_area()
        
        # 创建入库表单
        form_frame = tk.Frame(self.content_area, bg="white")
        form_frame.pack(fill=tk.BOTH, expand=True, padx=20, pady=20)
        
        # 标题
        title_label = tk.Label(form_frame, text="钢管入库", 
                              font=("微软雅黑", 16, "bold"), bg="white")
        title_label.pack(pady=(0, 20))
        
        # 创建表单字段
        fields = [
            ("钢管编号", "pipe_id"),
            ("规格型号", "specification"),
            ("材质", "material"),
            ("长度(米)", "length"),
            ("直径(毫米)", "diameter"),
            ("壁厚(毫米)", "thickness"),
            ("数量", "quantity"),
            ("存放位置", "location"),
            ("供应商", "supplier"),
            ("操作员", "operator")
        ]
        
        self.entry_vars = {}
        
        for i, (label_text, var_name) in enumerate(fields):
            frame = tk.Frame(form_frame, bg="white")
            frame.pack(fill=tk.X, pady=5)
            
            label = tk.Label(frame, text=label_text, font=("微软雅黑", 10),
                            bg="white", width=15, anchor="e")
            label.pack(side=tk.LEFT, padx=(0, 10))
            
            entry = tk.Entry(frame, font=("微软雅黑", 10))
            entry.pack(side=tk.LEFT, fill=tk.X, expand=True)
            self.entry_vars[var_name] = entry
        
        # 添加备注字段
        remarks_frame = tk.Frame(form_frame, bg="white")
        remarks_frame.pack(fill=tk.X, pady=5)
        
        remarks_label = tk.Label(remarks_frame, text="备注", font=("微软雅黑", 10),
                               bg="white", width=15, anchor="e")
        remarks_label.pack(side=tk.LEFT, padx=(0, 10))
        
        self.remarks_entry = tk.Text(remarks_frame, font=("微软雅黑", 10), height=3)
        self.remarks_entry.pack(side=tk.LEFT, fill=tk.X, expand=True)
        
        # 添加提交按钮
        submit_btn = tk.Button(form_frame, text="确认入库", bg="#3498db", fg="white",
                              font=("微软雅黑", 12), relief=tk.FLAT,
                              activebackground="#2980b9", activeforeground="white",
                              command=self.submit_entry)
        submit_btn.pack(pady=20)
    
    def submit_entry(self):
        # 获取表单数据
        pipe_id = self.entry_vars["pipe_id"].get()
        diameter = self.entry_vars["diameter"].get()
        thickness = self.entry_vars["thickness"].get()
        length = self.entry_vars["length"].get()
        material = self.entry_vars["material"].get()
        quantity = self.entry_vars["quantity"].get()
        location = self.entry_vars["location"].get()
        supplier = self.entry_vars["supplier"].get()
        operator = self.entry_vars["operator"].get()
        remarks = self.remarks_entry.get("1.0", tk.END).strip()
        
        # 验证必填字段
        if not all([pipe_id, diameter, thickness, length, material, quantity, operator]):
            messagebox.showerror("错误", "请填写所有必填字段！")
            return
        
        try:
            # 转换数值类型
            diameter = float(diameter)
            thickness = float(thickness)
            length = float(length)
            quantity = int(quantity)
            
            # 添加钢管到数据库
            self.db.add_pipe(pipe_id, diameter, thickness, length, material, quantity, location, supplier)
            
            # 记录入库操作
            self.db.add_inventory_record(pipe_id, "入库", quantity, operator, remarks)
            
            # 清空表单
            for var in self.entry_vars.values():
                var.delete(0, tk.END)
            self.remarks_entry.delete("1.0", tk.END)
            
            # 显示成功消息
            messagebox.showinfo("成功", "钢管入库操作成功！")
            
        except ValueError:
            messagebox.showerror("错误", "请输入有效的数值！")
        except Exception as e:
            messagebox.showerror("错误", f"入库操作失败：{str(e)}")
    
    def show_exit_form(self):
        self.clear_content_area()
        
        # 创建出库表单
        form_frame = tk.Frame(self.content_area, bg="white")
        form_frame.pack(fill=tk.BOTH, expand=True, padx=20, pady=20)
        
        # 标题
        title_label = tk.Label(form_frame, text="钢管出库", 
                              font=("微软雅黑", 16, "bold"), bg="white")
        title_label.pack(pady=(0, 20))
        
        # 创建表单字段
        fields = [
            ("钢管编号", "pipe_id"),
            ("出库数量", "quantity"),
            ("操作员", "operator")
        ]
        
        self.exit_vars = {}
        
        for i, (label_text, var_name) in enumerate(fields):
            frame = tk.Frame(form_frame, bg="white")
            frame.pack(fill=tk.X, pady=5)
            
            label = tk.Label(frame, text=label_text, font=("微软雅黑", 10),
                            bg="white", width=15, anchor="e")
            label.pack(side=tk.LEFT, padx=(0, 10))
            
            entry = tk.Entry(frame, font=("微软雅黑", 10))
            entry.pack(side=tk.LEFT, fill=tk.X, expand=True)
            self.exit_vars[var_name] = entry
        
        # 添加备注字段
        remarks_frame = tk.Frame(form_frame, bg="white")
        remarks_frame.pack(fill=tk.X, pady=5)
        
        remarks_label = tk.Label(remarks_frame, text="备注", font=("微软雅黑", 10),
                               bg="white", width=15, anchor="e")
        remarks_label.pack(side=tk.LEFT, padx=(0, 10))
        
        self.exit_remarks_entry = tk.Text(remarks_frame, font=("微软雅黑", 10), height=3)
        self.exit_remarks_entry.pack(side=tk.LEFT, fill=tk.X, expand=True)
        
        # 添加提交按钮
        submit_btn = tk.Button(form_frame, text="确认出库", bg="#e74c3c", fg="white",
                              font=("微软雅黑", 12), relief=tk.FLAT,
                              activebackground="#c0392b", activeforeground="white",
                              command=self.submit_exit)
        submit_btn.pack(pady=20)
    
    def submit_exit(self):
        # 获取表单数据
        pipe_id = self.exit_vars["pipe_id"].get()
        quantity = self.exit_vars["quantity"].get()
        operator = self.exit_vars["operator"].get()
        remarks = self.exit_remarks_entry.get("1.0", tk.END).strip()
        
        # 验证必填字段
        if not all([pipe_id, quantity, operator]):
            messagebox.showerror("错误", "请填写所有必填字段！")
            return
        
        try:
            # 转换数值类型
            quantity = int(quantity)
            
            # 检查钢管是否存在并获取当前库存
            pipe_data = self.db.get_pipe_by_id(pipe_id)
            
            if not pipe_data:
                messagebox.showerror("错误", "未找到该钢管编号！")
                return
            
            current_quantity = pipe_data[6]  # quantity字段在表中的位置
            
            if current_quantity < quantity:
                messagebox.showerror("错误", f"库存不足！当前库存：{current_quantity}")
                return
            
            # 更新库存数量
            self.db.update_pipe_quantity(pipe_id, -quantity)
            
            # 记录出库操作
            self.db.add_inventory_record(pipe_id, "出库", quantity, operator, remarks)
            
            # 清空表单
            for var in self.exit_vars.values():
                var.delete(0, tk.END)
            self.exit_remarks_entry.delete("1.0", tk.END)
            
            # 显示成功消息
            messagebox.showinfo("成功", "钢管出库操作成功！")
            
        except ValueError:
            messagebox.showerror("错误", "请输入有效的数值！")
        except Exception as e:
            messagebox.showerror("错误", f"出库操作失败：{str(e)}")
    
    def show_inventory(self):
        self.clear_content_area()
        
        # 创建库存查询界面
        inventory_frame = tk.Frame(self.content_area, bg="white")
        inventory_frame.pack(fill=tk.BOTH, expand=True, padx=20, pady=20)
        
        # 标题
        title_label = tk.Label(inventory_frame, text="库存查询", 
                              font=("微软雅黑", 16, "bold"), bg="white")
        title_label.pack(pady=(0, 20))
        
        # 创建搜索框
        search_frame = tk.Frame(inventory_frame, bg="white")
        search_frame.pack(fill=tk.X, pady=(0, 20))
        
        search_label = tk.Label(search_frame, text="搜索:", font=("微软雅黑", 10),
                               bg="white")
        search_label.pack(side=tk.LEFT, padx=(0, 10))
        
        self.search_var = tk.StringVar()
        search_entry = tk.Entry(search_frame, textvariable=self.search_var, 
                               font=("微软雅黑", 10))
        search_entry.pack(side=tk.LEFT, fill=tk.X, expand=True, padx=(0, 10))
        
        search_btn = tk.Button(search_frame, text="搜索", bg="#3498db", fg="white",
                              font=("微软雅黑", 10), relief=tk.FLAT,
                              activebackground="#2980b9", activeforeground="white",
                              command=self.search_inventory)
        search_btn.pack(side=tk.LEFT)
        
        refresh_btn = tk.Button(search_frame, text="刷新", bg="#2ecc71", fg="white",
                               font=("微软雅黑", 10), relief=tk.FLAT,
                               activebackground="#27ae60", activeforeground="white",
                               command=self.load_data)
        refresh_btn.pack(side=tk.LEFT, padx=(10, 0))
        
        # 创建表格
        table_frame = tk.Frame(inventory_frame, bg="white")
        table_frame.pack(fill=tk.BOTH, expand=True)
        
        # 创建Treeview表格
        columns = ("pipe_id", "diameter", "thickness", "length", "material", 
                  "quantity", "location", "entry_date", "status")
        
        self.inventory_tree = ttk.Treeview(table_frame, columns=columns, show="headings")
        
        # 设置列标题
        self.inventory_tree.heading("pipe_id", text="钢管编号")
        self.inventory_tree.heading("diameter", text="直径(毫米)")
        self.inventory_tree.heading("thickness", text="壁厚(毫米)")
        self.inventory_tree.heading("length", text="长度(米)")
        self.inventory_tree.heading("material", text="材质")
        self.inventory_tree.heading("quantity", text="数量")
        self.inventory_tree.heading("location", text="存放位置")
        self.inventory_tree.heading("entry_date", text="入库日期")
        self.inventory_tree.heading("status", text="状态")
        
        # 设置列宽
        for col in columns:
            self.inventory_tree.column(col, width=100, anchor=tk.CENTER)
        
        # 添加滚动条
        scrollbar = ttk.Scrollbar(table_frame, orient=tk.VERTICAL, 
                                  command=self.inventory_tree.yview)
        self.inventory_tree.configure(yscrollcommand=scrollbar.set)
        
        # 布局表格和滚动条
        self.inventory_tree.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        scrollbar.pack(side=tk.RIGHT, fill=tk.Y)
        
        # 加载数据
        self.load_data()
    
    def show_records(self):
        self.clear_content_area()
        
        # 创建出入库记录界面
        records_frame = tk.Frame(self.content_area, bg="white")
        records_frame.pack(fill=tk.BOTH, expand=True, padx=20, pady=20)
        
        # 标题
        title_label = tk.Label(records_frame, text="出入库记录", 
                              font=("微软雅黑", 16, "bold"), bg="white")
        title_label.pack(pady=(0, 20))
        
        # 创建筛选选项
        filter_frame = tk.Frame(records_frame, bg="white")
        filter_frame.pack(fill=tk.X, pady=(0, 20))
        
        tk.Label(filter_frame, text="操作类型:", font=("微软雅黑", 10),
                bg="white").pack(side=tk.LEFT, padx=(0, 10))
        
        self.filter_type = tk.StringVar(value="全部")
        type_combobox = ttk.Combobox(filter_frame, textvariable=self.filter_type,
                                    values=["全部", "入库", "出库"], state="readonly")
        type_combobox.pack(side=tk.LEFT, padx=(0, 20))
        
        tk.Label(filter_frame, text="钢管编号:", font=("微软雅黑", 10),
                bg="white").pack(side=tk.LEFT, padx=(0, 10))
        
        self.filter_pipe_id = tk.StringVar()
        pipe_id_entry = tk.Entry(filter_frame, textvariable=self.filter_pipe_id, 
                                 font=("微软雅黑", 10))
        pipe_id_entry.pack(side=tk.LEFT, padx=(0, 20))
        
        filter_btn = tk.Button(filter_frame, text="筛选", bg="#3498db", fg="white",
                             font=("微软雅黑", 10), relief=tk.FLAT,
                             activebackground="#2980b9", activeforeground="white",
                             command=self.filter_records)
        filter_btn.pack(side=tk.LEFT)
        
        # 创建表格
        table_frame = tk.Frame(records_frame, bg="white")
        table_frame.pack(fill=tk.BOTH, expand=True)
        
        # 创建Treeview表格
        columns = ("pipe_id", "operation_type", "quantity", "operation_date", "operator", "remarks")
        
        self.records_tree = ttk.Treeview(table_frame, columns=columns, show="headings")
        
        # 设置列标题
        self.records_tree.heading("pipe_id", text="钢管编号")
        self.records_tree.heading("operation_type", text="操作类型")
        self.records_tree.heading("quantity", text="数量")
        self.records_tree.heading("operation_date", text="操作日期")
        self.records_tree.heading("operator", text="操作员")
        self.records_tree.heading("remarks", text="备注")
        
        # 设置列宽
        for col in columns:
            self.records_tree.column(col, width=100, anchor=tk.CENTER)
        
        # 添加滚动条
        scrollbar = ttk.Scrollbar(table_frame, orient=tk.VERTICAL, 
                                command=self.records_tree.yview)
        self.records_tree.configure(yscrollcommand=scrollbar.set)
        
        # 布局表格和滚动条
        self.records_tree.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        scrollbar.pack(side=tk.RIGHT, fill=tk.Y)
        
        # 加载数据
        self.load_records()
    
    def show_statistics(self):
        self.clear_content_area()
        
        # 创建数据统计界面
        stats_frame = tk.Frame(self.content_area, bg="white")
        stats_frame.pack(fill=tk.BOTH, expand=True, padx=20, pady=20)
        
        # 标题
        title_label = tk.Label(stats_frame, text="数据统计", 
                             font=("微软雅黑", 16, "bold"), bg="white")
        title_label.pack(pady=(0, 20))
        
        # 获取统计数据
        stats = self.db.get_statistics()
        
        # 创建统计卡片
        stats_cards = [
            ("总钢管种类数", stats["total_types"], "#3498db"),
            ("总钢管数量", stats["total_quantity"], "#2ecc71"),
            ("入库总数", stats["total_in"], "#f39c12"),
            ("出库总数", stats["total_out"], "#e74c3c")
        ]
        
        cards_frame = tk.Frame(stats_frame, bg="white")
        cards_frame.pack(fill=tk.BOTH, expand=True)
        
        for i, (title, value, color) in enumerate(stats_cards):
            card = tk.Frame(cards_frame, bg=color, width=200, height=150)
            card.grid(row=i//2, column=i%2, padx=10, pady=10)
            card.pack_propagate(False)
            
            title_label = tk.Label(card, text=title, font=("微软雅黑", 12),
                                  bg=color, fg="white")
            title_label.pack(pady=(30, 10))
            
            value_label = tk.Label(card, text=str(value), font=("微软雅黑", 20, "bold"),
                                  bg=color, fg="white")
            value_label.pack()
    
    def load_data(self):
        # 加载库存数据到表格
        for item in self.inventory_tree.get_children():
            self.inventory_tree.delete(item)
        
        for pipe in self.db.get_pipes():
            self.inventory_tree.insert("", "end", values=pipe)
    
    def search_inventory(self):
        # 搜索库存
        search_text = self.search_var.get().lower()
        
        for item in self.inventory_tree.get_children():
            self.inventory_tree.delete(item)
        
        for pipe in self.db.get_pipes():
            # 检查是否匹配搜索条件
            if search_text in str(pipe).lower():
                self.inventory_tree.insert("", "end", values=pipe)
    
    def load_records(self):
        # 加载出入库记录到表格
        for item in self.records_tree.get_children():
            self.records_tree.delete(item)
        
        for record in self.db.get_inventory_records():
            self.records_tree.insert("", "end", values=record)
    
    def filter_records(self):
        # 筛选出入库记录
        operation_type = self.filter_type.get()
        pipe_id = self.filter_pipe_id.get()
        
        for item in self.records_tree.get_children():
            self.records_tree.delete(item)
        
        for record in self.db.get_inventory_records(pipe_id=pipe_id if pipe_id else None, 
                                                   operation_type=operation_type):
            self.records_tree.insert("", "end", values=record)

    def on_close(self):
        self.db.close()
        self.root.destroy()

if __name__ == "__main__":
    root = tk.Tk()
    app = PipeApp(root)
    root.protocol("WM_DELETE_WINDOW", app.on_close)
    root.mainloop()
