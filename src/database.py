"""SQLAlchemy async engine, session factory, and session dependency.

All raw connections execute the mandatory SQLite PRAGMAs via an
event listener on the sync engine.
"""

import datetime
import sqlite3
from collections.abc import AsyncGenerator

from sqlalchemy import event
from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker, create_async_engine

from src.config import settings

# Silence the Python 3.12+ "default datetime adapter is deprecated" warning
# that fires whenever a bare `datetime`/`date` reaches the DBAPI via a raw
# `text()` bind (e.g. stock_out writes `stock_out_time=:t`). The custom adapters
# below replicate the exact storage format the deprecated defaults produced
# (`YYYY-MM-DD HH:MM:SS.ffffff` for datetime, `YYYY-MM-DD` for date), so there is
# zero behavioural change — only the DeprecationWarning is removed. SQLAlchemy's
# own `DateTime`/`Date` types bypass sqlite3 adapters entirely (they serialise to
# strings before reaching the DBAPI), so this registration never interferes with
# ORM/Core typed-column writes.
sqlite3.register_adapter(datetime.datetime, lambda d: d.isoformat(sep=" "))
sqlite3.register_adapter(datetime.date, lambda d: d.isoformat())

engine = create_async_engine(
    settings.DATABASE_URL,
    echo=False,
)


@event.listens_for(engine.sync_engine, "connect")
def _set_sqlite_pragmas(dbapi_connection: object, _connection_record: object) -> None:
    """Set mandatory SQLite PRAGMAs on every new raw connection.

    WAL mode, synchronous=NORMAL, foreign_keys enforcement, busy timeout,
    and WAL auto-checkpoint are all set here.
    """
    cursor = dbapi_connection.cursor()  # type: ignore[union-attr]
    cursor.execute("PRAGMA foreign_keys=ON;")
    cursor.execute("PRAGMA journal_mode=WAL;")
    cursor.execute("PRAGMA synchronous=NORMAL;")
    cursor.execute("PRAGMA busy_timeout=5000;")
    cursor.execute("PRAGMA wal_autocheckpoint=1000;")
    cursor.close()


async_session_factory = async_sessionmaker(
    engine,
    class_=AsyncSession,
    expire_on_commit=False,
)


async def get_session() -> AsyncGenerator[AsyncSession, None]:
    """FastAPI dependency that yields an async session."""
    async with async_session_factory() as session:
        yield session
