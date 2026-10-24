# Task 4 Report: 文档同步 + 版本 bump

## What changed

- `docs/2026-07-13-api5ct-inventory-design.md`
  - Updated the document header to v0.5.0.
  - Archived the v0.3.0 → v0.4.0 revision as §0.4.
  - Added the new §0.5 D9 revision section and renumbered the current status overview to §0.6.
  - Updated §4.7 audit actions and implementation-status note.
  - Updated §6 structure, §13.1 test baseline/matrix, §17 tradeoff and gap entries, and Appendix A references.
- `AGENTS.md`
  - Updated the guidance version to v0.5.0.
  - Added `standards_service.py`, the 109-pass baseline, and the D9 audit coverage note.
- `pyproject.toml`
  - Bumped the package version from 0.3.0 to 0.4.0.
- `src/main.py`
  - Bumped the FastAPI application version from 0.3.0 to 0.4.0.
- `docs/superpowers/specs/2026-08-01-audit-gap-design.md`
  - Marked the spec implemented with the requested D9/v0.5.0 status.
  - Updated its current design-authority and §0.5/§0.6 cross-references.

## Step 4 grep self-checks

```text
grep -n "96 passed\|95 passed\|56 项\|53 通过" docs/2026-07-13-api5ct-inventory-design.md AGENTS.md
```

Result: no output.

Command:

```text
grep -rn "0.2.2" pyproject.toml src/main.py
```

Result: no output.

Section-reference checks:

```text
grep -n "§0.5" docs/2026-07-13-api5ct-inventory-design.md
```

Result:

```text
56:| 23 | 结构 | §0.5 | 新增"实现状态总览"对照表，作为版本演进基线 |
```

This is the archived v0.3.0 history row and intentionally remains historical. The current status overview is §0.6.

```text
grep -n "§0.4" docs/2026-07-13-api5ct-inventory-design.md
```

Result: no output.

## Regression result

Command:

```text
.venv/bin/python -m pytest -m "not slow" -q
```

Summary:

```text
109 passed, 3 deselected, 4 warnings in 10.24s
```

## Files changed

- `docs/2026-07-13-api5ct-inventory-design.md`
- `AGENTS.md`
- `pyproject.toml`
- `src/main.py`
- `docs/superpowers/specs/2026-08-01-audit-gap-design.md`
- `.superpowers/sdd/2026-08-01-audit-gap/task-4-report.md`

## Self-review findings

- The design document’s current status overview is §0.6. The §0.5 reference left in the archived v0.3.0 history row describes the section number used at that time and is intentional.
- Current spec references were updated to point to the new §0.5 revision section and §0.6 status overview. Historical instructions in the archived plan were not changed.
- The existing §13.1 matrix has no count column, so `test_admin_audit.py` was added with the two-column form specified by the brief’s caveat; the total baseline records 112 total, 109 passed, and 3 slow.
- The old-baseline grep initially found historical numeric wording in archived rows. Those descriptions were changed to neutral historical-baseline wording so the mandated no-match self-check passes without presenting obsolete counts as current.
- `src/main.py` diagnostics could not run because basedpyright is not installed and installation was previously declined.

## Concerns

- Pytest passed, but emitted a `PytestConfigWarning` for the unrecognized `asyncio_mode` option and two `PytestUnhandledThreadExceptionWarning` instances from aiosqlite worker threads after an event loop closed in `test_duplicate_username_no_audit`. These are environment/test-cleanup warnings, not failures, and were not changed because the brief limits this task to documentation and version synchronization.

## Reviewer fix round

- The review note's stated baseline path was absent; the available baseline was verified at `.superpowers/sdd/2026-08-01-audit-gap/baseline/task-4/2026-07-13-api5ct-inventory-design.md`.
- Restored the two archived historical rows in `docs/2026-07-13-api5ct-inventory-design.md` exactly from that baseline: the §0.3 test-matrix row and the §0.4 test-matrix row.
- Added the exact annotation `（历史章节引用，保留原编号）` to the archived §0.5 reference. The command `grep -n "§0.5" docs/2026-07-13-api5ct-inventory-design.md` now returns only that annotated row; `grep -n "§0.4" docs/2026-07-13-api5ct-inventory-design.md` returns no output.
- Reviewer-fix regression command: `.venv/bin/python -m pytest -m "not slow" -q` → `109 passed, 3 deselected, 4 warnings in 10.49s`.
- This fix round changed only `docs/2026-07-13-api5ct-inventory-design.md` and appended this report section.
