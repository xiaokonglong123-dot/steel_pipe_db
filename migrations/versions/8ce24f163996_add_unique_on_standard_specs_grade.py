"""add unique on standard_specs.grade

Revision ID: 8ce24f163996
Revises: 20260713_084000_add_audit_ts
Create Date: 2026-07-16 15:24:00.708802
"""
from collections.abc import Sequence

from alembic import op

revision: str = '8ce24f163996'
down_revision: str | None = '20260713_084000_add_audit_ts'
branch_labels: str | Sequence[str] | None = None
depends_on: str | Sequence[str] | None = None


def upgrade() -> None:
    with op.batch_alter_table('standard_specs', schema=None) as batch_op:
        batch_op.create_unique_constraint('uq_standard_specs_grade', ['grade'])


def downgrade() -> None:
    with op.batch_alter_table('standard_specs', schema=None) as batch_op:
        batch_op.drop_constraint('uq_standard_specs_grade', type_='unique')
