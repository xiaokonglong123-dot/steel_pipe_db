#!/usr/bin/env python3
"""Generate realistic API 5CT test data and seed into steel_pipe.db."""

import sqlite3
import random
import uuid
from datetime import datetime, timedelta
from pathlib import Path

DB_PATH = str(Path(__file__).parent / "data" / "steel_pipe.db")
ADMIN_HASH = "$argon2id$v=19$m=19456,t=2,p=1$9F0inGzvTDuczkTmmoEwXg$uML7yfMlcbkW3Y/cQgm04ax2Rt+Yg+YaplX24lg2WS4"

random.seed(42)

def dt(days_ago, base=datetime.now()):
    return (base - timedelta(days=days_ago)).strftime("%Y-%m-%d %H:%M:%S")

def date(days_ago, base=datetime.now()):
    return (base - timedelta(days=days_ago)).strftime("%Y-%m-%d")

conn = sqlite3.connect(DB_PATH)
conn.execute("PRAGMA foreign_keys = OFF")
cur = conn.cursor()

print("=== Updating admin password ===")
cur.execute("UPDATE users SET password_hash = ?, updated_at = datetime('now') WHERE username = 'admin'", (ADMIN_HASH,))
print(f"  Admin user password hash updated.")

# ========== SUPPLIERS ==========
print("=== Creating Suppliers ===")
suppliers_data = [
    ("SUP-001", "天津渤海钢管集团有限公司", "张卫国", "022-5888-1001", "zhangwg@bhpipe.cn", "天津市东丽区津塘公路396号"),
    ("SUP-002", "衡阳华菱钢管有限公司", "李国强", "0734-8222-1002", "ligq@hlpipe.cn", "湖南省衡阳市蒸湘区大栗新村10号"),
    ("SUP-003", "宝钢股份钢管条钢事业部", "王建国", "021-2664-1003", "wangjg@baosteel.com", "上海市宝山区富锦路885号"),
    ("SUP-004", "包钢集团无缝钢管有限公司", "赵铁柱", "0472-2188-1004", "zhaotz@btsteel.com", "内蒙古包头市昆都仑区河西工业区"),
    ("SUP-005", "烟台鲁宝钢管有限公司", "刘宝山", "0535-6377-1005", "liubs@lubao.cn", "山东省烟台市福山区高新路67号"),
]
for code, name, contact, phone, email, addr in suppliers_data:
    cur.execute(
        "INSERT OR IGNORE INTO suppliers (supplier_code, name, contact_person, phone, email, address, is_active, created_at) VALUES (?,?,?,?,?,?,1, datetime('now'))",
        (code, name, contact, phone, email, addr),
    )
    print(f"  Supplier: {name}")

# ========== CUSTOMERS ==========
print("=== Creating Customers ===")
customers_data = [
    ("CUS-001", "中国石油天然气集团公司", "陈爱国", "010-5998-2001", "chenag@cnpc.com.cn", "北京市东城区东直门北大街9号"),
    ("CUS-002", "中国石化胜利油田有限公司", "孙为民", "0546-8555-2002", "sunwm@sinopec.com", "山东省东营市济南路258号"),
    ("CUS-003", "中海石油(中国)有限公司", "黄海明", "010-8452-2003", "huanghm@cnooc.com.cn", "北京市东城区朝阳门北大街25号"),
    ("CUS-004", "陕西延长石油(集团)有限责任公司", "马志强", "029-8888-2004", "mazq@ycpc.com", "陕西省延安市宝塔区枣园路"),
    ("CUS-005", "中石油川庆钻探工程有限公司", "周建军", "028-8600-2005", "zhoujj@cqde.com", "四川省成都市成华区猛追湾街6号"),
]
for code, name, contact, phone, email, addr in customers_data:
    cur.execute(
        "INSERT OR IGNORE INTO customers (customer_code, name, contact_person, phone, email, address, is_active, created_at) VALUES (?,?,?,?,?,?,1, datetime('now'))",
        (code, name, contact, phone, email, addr),
    )
    print(f"  Customer: {name}")

# ========== LOCATIONS ==========
print("=== Creating Locations ===")
location_data = []
zone_names = {"A": "主仓库A区", "B": "主仓库B区", "C": "室外堆场C区"}
for zone in ("A", "B", "C"):
    for shelf in range(1, 4):
        for level in range(1, 4):
            full = f"{zone}-{shelf:02d}-{level:02d}"
            desc = f"{zone_names[zone]}, 货架{shelf:02d}, 第{level}层"
            cap = random.randint(30, 60)
            location_data.append((zone, str(shelf), str(level), full, desc, cap, 0))
for zone, shelf, level, full, desc, cap, used in location_data:
    cur.execute(
        "INSERT OR IGNORE INTO locations (zone_code, shelf_code, level_code, full_code, description, capacity, used_count, is_active) VALUES (?,?,?,?,?,?,?,1)",
        (zone, shelf, level, full, desc, cap, used),
    )
print(f"  Created {len(location_data)} locations")

# ========== SEAMLESS PIPES ==========
print("=== Creating Seamless Pipes ===")
# API 5CT pipe configurations
pipe_configs = [
    # (grade, pipe_type, od, wt, end_type, coupling_type, coupling_od, coupling_length, manufacturer_idx)
    ("J55", "casing", 114.3, 6.35, "STC", "BC", 127.0, 254.0, 0),
    ("J55", "casing", 139.7, 7.72, "STC", "BC", 153.67, 269.88, 0),
    ("K55", "casing", 177.8, 8.05, "LTC", "BC", 194.46, 285.75, 1),
    ("K55", "casing", 244.5, 8.94, "LTC", "BC", 265.1, 292.1, 1),
    ("N80", "casing", 177.8, 9.19, "BTC", "BC", 194.46, 285.75, 2),
    ("N80", "casing", 244.5, 10.03, "BTC", "BC", 265.1, 292.1, 2),
    ("L80", "casing", 139.7, 9.17, "STC", "BC", 153.67, 269.88, 3),
    ("L80", "casing", 177.8, 10.36, "LTC", "BC", 194.46, 285.75, 3),
    ("C90", "casing", 244.5, 11.05, "BTC", "BC", 265.1, 292.1, 4),
    ("C90", "casing", 339.7, 12.19, "BTC", "BC", 365.1, 304.8, 4),
    ("P110", "casing", 177.8, 11.51, "BTC", "BC", 194.46, 285.75, 0),
    ("Q125", "casing", 244.5, 13.84, "BTC", "BC", 265.1, 292.1, 1),
    ("J55", "tubing", 73.0, 5.51, "NU", "UB", 82.55, 177.8, 2),
    ("J55", "tubing", 88.9, 6.45, "NU", "UB", 101.6, 196.85, 2),
    ("N80", "tubing", 73.0, 7.01, "EU", "UB", 82.55, 177.8, 3),
    ("N80", "tubing", 88.9, 7.82, "EU", "UB", 101.6, 196.85, 3),
    ("L80", "tubing", 60.3, 4.83, "NU", "UB", 69.85, 165.1, 4),
    ("P110", "tubing", 73.0, 7.82, "EU", "UB", 82.55, 177.8, 0),
]

manu_names = ["天津渤海钢管", "衡阳华菱钢管", "宝钢股份", "包钢集团", "烟台鲁宝"]
loc_ids = [r[0] for r in cur.execute("SELECT id FROM locations WHERE is_active = 1").fetchall()]

pipe_count = 0
for grade, ptype, od, wt, end_type, coup_type, coup_od, coup_len, manu_idx in pipe_configs:
    num_pipes = random.randint(1, 3)  # 1-3 pipes of each spec
    for i in range(num_pipes):
        pipe_count += 1
        pipe_no = f"SP-{date(180).replace('-','')}-{pipe_count:04d}"
        batch_no = f"B{date(90).replace('-','')}-{random.randint(100,999)}"
        heat_no = f"H{random.randint(100000,999999)}"
        serial = f"S{random.randint(1000000,9999999)}"
        mfg_date = date(random.randint(30, 365))
        length = round(random.uniform(9.0, 12.5), 2)
        weight = round(3.14159 * (od - wt) * wt * 7.85 / 1000 * length, 2)  # approximate kg
        loc = random.choice(loc_ids)
        cert_no = f"QC-SP-{pipe_count:06d}"

        cur.execute("""
            INSERT INTO seamless_pipes
                (pipe_number, batch_number, pipe_type, grade, od, wt, length, weight_per_unit,
                 end_type, coupling_type, coupling_od, coupling_length,
                 heat_number, serial_number, manufacturer, production_date, cert_number,
                 location_id, status, notes, created_at)
            VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?, 'in_stock', ?, datetime('now'))
        """, (
            pipe_no, batch_no, ptype, grade, od, wt, length, weight,
            end_type, coup_type, coup_od, coup_len,
            heat_no, serial, manu_names[manu_idx], mfg_date, cert_no,
            loc, f"标准长度 {length}m, API 5CT {grade} {ptype}, 壁厚 {wt}mm"
        ))

print(f"  Created {pipe_count} seamless pipes")

# ========== SCREEN PIPES ==========
print("=== Creating Screen Pipes ===")
screen_configs = [
    ("wire_wrapped", 0.15, "FINE", "N80", 73.0, 5.51, "NU"),
    ("wire_wrapped", 0.20, "STANDARD", "N80", 88.9, 6.45, "NU"),
    ("slotted", 0.30, "COARSE", "J55", 114.3, 6.35, "STC"),
    ("slotted", 0.25, "STANDARD", "L80", 139.7, 7.72, "LTC"),
    ("punched", 0.50, "COARSE", "K55", 177.8, 8.05, "LTC"),
    ("metal_felt", 0.05, "EXTRA_FINE", "P110", 73.0, 7.82, "EU"),
    ("wire_wrapped", 0.15, "STANDARD", "L80", 88.9, 6.45, "EU"),
    ("slotted", 0.30, "STANDARD", "N80", 139.7, 7.72, "STC"),
    ("punched", 0.40, "COARSE", "J55", 168.3, 8.94, "LTC"),
    ("metal_felt", 0.08, "FINE", "C90", 88.9, 6.45, "EU"),
]
screen_count = 0
for sctype, slot, filt_grade, base_grade, od, wt, end_type in screen_configs:
    screen_count += 1
    pipe_no = f"SCR-{date(180).replace('-','')}-{screen_count:04d}"
    batch_no = f"SB{date(90).replace('-','')}-{random.randint(100,999)}"
    heat_no = f"H{random.randint(100000,999999)}"
    serial = f"S{random.randint(1000000,9999999)}"
    mfg_date = date(random.randint(30, 365))
    length = round(random.uniform(6.0, 11.5), 2)
    weight = round(3.14159 * (od - wt) * wt * 7.85 / 1000 * length, 2)
    loc = random.choice(loc_ids)
    cert_no = f"QC-SCR-{screen_count:06d}"

    cur.execute("""
        INSERT INTO screen_pipes
            (pipe_number, batch_number, screen_type, slot_size, filtration_grade,
             base_od, base_wt, base_grade, base_end_type,
             length, weight_per_unit, heat_number, serial_number,
             manufacturer, production_date, cert_number,
             location_id, status, notes, created_at)
        VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?, 'in_stock', ?, datetime('now'))
    """, (
        pipe_no, batch_no, sctype, slot, filt_grade,
        od, wt, base_grade, end_type,
        length, weight, heat_no, serial,
        random.choice(manu_names), mfg_date, cert_no,
        loc, f"{sctype} 筛管, {filt_grade} 过滤等级, 缝宽{slot}mm, 长度{length}m"
    ))

print(f"  Created {screen_count} screen pipes")

# ========== CONTRACTS ==========
print("=== Creating Contracts ===")
contracts_data = [
    ("purchase", "2025年渤海钢管采购框架合同", "天津渤海钢管集团有限公司", "SteelPipeDB库存管理公司", "draft", 5200000),
    ("purchase", "2025年华菱钢管年度采购合同", "衡阳华菱钢管有限公司", "SteelPipeDB库存管理公司", "active", 3800000),
    ("purchase", "2025年宝钢钢管采购合同", "宝钢股份钢管条钢事业部", "SteelPipeDB库存管理公司", "active", 7600000),
    ("sales", "CNPC年度油套管供应合同", "SteelPipeDB库存管理公司", "中国石油天然气集团公司", "active", 12000000),
    ("sales", "胜利油田筛管供应框架协议", "SteelPipeDB库存管理公司", "中国石化胜利油田有限公司", "active", 8500000),
    ("sales", "延长石油套管采购合同", "SteelPipeDB库存管理公司", "陕西延长石油(集团)有限责任公司", "completed", 4200000),
]
contract_ids = []
for ctype, title, party_a, party_b, status, amount in contracts_data:
    contract_no = f"CT-{random.randint(10000000,99999999)}"
    sign_date = date(random.randint(30, 180))
    start_date = date(random.randint(15, 180))
    end_date = date(-random.randint(60, 365))  # future date
    cur.execute("""
        INSERT INTO contracts
            (contract_no, contract_type, title, party_a, party_b,
             sign_date, start_date, end_date, total_amount, status, notes, created_by, created_at)
        VALUES (?,?,?,?,?,?,?,?,?,?,?,1, datetime('now'))
    """, (contract_no, ctype, title, party_a, party_b,
          sign_date, start_date, end_date, amount, status,
          f"{title} - 付款方式: 预付30%, 到货付60%, 质保金10%"))
    cid = cur.lastrowid
    contract_ids.append(cid)
    print(f"  Contract {contract_no}: {title}")

# Contract items
print("  Adding contract items...")
for i, cid in enumerate(contract_ids):
    is_purchase = i < 3
    num_items = random.randint(1, 3)
    for j in range(num_items):
        if is_purchase:
            grades = ["J55", "K55", "N80", "L80", "P110"]
        else:
            grades = ["N80", "L80", "C90", "P110", "Q125"]
        grade = random.choice(grades)
        pipe_type = random.choice(["casing", "tubing"])
        od = random.choice([73.0, 88.9, 114.3, 139.7, 177.8])
        wt = round(random.uniform(5.0, 13.0), 2)
        qty = random.randint(50, 300)
        unit_price = round(random.uniform(500, 2000), 2)
        total_price = round(qty * unit_price, 2)
        cur.execute("""
            INSERT INTO contract_items (contract_id, pipe_type, grade, od, wt, quantity, unit_price, total_price, notes)
            VALUES (?,?,?,?,?,?,?,?,?)
        """, (cid, pipe_type, grade, od, wt, qty, unit_price, total_price, f"第{j+1}批交付"))

# Contract payments
print("  Adding contract payment schedules...")
for cid in contract_ids:
    # 30% deposit
    cur.execute("""
        INSERT INTO contract_payments (contract_id, due_date, amount, payment_type, is_paid, paid_date, notes)
        VALUES (?,?,?,'deposit',1,?,?)
    """, (cid, date(random.randint(15, 30)),
          round(random.uniform(500000, 2000000), 2),
          date(random.randint(5, 20)), "预付款已支付"))
    # 60% milestone
    cur.execute("""
        INSERT INTO contract_payments (contract_id, due_date, amount, payment_type, is_paid, paid_date, notes)
        VALUES (?,?,?,'milestone',?,?,?)
    """, (cid, date(-random.randint(30, 90)),
          round(random.uniform(1000000, 5000000), 2),
          random.choice([1, 0]),
          date(random.randint(1, 10)) if random.choice([True, False]) else None,
          "到货款"))
    # 10% final
    cur.execute("""
        INSERT INTO contract_payments (contract_id, due_date, amount, payment_type, is_paid, paid_date, notes)
        VALUES (?,?,?,'final',0,?,?)
    """, (cid, date(-random.randint(365, 730)),
          round(random.uniform(100000, 1000000), 2),
          None, "质保金 - 到期货款"))

print(f"  Added contract items and payments")

# ========== PURCHASE ORDERS ==========
print("=== Creating Purchase Orders ===")
supplier_ids = [r[0] for r in cur.execute("SELECT id FROM suppliers").fetchall()]
po_data = [
    (supplier_ids[0], "pending", "渤海钢管 - N80套管采购", 1200000),
    (supplier_ids[1], "approved", "华菱钢管 - J55油管采购", 850000),
    (supplier_ids[2], "completed", "宝钢 - 高强度套管采购", 2300000),
]
po_ids = []
for sid, status, notes, amount in po_data:
    order_no = f"PO-{random.randint(10000000,99999999)}"
    ord_date = date(random.randint(15, 180))
    cur.execute("""
        INSERT INTO purchase_orders (order_no, supplier_id, order_date, status, total_amount, notes, created_by, created_at)
        VALUES (?,?,?,?,?,?,1, datetime('now'))
    """, (order_no, sid, ord_date, status, amount, notes))
    oid = cur.lastrowid
    po_ids.append(oid)
    print(f"  PO {order_no}: {notes}")

    # PO items
    num_items = random.randint(2, 4)
    for j in range(num_items):
        grade = random.choice(["J55", "K55", "N80", "L80", "P110"])
        ptype = random.choice(["casing", "tubing"])
        od = random.choice([73.0, 88.9, 114.3, 139.7, 177.8])
        wt = round(random.uniform(5.0, 13.0), 2)
        qty = random.randint(30, 200)
        unit_price = round(random.uniform(800, 2500), 2)
        total_price = round(qty * unit_price, 2)
        received = qty if status == "completed" else random.randint(0, qty)
        cur.execute("""
            INSERT INTO purchase_order_items (order_id, pipe_type, grade, od, wt, quantity, received_quantity, unit_price, total_price, notes)
            VALUES (?,?,?,?,?,?,?,?,?,?)
        """, (oid, ptype, grade, od, wt, qty, received, unit_price, total_price, f"第{j+1}项"))

# ========== INBOUND RECORDS ==========
print("=== Creating Inbound Records ===")
# For the "completed" PO, create inbound record
completed_po_id = po_ids[2]  # the completed one
cur.execute("SELECT id FROM purchase_order_items WHERE order_id = ?", (completed_po_id,))
po_items = cur.fetchall()

inbound_no = f"IN-{date(30).replace('-','')}-{random.randint(1000,9999)}"
cur.execute("""
    INSERT INTO inbound_records (inbound_no, inbound_type, order_id, supplier_id, notes, approval_status, handled_by, handled_at, created_at)
    VALUES (?, 'purchase', ?, ?, ?, 'approved', 1, datetime('now', '-15 days'), datetime('now', '-30 days'))
""", (inbound_no, completed_po_id, supplier_ids[2], "宝钢钢管到货验收"))

inbound_id = cur.lastrowid
print(f"  Inbound {inbound_no}: from 宝钢")

# Link pipes from this supplier's spec to inbound items
sp_ids = cur.execute("SELECT id FROM seamless_pipes WHERE location_id IS NOT NULL LIMIT 20").fetchall()
for sp_id in sp_ids[:8]:
    cur.execute("INSERT INTO inbound_items (inbound_id, pipe_type, pipe_id) VALUES (?, 'seamless', ?)",
                (inbound_id, sp_id[0]))

# Inventory logs for inbound
for sp_id in sp_ids[:8]:
    cur.execute("""INSERT INTO inventory_logs (pipe_type, pipe_id, change_type, ref_type, ref_id, to_location_id, notes, created_by, created_at)
                   VALUES ('seamless', ?, 'inbound', 'inbound', ?, ?, '采购入库', 1, datetime('now', '-30 days'))""",
               (sp_id[0], inbound_id, random.choice(loc_ids)))

# ========== SALES ORDERS ==========
print("=== Creating Sales Orders ===")
customer_ids = [r[0] for r in cur.execute("SELECT id FROM customers").fetchall()]
so_data = [
    (customer_ids[0], "approved", "CNPC N80套管销售单", 1800000),
    (customer_ids[1], "pending", "胜利油田筛管销售单", 950000),
    (customer_ids[3], "completed", "延长石油 J55油管销售单", 1200000),
]
so_ids = []
for cid, status, notes, amount in so_data:
    order_no = f"SO-{random.randint(10000000,99999999)}"
    ord_date = date(random.randint(10, 90))
    cur.execute("""
        INSERT INTO sales_orders (order_no, customer_id, order_date, status, total_amount, notes, created_by, created_at)
        VALUES (?,?,?,?,?,?,1, datetime('now'))
    """, (order_no, cid, ord_date, status, amount, notes))
    oid = cur.lastrowid
    so_ids.append(oid)
    print(f"  SO {order_no}: {notes}")

    # SO items
    num_items = random.randint(1, 3)
    for j in range(num_items):
        grade = random.choice(["N80", "L80", "C90", "P110"])
        ptype = random.choice(["casing", "tubing"])
        od = random.choice([73.0, 88.9, 114.3, 139.7])
        wt = round(random.uniform(5.0, 12.0), 2)
        qty = random.randint(10, 100)
        unit_price = round(random.uniform(1200, 3500), 2)
        total_price = round(qty * unit_price, 2)
        delivered = qty if status == "completed" else random.randint(0, qty)
        cur.execute("""
            INSERT INTO sales_order_items (order_id, pipe_type, grade, od, wt, quantity, delivered_quantity, unit_price, total_price, notes)
            VALUES (?,?,?,?,?,?,?,?,?,?)
        """, (oid, ptype, grade, od, wt, qty, delivered, unit_price, total_price, f"第{j+1}项"))

# ========== OUTBOUND RECORDS for completed SO ==========
completed_so_id = so_ids[2]  # completed one
outbound_no = f"OUT-{date(15).replace('-','')}-{random.randint(1000,9999)}"
cur.execute("""
    INSERT INTO outbound_records (outbound_no, outbound_type, order_id, customer_id, notes, approval_status, handled_by, handled_at, created_at)
    VALUES (?, 'sales', ?, ?, ?, 'approved', 1, datetime('now', '-5 days'), datetime('now', '-20 days'))
""", (outbound_no, completed_so_id, customer_ids[3], "延长石油提货 - 已完成"))

outbound_id = cur.lastrowid
print(f"  Outbound {outbound_no}: to 延长石油")

# Link some pipes to outbound
sp_for_out = cur.execute("SELECT id FROM seamless_pipes WHERE status = 'in_stock' LIMIT 10").fetchall()
for sp_id in sp_for_out[:5]:
    cur.execute("INSERT INTO outbound_items (outbound_id, pipe_type, pipe_id) VALUES (?, 'seamless', ?)",
                (outbound_id, sp_id[0]))
    cur.execute("UPDATE seamless_pipes SET status = 'outbound' WHERE id = ?", (sp_id[0],))
    cur.execute("""INSERT INTO inventory_logs (pipe_type, pipe_id, change_type, ref_type, ref_id, from_location_id, notes, created_by, created_at)
                   VALUES ('seamless', ?, 'outbound', 'outbound', ?, ?, '销售出库', 1, datetime('now', '-20 days'))""",
               (sp_id[0], outbound_id, random.choice(loc_ids)))

# ========== QUALITY CERTS ==========
print("=== Creating Quality Certificates ===")
# Create certs for some seamless pipes
sp_all = cur.execute("SELECT id, pipe_number, grade, pipe_type FROM seamless_pipes LIMIT 25").fetchall()
inspectors = ["张工", "李工", "王工", "刘工"]
bodies = ["天津钢管检验中心", "国家钢管质量监督检验中心", "SGS通标标准技术服务有限公司"]

for sp in sp_all[:15]:
    sp_id, pipe_no, grade, ptype = sp
    cert_no = f"QC-{date(90).replace('-','')}-{random.randint(1000,9999)}"
    result = random.choices(["pass", "pass", "pass", "pass", "fail"], weights=[40, 40, 10, 5, 5])[0]
    insp = random.choice(inspectors)
    body = random.choice(bodies)
    cert_date = date(random.randint(10, 90))
    notes = f"{grade} {ptype}, 热处理检验合格, 理化性能达标" if result == "pass" else f"{grade} {ptype}, 抗拉强度不合格, 需复验"
    cur.execute("""
        INSERT INTO quality_certs (cert_number, pipe_type, pipe_id, cert_date, result, inspector, inspection_body, notes, created_at)
        VALUES (?, 'seamless', ?, ?, ?, ?, ?, ?, datetime('now'))
    """, (cert_no, sp_id, cert_date, result, insp, body, notes))
print(f"  Created 15 quality certs for seamless pipes")

# ========== FINAL COUNTS ==========
conn.commit()
print("\n=== Seed Summary ===")
tables = [
    ("users", "SELECT COUNT(*) FROM users"),
    ("suppliers", "SELECT COUNT(*) FROM suppliers"),
    ("customers", "SELECT COUNT(*) FROM customers"),
    ("locations", "SELECT COUNT(*) FROM locations"),
    ("contracts", "SELECT COUNT(*) FROM contracts"),
    ("contract_items", "SELECT COUNT(*) FROM contract_items"),
    ("contract_payments", "SELECT COUNT(*) FROM contract_payments"),
    ("seamless_pipes", "SELECT COUNT(*) FROM seamless_pipes"),
    ("screen_pipes", "SELECT COUNT(*) FROM screen_pipes"),
    ("purchase_orders", "SELECT COUNT(*) FROM purchase_orders"),
    ("purchase_order_items", "SELECT COUNT(*) FROM purchase_order_items"),
    ("sales_orders", "SELECT COUNT(*) FROM sales_orders"),
    ("sales_order_items", "SELECT COUNT(*) FROM sales_order_items"),
    ("inbound_records", "SELECT COUNT(*) FROM inbound_records"),
    ("inbound_items", "SELECT COUNT(*) FROM inbound_items"),
    ("outbound_records", "SELECT COUNT(*) FROM outbound_records"),
    ("outbound_items", "SELECT COUNT(*) FROM outbound_items"),
    ("quality_certs", "SELECT COUNT(*) FROM quality_certs"),
    ("inventory_logs", "SELECT COUNT(*) FROM inventory_logs"),
]
for name, query in tables:
    count = cur.execute(query).fetchone()[0]
    print(f"  {name}: {count} rows")

conn.close()
print("\nSeed complete!")
