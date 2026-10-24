"""Add check_sessions + check_items for inventory check (盘点, D11).

Revision ID: 20260801_000000_add_check_tables
Revises: 8ce24f163996
Create Date: 2026-08-01
"""

from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa

revision: str = "20260801_000000_add_check_tables"
down_revision: Union[str, None] = "8ce24f163996"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.create_table(
        "check_sessions",
        sa.Column("id", sa.Integer, primary_key=True, autoincrement=True),
        sa.Column("location", sa.String(32), nullable=False),
        sa.Column("status", sa.String(16), server_default=sa.text("'open'"), nullable=False),
        sa.Column("created_by", sa.Integer, nullable=False),
        sa.Column("confirmed_by", sa.Integer, nullable=True),
        sa.Column("created_at", sa.DateTime, nullable=False),
        sa.Column("confirmed_at", sa.DateTime, nullable=True),
        sa.Column("remark", sa.Text, nullable=True),
    )
    op.create_table(
        "check_items",
        sa.Column("id", sa.Integer, primary_key=True, autoincrement=True),
        sa.Column(
            "check_id",
            sa.Integer,
            sa.ForeignKey("check_sessions.id", ondelete="CASCADE"),
            nullable=False,
        ),
        sa.Column("pipe_id", sa.String, nullable=False),
        sa.Column("result", sa.String(16), nullable=False),
        sa.Column("actual_location", sa.String(32), nullable=True),
        sa.Column("surplus_data", sa.Text, nullable=True),
        sa.Column("created_at", sa.DateTime, nullable=False),
    )
    op.create_index("ix_check_items_check_id", "check_items", ["check_id"])


def downgrade() -> None:
    op.drop_index("ix_check_items_check_id", table_name="check_items")
    op.drop_table("check_items")
    op.drop_table("check_sessions")
