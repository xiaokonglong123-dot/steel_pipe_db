"""Modules package — register_all_routers(app) wires up all domain routers."""


def register_all_routers(app) -> None:
    """Iterate known module routers and include them on *app*."""
    from src.modules.auth.routers.auth import router as auth_router
    from src.modules.users.routers.users import router as users_router
    from src.modules.warehouse.routers.audit import router as audit_router
    from src.modules.warehouse.routers.standards import router as standards_router
    from src.modules.warehouse.routers.tubes import router as tubes_router

    app.include_router(auth_router)
    app.include_router(audit_router)
    app.include_router(standards_router)
    app.include_router(tubes_router)
    app.include_router(users_router)
