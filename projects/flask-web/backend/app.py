import io
import csv
from flask import Flask, request, jsonify, g, Response
from flask_cors import CORS
import sqlite3
import os

DB_PATH = os.path.join(os.path.dirname(__file__), 'pipes.db')

def get_db():
    if 'db' not in g:
        g.db = sqlite3.connect(DB_PATH)
        g.db.row_factory = sqlite3.Row
    return g.db

def init_db():
    db = get_db()
    db.execute('''
        CREATE TABLE IF NOT EXISTS pipes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            diameter REAL,
            thickness REAL,
            length REAL,
            material TEXT,
            quantity INTEGER
        )
    ''')
    db.commit()

def dict_from_row(row):
    return {k: row[k] for k in row.keys()}

app = Flask(__name__, static_folder=os.path.join(os.path.dirname(__file__), '../frontend'), static_url_path="")
CORS(app)

@app.before_first_request
def setup():
    os.makedirs(os.path.dirname(DB_PATH), exist_ok=True)
    if not os.path.exists(DB_PATH):
        open(DB_PATH, 'a').close()
    init_db()

@app.teardown_appcontext
def close_db(exc):
    db = g.pop('db', None)
    if db is not None:
        db.close()

@app.route("/")
def index():
    return app.send_static_file('index.html')

def query_pipes(params):
    db = get_db()
    sql = ["SELECT * FROM pipes WHERE 1=1"]
    args = []
    if params.get('material'):
        sql.append("AND material = ?")
        args.append(params['material'])
    if params.get('min_diameter') is not None:
        sql.append("AND diameter >= ?")
        args.append(params['min_diameter'])
    if params.get('max_diameter') is not None:
        sql.append("AND diameter <= ?")
        args.append(params['max_diameter'])
    if params.get('min_length') is not None:
        sql.append("AND length >= ?")
        args.append(params['min_length'])
    if params.get('max_length') is not None:
        sql.append("AND length <= ?")
        args.append(params['max_length'])
    if params.get('q'):
        sql.append("AND (material LIKE ?)")
        args.append(f"%{params['q']}%")
    sort_by = params.get('sort_by')
    if sort_by not in {"id","diameter","thickness","length","material","quantity"}:
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
    if params.get('min_diameter') is not None:
        count_sql += "AND diameter >= ?"
        count_args.append(params['min_diameter'])
    if params.get('max_diameter') is not None:
        count_sql += "AND diameter <= ?"
        count_args.append(params['max_diameter'])
    if params.get('min_length') is not None:
        count_sql += "AND length >= ?"
        count_args.append(params['min_length'])
    if params.get('max_length') is not None:
        count_sql += "AND length <= ?"
        count_args.append(params['max_length'])
    if params.get('q'):
        count_sql += "AND (material LIKE ?)"
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
        diameter = data.get("diameter")
        thickness = data.get("thickness")
        length = data.get("length")
        material = data.get("material")
        quantity = data.get("quantity")
        errors = {}
        if any(v is None for v in [diameter, thickness, length, material, quantity]):
            if diameter is None: errors['diameter'] = 'required'
            if thickness is None: errors['thickness'] = 'required'
            if length is None: errors['length'] = 'required'
            if material is None: errors['material'] = 'required'
            if quantity is None: errors['quantity'] = 'required'
            return jsonify({'errors': errors}), 400
        db = get_db()
        db.execute("INSERT INTO pipes (diameter, thickness, length, material, quantity) VALUES (?,?,?,?,?)",
                   (diameter, thickness, length, material, quantity))
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

if __name__ == "__main__":
    app.run(host="0.0.0.0", port=5000, debug=True)
