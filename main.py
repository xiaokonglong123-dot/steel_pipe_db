import sqlite3
import tkinter as tk
from tkinter import ttk, messagebox

class SteelPipeDB:
    def __init__(self, db_path="pipes.db"):
        self.conn = sqlite3.connect(db_path)
        self.cursor = self.conn.cursor()
        self.create_table()

    def create_table(self):
        self.cursor.execute('''
            CREATE TABLE IF NOT EXISTS pipes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                diameter REAL,
                thickness REAL,
                length REAL,
                material TEXT,
                quantity INTEGER
            )
        ''')
        self.conn.commit()

    def add_pipe(self, diameter, thickness, length, material, quantity):
        self.cursor.execute('''
            INSERT INTO pipes (diameter, thickness, length, material, quantity)
            VALUES (?, ?, ?, ?, ?)
        ''', (diameter, thickness, length, material, quantity))
        self.conn.commit()

    def get_pipes(self):
        self.cursor.execute('SELECT * FROM pipes')
        return self.cursor.fetchall()

    def delete_pipe(self, pipe_id):
        self.cursor.execute('DELETE FROM pipes WHERE id = ?', (pipe_id,))
        self.conn.commit()

    def update_pipe(self, pipe_id, diameter, thickness, length, material, quantity):
        self.cursor.execute('''
            UPDATE pipes SET diameter=?, thickness=?, length=?, material=?, quantity=?
            WHERE id=?
        ''', (diameter, thickness, length, material, quantity, pipe_id))
        self.conn.commit()

    def close(self):
        self.conn.close()

class PipeApp:
    def __init__(self, root):
        self.root = root
        self.root.title("Steel Pipe Database")
        self.db = SteelPipeDB()

        self.create_widgets()
        self.load_data()

    def create_widgets(self):
        # Input Frame
        input_frame = ttk.LabelFrame(self.root, text="Pipe Details")
        input_frame.pack(fill="x", padx=10, pady=5)

        ttk.Label(input_frame, text="Diameter (mm):").grid(row=0, column=0, padx=5, pady=5)
        self.diameter_entry = ttk.Entry(input_frame, width=15)
        self.diameter_entry.grid(row=0, column=1, padx=5, pady=5)

        ttk.Label(input_frame, text="Thickness (mm):").grid(row=0, column=2, padx=5, pady=5)
        self.thickness_entry = ttk.Entry(input_frame, width=15)
        self.thickness_entry.grid(row=0, column=3, padx=5, pady=5)

        ttk.Label(input_frame, text="Length (m):").grid(row=1, column=0, padx=5, pady=5)
        self.length_entry = ttk.Entry(input_frame, width=15)
        self.length_entry.grid(row=1, column=1, padx=5, pady=5)

        ttk.Label(input_frame, text="Material:").grid(row=1, column=2, padx=5, pady=5)
        self.material_combo = ttk.Combobox(input_frame, values=["Carbon Steel", "Stainless Steel", "Alloy Steel"], width=13)
        self.material_combo.grid(row=1, column=3, padx=5, pady=5)

        ttk.Label(input_frame, text="Quantity:").grid(row=0, column=4, padx=5, pady=5)
        self.quantity_entry = ttk.Entry(input_frame, width=10)
        self.quantity_entry.grid(row=0, column=5, padx=5, pady=5)

        btn_frame = ttk.Frame(input_frame)
        btn_frame.grid(row=1, column=4, columnspan=3, padx=5, pady=5)
        ttk.Button(btn_frame, text="Add", command=self.add_pipe).pack(side="left", padx=2)
        ttk.Button(btn_frame, text="Update", command=self.update_pipe).pack(side="left", padx=2)
        ttk.Button(btn_frame, text="Delete", command=self.delete_pipe).pack(side="left", padx=2)
        ttk.Button(btn_frame, text="Clear", command=self.clear_inputs).pack(side="left", padx=2)

        # Treeview Frame
        tree_frame = ttk.LabelFrame(self.root, text="Pipe Inventory")
        tree_frame.pack(fill="both", expand=True, padx=10, pady=5)

        columns = ("ID", "Diameter", "Thickness", "Length", "Material", "Quantity")
        self.tree = ttk.Treeview(tree_frame, columns=columns, show="headings")
        for col in columns:
            self.tree.heading(col, text=col)
            self.tree.column(col, width=80, anchor="center")
        self.tree.pack(fill="both", expand=True, padx=5, pady=5)

        scrollbar = ttk.Scrollbar(tree_frame, orient="vertical", command=self.tree.yview)
        self.tree.configure(yscrollcommand=scrollbar.set)
        scrollbar.pack(side="right", fill="y")

        self.tree.bind("<<TreeviewSelect>>", self.on_select)

    def load_data(self):
        for item in self.tree.get_children():
            self.tree.delete(item)
        for pipe in self.db.get_pipes():
            self.tree.insert("", "end", values=pipe)

    def add_pipe(self):
        try:
            diameter = float(self.diameter_entry.get())
            thickness = float(self.thickness_entry.get())
            length = float(self.length_entry.get())
            material = self.material_combo.get()
            quantity = int(self.quantity_entry.get())

            if not material:
                messagebox.showerror("Error", "Please select a material")
                return

            self.db.add_pipe(diameter, thickness, length, material, quantity)
            self.load_data()
            self.clear_inputs()
            messagebox.showinfo("Success", "Pipe added successfully")
        except ValueError:
            messagebox.showerror("Error", "Please enter valid numbers")

    def delete_pipe(self):
        selected = self.tree.selection()
        if not selected:
            messagebox.showwarning("Warning", "Please select a pipe to delete")
            return

        pipe_id = self.tree.item(selected[0])["values"][0]
        if messagebox.askyesno("Confirm", "Delete this pipe?"):
            self.db.delete_pipe(pipe_id)
            self.load_data()
            self.clear_inputs()

    def update_pipe(self):
        selected = self.tree.selection()
        if not selected:
            messagebox.showwarning("Warning", "Please select a pipe to update")
            return

        try:
            pipe_id = self.tree.item(selected[0])["values"][0]
            diameter = float(self.diameter_entry.get())
            thickness = float(self.thickness_entry.get())
            length = float(self.length_entry.get())
            material = self.material_combo.get()
            quantity = int(self.quantity_entry.get())

            self.db.update_pipe(pipe_id, diameter, thickness, length, material, quantity)
            self.load_data()
            messagebox.showinfo("Success", "Pipe updated successfully")
        except ValueError:
            messagebox.showerror("Error", "Please enter valid numbers")

    def on_select(self, event):
        selected = self.tree.selection()
        if selected:
            values = self.tree.item(selected[0])["values"]
            self.diameter_entry.delete(0, "end")
            self.diameter_entry.insert(0, str(values[1]))
            self.thickness_entry.delete(0, "end")
            self.thickness_entry.insert(0, str(values[2]))
            self.length_entry.delete(0, "end")
            self.length_entry.insert(0, str(values[3]))
            self.material_combo.set(values[4])
            self.quantity_entry.delete(0, "end")
            self.quantity_entry.insert(0, str(values[5]))

    def clear_inputs(self):
        for entry in [self.diameter_entry, self.thickness_entry, self.length_entry, self.quantity_entry]:
            entry.delete(0, "end")
        self.material_combo.set("")

    def on_close(self):
        self.db.close()
        self.root.destroy()

if __name__ == "__main__":
    root = tk.Tk()
    app = PipeApp(root)
    root.protocol("WM_DELETE_WINDOW", app.on_close)
    root.mainloop()
