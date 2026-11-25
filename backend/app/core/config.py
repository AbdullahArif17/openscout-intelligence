from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    """Application configuration loaded from environment variables."""

    app_name: str = "OpenScout Intelligence API"
    environment: str = "development"
    database_url: str = (
        "postgresql+asyncpg://openscout:openscout_dev_password@localhost:5432/openscout_db"
    )

    model_config = SettingsConfigDict(env_file=".env", extra="ignore")


settings = Settings()
