"""Jinja2 templates environment, extracted from `main` to break circular imports.

Routers historically wrote ``from src.main import templates`` — but
``src.main`` calls ``register_all_routers(app)`` at import time, which loads
the routers right back, producing a partial-init ImportError.  Importing
``templates`` from this leaf module has no side effects, so any router (or
test) can safely do ``from src.templates_env import templates``.

We also expose a small ``CompatTemplates`` wrapper that translates the
Starlette 0.x signature ``TemplateResponse(name, {request: ...})`` to the
Starlette 1.x required signature ``TemplateResponse(request, name, ctx)``.
The project's routers were written against the 0.x signature; calling them
against Starlette 1.x without this shim raises ``TypeError: cannot use
'tuple' as a dict key (unhashable type: 'dict')`` because Starlette 1.x
treats the 2nd positional as ``request`` and ends up mis-shaping the
template cache key.
"""

from __future__ import annotations

from typing import Any, Mapping

from starlette.requests import Request
from starlette.templating import Jinja2Templates


class CompatTemplates(Jinja2Templates):
    """Backwards-compatible wrapper around ``Jinja2Templates``.

    Accepts both:
        ``templates.TemplateResponse(name, ctx)``          (Starlette 0.x)
        ``templates.TemplateResponse(request, name, ctx)`` (Starlette 1.x)

    The first form is detected when the first positional is a ``str``
    (the template name); the second form is detected when it is a
    ``Request``.  Either arms the right keyword path so that the underlying
    Starlette 1.x implementation never sees a bare dict as a cache key.
    """

    def TemplateResponse(  # type: ignore[override]
        self,
        first: Request | str,
        second: str | Mapping[str, Any] | None = None,
        third: Mapping[str, Any] | None = None,
        **kwargs: Any,
    ) -> Any:
        if isinstance(first, Request):
            # New-style: (request, name, ctx?)
            request: Request = first
            name: str = second  # type: ignore[assignment]
            ctx = third or {}
            if "request" not in ctx:
                ctx = {**ctx, "request": request}
            return super().TemplateResponse(request, name, ctx, **kwargs)

        # Old-style: (name, ctx) with ctx["request"] mandatory.
        name = first  # type: ignore[assignment]
        ctx = second or {}
        if not isinstance(ctx, Mapping):
            raise TypeError(
                "CompatTemplates.TemplateResponse(name, ctx) requires ctx to be a mapping"
            )
        request = ctx.get("request")
        if request is None or not isinstance(request, Request):
            raise TypeError(
                "CompatTemplates.TemplateResponse(name, ctx) requires ctx['request'] to be a starlette Request"
            )
        return super().TemplateResponse(request, name, ctx, **kwargs)


templates: CompatTemplates = CompatTemplates(directory="src/templates")

