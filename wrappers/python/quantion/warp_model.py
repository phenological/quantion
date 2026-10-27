import ctypes
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from quantion._bridge import _ABI


class WarpModel:
    def __init__(self, handle: ctypes.c_void_p, abi: "_ABI") -> None:
        self._handle: ctypes.c_void_p | None = handle
        self._abi = abi

    @property
    def handle(self) -> ctypes.c_void_p:
        if self._handle is None:
            raise RuntimeError("WarpModel: model has been disposed")
        return self._handle

    def dispose(self) -> None:
        if self._handle is not None:
            self._abi.warp_model_free(self._handle)
            self._handle = None

    def __del__(self) -> None:
        self.dispose()

    def __enter__(self) -> "WarpModel":
        return self

    def __exit__(self, *_: Any) -> None:
        self.dispose()
