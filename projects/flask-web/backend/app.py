import io
import csv
from flask import Flask, request, jsonify, g, Response
from flask_cors import CORS
import sqlite3
import os
from datetime import datetime

DB_PATH = os.path.join(os.path.dirname(__file__), 'pipes.db')

def get_db():
    if 'db' not in g:
        g.db = sqlite3.connect(DB_PATH)
        g.db.row_factory = sqlite3.Row
    return g.db

def dict_from_row(row):
    return {k: row[k] for k in row.keys()}

app = Flask(__name__, static_folder=os.path.join(os.path.dirname(__file__), '../frontend'), static_url_path="")
CORS(app)

def init_db():
    with app.app_context():
        db = get_db()
        db.execute('''
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
        db.execute('''
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
        db.commit()

os.makedirs(os.path.dirname(DB_PATH), exist_ok=True)
if not os.path.exists(DB_PATH):
    open(DB_PATH, 'a').close()
init_db()

def query_pipes(params):
    db = get_db()
    sql = ["SELECT * FROM pipes WHERE 1=1"]
    args = []
    if params.get('material'):
        sql.append("AND material = ?")
        args.append(params['material'])
    if params.get('q'):
        sql.append("AND (material LIKE ? OR pipe_id LIKE ?)")
        args.append(f"%{params['q']}%")
        args.append(f"%{params['q']}%")
    sort_by = params.get('sort_by')
    if sort_by not in {"id","diameter","thickness","length","material","quantity","pipe_id"}:
        sort_by = 'id'
    sort_dir = params.get('sort_dir')
    if sort_dir not in {"asc","desc"}:
        sort_dir = 'asc'
    sql.append(f"ORDER BY {sort_by} {sort_dir}")
    page = int(params.get('page', 1))
    per_page = int(params.get('per_page', 10))
    offset = (page - 1) * per_page
    sql.append("LIMIT ? OFFSET ?")
    args.extend([per_page, offset])
    cur = db.execute(" ".join(sql), tuple(args))
    rows = [dict_from_row(r) for r in cur.fetchall()]
    count_sql = "SELECT COUNT(*) FROM pipes WHERE 1=1 "
    count_args = []
    if params.get('material'):
        count_sql += "AND material = ?"
        count_args.append(params['material'])
    if params.get('q'):
        count_sql += "AND (material LIKE ? OR pipe_id LIKE ?)"
        count_args.append(f"%{params['q']}%")
        count_args.append(f"%{params['q']}%")
    total = db.execute(count_sql, tuple(count_args)).fetchone()[0]
    return {'total': total, 'page': page, 'per_page': per_page, 'pipes': rows}

@app.route("/pipes", methods=["GET","POST"])
def pipes():
    if request.method == 'GET':
        params = {
            'q': request.args.get('q'),
            'material': request.args.get('material'),
            'min_diameter': request.args.get('min_diameter', type=float),
            'max_diameter': request.args.get('max_diameter', type=float),
            'min_length': request.args.get('min_length', type=float),
            'max_length': request.args.get('max_length', type=float),
            'sort_by': request.args.get('sort_by'),
            'sort_dir': request.args.get('sort_dir'),
            'page': request.args.get('page', type=int) or 1,
            'per_page': request.args.get('per_page', type=int) or 10,
        }
        res = query_pipes(params)
        return jsonify(res)
    else:
        data = request.json or {}
        pipe_id = data.get("pipe_id")
        diameter = data.get("diameter")
        thickness = data.get("thickness")
        length = data.get("length")
        material = data.get("material")
        quantity = data.get("quantity")
        errors = {}
        if any(v is None for v in [pipe_id, diameter, thickness, length, material, quantity]):
            if pipe_id is None: errors['pipe_id'] = 'required'
            if diameter is None: errors['diameter'] = 'required'
            if thickness is None: errors['thickness'] = 'required'
            if length is None: errors['length'] = 'required'
            if material is None: errors['material'] = 'required'
            if quantity is None: errors['quantity'] = 'required'
            return jsonify({'errors': errors}), 400
        now = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        db = get_db()
        existing = db.execute("SELECT id FROM pipes WHERE pipe_id = ?", (pipe_id,)).fetchone()
        if existing:
            db.execute('''
                UPDATE pipes SET diameter=?, thickness=?, length=?, material=?, quantity=quantity+?, last_update=?
                WHERE pipe_id=?
            ''', (diameter, thickness, length, material, quantity, now, pipe_id))
        else:
            db.execute('''
                INSERT INTO pipes (pipe_id, diameter, thickness, length, material, quantity, entry_date, status)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ''', (pipe_id, diameter, thickness, length, material, quantity, now, "在库"))
        db.commit()
        return jsonify({"status": "created"}), 201

@app.route("/pipes/<int:pipe_id>", methods=["GET","PUT","DELETE"])
def pipe(pipe_id):
    db = get_db()
    if request.method == 'GET':
        cur = db.execute("SELECT * FROM pipes WHERE id=?", (pipe_id,))
        row = cur.fetchone()
        if row:
            return jsonify(dict_from_row(row))
        return jsonify({"error": "not found"}), 404
    elif request.method == 'PUT':
        data = request.json or {}
        diameter = data.get("diameter")
        thickness = data.get("thickness")
        length = data.get("length")
        material = data.get("material")
        quantity = data.get("quantity")
        if any(v is None for v in [diameter, thickness, length, material, quantity]):
            return jsonify({"error": "missing fields"}), 400
        db.execute("UPDATE pipes SET diameter=?, thickness=?, length=?, material=?, quantity=? WHERE id=?",
                   (diameter, thickness, length, material, quantity, pipe_id))
        db.commit()
        return jsonify({"status": "updated"})
    else:
        db.execute("DELETE FROM pipes WHERE id=?", (pipe_id,))
        db.commit()
        return jsonify({"status": "deleted"})

@app.route('/pipes/export', methods=['GET'])
def export_pipes():
    fmt = request.args.get('format', 'csv')
    params = {
        'q': request.args.get('q'),
        'material': request.args.get('material'),
        'min_diameter': request.args.get('min_diameter', type=float),
        'max_diameter': request.args.get('max_diameter', type=float),
        'min_length': request.args.get('min_length', type=float),
        'max_length': request.args.get('max_length', type=float),
        'sort_by': request.args.get('sort_by'),
        'sort_dir': request.args.get('sort_dir'),
    }
    res = query_pipes(params)
    pipes = res.get('pipes', [])
    if fmt == 'json':
        return jsonify(pipes)
    # default csv
    def generate():
        data = io.StringIO()
        writer = csv.writer(data)
        writer.writerow(['id','diameter','thickness','length','material','quantity'])
        yield data.getvalue()
        data.seek(0)
        data.truncate(0)
        for p in pipes:
            writer.writerow([p['id'], p['diameter'], p['thickness'], p['length'], p['material'], p['quantity']])
            yield data.getvalue()
            data.seek(0)
            data.truncate(0)
    return Response(generate(), mimetype='text/csv', headers={'Content-Disposition':'attachment; filename="pipes.csv"'})

@app.route('/pipes/import', methods=['POST'])
def import_pipes():
    if 'file' not in request.files:
        return jsonify({'error': 'no file'}), 400
    file = request.files['file']
    if not file.filename.endswith('.csv'):
        return jsonify({'error': 'only CSV allowed'}), 400
    db = get_db()
    content = file.read().decode('utf-8')
    reader = csv.DictReader(io.StringIO(content))
    count = 0
    errors = []
    for row in reader:
        try:
            diameter = float(row.get('diameter'))
            thickness = float(row.get('thickness'))
            length = float(row.get('length'))
            material = row.get('material')
            quantity = int(row.get('quantity'))
            if None in (diameter, thickness, length, material, quantity):
                raise ValueError('missing fields')
            db.execute("INSERT INTO pipes (diameter, thickness, length, material, quantity) VALUES (?,?,?,?,?)",
                       (diameter, thickness, length, material, quantity))
            count += 1
        except Exception as e:
            errors.append(str(e))
    db.commit()
    return jsonify({'imported': count, 'errors': errors})

@app.route("/pipes/entry", methods=["POST"])
def entry_pipe():
    data = request.json or {}
    pipe_id = data.get("pipe_id")
    diameter = data.get("diameter")
    thickness = data.get("thickness")
    length = data.get("length")
    material = data.get("material")
    quantity = data.get("quantity")
    location = data.get("location")
    supplier = data.get("supplier")
    operator = data.get("operator", "system")
    remarks = data.get("remarks", "")

    if not pipe_id or not quantity:
        return jsonify({'error': 'pipe_id and quantity required'}), 400

    now = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    db = get_db()

    existing = db.execute("SELECT id FROM pipes WHERE pipe_id = ?", (pipe_id,)).fetchone()
    if existing:
        db.execute('''
            UPDATE pipes SET diameter=?, thickness=?, length=?, material=?, quantity=quantity+?, location=?, supplier=?, last_update=?
            WHERE pipe_id=?
        ''', (diameter, thickness, length, material, quantity, location, supplier, now, pipe_id))
    else:
        db.execute('''
            INSERT INTO pipes (pipe_id, diameter, thickness, length, material, quantity, location, supplier, entry_date, status)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ''', (pipe_id, diameter, thickness, length, material, quantity, location, supplier, now, "在库"))

    db.execute('''
        INSERT INTO inventory_records (pipe_id, operation_type, quantity, operation_date, operator, remarks)
        VALUES (?, ?, ?, ?, ?, ?)
    ''', (pipe_id, "入库", quantity, now, operator, remarks))
    db.commit()
    return jsonify({"status": "created"}), 201

@app.route("/pipes/exit", methods=["POST"])
def exit_pipe():
    data = request.json or {}
    pipe_id = data.get("pipe_id")
    quantity = data.get("quantity")
    operator = data.get("operator", "system")
    remarks = data.get("remarks", "")

    if not pipe_id or not quantity:
        return jsonify({'error': 'pipe_id and quantity required'}), 400

    now = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    db = get_db()

    pipe = db.execute("SELECT quantity FROM pipes WHERE pipe_id = ?", (pipe_id,)).fetchone()
    if not pipe:
        return jsonify({"error": "未找到该钢管编号"}), 404

    if pipe["quantity"] < quantity:
        return jsonify({"error": "库存不足", "current": pipe["quantity"]}), 400

    db.execute("UPDATE pipes SET quantity = quantity - ?, last_update = ? WHERE pipe_id = ?",
               (quantity, now, pipe_id))
    db.execute('''
        INSERT INTO inventory_records (pipe_id, operation_type, quantity, operation_date, operator, remarks)
        VALUES (?, ?, ?, ?, ?, ?)
    ''', (pipe_id, "出库", quantity, now, operator, remarks))
    db.commit()
    return jsonify({"status": "success"})

@app.route("/records", methods=["GET"])
def records():
    db = get_db()
    params = {}
    if request.args.get('pipe_id'):
        params['pipe_id'] = request.args.get('pipe_id')
    if request.args.get('operation_type'):
        params['operation_type'] = request.args.get('operation_type')

    query = "SELECT * FROM inventory_records WHERE 1=1"
    args = []
    if params.get('pipe_id'):
        query += " AND pipe_id = ?"
        args.append(params['pipe_id'])
    if params.get('operation_type'):
        query += " AND operation_type = ?"
        args.append(params['operation_type'])
    query += " ORDER BY operation_date DESC"

    cur = db.execute(query, tuple(args))
    rows = [dict_from_row(r) for r in cur.fetchall()]
    return jsonify(rows)

@app.route("/statistics", methods=["GET"])
def statistics():
    db = get_db()
    stats = {}

    stats["total_types"] = db.execute("SELECT COUNT(*) FROM pipes").fetchone()[0]
    stats["total_quantity"] = db.execute("SELECT COALESCE(SUM(quantity),0) FROM pipes").fetchone()[0]
    stats["total_in"] = db.execute("SELECT COALESCE(SUM(quantity),0) FROM inventory_records WHERE operation_type='入库'").fetchone()[0]
    stats["total_out"] = db.execute("SELECT COALESCE(SUM(quantity),0) FROM inventory_records WHERE operation_type='出库'").fetchone()[0]

    return jsonify(stats)

if __name__ == "__main__":
    app.run(host="0.0.0.0", port=5000, debug=True)
