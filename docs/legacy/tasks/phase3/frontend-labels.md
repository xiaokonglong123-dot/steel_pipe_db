# Phase 3 — Frontend: Label Printing Module (P2)

> Based on: `docs/frontend-design.en.md` §4.1

## Task List

### 1.1 Shared Types & API
- [ ] Define `features/labels/types.ts`: LabelTemplate, LabelConfig, etc.
- [ ] Define `features/labels/api/labelApi.ts`:
  - Template CRUD
  - Label generation
  - Print history

### 1.2 Label Template Management Page
- [ ] Implement `LabelTemplateListPage`:
  - Template list (name, width/height, last modified, actions)
  - Create / edit / delete templates
- [ ] Implement `LabelTemplateFormPage`:
  - Basic info: template name, width, height (mm)
  - Visual editor (simplified): pick fields to display + font size + alignment
  - Preview: simulate label rendering

### 1.3 Label Generation & Print Page
- [ ] Implement `LabelGeneratePage`:
  - Select label template
  - Select pipes (batch): comma-separated input + pick from list
  - Preview generated label layout
  - Print button → calls generate API → downloads PDF / triggers print

### 1.4 Shared Components
- [ ] Implement `TemplatePreview`: live preview of label template configuration
- [ ] Implement `PipeSelector`: batch pipe selector (search by number + multi-select list)

> **Dependencies**: Pipe management frontend module (PipeSelector)
