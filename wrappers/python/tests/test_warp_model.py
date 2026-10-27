import math
import unittest

from tests import support

MODEL = support.REPO_ROOT / "core" / "tests" / "fixtures" / "shift_model.onnx"
EMPTY_FILE = support.REPO_ROOT / "core" / "tests" / "fixtures" / "test.ion"
TARGETS = [
    {"id": "first", "mz": 250.1, "rt": 1.2},
    {"id": "second", "mz": 310.2, "rt": 3.1},
]


@unittest.skipIf(support.library_problem, support.library_problem)
@unittest.skipIf(support.fixture_problem, support.fixture_problem)
class WarpModel(unittest.TestCase):
    def setUp(self):
        self.file = support.quantion.parse_ion_path(str(support.ION_FIXTURE))
        self.addCleanup(self.file.dispose)

    def load(self, source):
        warp = support.quantion.load_warp_model(source)
        self.addCleanup(warp.dispose)
        return warp

    def test_one_prediction_per_target_in_order(self):
        predicted = support.quantion.predict_rts(self.load(str(MODEL)), self.file, TARGETS)
        self.assertEqual(len(predicted), len(TARGETS))
        for prediction in predicted:
            self.assertEqual(set(prediction), {"rt", "uncertainty"})
            self.assertTrue(math.isfinite(prediction["rt"]))

    def test_a_model_from_bytes_predicts_the_same(self):
        from_path = support.quantion.predict_rts(self.load(str(MODEL)), self.file, TARGETS)
        from_bytes = support.quantion.predict_rts(self.load(MODEL.read_bytes()), self.file, TARGETS)
        self.assertEqual(from_path, from_bytes)

    def test_a_model_from_a_url_predicts_the_same(self):
        server = support.RangeServer(MODEL)
        self.addCleanup(server.stop)
        from_path = support.quantion.predict_rts(self.load(str(MODEL)), self.file, TARGETS)
        from_url = support.quantion.predict_rts(self.load(server.url), self.file, TARGETS)
        self.assertEqual(from_path, from_url)

    def test_a_remote_file_predicts_like_a_local_one(self):
        server = support.RangeServer(support.ION_FIXTURE)
        self.addCleanup(server.stop)
        remote = support.quantion.parse_ion_remote(server.url)
        self.addCleanup(remote.dispose)
        warp = self.load(str(MODEL))
        local = support.quantion.predict_rts(warp, self.file, TARGETS)
        self.assertEqual(support.quantion.predict_rts(warp, remote, TARGETS), local)

    def test_a_target_without_mz_is_named_by_its_index(self):
        targets = [TARGETS[0], {"id": "broken", "rt": 2.0}]
        with self.assertRaisesRegex(RuntimeError, "predict_rts: target 2 has no valid m/z or rt"):
            support.quantion.predict_rts(self.load(str(MODEL)), self.file, targets)

    def test_a_file_without_scans_in_the_model_range_is_refused(self):
        empty = support.quantion.parse_ion_path(str(EMPTY_FILE))
        self.addCleanup(empty.dispose)
        with self.assertRaisesRegex(RuntimeError, "predict_rts: the file has no scans, the warp model needs"):
            support.quantion.predict_rts(self.load(str(MODEL)), empty, TARGETS)

    def test_a_missing_model_is_refused(self):
        with self.assertRaisesRegex(RuntimeError, "load_warp_model: the warp model was not read"):
            support.quantion.load_warp_model(str(MODEL.with_name("missing.onnx")))
