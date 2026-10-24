"""Audit log routes: view and verify hash chain."""

from fastapi import APIRouter, Depends, Request
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from rust_core import audit as rust_audit
from src.database import get_session
from src.models import AuditLog, GENESIS_HASH, User
from src.security import (
    generate_csrf_token,
    get_current_user,
    require_admin,
    require_csrf,
    set_csrf_cookie,
)
from src.templates_env import templates
import anyio

CHUNK_SIZE = 20_000

router = APIRouter()


@router.get("/audit")
async def audit_logs(
    request: Request,
    session: AsyncSession = Depends(get_session),
    user=Depends(get_current_user),
):
    """Display the last 50 audit log entries."""
    token = request.cookies.get("csrf_token") or generate_csrf_token()

    result = await session.execute(
        select(AuditLog).order_by(AuditLog.id.desc()).limit(50)
    )
    rows = result.scalars().all()

    rendered = templates.TemplateResponse(
        "audit/audit_logs.html",
        {
            "request": request,
            "user": user,
            "logs": rows,
            "csrf_token": token,
            "verify_result": None,
        },
    )
    set_csrf_cookie(rendered, token)
    return rendered


def _row_to_dict(r: AuditLog) -> dict:
    return {
        "id": r.id,
        "prev_hash": r.prev_hash,
        "action": r.action,
        "detail": r.detail,
        "timestamp": r.ts,
        "current_hash": r.current_hash,
    }


async def _verify_chunked(rows: list[AuditLog]) -> dict:
    """Walk all audit rows in 20K chunks via rust_audit.verify_chain_chunk.

    Stops at the first broken entry; returns a summary dict compatible with
    the existing template render (`{ok, broken}`) where `broken` lists each
    entry that failed (id + expected/actual).
    """
    broken: list[dict] = []
    expected_prev = GENESIS_HASH
    start_id = 0

    for offset in range(0, len(rows), CHUNK_SIZE):
        chunk_rows = rows[offset:offset + CHUNK_SIZE]
        chunk_dicts = [_row_to_dict(r) for r in chunk_rows]
        chunk_pyobjects: list = await anyio.to_thread.run_sync(
            lambda cd=chunk_dicts: [d for d in cd], chunk_dicts
        )
        result = await anyio.to_thread.run_sync(
            lambda: rust_audit.verify_chain_chunk(start_id, expected_prev, chunk_pyobjects)
        )

        ok = bool(result.get("ok", False))
        broken_idx = result.get("broken_index")
        new_tip = result.get("tip") or expected_prev

        if not ok:
            if broken_idx is not None:
                bad = chunk_dicts[int(broken_idx)]
                broken.append({
                    "id": bad["id"],
                    "expected": "",
                    "actual": bad["current_hash"],
                })
            break

        expected_prev = new_tip
        if chunk_rows:
            start_id = chunk_rows[-1].id

    return {"ok": not broken, "broken": broken, "verified_total": len(rows)}


@router.post("/audit/verify")
async def verify_chain_post(
    request: Request,
    session: AsyncSession = Depends(get_session),
    user: User = Depends(get_current_user),
    _admin: User = Depends(require_admin),
    _: None = Depends(require_csrf),
):
    """Verify the entire audit hash chain via batched Rust calls (admin-only)."""
    return await _render_verify_result(request, session, user)


@router.get("/audit/verify")
async def verify_chain_get(
    request: Request,
    session: AsyncSession = Depends(get_session),
    user: User = Depends(get_current_user),
    _admin: User = Depends(require_admin),
):
    """Chain verification (admin-only, per AGENTS.md role rules)."""
    return await _render_verify_result(request, session, user)


async def _render_verify_result(
    request: Request,
    session: AsyncSession,
    user: User,
):
    token = request.cookies.get("csrf_token") or generate_csrf_token()

    result = await session.execute(
        select(AuditLog).order_by(AuditLog.id.asc())
    )
    rows = result.scalars().all()

    verify_result = await _verify_chunked(list(rows))

    recent_result = await session.execute(
        select(AuditLog).order_by(AuditLog.id.desc()).limit(50)
    )
    recent_rows = recent_result.scalars().all()

    rendered = templates.TemplateResponse(
        "audit/audit_logs.html",
        {
            "request": request,
            "user": user,
            "logs": recent_rows,
            "csrf_token": token,
            "verify_result": verify_result,
        },
    )
    set_csrf_cookie(rendered, token)
    return rendered
