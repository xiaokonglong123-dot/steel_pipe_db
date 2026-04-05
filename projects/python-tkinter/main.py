import sqlite3
import tkinter as tk
from tkinter import ttk, messagebox
from datetime import datetime
import configparser
import os

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

        table_frame = tk.Frame(inventory_frame, bg="white")
        table_frame.pack(fill=tk.BOTH, expand=True)

        columns = ("pipe_id", "diameter", "thickness", "length", "material",
                   "quantity", "location", "entry_date", "status")

        self.inventory_tree = ttk.Treeview(table_frame, columns=columns, show="headings")

        self.inventory_tree.heading("pipe_id", text="钢管编号")
        self.inventory_tree.heading("diameter", text="直径(毫米)")
        self.inventory_tree.heading("thickness", text="壁厚(毫米)")
        self.inventory_tree.heading("length", text="长度(米)")
        self.inventory_tree.heading("material", text="材质")
        self.inventory_tree.heading("quantity", text="数量")
        self.inventory_tree.heading("location", text="存放位置")
        self.inventory_tree.heading("entry_date", text="入库日期")
        self.inventory_tree.heading("status", text="状态")

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

    def load_data(self):
        if not hasattr(self, "inventory_tree"):
            return
        for item in self.inventory_tree.get_children():
            self.inventory_tree.delete(item)

        for pipe in self.db.get_pipes():
            values = tuple(pipe[col] for col in pipe.keys())
            self.inventory_tree.insert("", "end", values=values)

    def search_inventory(self):
        search_text = self.search_var.get().lower()

        for item in self.inventory_tree.get_children():
            self.inventory_tree.delete(item)

        for pipe in self.db.get_pipes():
            if search_text in str(dict(pipe)).lower():
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

    def on_close(self):
        self.db.close()
        self.root.destroy()

if __name__ == "__main__":
    root = tk.Tk()
    app = PipeApp(root)
    root.protocol("WM_DELETE_WINDOW", app.on_close)
    root.mainloop()
