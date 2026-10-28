from typing import Any

def to_threads(threads: Any) -> int:
    try:
        v = int(threads)
    except (TypeError, ValueError):
        return 1
    return max(1, v)