from flask import Flask, request, jsonify, g
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
    db.execute(
        '''
        CREATE TABLE IF NOT EXISTS pipes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            diameter REAL,
            thickness REAL,
            length REAL,
            material TEXT,
            quantity INTEGER
        )
        '''
    )
    db.commit()

def dict_from_row(row):
    return {k: row[k] for k in row.keys()}

app = Flask(__name__, static_folder=os.path.join(os.path.dirname(__file__), '../frontend'), static_url_path="")

@app.before_first_request
def setup():
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

@app.route("/pipes", methods=["GET","POST"])
def pipes():
    db = get_db()
    if request.method == 'GET':
        cur = db.execute("SELECT * FROM pipes")
        rows = [dict_from_row(r) for r in cur.fetchall()]
        return jsonify(rows)
    else:
        data = request.json or {}
        diameter = data.get("diameter")
        thickness = data.get("thickness")
        length = data.get("length")
        material = data.get("material")
        quantity = data.get("quantity")
        if any(v is None for v in [diameter, thickness, length, material, quantity]):
            return jsonify({"error": "missing fields"}), 400
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

if __name__ == "__main__":
    app.run(host="0.0.0.0", port=5000, debug=True)
