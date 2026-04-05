import sqlite3
import tkinter as tk
from tkinter import ttk, messagebox, filedialog
from datetime import datetime
import configparser
import os
import csv

class SteelPipeDB:
    def __init__(self, db_path="pipes.db"):
        self.conn = sqlite3.connect(db_path)
        self.conn.row_factory = sqlite3.Row
        self.cursor = self.conn.cursor()
        self.create_tables()

    def create_tables(self):
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
                last_update TEXT,
                status TEXT DEFAULT "在库"
            )
        ''')

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
        now = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        self.cursor.execute("SELECT id, quantity FROM pipes WHERE pipe_id = ?", (pipe_id,))
        existing = self.cursor.fetchone()
        if existing:
            self.cursor.execute('''
                UPDATE pipes SET diameter=?, thickness=?, length=?, material=?, quantity=quantity+?, location=?, supplier=?, last_update=?
                WHERE pipe_id=?
            ''', (diameter, thickness, length, material, quantity, location, supplier, now, pipe_id))
        else:
            self.cursor.execute('''
                INSERT INTO pipes (pipe_id, diameter, thickness, length, material, quantity, location, supplier, entry_date, status)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ''', (pipe_id, diameter, thickness, length, material, quantity, location, supplier, now, "在库"))
        self.conn.commit()

    def get_pipes(self):
        self.cursor.execute("SELECT * FROM pipes")
        return self.cursor.fetchall()

    def get_pipe_by_id(self, pipe_id):
        self.cursor.execute("SELECT * FROM pipes WHERE pipe_id = ?", (pipe_id,))
        return self.cursor.fetchone()

    def delete_pipe(self, pipe_id):
        self.cursor.execute("DELETE FROM inventory_records WHERE pipe_id = ?", (pipe_id,))
        self.cursor.execute("DELETE FROM pipes WHERE pipe_id = ?", (pipe_id,))
        self.conn.commit()

    def update_pipe(self, pipe_id, diameter, thickness, length, material, quantity, location, supplier):
        now = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        self.cursor.execute('''
            UPDATE pipes SET diameter=?, thickness=?, length=?, material=?, quantity=?, location=?, supplier=?, last_update=?
            WHERE pipe_id=?
        ''', (diameter, thickness, length, material, quantity, location, supplier, now, pipe_id))
        self.conn.commit()

    def update_pipe_quantity(self, pipe_id, quantity_change):
        now = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        self.cursor.execute("SELECT quantity FROM pipes WHERE pipe_id = ?", (pipe_id,))
        row = self.cursor.fetchone()
        if row and row["quantity"] + quantity_change < 0:
            return False
        self.cursor.execute('''
            UPDATE pipes SET quantity = quantity + ?, last_update = ?
            WHERE pipe_id = ?
        ''', (quantity_change, now, pipe_id))
        self.conn.commit()
        return True

    def add_inventory_record(self, pipe_id, operation_type, quantity, operator, remarks):
        now = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        self.cursor.execute('''
            INSERT INTO inventory_records (pipe_id, operation_type, quantity, operation_date, operator, remarks)
            VALUES (?, ?, ?, ?, ?, ?)
        ''', (pipe_id, operation_type, quantity, now, operator, remarks))
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

        self.cursor.execute("SELECT COUNT(*) FROM pipes")
        stats["total_types"] = self.cursor.fetchone()[0]

        self.cursor.execute("SELECT COALESCE(SUM(quantity),0) FROM pipes")
        stats["total_quantity"] = self.cursor.fetchone()[0]

        self.cursor.execute("SELECT COALESCE(SUM(quantity),0) FROM inventory_records WHERE operation_type = '入库'")
        stats["total_in"] = self.cursor.fetchone()[0]

        self.cursor.execute("SELECT COALESCE(SUM(quantity),0) FROM inventory_records WHERE operation_type = '出库'")
        stats["total_out"] = self.cursor.fetchone()[0]

        return stats

    def batch_delete_pipes(self, pipe_ids):
        for pipe_id in pipe_ids:
            self.cursor.execute("DELETE FROM inventory_records WHERE pipe_id = ?", (pipe_id,))
            self.cursor.execute("DELETE FROM pipes WHERE pipe_id = ?", (pipe_id,))
        self.conn.commit()
        return len(pipe_ids)

    def backup_database(self, backup_path):
        import shutil
        self.conn.close()
        shutil.copy2(self.conn.cursor(), backup_path)
        self.conn = sqlite3.connect(self.conn.cursor())
        self.conn.row_factory = sqlite3.Row
        return backup_path

    def get_daily_report(self):
        today = datetime.now().strftime("%Y-%m-%d")
        self.cursor.execute("SELECT COALESCE(SUM(quantity),0) FROM inventory_records WHERE operation_type='入库' AND operation_date LIKE ?", (f"{today}%",))
        entry_count = self.cursor.fetchone()[0]
        self.cursor.execute("SELECT COALESCE(SUM(quantity),0) FROM inventory_records WHERE operation_type='出库' AND operation_date LIKE ?", (f"{today}%",))
        exit_count = self.cursor.fetchone()[0]
        self.cursor.execute("SELECT COALESCE(SUM(quantity),0) FROM pipes")
        current_stock = self.cursor.fetchone()[0]
        return {
            "date": today,
            "entry_count": entry_count,
            "exit_count": exit_count,
            "current_stock": current_stock
        }

    def export_to_csv(self, filepath):
        self.cursor.execute("SELECT pipe_id, diameter, thickness, length, material, quantity, location, supplier, entry_date, status FROM pipes")
        pipes = self.cursor.fetchall()
        with open(filepath, 'w', newline='', encoding='utf-8') as f:
            writer = csv.writer(f)
            writer.writerow(['钢管编号', '直径(mm)', '壁厚(mm)', '长度(m)', '材质', '数量', '存放位置', '供应商', '入库日期', '状态'])
            for row in pipes:
                writer.writerow([row[i] for i in range(len(row))])
        return len(pipes)

    def import_from_csv(self, filepath):
        count = 0
        errors = []
        with open(filepath, 'r', encoding='utf-8') as f:
            reader = csv.reader(f)
            next(reader)
            for row in reader:
                try:
                    if len(row) < 6:
                        errors.append(f"数据行不完整: {row}")
                        continue
                    pipe_id = row[0].strip()
                    diameter = float(row[1].strip())
                    thickness = float(row[2].strip())
                    length = float(row[3].strip())
                    material = row[4].strip()
                    quantity = int(row[5].strip())
                    location = row[6].strip() if len(row) > 6 and row[6].strip() else None
                    supplier = row[7].strip() if len(row) > 7 and row[7].strip() else None
                    self.add_pipe(pipe_id, diameter, thickness, length, material, quantity, location, supplier)
                    count += 1
                except Exception as e:
                    errors.append(f"行数据错误: {row}, 错误: {e}")
        return count, errors

    def get_low_stock(self, threshold=10):
        self.cursor.execute("SELECT * FROM pipes WHERE quantity <= ? ORDER BY quantity ASC", (threshold,))
        return self.cursor.fetchall()

    def close(self):
        self.conn.close()

class PipeApp:
    def __init__(self, root):
        self.config = configparser.ConfigParser()
        config_file = os.path.join(os.path.dirname(os.path.abspath(__file__)), "config.ini")
        if os.path.exists(config_file):
            self.config.read(config_file, encoding="utf-8")

        db_path = self.config.get("database", "db_path", fallback="pipes.db")
        window_title = self.config.get("ui", "window_title", fallback="钢管原料进出入库管理系统")
        window_width = self.config.getint("ui", "window_width", fallback=1200)
        window_height = self.config.getint("ui", "window_height", fallback=800)

        self.root = root
        self.root.title(window_title)
        self.root.geometry(f"{window_width}x{window_height}")
        self.db = SteelPipeDB(db_path)

        self.colors = {
            "nav_bg": self.config.get("colors", "nav_bg", fallback="#2c3e50"),
            "content_bg": self.config.get("colors", "content_bg", fallback="#ecf0f1"),
            "button_bg": self.config.get("colors", "button_bg", fallback="#34495e"),
            "entry_btn": self.config.get("colors", "entry_btn", fallback="#3498db"),
            "exit_btn": self.config.get("colors", "exit_btn", fallback="#e74c3c"),
            "inventory_btn": self.config.get("colors", "inventory_btn", fallback="#2ecc71"),
            "records_btn": self.config.get("colors", "records_btn", fallback="#f39c12"),
            "stats_btn": self.config.get("colors", "stats_btn", fallback="#9b59b6"),
            "close_btn": self.config.get("colors", "close_btn", fallback="#95a5a6"),
        }

        self.font_family = self.config.get("fonts", "font_family", fallback="微软雅黑")
        self.title_size = self.config.getint("fonts", "title_size", fallback=18)
        self.button_size = self.config.getint("fonts", "button_size", fallback=12)
        self.label_size = self.config.getint("fonts", "label_size", fallback=10)
        self.entry_size = self.config.getint("fonts", "entry_size", fallback=10)

        try:
            icon_path = self.config.get("ui", "icon_path", fallback="")
            if icon_path and os.path.exists(icon_path):
                self.root.iconbitmap(icon_path)
        except Exception:
            pass

        self.create_widgets()
        self.load_data()

    def create_widgets(self):
        nav_frame = tk.Frame(self.root, bg=self.colors["nav_bg"], height=60)
        nav_frame.pack(fill=tk.X)

        title_label = tk.Label(nav_frame, text=self.root.title(),
                               font=(self.font_family, self.title_size, "bold"),
                               bg=self.colors["nav_bg"], fg="white")
        title_label.pack(pady=10)

        content_frame = tk.Frame(self.root, bg=self.colors["content_bg"])
        content_frame.pack(fill=tk.BOTH, expand=True, padx=20, pady=20)

        button_frame = tk.Frame(content_frame, bg=self.colors["button_bg"], width=250)
        button_frame.pack(side=tk.LEFT, fill=tk.Y, padx=(0, 10))
        button_frame.pack_propagate(False)

        buttons = [
            ("钢管入库", self.colors["entry_btn"], self.show_entry_form),
            ("钢管出库", self.colors["exit_btn"], self.show_exit_form),
            ("库存查询", self.colors["inventory_btn"], self.show_inventory),
            ("出入库记录", self.colors["records_btn"], self.show_records),
            ("数据统计", self.colors["stats_btn"], self.show_statistics),
            ("CSV导入导出", self.colors["inventory_btn"], self.show_import_export),
            ("低库存提醒", self.colors["records_btn"], self.show_low_stock),
            ("数据备份", self.colors["stats_btn"], self.show_backup),
            ("退出系统", self.colors["close_btn"], self.on_close)
        ]

        for text, color, command in buttons:
            btn = tk.Button(button_frame, text=text, bg=color, fg="white",
                            font=(self.font_family, self.button_size), relief=tk.FLAT,
                            activebackground=color, activeforeground="white",
                            command=command)
            btn.pack(fill=tk.X, padx=10, pady=10)

        self.content_area = tk.Frame(content_frame, bg="white", relief=tk.RAISED, bd=2)
        self.content_area.pack(side=tk.RIGHT, fill=tk.BOTH, expand=True)

        self.show_inventory()

    def clear_content_area(self):
        for widget in self.content_area.winfo_children():
            widget.destroy()

    def show_entry_form(self):
        self.clear_content_area()

        form_frame = tk.Frame(self.content_area, bg="white")
        form_frame.pack(fill=tk.BOTH, expand=True, padx=20, pady=20)

        title_label = tk.Label(form_frame, text="钢管入库",
                               font=(self.font_family, 16, "bold"), bg="white")
        title_label.pack(pady=(0, 20))

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

        for label_text, var_name in fields:
            frame = tk.Frame(form_frame, bg="white")
            frame.pack(fill=tk.X, pady=5)

            label = tk.Label(frame, text=label_text, font=(self.font_family, self.label_size),
                             bg="white", width=15, anchor="e")
            label.pack(side=tk.LEFT, padx=(0, 10))

            entry = tk.Entry(frame, font=(self.font_family, self.entry_size))
            entry.pack(side=tk.LEFT, fill=tk.X, expand=True)
            self.entry_vars[var_name] = entry

        remarks_frame = tk.Frame(form_frame, bg="white")
        remarks_frame.pack(fill=tk.X, pady=5)

        remarks_label = tk.Label(remarks_frame, text="备注", font=(self.font_family, self.label_size),
                                 bg="white", width=15, anchor="e")
        remarks_label.pack(side=tk.LEFT, padx=(0, 10))

        self.remarks_entry = tk.Text(remarks_frame, font=(self.font_family, self.entry_size), height=3)
        self.remarks_entry.pack(side=tk.LEFT, fill=tk.X, expand=True)

        submit_btn = tk.Button(form_frame, text="确认入库", bg=self.colors["entry_btn"], fg="white",
                               font=(self.font_family, self.button_size + 2), relief=tk.FLAT,
                               activebackground="#2980b9", activeforeground="white",
                               command=self.submit_entry)
        submit_btn.pack(pady=20)

    def submit_entry(self):
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

        if not all([pipe_id, diameter, thickness, length, material, quantity, operator]):
            messagebox.showerror("错误", "请填写所有必填字段！")
            return

        try:
            diameter = float(diameter)
            thickness = float(thickness)
            length = float(length)
            quantity = int(quantity)

            if diameter <= 0 or diameter > 10000:
                messagebox.showerror("错误", "直径必须在0-10000mm之间")
                return
            if thickness <= 0 or thickness > 500:
                messagebox.showerror("错误", "壁厚必须在0-500mm之间")
                return
            if length <= 0 or length > 1000:
                messagebox.showerror("错误", "长度必须在0-1000m之间")
                return
            if quantity <= 0:
                messagebox.showerror("错误", "数量必须大于0")
                return

            self.db.add_pipe(pipe_id, diameter, thickness, length, material, quantity, location, supplier)
            self.db.add_inventory_record(pipe_id, "入库", quantity, operator, remarks)

            for var in self.entry_vars.values():
                var.delete(0, tk.END)
            self.remarks_entry.delete("1.0", tk.END)

            messagebox.showinfo("成功", "钢管入库操作成功！")

        except ValueError:
            messagebox.showerror("错误", "请输入有效的数值！")
        except Exception as e:
            messagebox.showerror("错误", f"入库操作失败：{str(e)}")

    def show_exit_form(self):
        self.clear_content_area()

        form_frame = tk.Frame(self.content_area, bg="white")
        form_frame.pack(fill=tk.BOTH, expand=True, padx=20, pady=20)

        title_label = tk.Label(form_frame, text="钢管出库",
                               font=(self.font_family, 16, "bold"), bg="white")
        title_label.pack(pady=(0, 20))

        fields = [
            ("钢管编号", "pipe_id"),
            ("出库数量", "quantity"),
            ("操作员", "operator")
        ]

        self.exit_vars = {}

        for label_text, var_name in fields:
            frame = tk.Frame(form_frame, bg="white")
            frame.pack(fill=tk.X, pady=5)

            label = tk.Label(frame, text=label_text, font=(self.font_family, self.label_size),
                             bg="white", width=15, anchor="e")
            label.pack(side=tk.LEFT, padx=(0, 10))

            entry = tk.Entry(frame, font=(self.font_family, self.entry_size))
            entry.pack(side=tk.LEFT, fill=tk.X, expand=True)
            self.exit_vars[var_name] = entry

        remarks_frame = tk.Frame(form_frame, bg="white")
        remarks_frame.pack(fill=tk.X, pady=5)

        remarks_label = tk.Label(remarks_frame, text="备注", font=(self.font_family, self.label_size),
                                 bg="white", width=15, anchor="e")
        remarks_label.pack(side=tk.LEFT, padx=(0, 10))

        self.exit_remarks_entry = tk.Text(remarks_frame, font=(self.font_family, self.entry_size), height=3)
        self.exit_remarks_entry.pack(side=tk.LEFT, fill=tk.X, expand=True)

        submit_btn = tk.Button(form_frame, text="确认出库", bg=self.colors["exit_btn"], fg="white",
                               font=(self.font_family, self.button_size + 2), relief=tk.FLAT,
                               activebackground="#c0392b", activeforeground="white",
                               command=self.submit_exit)
        submit_btn.pack(pady=20)

    def submit_exit(self):
        pipe_id = self.exit_vars["pipe_id"].get()
        quantity = self.exit_vars["quantity"].get()
        operator = self.exit_vars["operator"].get()
        remarks = self.exit_remarks_entry.get("1.0", tk.END).strip()

        if not all([pipe_id, quantity, operator]):
            messagebox.showerror("错误", "请填写所有必填字段！")
            return

        try:
            quantity = int(quantity)

            if quantity <= 0:
                messagebox.showerror("错误", "出库数量必须大于0")
                return

            pipe_data = self.db.get_pipe_by_id(pipe_id)

            if not pipe_data:
                messagebox.showerror("错误", "未找到该钢管编号！")
                return

            current_quantity = pipe_data["quantity"]

            if current_quantity < quantity:
                messagebox.showerror("错误", f"库存不足！当前库存：{current_quantity}")
                return

            self.db.update_pipe_quantity(pipe_id, -quantity)
            self.db.add_inventory_record(pipe_id, "出库", quantity, operator, remarks)

            for var in self.exit_vars.values():
                var.delete(0, tk.END)
            self.exit_remarks_entry.delete("1.0", tk.END)

            messagebox.showinfo("成功", "钢管出库操作成功！")

        except ValueError:
            messagebox.showerror("错误", "请输入有效的数值！")
        except Exception as e:
            messagebox.showerror("错误", f"出库操作失败：{str(e)}")

    def show_inventory(self):
        self.clear_content_area()

        inventory_frame = tk.Frame(self.content_area, bg="white")
        inventory_frame.pack(fill=tk.BOTH, expand=True, padx=20, pady=20)

        title_label = tk.Label(inventory_frame, text="库存查询",
                               font=(self.font_family, 16, "bold"), bg="white")
        title_label.pack(pady=(0, 20))

        search_frame = tk.Frame(inventory_frame, bg="white")
        search_frame.pack(fill=tk.X, pady=(0, 20))

        search_label = tk.Label(search_frame, text="搜索:", font=(self.font_family, self.label_size),
                                bg="white")
        search_label.pack(side=tk.LEFT, padx=(0, 10))

        self.search_var = tk.StringVar()
        search_entry = tk.Entry(search_frame, textvariable=self.search_var,
                                font=(self.font_family, self.entry_size))
        search_entry.pack(side=tk.LEFT, fill=tk.X, expand=True, padx=(0, 10))

        search_btn = tk.Button(search_frame, text="搜索", bg=self.colors["entry_btn"], fg="white",
                               font=(self.font_family, self.label_size), relief=tk.FLAT,
                               activebackground="#2980b9", activeforeground="white",
                               command=self.search_inventory)
        search_btn.pack(side=tk.LEFT)

        refresh_btn = tk.Button(search_frame, text="刷新", bg=self.colors["inventory_btn"], fg="white",
                                font=(self.font_family, self.label_size), relief=tk.FLAT,
                                activebackground="#27ae60", activeforeground="white",
                                command=self.load_data)
        refresh_btn.pack(side=tk.LEFT, padx=(10, 0))

        self.batch_delete_btn = tk.Button(search_frame, text="批量删除", bg="#e74c3c", fg="white",
                                font=(self.font_family, self.label_size), relief=tk.FLAT,
                                activebackground="#c0392b", activeforeground="white",
                                command=self.batch_delete_selected, state=tk.DISABLED)
        self.batch_delete_btn.pack(side=tk.LEFT, padx=(10, 0))

        table_frame = tk.Frame(inventory_frame, bg="white")
        table_frame.pack(fill=tk.BOTH, expand=True)

        columns = ("pipe_id", "diameter", "thickness", "length", "material",
                   "quantity", "location", "entry_date", "status")

        self.inventory_tree = ttk.Treeview(table_frame, columns=columns, show="headings", selectmode="extended")

        self.inventory_tree.heading("pipe_id", text="钢管编号")
        self.inventory_tree.heading("diameter", text="直径(毫米)")
        self.inventory_tree.heading("thickness", text="壁厚(毫米)")
        self.inventory_tree.heading("length", text="长度(米)")
        self.inventory_tree.heading("material", text="材质")
        self.inventory_tree.heading("quantity", text="数量")
        self.inventory_tree.heading("location", text="存放位置")
        self.inventory_tree.heading("entry_date", text="入库日期")
        self.inventory_tree.heading("status", text="状态")

        self.inventory_tree.bind("<<TreeviewSelect>>", self.on_tree_select)

        for col in columns:
            self.inventory_tree.column(col, width=100, anchor=tk.CENTER)

        scrollbar = ttk.Scrollbar(table_frame, orient=tk.VERTICAL,
                                  command=self.inventory_tree.yview)
        self.inventory_tree.configure(yscrollcommand=scrollbar.set)

        self.inventory_tree.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        scrollbar.pack(side=tk.RIGHT, fill=tk.Y)

        self.load_data()

    def show_records(self):
        self.clear_content_area()

        records_frame = tk.Frame(self.content_area, bg="white")
        records_frame.pack(fill=tk.BOTH, expand=True, padx=20, pady=20)

        title_label = tk.Label(records_frame, text="出入库记录",
                               font=(self.font_family, 16, "bold"), bg="white")
        title_label.pack(pady=(0, 20))

        filter_frame = tk.Frame(records_frame, bg="white")
        filter_frame.pack(fill=tk.X, pady=(0, 20))

        tk.Label(filter_frame, text="操作类型:", font=(self.font_family, self.label_size),
                 bg="white").pack(side=tk.LEFT, padx=(0, 10))

        self.filter_type = tk.StringVar(value="全部")
        type_combobox = ttk.Combobox(filter_frame, textvariable=self.filter_type,
                                     values=["全部", "入库", "出库"], state="readonly")
        type_combobox.pack(side=tk.LEFT, padx=(0, 20))

        tk.Label(filter_frame, text="钢管编号:", font=(self.font_family, self.label_size),
                 bg="white").pack(side=tk.LEFT, padx=(0, 10))

        self.filter_pipe_id = tk.StringVar()
        pipe_id_entry = tk.Entry(filter_frame, textvariable=self.filter_pipe_id,
                                 font=(self.font_family, self.entry_size))
        pipe_id_entry.pack(side=tk.LEFT, padx=(0, 20))

        filter_btn = tk.Button(filter_frame, text="筛选", bg=self.colors["entry_btn"], fg="white",
                               font=(self.font_family, self.label_size), relief=tk.FLAT,
                               activebackground="#2980b9", activeforeground="white",
                               command=self.filter_records)
        filter_btn.pack(side=tk.LEFT)

        table_frame = tk.Frame(records_frame, bg="white")
        table_frame.pack(fill=tk.BOTH, expand=True)

        columns = ("pipe_id", "operation_type", "quantity", "operation_date", "operator", "remarks")

        self.records_tree = ttk.Treeview(table_frame, columns=columns, show="headings")

        self.records_tree.heading("pipe_id", text="钢管编号")
        self.records_tree.heading("operation_type", text="操作类型")
        self.records_tree.heading("quantity", text="数量")
        self.records_tree.heading("operation_date", text="操作日期")
        self.records_tree.heading("operator", text="操作员")
        self.records_tree.heading("remarks", text="备注")

        for col in columns:
            self.records_tree.column(col, width=100, anchor=tk.CENTER)

        scrollbar = ttk.Scrollbar(table_frame, orient=tk.VERTICAL,
                                  command=self.records_tree.yview)
        self.records_tree.configure(yscrollcommand=scrollbar.set)

        self.records_tree.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        scrollbar.pack(side=tk.RIGHT, fill=tk.Y)

        self.load_records()

    def show_statistics(self):
        self.clear_content_area()

        stats_frame = tk.Frame(self.content_area, bg="white")
        stats_frame.pack(fill=tk.BOTH, expand=True, padx=20, pady=20)

        title_label = tk.Label(stats_frame, text="数据统计",
                               font=(self.font_family, 16, "bold"), bg="white")
        title_label.pack(pady=(0, 20))

        stats = self.db.get_statistics()

        stats_cards = [
            ("总钢管种类数", stats["total_types"], self.colors["entry_btn"]),
            ("总钢管数量", stats["total_quantity"], self.colors["inventory_btn"]),
            ("入库总数", stats["total_in"], self.colors["records_btn"]),
            ("出库总数", stats["total_out"], self.colors["exit_btn"])
        ]

        cards_frame = tk.Frame(stats_frame, bg="white")
        cards_frame.pack(fill=tk.BOTH, expand=True)

        for i, (title, value, color) in enumerate(stats_cards):
            card = tk.Frame(cards_frame, bg=color, width=200, height=150)
            card.grid(row=i // 2, column=i % 2, padx=10, pady=10)
            card.pack_propagate(False)

            title_label = tk.Label(card, text=title, font=(self.font_family, 12),
                                   bg=color, fg="white")
            title_label.pack(pady=(30, 10))

            value_label = tk.Label(card, text=str(value), font=(self.font_family, 20, "bold"),
                                   bg=color, fg="white")
            value_label.pack()

    def show_import_export(self):
        self.clear_content_area()

        ie_frame = tk.Frame(self.content_area, bg="white")
        ie_frame.pack(fill=tk.BOTH, expand=True, padx=20, pady=20)

        title_label = tk.Label(ie_frame, text="CSV 导入/导出",
                               font=(self.font_family, 16, "bold"), bg="white")
        title_label.pack(pady=(0, 20))

        btn_frame = tk.Frame(ie_frame, bg="white")
        btn_frame.pack(pady=20)

        export_btn = tk.Button(btn_frame, text="导出库存数据 (CSV)", 
                               bg=self.colors["entry_btn"], fg="white",
                               font=(self.font_family, self.button_size), relief=tk.FLAT,
                               command=self.export_csv)
        export_btn.pack(side=tk.LEFT, padx=10)

        import_btn = tk.Button(btn_frame, text="导入库存数据 (CSV)", 
                               bg=self.colors["inventory_btn"], fg="white",
                               font=(self.font_family, self.button_size), relief=tk.FLAT,
                               command=self.import_csv)
        import_btn.pack(side=tk.LEFT, padx=10)

        self.import_result_label = tk.Label(ie_frame, text="", font=(self.font_family, self.label_size),
                                            bg="white", fg="green")
        self.import_result_label.pack(pady=20)

    def export_csv(self):
        filepath = filedialog.asksaveasfilename(
            defaultextension=".csv",
            filetypes=[("CSV files", "*.csv")],
            title="导出库存数据"
        )
        if filepath:
            try:
                count = self.db.export_to_csv(filepath)
                messagebox.showinfo("成功", f"已导出 {count} 条记录到 {filepath}")
            except Exception as e:
                messagebox.showerror("错误", f"导出失败: {e}")

    def import_csv(self):
        filepath = filedialog.askopenfilename(
            filetypes=[("CSV files", "*.csv")],
            title="导入库存数据"
        )
        if filepath:
            try:
                count, errors = self.db.import_from_csv(filepath)
                if errors:
                    messagebox.showwarning("部分失败", f"成功导入 {count} 条，失败 {len(errors)} 条\n错误详情:\n" + "\n".join(errors[:5]))
                else:
                    messagebox.showinfo("成功", f"成功导入 {count} 条记录")
                self.load_data()
            except Exception as e:
                messagebox.showerror("错误", f"导入失败: {e}")

    def show_low_stock(self):
        self.clear_content_area()

        ls_frame = tk.Frame(self.content_area, bg="white")
        ls_frame.pack(fill=tk.BOTH, expand=True, padx=20, pady=20)

        title_label = tk.Label(ls_frame, text="低库存提醒",
                               font=(self.font_family, 16, "bold"), bg="white")
        title_label.pack(pady=(0, 20))

        threshold_frame = tk.Frame(ls_frame, bg="white")
        threshold_frame.pack(fill=tk.X, pady=(0, 10))

        tk.Label(threshold_frame, text="库存阈值:", font=(self.font_family, self.label_size),
                 bg="white").pack(side=tk.LEFT, padx=(0, 10))

        self.threshold_var = tk.StringVar(value="10")
        threshold_entry = tk.Entry(threshold_frame, textvariable=self.threshold_var,
                                    font=(self.font_family, self.entry_size), width=10)
        threshold_entry.pack(side=tk.LEFT, padx=(0, 10))

        tk.Button(threshold_frame, text="查询", bg=self.colors["entry_btn"], fg="white",
                  font=(self.font_family, self.label_size), relief=tk.FLAT,
                  command=self.check_low_stock).pack(side=tk.LEFT)

        table_frame = tk.Frame(ls_frame, bg="white")
        table_frame.pack(fill=tk.BOTH, expand=True)

        columns = ("pipe_id", "diameter", "thickness", "length", "material", "quantity", "location")

        self.low_stock_tree = ttk.Treeview(table_frame, columns=columns, show="headings")

        self.low_stock_tree.heading("pipe_id", text="钢管编号")
        self.low_stock_tree.heading("diameter", text="直径(毫米)")
        self.low_stock_tree.heading("thickness", text="壁厚(毫米)")
        self.low_stock_tree.heading("length", text="长度(米)")
        self.low_stock_tree.heading("material", text="材质")
        self.low_stock_tree.heading("quantity", text="数量")
        self.low_stock_tree.heading("location", text="存放位置")

        for col in columns:
            self.low_stock_tree.column(col, width=100, anchor=tk.CENTER)

        scrollbar = ttk.Scrollbar(table_frame, orient=tk.VERTICAL,
                                  command=self.low_stock_tree.yview)
        self.low_stock_tree.configure(yscrollcommand=scrollbar.set)

        self.low_stock_tree.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        scrollbar.pack(side=tk.RIGHT, fill=tk.Y)

        self.check_low_stock()

    def check_low_stock(self):
        if not hasattr(self, "low_stock_tree"):
            return
        for item in self.low_stock_tree.get_children():
            self.low_stock_tree.delete(item)

        try:
            threshold = int(self.threshold_var.get())
        except ValueError:
            threshold = 10

        for pipe in self.db.get_low_stock(threshold):
            values = tuple(pipe[col] for col in pipe.keys())
            self.low_stock_tree.insert("", "end", values=values)

    def load_data(self):
        if not hasattr(self, "inventory_tree"):
            return
        for item in self.inventory_tree.get_children():
            self.inventory_tree.delete(item)

        for pipe in self.db.get_pipes():
            values = tuple(pipe[col] for col in pipe.keys())
            self.inventory_tree.insert("", "end", values=values)

        if hasattr(self, 'batch_delete_btn'):
            self.batch_delete_btn.config(state=tk.DISABLED)

    def on_tree_select(self, event):
        selected = self.inventory_tree.selection()
        if hasattr(self, 'batch_delete_btn'):
            if selected:
                self.batch_delete_btn.config(state=tk.NORMAL)
            else:
                self.batch_delete_btn.config(state=tk.DISABLED)

    def batch_delete_selected(self):
        selected = self.inventory_tree.selection()
        if not selected:
            return
        
        pipe_ids = [self.inventory_tree.item(item)['values'][0] for item in selected]
        
        if not messagebox.askyesno("确认", f"确定删除选中的 {len(pipe_ids)} 条记录吗？"):
            return
        
        try:
            count = self.db.batch_delete_pipes(pipe_ids)
            messagebox.showinfo("成功", f"已删除 {count} 条记录")
            self.load_data()
        except Exception as e:
            messagebox.showerror("错误", f"删除失败: {e}")

    def search_inventory(self):
        search_text = self.search_var.get().lower()

        for item in self.inventory_tree.get_children():
            self.inventory_tree.delete(item)

        for pipe in self.db.get_pipes():
            pipe_dict = dict(pipe)
            search_fields = [
                str(pipe_dict.get('pipe_id', '')),
                str(pipe_dict.get('material', '')),
                str(pipe_dict.get('location', '')),
                str(pipe_dict.get('supplier', '')),
            ]
            if search_text in ' '.join(search_fields).lower():
                values = tuple(pipe[col] for col in pipe.keys())
                self.inventory_tree.insert("", "end", values=values)

    def load_records(self):
        if not hasattr(self, "records_tree"):
            return
        for item in self.records_tree.get_children():
            self.records_tree.delete(item)

        for record in self.db.get_inventory_records():
            values = tuple(record[col] for col in record.keys())
            self.records_tree.insert("", "end", values=values)

    def filter_records(self):
        operation_type = self.filter_type.get()
        pipe_id = self.filter_pipe_id.get()

        for item in self.records_tree.get_children():
            self.records_tree.delete(item)

        for record in self.db.get_inventory_records(pipe_id=pipe_id if pipe_id else None,
                                                    operation_type=operation_type):
            values = tuple(record[col] for col in record.keys())
            self.records_tree.insert("", "end", values=values)

    def show_backup(self):
        self.clear_content_area()

        backup_frame = tk.Frame(self.content_area, bg="white")
        backup_frame.pack(fill=tk.BOTH, expand=True, padx=20, pady=20)

        title_label = tk.Label(backup_frame, text="数据备份与报表",
                               font=(self.font_family, 16, "bold"), bg="white")
        title_label.pack(pady=(0, 20))

        btn_frame = tk.Frame(backup_frame, bg="white")
        btn_frame.pack(pady=20)

        backup_btn = tk.Button(btn_frame, text="备份数据库", 
                               bg=self.colors["entry_btn"], fg="white",
                               font=(self.font_family, self.button_size), relief=tk.FLAT,
                               command=self.backup_database)
        backup_btn.pack(side=tk.LEFT, padx=10)

        report_btn = tk.Button(btn_frame, text="今日报表", 
                               bg=self.colors["inventory_btn"], fg="white",
                               font=(self.font_family, self.button_size), relief=tk.FLAT,
                               command=self.show_daily_report)
        report_btn.pack(side=tk.LEFT, padx=10)

        self.report_text = tk.Text(backup_frame, font=(self.font_family, self.label_size),
                                   height=15, width=60)
        self.report_text.pack(pady=20)

    def backup_database(self):
        filepath = filedialog.asksaveasfilename(
            defaultextension=".db",
            filetypes=[("Database files", "*.db")],
            title="备份数据库"
        )
        if filepath:
            try:
                self.db.backup_database(filepath)
                messagebox.showinfo("成功", f"数据库已备份到 {filepath}")
            except Exception as e:
                messagebox.showerror("错误", f"备份失败: {e}")

    def show_daily_report(self):
        if not hasattr(self, 'report_text'):
            return
        try:
            report = self.db.get_daily_report()
            report_str = f"""
===========================================
            今日报表 - {report['date']}
===========================================
入库数量: {report['entry_count']}
出库数量: {report['exit_count']}
当前库存: {report['current_stock']}
===========================================
"""
            self.report_text.delete("1.0", tk.END)
            self.report_text.insert("1.0", report_str)
        except Exception as e:
            messagebox.showerror("错误", f"生成报表失败: {e}")

    def on_close(self):
        self.db.close()
        self.root.destroy()

if __name__ == "__main__":
    root = tk.Tk()
    app = PipeApp(root)
    root.protocol("WM_DELETE_WINDOW", app.on_close)
    root.mainloop()
