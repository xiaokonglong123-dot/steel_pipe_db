#!/usr/bin/env python3
"""
Enhanced seed data generator for Steel Pipe DB.
Adds substantial API 5CT-compliant random data on top of existing DB data.
Run after seed_data.py or on a fresh migrated database.

Usage: python3 backend/seed_data_enhanced.py
"""

import sqlite3
import random
import uuid
from datetime import datetime, timedelta
from pathlib import Path

DB_PATH = Path(__file__).parent / "data" / "steel_pipe.db"
random.seed(2024)

def dt(days_ago, base=None):
    base = base or datetime.now()
    return (base - timedelta(days=days_ago)).strftime("%Y-%m-%d %H:%M:%S")

def date(days_ago, base=None):
    base = base or datetime.now()
    return (base - timedelta(days=days_ago)).strftime("%Y-%m-%d")

def weighted_choice(options, weights):
    total = sum(weights)
    r = random.uniform(0, total)
    upto = 0
    for opt, w in zip(options, weights):
        if upto + w >= r:
            return opt
        upto += w
    return options[-1]

# ─── Real API 5CT Casing Specs ─────────────────────────────────
# (OD_mm, wt_mm, grade, end_type, coupling_type, coupling_od, coupling_length)
API_5CT_CASING_SPECS = [
    # 4-1/2" (114.3 mm)
    (114.3, 5.21, "J55", "STC", "BC", 127.0, 254.0),
    (114.3, 5.69, "K55", "STC", "BC", 127.0, 254.0),
    (114.3, 6.35, "J55", "STC", "BC", 127.0, 254.0),
    (114.3, 6.35, "N80", "LTC", "BC", 127.0, 269.88),
    (114.3, 7.37, "P110", "BTC", "BC", 127.0, 269.88),
    # 5-1/2" (139.7 mm)
    (139.7, 6.20, "J55", "STC", "BC", 153.67, 269.88),
    (139.7, 6.98, "K55", "LTC", "BC", 153.67, 285.75),
    (139.7, 7.72, "J55", "STC", "BC", 153.67, 269.88),
    (139.7, 7.72, "N80", "BTC", "BC", 153.67, 285.75),
    (139.7, 9.17, "L80", "LTC", "BC", 153.67, 285.75),
    (139.7, 9.17, "P110", "BTC", "BC", 153.67, 285.75),
    (139.7, 10.54, "C90", "BTC", "BC", 153.67, 285.75),
    # 7" (177.8 mm)
    (177.8, 6.91, "J55", "STC", "BC", 194.46, 285.75),
    (177.8, 8.05, "K55", "LTC", "BC", 194.46, 285.75),
    (177.8, 9.19, "N80", "BTC", "BC", 194.46, 285.75),
    (177.8, 10.36, "L80", "LTC", "BC", 194.46, 285.75),
    (177.8, 11.51, "P110", "BTC", "BC", 194.46, 285.75),
    (177.8, 12.65, "Q125", "BTC", "BC", 194.46, 285.75),
    # 9-5/8" (244.5 mm)
    (244.5, 7.92, "J55", "STC", "BC", 265.1, 292.1),
    (244.5, 8.94, "K55", "LTC", "BC", 265.1, 292.1),
    (244.5, 10.03, "N80", "BTC", "BC", 265.1, 292.1),
    (244.5, 11.05, "C90", "BTC", "BC", 265.1, 292.1),
    (244.5, 11.99, "L80", "BTC", "BC", 265.1, 292.1),
    (244.5, 13.84, "P110", "BTC", "BC", 265.1, 292.1),
    # 13-3/8" (339.7 mm)
    (339.7, 9.65, "J55", "STC", "BC", 365.1, 304.8),
    (339.7, 10.92, "K55", "LTC", "BC", 365.1, 304.8),
    (339.7, 12.19, "N80", "BTC", "BC", 365.1, 304.8),
]

# ─── Real API 5CT Tubing Specs ─────────────────────────────────
API_5CT_TUBING_SPECS = [
    # 2-3/8" (60.3 mm)
    (60.3, 4.83, "J55", "NU", "UB", 69.85, 165.1),
    (60.3, 4.83, "N80", "NU", "UB", 69.85, 165.1),
    (60.3, 5.51, "P110", "EU", "UB", 69.85, 177.8),
    # 2-7/8" (73.0 mm)
    (73.0, 5.51, "J55", "NU", "UB", 82.55, 177.8),
    (73.0, 5.51, "N80", "NU", "UB", 82.55, 177.8),
    (73.0, 7.01, "L80", "EU", "UB", 82.55, 177.8),
    (73.0, 7.82, "P110", "EU", "UB", 82.55, 177.8),
    # 3-1/2" (88.9 mm)
    (88.9, 6.45, "J55", "NU", "UB", 101.6, 196.85),
    (88.9, 6.45, "N80", "NU", "UB", 101.6, 196.85),
    (88.9, 7.82, "L80", "EU", "UB", 101.6, 196.85),
    (88.9, 7.82, "P110", "EU", "UB", 101.6, 196.85),
    (88.9, 9.52, "C90", "EU", "UB", 101.6, 196.85),
    # 4" (101.6 mm)
    (101.6, 5.74, "J55", "NU", "UB", 114.3, 203.2),
    (101.6, 6.88, "N80", "EU", "UB", 114.3, 203.2),
    (101.6, 8.38, "P110", "EU", "UB", 114.3, 203.2),
]

# ─── Screen pipe configs ──────────────────────────────────
SCREEN_SPECS = [
    ("wire_wrapped", 0.10, "EXTRA_FINE", "N80", 73.0, 5.51, "NU"),
    ("wire_wrapped", 0.15, "FINE", "N80", 88.9, 6.45, "NU"),
    ("wire_wrapped", 0.15, "FINE", "L80", 73.0, 7.01, "EU"),
    ("wire_wrapped", 0.20, "STANDARD", "N80", 114.3, 6.35, "STC"),
    ("wire_wrapped", 0.25, "STANDARD", "L80", 139.7, 7.72, "LTC"),
    ("slotted", 0.30, "COARSE", "J55", 139.7, 7.72, "LTC"),
    ("slotted", 0.30, "STANDARD", "N80", 177.8, 9.19, "BTC"),
    ("slotted", 0.40, "COARSE", "K55", 177.8, 8.05, "LTC"),
    ("slotted", 0.35, "STANDARD", "L80", 244.5, 10.03, "BTC"),
    ("punched", 0.40, "COARSE", "J55", 168.3, 8.94, "LTC"),
    ("punched", 0.50, "COARSE", "K55", 219.1, 8.94, "STC"),
    ("punched", 0.60, "STANDARD", "N80", 244.5, 8.94, "LTC"),
    ("metal_felt", 0.05, "EXTRA_FINE", "P110", 73.0, 7.82, "EU"),
    ("metal_felt", 0.08, "FINE", "C90", 88.9, 6.45, "EU"),
    ("metal_felt", 0.08, "FINE", "L80", 114.3, 6.35, "STC"),
]

# ─── More suppliers ───────────────────────────────────────
MORE_SUPPLIERS = [
    ("SUP-006", "江苏常宝钢管股份有限公司", "陈志强", "0519-8877-1006", "chenzq@changbao.com", "江苏省常州市天宁区延陵中路188号"),
    ("SUP-007", "浙江金洲管道科技股份有限公司", "周建国", "0572-2777-1007", "zhoujg@jinzhou.com", "浙江省湖州市凤凰路8号"),
    ("SUP-008", "河北普阳钢铁有限公司", "张建平", "0310-5177-1008", "zhangjp@pygt.com", "河北省武安市阳邑镇普阳大道1号"),
    ("SUP-009", "鞍钢集团无缝钢管厂", "刘国栋", "0412-6722-1009", "liugd@ansteel.com.cn", "辽宁省鞍山市铁西区鞍钢厂区"),
    ("SUP-010", "太原钢铁(集团)有限公司", "马志勇", "0351-2133-1010", "mazy@tisco.com.cn", "山西省太原市尖草坪区尖草坪街2号"),
]

MORE_CUSTOMERS = [
    ("CUS-006", "中石油长城钻探工程有限公司", "孙伟", "010-8486-2006", "sunw@gwdc.com.cn", "北京市朝阳区屏翠东路8号"),
    ("CUS-007", "中国石化中原油田分公司", "赵建设", "0393-4822-2007", "zhaojs@zyyt.com", "河南省濮阳市华龙区中原路277号"),
    ("CUS-008", "中海油田服务股份有限公司（油服）", "王海龙", "010-8452-2008", "wanghl@cosl.com.cn", "北京市东城区东直门外小街甲2号"),
    ("CUS-009", "中化石油勘探开发有限公司", "李国华", "010-5936-2009", "ligh@sinochem.com", "北京市西城区复兴门内大街28号"),
    ("CUS-010", "中石油塔里木油田分公司", "刘合", "0996-2172-2010", "liuh@tlm.cn", "新疆库尔勒市石化大道26号"),
]

MANUFACTURERS = [
    "天津渤海钢管", "衡阳华菱钢管", "宝钢股份", "包钢集团", "烟台鲁宝",
    "江苏常宝钢管", "浙江金洲管道", "河北普阳钢铁", "鞍钢集团", "太原钢铁",
]

INSPECTORS = ["张工", "李工", "王工", "刘工", "陈工", "杨工", "赵工", "周工"]
INSPECTION_BODIES = [
    "天津钢管检验中心",
    "国家钢管质量监督检验中心",
    "SGS通标标准技术服务有限公司",
    "中国石油管材研究院检验中心",
    "西安摩尔石油工程实验室",
    "BV必维国际检验集团",
]

PURCHASE_NOTES_POOL = [
    "{}套管框架采购合同执行",
    "{}油管年度采购订单",
    "{}高强度套管批量采购",
    "{}合金管材紧急采购",
    "{}API 5CT认证钢管采购",
    "{}石油管材季度采购计划",
    "{}特殊扣套管采购",
    "{}抗硫管材批量采购",
]

SALES_NOTES_POOL = [
    "{}套管销售订单",
    "{}油管供应合同执行",
    "{}油田钻采用管订单",
    "{}管材配套供应合同",
    "{}API 5CT认证成品管销售",
    "{}项目用管紧急订单",
]

CONTRACT_TITLES_POOL_PURCHASE = [
    "{}年度采购框架合同",
    "{}钢管供应合作协议",
    "{}API 5CT管材专项采购合同",
    "{}石油套管长期供货协议",
    "{}年度钢管采购合同",
]

CONTRACT_TITLES_POOL_SALES = [
    "{}年度套管供应框架协议",
    "{}油管批量供货合同",
    "{}钻采用管项目供货合同",
    "{}API 5CT管材出口销售合同",
    "{}常规管材供应合同",
]


def main():
    conn = sqlite3.connect(str(DB_PATH))
    conn.execute("PRAGMA foreign_keys = OFF")
    cur = conn.cursor()

    # ─── Determine existing data ranges ───
    def max_id(table):
        row = cur.execute(f"SELECT COALESCE(MAX(id), 0) FROM {table}").fetchone()
        return row[0]

    print(f"Connected to: {DB_PATH}")
    print(f"Existing data: users={max_id('users')}, suppliers={max_id('suppliers')}, "
          f"customers={max_id('customers')}, seamless_pipes={max_id('seamless_pipes')}, "
          f"screen_pipes={max_id('screen_pipes')}, locations={max_id('locations')}")

    print("\n=== Adding More Suppliers ===")
    for code, name, contact, phone, email, addr in MORE_SUPPLIERS:
        cur.execute(
            "INSERT OR IGNORE INTO suppliers (supplier_code, name, contact_person, phone, email, address, is_active, notes, created_at) "
            "VALUES (?,?,?,?,?,?,1,?, datetime('now'))",
            (code, name, contact, phone, email, addr, f"资质齐全，可供应API 5CT {random.choice(['J55-K55','N80-L80','C90-P110','各类钢级'])}管材"),
        )
        if cur.rowcount > 0:
            print(f"  + Supplier: {name}")
        else:
            print(f"  • Skipped (exists): {name}")

    # ─── 2. ADD MORE CUSTOMERS ───
    print("\n=== Adding More Customers ===")
    for code, name, contact, phone, email, addr in MORE_CUSTOMERS:
        cur.execute(
            "INSERT OR IGNORE INTO customers (customer_code, name, contact_person, phone, email, address, is_active, notes, created_at) "
            "VALUES (?,?,?,?,?,?,1,?, datetime('now'))",
            (code, name, contact, phone, email, addr, f"大型油企，长期合作客户"),
        )
        if cur.rowcount > 0:
            print(f"  + Customer: {name}")
        else:
            print(f"  • Skipped (exists): {name}")

    # ─── 3. ADD MORE LOCATIONS ───
    print("\n=== Adding More Locations ===")
    existing_locs = set(r[0] for r in cur.execute("SELECT full_code FROM locations").fetchall())
    zone_names = {"D": "精密管材库D区", "E": "出口专用堆场E区", "F": "待检区F区"}
    new_locs = []
    for zone in ("D", "E", "F"):
        for shelf in range(1, 4):
            for level in range(1, 3):
                full = f"{zone}-{shelf:02d}-{level:02d}"
                if full in existing_locs:
                    continue
                desc = f"{zone_names[zone]}, 货架{shelf:02d}, 第{level}层"
                cap = random.randint(40, 80)
                new_locs.append((zone, str(shelf), str(level), full, desc, cap, 0))
    for zone, shelf, level, full, desc, cap, used in new_locs:
        cur.execute(
            "INSERT OR IGNORE INTO locations (zone_code, shelf_code, level_code, full_code, description, capacity, used_count, is_active) "
            "VALUES (?,?,?,?,?,?,?,1)",
            (zone, shelf, level, full, desc, cap, used),
        )
    print(f"  + Added {len(new_locs)} new locations (D/E/F zones)")

    # ─── Helper: get all location IDs ───
    loc_ids = [r[0] for r in cur.execute("SELECT id FROM locations WHERE is_active = 1").fetchall()]
    sup_ids = [r[0] for r in cur.execute("SELECT id FROM suppliers").fetchall()]
    cust_ids = [r[0] for r in cur.execute("SELECT id FROM customers").fetchall()]

    # ─── 4a. ADD MORE SEAMLESS PIPES (Casing) ───
    print("\n=== Adding More Seamless Pipes (Casing) ===")
    existing_pipe_numbers = set(r[0] for r in cur.execute("SELECT pipe_number FROM seamless_pipes").fetchall())
    next_id = max_id("seamless_pipes") + 1
    casing_count = 0

    for od, wt, grade, end_type, coup_type, coup_od, coup_len in API_5CT_CASING_SPECS:


        for _ in range(random.randint(2, 4)):
            base_date = datetime.now() - timedelta(days=random.randint(30, 365))
            pipe_no = f"SP-{base_date.strftime('%Y%m%d')}-{next_id:04d}"
            if pipe_no in existing_pipe_numbers:
                pipe_no = f"SP-{base_date.strftime('%Y%m%d')}-{next_id + random.randint(1000,9999):04d}"
            existing_pipe_numbers.add(pipe_no)
            next_id += 1

            batch_no = f"B{base_date.strftime('%Y%m%d')}-{random.randint(100,999)}"
            heat_no = f"H{random.randint(100000,999999)}"
            serial = f"S{random.randint(1000000,9999999)}"
            mfg_date = date(random.randint(30, 365))
            length = round(random.uniform(9.5, 12.5), 2)
            # Weight ≈ π*(OD-WT)*WT*7.85/1000 * length (kg)
            weight = round(3.14159 * (od - wt) * wt * 7.85 / 1000 * length, 2)
            loc = random.choice(loc_ids)
            cert_no = f"QC-SP-{next_id:06d}"
            manufacturer = random.choice(MANUFACTURERS)
            status = random.choices(["in_stock", "in_stock", "in_stock", "outbound", "scrapped"],
                                    weights=[60, 20, 10, 8, 2])[0]

            cur.execute("""
                INSERT INTO seamless_pipes
                    (pipe_number, batch_number, pipe_type, grade, od, wt, length, weight_per_unit,
                     end_type, coupling_type, coupling_od, coupling_length,
                     heat_number, serial_number, manufacturer, production_date, cert_number,
                     location_id, status, notes, created_at)
                VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,datetime('now'))
            """, (
                pipe_no, batch_no, "casing", grade, od, wt, length, weight,
                end_type, coup_type, coup_od, coup_len,
                heat_no, serial, manufacturer, mfg_date, cert_no,
                loc, status,
                f"API 5CT {grade} casing {od}×{wt}mm, R2 length {length}m"
            ))
            casing_count += 1

    print(f"  + Added {casing_count} casing pipes")

    # ─── 4b. ADD MORE SEAMLESS PIPES (Tubing) ───
    print("=== Adding More Seamless Pipes (Tubing) ===")
    tubing_count = 0
    for od, wt, grade, end_type, coup_type, coup_od, coup_len in API_5CT_TUBING_SPECS:
        for _ in range(random.randint(2, 4)):
            base_date = datetime.now() - timedelta(days=random.randint(30, 365))
            pipe_no = f"SP-{base_date.strftime('%Y%m%d')}-{next_id:04d}"
            if pipe_no in existing_pipe_numbers:
                pipe_no = f"SP-{base_date.strftime('%Y%m%d')}-{next_id + random.randint(1000,9999):04d}"
            existing_pipe_numbers.add(pipe_no)
            next_id += 1

            batch_no = f"B{base_date.strftime('%Y%m%d')}-{random.randint(100,999)}"
            heat_no = f"H{random.randint(100000,999999)}"
            serial = f"S{random.randint(1000000,9999999)}"
            mfg_date = date(random.randint(30, 365))
            length = round(random.uniform(9.0, 12.0), 2)
            weight = round(3.14159 * (od - wt) * wt * 7.85 / 1000 * length, 2)
            loc = random.choice(loc_ids)
            cert_no = f"QC-SP-{next_id:06d}"
            manufacturer = random.choice(MANUFACTURERS)
            status = random.choices(["in_stock", "in_stock", "in_stock", "outbound", "scrapped"],
                                    weights=[60, 20, 10, 8, 2])[0]

            cur.execute("""
                INSERT INTO seamless_pipes
                    (pipe_number, batch_number, pipe_type, grade, od, wt, length, weight_per_unit,
                     end_type, coupling_type, coupling_od, coupling_length,
                     heat_number, serial_number, manufacturer, production_date, cert_number,
                     location_id, status, notes, created_at)
                VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,datetime('now'))
            """, (
                pipe_no, batch_no, "tubing", grade, od, wt, length, weight,
                end_type, coup_type, coup_od, coup_len,
                heat_no, serial, manufacturer, mfg_date, cert_no,
                loc, status,
                f"API 5CT {grade} tubing {od}×{wt}mm, R2 length {length}m"
            ))
            tubing_count += 1

    print(f"  + Added {tubing_count} tubing pipes")

    # ─── 4c. ADD MORE SCREEN PIPES ───
    print("=== Adding More Screen Pipes ===")
    existing_screen_nums = set(r[0] for r in cur.execute("SELECT pipe_number FROM screen_pipes").fetchall())
    next_screen_id = max_id("screen_pipes") + 1
    screen_count = 0
    for sctype, slot, filt_grade, base_grade, od, wt, end_type in SCREEN_SPECS:
        for _ in range(random.randint(1, 3)):
            base_date = datetime.now() - timedelta(days=random.randint(30, 365))
            pipe_no = f"SCR-{base_date.strftime('%Y%m%d')}-{next_screen_id:04d}"
            if pipe_no in existing_screen_nums:
                pipe_no = f"SCR-{base_date.strftime('%Y%m%d')}-{next_screen_id + random.randint(1000,9999):04d}"
            existing_screen_nums.add(pipe_no)
            next_screen_id += 1

            batch_no = f"SB{base_date.strftime('%Y%m%d')}-{random.randint(100,999)}"
            heat_no = f"H{random.randint(100000,999999)}"
            serial = f"S{random.randint(1000000,9999999)}"
            mfg_date = date(random.randint(30, 365))
            length = round(random.uniform(6.0, 11.5), 2)
            weight = round(3.14159 * (od - wt) * wt * 7.85 / 1000 * length, 2)
            loc = random.choice(loc_ids)
            cert_no = f"QC-SCR-{next_screen_id:06d}"
            manufacturer = random.choice(MANUFACTURERS)
            status = random.choices(["in_stock", "in_stock", "in_stock", "outbound"],
                                    weights=[60, 20, 15, 5])[0]

            cur.execute("""
                INSERT INTO screen_pipes
                    (pipe_number, batch_number, screen_type, slot_size, filtration_grade,
                     base_od, base_wt, base_grade, base_end_type,
                     length, weight_per_unit, heat_number, serial_number,
                     manufacturer, production_date, cert_number,
                     location_id, status, notes, created_at)
                VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,datetime('now'))
            """, (
                pipe_no, batch_no, sctype, slot, filt_grade,
                od, wt, base_grade, end_type,
                length, weight, heat_no, serial,
                manufacturer, mfg_date, cert_no,
                loc, status,
                f"{sctype} screen pipe, {filt_grade}, slot={slot}mm, L={length}m"
            ))
            screen_count += 1

    print(f"  + Added {screen_count} screen pipes")

    # ─── 5. ADD MORE PURCHASE ORDERS ───
    print("\n=== Adding More Purchase Orders ===")
    po_count = 0
    po_ids_new = []
    statuses = ["draft", "pending", "approved", "completed", "cancelled"]
    status_weights = [10, 20, 30, 25, 15]

    for i in range(6):
        sup = random.choice(sup_ids)
        sup_name = cur.execute("SELECT name FROM suppliers WHERE id=?", (sup,)).fetchone()[0]
        status = weighted_choice(statuses, status_weights)
        amount = round(random.uniform(30, 500) * 10000, 2)

        order_no = f"PO-{random.randint(10000000,99999999)}"
        ord_date = date(random.randint(5, 180))
        note = random.choice(PURCHASE_NOTES_POOL).format(sup_name[:4])

        cur.execute("""
            INSERT INTO purchase_orders (order_no, supplier_id, order_date, status, total_amount, notes, created_by, created_at)
            VALUES (?,?,?,?,?,?,1, datetime('now'))
        """, (order_no, sup, ord_date, status, amount, note))
        oid = cur.lastrowid
        po_ids_new.append(oid)
        po_count += 1

        # PO items
        num_items = random.randint(2, 5)
        items_total = 0
        for j in range(num_items):
            spec = random.choice(API_5CT_CASING_SPECS + API_5CT_TUBING_SPECS)
            od, wt, grade, *_ = spec
            ptype = "casing" if random.random() < 0.6 else "tubing"
            qty = random.randint(20, 300)
            unit_price = round(random.uniform(800, 3500), 2)
            total_price = round(qty * unit_price, 2)
            items_total += total_price
            received = qty if status == "completed" else (random.randint(0, qty) if status == "approved" else 0)
            cur.execute("""
                INSERT INTO purchase_order_items (order_id, pipe_type, grade, od, wt, quantity, received_quantity, unit_price, total_price, notes)
                VALUES (?,?,?,?,?,?,?,?,?,?)
            """, (oid, ptype, grade, od, wt, qty, received, unit_price, total_price, f"第{j+1}项 - {grade} {ptype} {od}×{wt}mm"))

        cur.execute(
            "UPDATE purchase_orders SET total_amount = ? WHERE id = ?",
            (items_total, oid)
        )

    # ─── 6. ADD MORE SALES ORDERS ───
    print("\n=== Adding More Sales Orders ===")
    so_ids_new = []
    so_count = 0
    for i in range(6):
        cust = random.choice(cust_ids)
        cust_name = cur.execute("SELECT name FROM customers WHERE id=?", (cust,)).fetchone()[0]
        status = weighted_choice(statuses, status_weights)
        amount = round(random.uniform(20, 800) * 10000, 2)
        order_no = f"SO-{random.randint(10000000,99999999)}"
        ord_date = date(random.randint(5, 180))
        note = random.choice(SALES_NOTES_POOL).format(cust_name[:4])

        cur.execute("""
            INSERT INTO sales_orders (order_no, customer_id, order_date, status, total_amount, notes, created_by, created_at)
            VALUES (?,?,?,?,?,?,1, datetime('now'))
        """, (order_no, cust, ord_date, status, amount, note))
        oid = cur.lastrowid
        so_ids_new.append(oid)
        so_count += 1

        num_items = random.randint(1, 4)
        items_total = 0
        for j in range(num_items):
            spec = random.choice(API_5CT_CASING_SPECS + API_5CT_TUBING_SPECS)
            od, wt, grade, *_ = spec
            ptype = random.choice(["casing", "tubing"])
            qty = random.randint(10, 150)
            unit_price = round(random.uniform(1500, 5000), 2)
            total_price = round(qty * unit_price, 2)
            items_total += total_price
            delivered = qty if status == "completed" else (random.randint(0, qty) if status == "approved" else 0)
            cur.execute("""
                INSERT INTO sales_order_items (order_id, pipe_type, grade, od, wt, quantity, delivered_quantity, unit_price, total_price, notes)
                VALUES (?,?,?,?,?,?,?,?,?,?)
            """, (oid, ptype, grade, od, wt, qty, delivered, unit_price, total_price, f"第{j+1}项 - {grade} {ptype} {od}×{wt}mm"))

        cur.execute("UPDATE sales_orders SET total_amount = ? WHERE id = ?", (round(items_total, 2), oid))
        print(f"  + SO {order_no}: {note[:40]}... (status={status})")

    # ─── 7. ADD INBOUND RECORDS ───
    print("\n=== Adding Inbound Records ===")
    all_po = cur.execute(
        "SELECT id, supplier_id, order_no FROM purchase_orders WHERE status IN ('approved','completed')"
    ).fetchall()
    in_count = 0
    for po_id, sup_id, po_no in all_po:
        inbound_no = f"IN-{date(30).replace('-','')}-{random.randint(1000,9999)}"
        inbound_type = random.choice(["purchase", "purchase", "purchase", "return"])
        approval = random.choice(["approved", "approved", "approved", "auto_approved", "pending"])
        handled_days = random.randint(3, 30)
        cur.execute("""
            INSERT INTO inbound_records (inbound_no, inbound_type, order_id, supplier_id, notes, approval_status, handled_by, handled_at, created_at)
            VALUES (?,?,?,?,?,?,1,?, datetime('now', ?))
        """, (inbound_no, inbound_type, po_id, sup_id,
              f"采购到货 - PO:{po_no}",
              approval,
              date(handled_days),
              f'-{handled_days + 5} days'))
        inbound_id = cur.lastrowid
        in_count += 1

        pipe_types = [("seamless", r[0]) for r in
                       cur.execute("SELECT id FROM seamless_pipes WHERE status='in_stock' LIMIT ?",
                                   (random.randint(3, 8),)).fetchall()]
        pipe_types += [("screen", r[0]) for r in
                        cur.execute("SELECT id FROM screen_pipes WHERE status='in_stock' LIMIT ?",
                                    (random.randint(0, 3),)).fetchall()]
        for ptype, pid in pipe_types:
            cur.execute("INSERT INTO inbound_items (inbound_id, pipe_type, pipe_id) VALUES (?,?,?)",
                        (inbound_id, ptype, pid))
            to_loc = random.choice(loc_ids)
            cur.execute("""
                INSERT INTO inventory_logs (pipe_type, pipe_id, change_type, ref_type, ref_id, to_location_id, notes, created_by, created_at)
                VALUES (?,?, 'inbound', 'inbound', ?, ?, '采购入库', 1, datetime('now'))
            """, (ptype, pid, inbound_id, to_loc))

        if approval == "approved":
            for ptype, pid in pipe_types:
                if ptype == "seamless":
                    cur.execute("UPDATE seamless_pipes SET location_id = ? WHERE id = ?", (to_loc, pid))
                else:
                    cur.execute("UPDATE screen_pipes SET location_id = ? WHERE id = ?", (to_loc, pid))

        print(f"  + Inbound {inbound_no}: PO#{po_no}, {len(pipe_types)} pipes, status={approval}")

    # ─── 8. ADD OUTBOUND RECORDS ───
    print("\n=== Adding Outbound Records ===")
    all_so = cur.execute(
        "SELECT id, customer_id, order_no FROM sales_orders WHERE status IN ('approved','completed')"
    ).fetchall()
    out_count = 0
    for so_id, cust_id, so_no in all_so:
        outbound_no = f"OUT-{date(15).replace('-','')}-{random.randint(1000,9999)}"
        approval = random.choice(["approved", "approved", "pending", "auto_approved"])
        handled_days = random.randint(2, 20)
        cur.execute("""
            INSERT INTO outbound_records (outbound_no, outbound_type, order_id, customer_id, notes, approval_status, handled_by, handled_at, created_at)
            VALUES (?, 'sales', ?, ?, ?, ?, 1, ?, datetime('now', ?))
        """, (outbound_no, so_id, cust_id,
              f"销售出库 - SO:{so_no}",
              approval,
              date(handled_days),
              f'-{handled_days + 5} days'))
        outbound_id = cur.lastrowid
        out_count += 1

        num_pipes = random.randint(2, 6)
        available_pipes = cur.execute(
            "SELECT id FROM seamless_pipes WHERE status='in_stock' LIMIT ?",
            (num_pipes,)
        ).fetchall()
        from_loc = None
        for (pid,) in available_pipes:
            cur.execute("INSERT INTO outbound_items (outbound_id, pipe_type, pipe_id) VALUES (?, 'seamless', ?)",
                        (outbound_id, pid))
            loc_row = cur.execute("SELECT location_id FROM seamless_pipes WHERE id=?", (pid,)).fetchone()
            from_loc = loc_row[0] if loc_row else None
            cur.execute("UPDATE seamless_pipes SET status='outbound' WHERE id=?", (pid,))
            cur.execute("""
                INSERT INTO inventory_logs (pipe_type, pipe_id, change_type, ref_type, ref_id, from_location_id, notes, created_by, created_at)
                VALUES ('seamless', ?, 'outbound', 'outbound', ?, ?, '销售出库', 1, datetime('now'))
            """, (pid, outbound_id, from_loc))

        print(f"  + Outbound {outbound_no}: SO#{so_no}, {len(available_pipes)} pipes, status={approval}")

    # ─── 9. ADD MORE CONTRACTS ───
    print("\n=== Adding More Contracts ===")
    contract_count = 0
    for i in range(6):
        ctype = "purchase" if i < 3 else "sales"
        if ctype == "purchase":
            sup = random.choice(sup_ids)
            party_a_name = cur.execute("SELECT name FROM suppliers WHERE id=?", (sup,)).fetchone()[0]
            party_b_name = "SteelPipeDB库存管理公司"
            title = random.choice(CONTRACT_TITLES_POOL_PURCHASE).format(party_a_name[:4])
        else:
            cust = random.choice(cust_ids)
            party_a_name = "SteelPipeDB库存管理公司"
            party_b_name = cur.execute("SELECT name FROM customers WHERE id=?", (cust,)).fetchone()[0]
            title = random.choice(CONTRACT_TITLES_POOL_SALES).format(party_b_name[:4])

        contract_no = f"CT-{random.randint(10000000,99999999)}"
        status = random.choice(["draft", "active", "active", "active", "completed", "terminated"])
        sign_date = date(random.randint(30, 365))
        start_date = date(random.randint(15, 360))
        end_date = date(-random.randint(60, 730))

        cur.execute("""
            INSERT INTO contracts (contract_no, contract_type, title, party_a, party_b,
                                   sign_date, start_date, end_date, status, created_by, created_at)
            VALUES (?,?,?,?,?,?,?,?,?,1, datetime('now'))
        """, (contract_no, ctype, title, party_a_name, party_b_name,
              sign_date, start_date, end_date, status))
        cid = cur.lastrowid
        contract_count += 1

        num_items = random.randint(2, 4)
        total_amt = 0
        for j in range(num_items):
            spec = random.choice(API_5CT_CASING_SPECS + API_5CT_TUBING_SPECS)
            od, wt, grade, *_ = spec
            ptype = random.choice(["casing", "tubing"])
            qty = random.randint(50, 500)
            unit_price = round(random.uniform(900, 3000), 2)
            total_price = round(qty * unit_price, 2)
            total_amt += total_price
            cur.execute("""
                INSERT INTO contract_items (contract_id, pipe_type, grade, od, wt, quantity, unit_price, total_price, notes)
                VALUES (?,?,?,?,?,?,?,?,?)
            """, (cid, ptype, grade, od, wt, qty, unit_price, total_price, f"第{j+1}批交付"))

        cur.execute("UPDATE contracts SET total_amount = ? WHERE id = ?", (round(total_amt, 2), cid))

        deposit_amt = round(total_amt * 0.30, 2)
        milestone_amt = round(total_amt * 0.60, 2)
        final_amt = round(total_amt * 0.10, 2)

        cur.execute("""
            INSERT INTO contract_payments (contract_id, due_date, amount, payment_type, is_paid, paid_date, notes)
            VALUES (?,?,?,'deposit',1,?, '预付款')
        """, (cid, date(random.randint(10, 30)), deposit_amt, date(random.randint(5, 25))))

        ms_paid = 1 if status in ("active", "completed") else 0
        cur.execute("""
            INSERT INTO contract_payments (contract_id, due_date, amount, payment_type, is_paid, paid_date, notes)
            VALUES (?,?,?,'milestone',?,?, '到货款')
        """, (cid, date(-random.randint(30, 90)), milestone_amt,
              ms_paid, date(random.randint(1, 15)) if ms_paid else None))

        cur.execute("""
            INSERT INTO contract_payments (contract_id, due_date, amount, payment_type, is_paid, paid_date, notes)
            VALUES (?,?,?,'final',0,?, '质保金 - 合同完成12个月后支付')
        """, (cid, date(-random.randint(365, 730)), final_amt, None))

        print(f"  + Contract {contract_no}: {title[:40]}... (status={status}, ¥{total_amt:,.0f})")

    # ─── 10. ADD QUALITY CERTIFICATES ───
    print("\n=== Adding Quality Certificates ===")
    cert_count = 0
    piped_with_certs = set(r[0] for r in cur.execute("SELECT pipe_id FROM quality_certs WHERE pipe_type='seamless'").fetchall())
    all_seamless = cur.execute("SELECT id, pipe_number, grade, pipe_type, od, wt, manufacturer, production_date FROM seamless_pipes").fetchall()
    all_screen = cur.execute("SELECT id, pipe_number, base_grade, screen_type, base_od, base_wt, manufacturer, production_date FROM screen_pipes").fetchall()

    to_cert = [p for p in all_seamless if p[0] not in piped_with_certs]
    random.shuffle(to_cert)
    for pipe in to_cert[:int(len(to_cert) * 0.6)]:
        pid, pipe_no, grade, ptype, od, wt, mfr, prod_date = pipe
        cert_no = f"QC-{date(random.randint(10, 90)).replace('-','')}-{random.randint(1000,9999)}"
        result = random.choices(["pass", "pass", "pass", "pass", "fail"], weights=[40, 30, 15, 10, 5])[0]
        inspector = random.choice(INSPECTORS)
        body = random.choice(INSPECTION_BODIES)
        cert_date = date(random.randint(5, 60))

        if result == "pass":
            note = f"{grade} {ptype}, OD={od}mm WT={wt}mm, 拉伸/压扁/水压检验合格, 参考标准API 5CT"
        else:
            note = f"{grade} {ptype}, OD={od}mm WT={wt}mm, 屈服强度不达标, 需复验"

        cur.execute("""
            INSERT INTO quality_certs (cert_number, pipe_type, pipe_id, cert_date, result, inspector, inspection_body, notes, created_at)
            VALUES (?, 'seamless', ?, ?, ?, ?, ?, ?, datetime('now'))
        """, (cert_no, pid, cert_date, result, inspector, body, note))
        cert_count += 1

    for pipe in all_screen[:int(len(all_screen) * 0.5)]:
        pid, pipe_no, grade, sctype, od, wt, mfr, prod_date = pipe
        cert_no = f"QC-{date(random.randint(10, 90)).replace('-','')}-{random.randint(1000,9999)}"
        result = random.choices(["pass", "pass", "pass", "fail"], weights=[40, 30, 20, 10])[0]
        inspector = random.choice(INSPECTORS)
        body = random.choice(INSPECTION_BODIES)
        cert_date = date(random.randint(5, 60))
        cur.execute("""
            INSERT INTO quality_certs (cert_number, pipe_type, pipe_id, cert_date, result, inspector, inspection_body, notes, created_at)
            VALUES (?, 'screen', ?, ?, ?, ?, ?, ?, datetime('now'))
        """, (cert_no, pid, cert_date, result, inspector, body,
              f"{sctype} screen pipe, {grade} base, slot={od if sctype=='wire_wrapped' else 'N/A'}, 过滤精度检验{'合格' if result=='pass' else '不合格'}"))
        cert_count += 1

    print(f"  + Added {cert_count} quality certificates")

    # ─── 11. ADD INVENTORY CHECK RECORDS ───
    print("\n=== Adding Inventory Check Records ===")
    check_count = 0
    for i in range(3):
        check_no = f"CK-{date(random.randint(5, 60)).replace('-','')}-{random.randint(1000,9999)}"
        loc = random.choice(loc_ids)
        status = random.choice(["completed", "completed", "completed", "in_progress"])
        notes = random.choice([
            "月度例行盘点", "季度库存盘点", "年终全面盘点",
            "专项抽检", "移交盘点", "区域轮盘",
        ])
        cur.execute("""
            INSERT INTO inventory_check_records (check_no, location_id, status, notes, created_by, created_at)
            VALUES (?,?,?,?,1, datetime('now'))
        """, (check_no, loc, status, notes))
        check_id = cur.lastrowid
        check_count += 1

        pipes_at_loc = cur.execute(
            "SELECT id, status FROM seamless_pipes WHERE location_id=? LIMIT ?",
            (loc, random.randint(5, 15))
        ).fetchall()
        for pid, expected_status in pipes_at_loc:
            if random.random() < 0.9:
                found_status = expected_status
                is_match = 1
            else:
                found_status = random.choice(["in_stock", "outbound"])
                is_match = 0 if found_status != expected_status else 1
            cur.execute("""
                INSERT INTO inventory_check_items (check_id, pipe_type, pipe_id, expected_status, found_status, is_match, notes)
                VALUES (?, 'seamless', ?, ?, ?, ?, ?)
            """, (check_id, pid, expected_status, found_status, is_match,
                  "账实相符" if is_match else "差异: 实物状态不符"))

        print(f"  + Check {check_no}: location {loc}, {len(pipes_at_loc)} items, status={status}")

    # ─── 12. ADD OPERATION LOGS ───
    print("\n=== Adding Operation Logs ===")
    actions = [
        ("create", "采购订单创建"),
        ("approve", "采购订单审批通过"),
        ("create", "销售订单创建"),
        ("approve", "销售订单审批通过"),
        ("inbound", "钢管入库操作"),
        ("outbound", "钢管出库操作"),
        ("quality_check", "质量证书生成"),
        ("contract_sign", "合同签署确认"),
        ("inventory_check", "库存盘点完成"),
        ("pipe_move", "库位转移操作"),
    ]
    entity_table = {
        "purchase_order": "purchase_orders",
        "sales_order": "sales_orders",
        "seamless_pipe": "seamless_pipes",
        "screen_pipe": "screen_pipes",
        "contract": "contracts",
        "inbound_record": "inbound_records",
        "outbound_record": "outbound_records",
    }
    entities = list(entity_table.keys())
    log_count = 0
    for i in range(20):
        action, desc = random.choice(actions)
        entity = random.choice(entities)
        entity_id = random.randint(1, max(max_id(entity_table[entity]), 1))
        cur.execute("""
            INSERT INTO operation_logs (user_id, username, action, entity_type, entity_id, details, ip_address, created_at)
            VALUES (1, 'admin', ?, ?, ?, ?, ?, datetime('now'))
        """, (action, entity, entity_id,
              f"{desc} - {entity}#{entity_id}",
              f"192.168.1.{random.randint(10, 200)}"))
        log_count += 1
    print(f"  + Added {log_count} operation logs")

    conn.commit()


    print("\n" + "="*60)
    print("ENHANCED SEED DATA SUMMARY")
    print("="*60)
    tables = [
        ("users", "SELECT COUNT(*) FROM users"),
        ("suppliers", "SELECT COUNT(*) FROM suppliers"),
        ("customers", "SELECT COUNT(*) FROM customers"),
        ("locations", "SELECT COUNT(*) FROM locations"),
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
        ("contracts", "SELECT COUNT(*) FROM contracts"),
        ("contract_items", "SELECT COUNT(*) FROM contract_items"),
        ("contract_payments", "SELECT COUNT(*) FROM contract_payments"),
        ("quality_certs", "SELECT COUNT(*) FROM quality_certs"),
        ("inventory_logs", "SELECT COUNT(*) FROM inventory_logs"),
        ("inventory_check_records", "SELECT COUNT(*) FROM inventory_check_records"),
        ("inventory_check_items", "SELECT COUNT(*) FROM inventory_check_items"),
        ("operation_logs", "SELECT COUNT(*) FROM operation_logs"),
    ]
    for name, query in tables:
        count = cur.execute(query).fetchone()[0]
        print(f"  {name:30s}: {count} rows")

    conn.close()
    print("\n✓ Enhanced seed data complete!")


if __name__ == "__main__":
    main()
