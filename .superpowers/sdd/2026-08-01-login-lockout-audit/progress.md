# SDD ledger — plan: docs/superpowers/plans/2026-08-01-login-lockout-audit.md

Environment: NOT a git repo — no commits; task completion tracked by file changes + test results.

Task 1: pending
Task 2: pending
Task 1: complete (files: security.py, auth.py, test_login_lockout_audit.py; 6 passed + 115 passed/3 deselected regression; review PASS)
  fix round 1: brief defects (locked_for :g format; rust_auth.init in test file) - both resolved
  minors (deferred): isinstance(locked_before,float) fragility; dropped inline comment (cosmetic)
  resolved ⚠️: XFF-sourced ip in audit detail = spec-intent (accepted); in-memory lockout restart loss = pre-existing design (accepted, noted)
Task 1: complete (security.py: MAX_FAILURES/LOCKOUT_SECONDS constants + record_failure -> bool; auth.py: login_lockout audit write in 401 branch; tests/test_login_lockout_audit.py 6 tests). 2 fix rounds for brief defects (locked_for :g format; rust_auth.init in test). Review PASS (0 critical/important; 2 accepted warnings: XFF spoofable IP in detail = design intent; in-memory lockout state = pre-existing design).
Task 2: complete (design doc v0.6.0, AGENTS.md v0.6.0, versions 0.5.0). 2 fix rounds (121->118 arithmetic controller error; round 1 did not apply). Review PASS (0 critical/important).
Deferred minors: 13.1 "21K" vs AGENTS.md "20K" naming drift (pre-existing); main.py version in FastAPI(version=...) not __version__ (memory #168 corrected).
