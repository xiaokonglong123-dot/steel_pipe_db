# ERP P0 Design System

## 0. Research Log
- Embedded refs: operational dashboard guidance from `taste-skill.md` and Element Plus; chosen because this is a dense internal ERP scaffold.
- Lazyweb: skipped, because the user explicitly requested a functional P0 scaffold without design polish.
- Imagen drafts: skipped, because this is an internal data application with no visual asset requirement.

## 1. Atmosphere & Identity
清晰、克制、可快速扫描的业务工作台。签名是蓝色操作强调与浅灰分层背景，让表格、状态和审批动作优先于装饰。

## 2. Color
| Role | Token | Value | Usage |
|---|---|---|---|
| Surface | `--surface` | `#f5f7fa` | App background |
| Panel | `--panel` | `#ffffff` | Content panels |
| Text | `--text` | `#303133` | Primary text |
| Muted | `--muted` | `#606266` | Secondary text |
| Border | `--border` | `#dcdfe6` | Dividers |
| Accent | `--accent` | `#409eff` | Actions and links |

## 3. Typography
- Primary: system sans-serif stack, Chinese glyph fallback.
- Body: 14px; page title: 20px; metadata: 12px.

## 4. Spacing & Layout
- Base unit: 4px. Content padding: 20px. Toolbar gap: 12px.
- Fixed sidebar shell, scroll-owned main content. Mobile collapses sidebar width to 64px.

## 5. Components
- **DataTable**: table, pagination, loading and empty states; keyboard-accessible actions.
- **SearchBar**: labeled filter form with explicit search/reset actions.
- **StatusTag**: semantic status color mapping.
- **PermissionButton**: hides unauthorized actions and preserves keyboard order.
- **PageLayout**: title, description, actions, and content slot.

## 6. Motion & Interaction
- Element Plus standard transitions only. No decorative motion. Reduced-motion behavior is delegated to Element Plus.

## 7. Depth & Surface
- Mixed: tonal background separation plus Element Plus borders. No custom shadows.

## 8. Accessibility Constraints & Accepted Debt
- WCAG 2.2 AA target, visible focus states, labeled inputs, keyboard-reachable actions.
- P0 debt: no visual regression harness and no empty-state illustrations; accepted to prioritize backend-connected workflows.
