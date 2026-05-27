# Phase 3 — Backend: Label Printing Module (P2)

> Based on: `docs/requirements.en.md` §3.8, `docs/detailed-design.en.md` §10.3

## Task List

### 1.1 Label Template Management
- [ ] Create `label_templates` table migration (template name, content config)
- [ ] Implement `GET /api/v1/label-templates` — Template list
- [ ] Implement `POST /api/v1/label-templates` — Create template
- [ ] Implement `PUT /api/v1/label-templates/{id}` — Update template
- [ ] Implement `DELETE /api/v1/label-templates/{id}` — Delete template
- [ ] Each template stores: label width/height, font size, field positions (JSON config)

### 1.2 Label Generation & Printing
- [ ] Implement `POST /api/v1/labels/generate`:
  - Request: template ID + list of pipe numbers
  - Validate pipes exist + template exists
  - Returns: generated label PDF (A4 layout, multi-label grid)
- [ ] Use PDF library (`printpdf`) to generate labels with barcodes or QR codes
- [ ] Differentiate output for bare pipe / finished pipe / coupling
- [ ] Label content: pipe number, grade, spec, length, weight, production date, heat/lot number
- [ ] `GET /api/v1/labels/print-history` — Print history records

### 1.3 Barcodes / QR Codes
- [ ] Generate QR code: encodes pipe number URL (points to detail page)
- [ ] Generate Code 128 barcode from pipe number (use `barcoder` crate or hand-roll it)

> **Dependencies**: Pipe management module (references pipe data)
