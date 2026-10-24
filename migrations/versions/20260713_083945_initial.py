"""Initial schema — creates all 9 tables with indexes.

Revision ID: 20260713_083945_initial
Revises:
Create Date: 2026-07-13 08:39:45
"""

from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa

revision: str = "20260713_083945_initial"
down_revision: Union[str, None] = None
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    # --- locations (no FK dependencies) ---
    op.create_table(
        "locations",
        sa.Column("code", sa.String(32), primary_key=True),
        sa.Column("name", sa.String, nullable=True),
        sa.Column("used_count", sa.Integer, server_default=sa.text("0"), nullable=False),
        sa.Column("capacity", sa.Integer, nullable=True),
        sa.Column("is_active", sa.Boolean, server_default=sa.text("1"), nullable=False),
    )

    # --- users (no FK dependencies) ---
    op.create_table(
        "users",
        sa.Column("id", sa.Integer, primary_key=True, autoincrement=True),
        sa.Column("username", sa.String, unique=True, nullable=False),
        sa.Column("password_hash", sa.String, nullable=False),
        sa.Column("display_name", sa.String, nullable=False),
        sa.Column("role", sa.String(16), nullable=False),
        sa.Column("token_version", sa.Integer, server_default=sa.text("0"), nullable=False),
        sa.Column("last_refresh_jti", sa.String, nullable=True),
        sa.Column("active", sa.Boolean, server_default=sa.text("1"), nullable=False),
        sa.Column("created_at", sa.DateTime, nullable=False),
    )

    # --- standard_specs (no FK dependencies) ---
    op.create_table(
        "standard_specs",
        sa.Column("id", sa.Integer, primary_key=True, autoincrement=True),
        sa.Column("grade", sa.String, nullable=False),
        sa.Column("outer_diameter_mm", sa.Float, nullable=True),
        sa.Column("min_wall_thickness_mm", sa.Float, nullable=True),
        sa.Column("max_wall_thickness_mm", sa.Float, nullable=True),
        sa.Column("length_tolerance_mm", sa.Float, nullable=True),
        sa.Column("source", sa.String, nullable=True),
    )

    # --- tubes (FK → locations.code) ---
    op.create_table(
        "tubes",
        sa.Column("id", sa.Integer, primary_key=True, autoincrement=True),
        sa.Column("pipe_id", sa.String, unique=True, nullable=False),
        sa.Column("type", sa.String(32), nullable=False),
        sa.Column("outer_diameter_mm", sa.Float, nullable=False),
        sa.Column("wall_thickness_mm", sa.Float, nullable=False),
        sa.Column("length_m", sa.Float, nullable=False),
        sa.Column("steel_grade", sa.String(32), nullable=False),
        sa.Column("thread_type", sa.String(32), nullable=True),
        sa.Column("api_5ct_cert", sa.String, nullable=True),
        sa.Column("manufacturer", sa.String, nullable=False),
        sa.Column("batch_no", sa.String, nullable=True),
        sa.Column("heat_no", sa.String, nullable=True),
        sa.Column("heat_treat_method", sa.String, nullable=True),
        sa.Column("location", sa.String(32), sa.ForeignKey("locations.code", ondelete="RESTRICT"), nullable=False),
        sa.Column("status", sa.String(16), nullable=False),
        sa.Column("quality_mark", sa.String(16), server_default=sa.text("'normal'"), nullable=False),
        sa.Column("quality_note", sa.Text, nullable=True),
        sa.Column("stock_in_time", sa.DateTime, nullable=True),
        sa.Column("stock_out_time", sa.DateTime, nullable=True),
        sa.Column("compliance_pass", sa.Boolean, nullable=True),
        sa.Column("deleted_at", sa.DateTime, nullable=True),
        sa.Column("created_at", sa.DateTime, nullable=False),
        sa.Column("updated_at", sa.DateTime, nullable=False),
    )

    # --- seamless_pipe_specs (1:1, FK → tubes.pipe_id) ---
    op.create_table(
        "seamless_pipe_specs",
        sa.Column("pipe_id", sa.String, sa.ForeignKey("tubes.pipe_id", ondelete="CASCADE"), primary_key=True),
        sa.Column("surface_treatment", sa.String, nullable=True),
        sa.Column("end_protector", sa.String, nullable=True),
    )

    # --- screen_pipe_specs (1:1, FK → tubes.pipe_id) ---
    op.create_table(
        "screen_pipe_specs",
        sa.Column("pipe_id", sa.String, sa.ForeignKey("tubes.pipe_id", ondelete="CASCADE"), primary_key=True),
        sa.Column("screen_type", sa.String, nullable=True),
        sa.Column("slot_width_mm", sa.Float, nullable=True),
        sa.Column("slot_density", sa.Float, nullable=True),
        sa.Column("filtration_rating_um", sa.Float, nullable=True),
        sa.Column("base_pipe_grade", sa.String, nullable=True),
        sa.Column("slot_pattern", sa.String, nullable=True),
    )

    # --- transactions (no FK dependencies) ---
    op.create_table(
        "transactions",
        sa.Column("id", sa.Integer, primary_key=True, autoincrement=True),
        sa.Column("type", sa.String(16), nullable=False),  # in / out / move / check
        sa.Column("operator_id", sa.Integer, nullable=False),
        sa.Column("reference_no", sa.String, nullable=True),
        sa.Column("remark", sa.Text, nullable=True),
        sa.Column("created_at", sa.DateTime, nullable=False),
    )

    # --- transaction_items (FK → transactions.id) ---
    op.create_table(
        "transaction_items",
        sa.Column("id", sa.Integer, primary_key=True, autoincrement=True),
        sa.Column("transaction_id", sa.Integer, sa.ForeignKey("transactions.id", ondelete="CASCADE"), nullable=False),
        sa.Column("pipe_id", sa.String, nullable=False),
        sa.Column("source_location", sa.String, nullable=True),
        sa.Column("target_location", sa.String, nullable=True),
    )

    # --- audit_logs (no FK dependencies at schema level) ---
    op.create_table(
        "audit_logs",
        sa.Column("id", sa.Integer, primary_key=True, autoincrement=True),
        sa.Column("user_id", sa.Integer, nullable=True),
        sa.Column("action", sa.String, nullable=False),
        sa.Column("detail", sa.Text, nullable=True),
        sa.Column("prev_hash", sa.String, nullable=False),
        sa.Column("current_hash", sa.String, nullable=False),
        sa.Column("created_at", sa.DateTime, nullable=False),
    )

    # --- indexes ---
    op.create_index("ix_tubes_location", "tubes", ["location"])
    op.create_index("ix_tubes_status_location", "tubes", ["status", "location"])
    op.create_index("ix_tubes_steel_grade", "tubes", ["steel_grade"])
    op.create_index("ix_tubes_quality_mark", "tubes", ["quality_mark"])
    op.create_index("ix_tubes_deleted_at", "tubes", ["deleted_at"])
    op.create_index("ix_transaction_items_transaction_id", "transaction_items", ["transaction_id"])
    op.create_index("ix_seamless_pipe_specs_pipe_id", "seamless_pipe_specs", ["pipe_id"])
    op.create_index("ix_screen_pipe_specs_pipe_id", "screen_pipe_specs", ["pipe_id"])


def downgrade() -> None:
    """Drop all tables in reverse FK dependency order.

    Child tables first, then parents.
    """
    op.drop_table("transaction_items")
    op.drop_table("seamless_pipe_specs")
    op.drop_table("screen_pipe_specs")
    op.drop_table("tubes")
    op.drop_table("transactions")
    op.drop_table("audit_logs")
    op.drop_table("locations")
    op.drop_table("users")
    op.drop_table("standard_specs")
