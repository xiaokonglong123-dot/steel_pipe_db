"""Application configuration via pydantic-settings."""

from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    model_config = SettingsConfigDict(env_file=".env", extra="ignore")

    DATABASE_URL: str = "sqlite+aiosqlite:///./data/inventory.db"
    # No default on purpose: an empty value is INVALID at runtime. The app
    # MUST fail-fast at startup (see main.py) if this is empty. It is NOT
    # required at import time so that tooling like `alembic` (which only
    # needs DATABASE_URL) works before the secret is provisioned.
    JWT_SECRET_KEY: str = ""
    MAX_UPLOAD_SIZE: int = 20 * 1024 * 1024
    UVICORN_WORKERS: int = 1


settings = Settings()
