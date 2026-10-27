import ctypes
from typing import TYPE_CHECKING, Any, Dict, List, Sequence

from quantion.model import Metadata

if TYPE_CHECKING:
    from quantion._bridge import _ABI
    from quantion.file import SampleFile


class WarpModel(Metadata):
    def __init__(
        self, handle: ctypes.c_void_p, abi: "_ABI", metadata: List[Dict[str, str]]
    ) -> None:
        self._handle: ctypes.c_void_p | None = handle
        self._abi = abi
        self._metadata = metadata

    @property
    def handle(self) -> ctypes.c_void_p:
        if self._handle is None:
            raise RuntimeError("WarpModel: model has been disposed")
        return self._handle

    def _get_metadata(self) -> List[Dict[str, str]]:
        return self._metadata

    def predict_rts(
        self, file: "SampleFile", targets: Sequence[Dict[str, Any]]
    ) -> List[Dict[str, float]]:
        """Predict each target's RT in this file.

        Args:
            file: The sample file.
            targets: Dicts with "mz" and "rt".

        Returns:
            One {"rt", "uncertainty"} per target, in the same order.
        """
        from quantion.api import _predict_rts

        return _predict_rts(self, file, targets)

    def calibrate(
        self,
        anchor_samples: Sequence["SampleFile"],
        target_lists: Sequence[Sequence[Dict[str, Any]]],
    ) -> List[Dict[str, Any]]:
        """Turn the RTs seen in anchor samples into targets for predict_rts.

        Args:
            anchor_samples: The samples where the targets were seen, like LTRs or QCs.
            target_lists: One list per sample, with dicts of "id", "mz" and the "rt" seen in it.

        Returns:
            One {"id", "mz", "rt"} per id, with the median calibrated rt over the samples.
        """
        from quantion.api import _calibrate_rts

        return _calibrate_rts(self, anchor_samples, target_lists)

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
