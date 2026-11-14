from __future__ import annotations

import ctypes
from typing import Optional, Sequence

_FIELD_SEPARATOR = "\x1f"

_KIND_CODES = {
    "invalid_argument": 1,
    "missing_argument": 2,
    "unknown_option": 3,
    "remote_file": 4,
    "file_not_found": 5,
    "parse_failed": 6,
    "model_mismatch": 7,
    "file_exists": 8,
    "upload_failed": 9,
    "internal": 99,
}


class QuantionError(Exception):
    """QuantionError · Errors

    The error every function raises. It says what went wrong (kind and code), where (function and argument) and why (message).

    Fields:
        kind: The type of problem, for example an invalid input or a file that cannot be read.
        code: A fixed number for the kind, useful in scripts.
        function: The function that failed.
        argument: The input that caused it, such as targets[3].range.
        message: Why it failed, in plain words. It starts with the function name.

    Documentation: https://phenological.github.io/quantion/?language=python#QuantionError
    """
    def __init__(
        self,
        kind: str,
        function: str,
        message: str,
        argument: Optional[str] = None,
        code: Optional[int] = None,
    ) -> None:
        super().__init__(f"{function}: {message}")
        self.kind = kind
        self.code = _KIND_CODES.get(kind, 99) if code is None else code
        self.function = function
        self.argument = argument
        self.message = message


def _invalid(function: str, argument: str, message: str) -> QuantionError:
    return QuantionError("invalid_argument", function, f"{argument} {message}", argument)


def _missing(function: str, argument: str) -> QuantionError:
    return QuantionError("missing_argument", function, f"{argument} is required", argument)


def _unknown_option(function: str, argument: str, allowed: Sequence[str]) -> QuantionError:
    message = f"{argument} is not an option; the options are {', '.join(allowed)}"
    return QuantionError("unknown_option", function, message, argument)


def _read_last_error(abi, function: str, code: int) -> QuantionError:
    from quantion._bridge import _Buf, buf_to_str

    buffer = _Buf()
    abi.quantion_last_error(ctypes.byref(buffer))
    text = buf_to_str(abi, buffer) if buffer.len else ""
    parts = text.split(_FIELD_SEPARATOR)
    if len(parts) != 5:
        return QuantionError("internal", function, f"the call failed with code {code}", code=code)
    error_code, kind, error_function, argument, message = parts
    return QuantionError(kind, error_function, message, argument or None, int(error_code))


def _check(abi, function: str, code: int) -> None:
    if code != 0:
        raise _read_last_error(abi, function, code)
