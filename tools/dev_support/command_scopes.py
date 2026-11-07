"""Command report scope registry assembled from command modules."""

from .commands.abi import SCOPES as ABI_SCOPES
from .commands.cpp import SCOPES as CPP_SCOPES
from .commands.csharp import SCOPES as CSHARP_SCOPES
from .commands.python import SCOPES as PYTHON_SCOPES
from .commands.rust import SCOPES as RUST_SCOPES


COMMAND_SCOPES = {
    **ABI_SCOPES,
    **CPP_SCOPES,
    **CSHARP_SCOPES,
    **PYTHON_SCOPES,
    **RUST_SCOPES,
}
DEFAULT_SCOPE = "development-bootstrap"


def scope_for(command):
    """Return the existing report scope for a command."""
    return COMMAND_SCOPES.get(command, DEFAULT_SCOPE)
