"""Add ts column to audit_logs for deterministic hash verification.

Revision ID: 20260713_084000_add_audit_ts
Revises: 20260713_083945_initial
Create Date: 2026-07-13 08:40:00
"""

from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa

revision: str = "20260713_084000_add_audit_ts"
down_revision: Union[str, None] = "20260713_083945_initial"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    with op.batch_alter_table("audit_logs") as batch_op:
        batch_op.add_column(sa.Column("ts", sa.String(), nullable=True))


def downgrade() -> None:
    with op.batch_alter_table("audit_logs") as batch_op:
        batch_op.drop_column("ts")
