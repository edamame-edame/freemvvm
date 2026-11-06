"""Build the child-process environment used by bootstrap commands."""


def bootstrap_environment(environment, bin_dir, path_separator):
    """Return the pinned UTF-8/.NET environment with the venv first on PATH."""
    result = dict(environment)
    result["PYTHONUTF8"] = "1"
    result["PYTHONIOENCODING"] = "utf-8"
    result["DOTNET_CLI_TELEMETRY_OPTOUT"] = "1"
    result["DOTNET_NOLOGO"] = "1"
    result["PATH"] = (
        f"{bin_dir}{path_separator}{result.get('PATH', '')}"
    )
    return result
