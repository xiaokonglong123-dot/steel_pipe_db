"""User management service — create, update, revoke sessions, toggle active.

All password operations go through Rust auth module. Token revocation
uses token_version (global) and last_refresh_jti (single-session).
"""

from __future__ import annotations

from typing import Optional

import anyio
from sqlalchemy import select

from rust_core import auth as rust_auth
from src import database as db
from src.models import User
from src.modules.warehouse.services.audit_service import (
    _write_audit,
    begin_immediate,
)


class UserConflictError(Exception):
    """Username already exists."""


class LastAdminError(Exception):
    """Cannot deactivate/revoke the last remaining admin."""


async def create_user(
    username: str,
    password: str,
    display_name: str,
    role: str,
    engine: object = None,
    *,
    actor_id: int,
) -> User:
    """Create a new user with argon2id hashed password.

    Returns the created User ORM instance.
    """
    eng = engine or db.engine

    async with eng.connect() as conn:
        await begin_immediate(conn)
        # Check uniqueness
        existing = (
            await conn.execute(select(User).where(User.username == username))
        ).scalar_one_or_none()
        if existing is not None:
            await conn.rollback()
            raise UserConflictError(f"username '{username}' already exists")

        password_hash = await anyio.to_thread.run_sync(
            rust_auth.hash_password, password
        )

        result = await conn.execute(
            User.__table__.insert().values(
                username=username,
                password_hash=password_hash,
                display_name=display_name,
                role=role,
                active=True,
            )
        )
        await _write_audit(
            conn, "create_user", f"username={username} role={role}",
            user_id=actor_id,
        )
        await conn.commit()

        user = User(
            id=result.inserted_primary_key[0],
            username=username,
            password_hash=password_hash,
            display_name=display_name,
            role=role,
            token_version=0,
            last_refresh_jti=None,
            active=True,
        )
        return user


async def update_user(
    user_id: int,
    display_name: Optional[str] = None,
    role: Optional[str] = None,
    password: Optional[str] = None,
    engine: object = None,
    *,
    actor_id: int,
) -> User:
    """Update user fields. Password re-hashes via Rust if provided.

    Returns the updated User ORM instance.
    """
    eng = engine or db.engine

    async with eng.connect() as conn:
        await begin_immediate(conn)
        user_row = (
            await conn.execute(select(User).where(User.id == user_id))
        ).scalar_one_or_none()
        if user_row is None:
            await conn.rollback()
            raise ValueError(f"user {user_id} not found")

        values: dict = {}
        if display_name is not None:
            values["display_name"] = display_name
        if role is not None:
            values["role"] = role
        if password is not None:
            values["password_hash"] = await anyio.to_thread.run_sync(
                rust_auth.hash_password, password
            )

        if values:
            await conn.execute(
                User.__table__.update().where(User.id == user_id).values(**values)
            )
            parts = []
            if display_name is not None:
                parts.append(f"display_name={display_name}")
            if role is not None:
                parts.append(f"role={role}")
            if password is not None:
                parts.append("password_changed=true")
            await _write_audit(
                conn, "update_user", " ".join(parts), user_id=actor_id,
            )
            await conn.commit()
        else:
            await conn.rollback()

        # Re-read for updated state
        result = await conn.execute(select(User).where(User.id == user_id))
        return result.scalar_one()


async def revoke_all_sessions(
    user_id: int,
    engine: object = None,
    *,
    actor_id: int,
) -> None:
    """Globally revoke all active sessions by incrementing token_version.

    Also clears last_refresh_jti so no pending refresh tokens remain valid.
    """
    eng = engine or db.engine

    async with eng.connect() as conn:
        await begin_immediate(conn)
        await conn.execute(
            User.__table__.update()
            .where(User.id == user_id)
            .values(
                token_version=User.token_version + 1,
                last_refresh_jti=None,
            )
        )
        await _write_audit(
            conn, "revoke_sessions", f"user_id={user_id} token_version+1",
            user_id=actor_id,
        )
        await conn.commit()


async def toggle_active(
    user_id: int,
    active: bool,
    engine: object = None,
    *,
    actor_id: int,
) -> None:
    """Enable or disable a user account.

    Raises LastAdminError if deactivating the last remaining admin.
    """
    eng = engine or db.engine

    async with eng.connect() as conn:
        await begin_immediate(conn)
        if not active:
            # Guard: must have at least one other active admin
            admin_count = (
                await conn.execute(
                    select(User).where(
                        User.role == "admin",
                        User.active == True,  # noqa: E712
                        User.id != user_id,
                    )
                )
            ).scalar()
            if not admin_count:
                await conn.rollback()
                raise LastAdminError(
                    "cannot deactivate the last remaining admin account"
                )

        await conn.execute(
            User.__table__.update()
            .where(User.id == user_id)
            .values(active=active)
        )
        await _write_audit(
            conn, "toggle_user", f"user_id={user_id} active={active}",
            user_id=actor_id,
        )
        await conn.commit()
