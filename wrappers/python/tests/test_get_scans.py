import unittest

from tests import support


@unittest.skipIf(support.library_problem, support.library_problem)
@unittest.skipIf(support.fixture_problem, support.fixture_problem)
class ScanQueries(unittest.TestCase):
    def setUp(self):
        self.file = support.quantion.parse_ion(str(support.ION_FIXTURE))
        self.addCleanup(self.file.dispose)

    def test_an_rt_range_gives_every_scan_inside_it(self):
        scans = support.quantion.get_scans(self.file, rt={"from": 2.3, "to": 2.5})
        self.assertGreater(len(scans), 0)
        self.assertTrue(all(2.3 <= scan["rt"] <= 2.5 for scan in scans))

    def test_one_rt_gives_a_list_with_the_closest_scan(self):
        scans = support.quantion.get_scans(self.file, rt=2.385)
        self.assertEqual(len(scans), 1)
        self.assertAlmostEqual(scans[0]["rt"], 2.385, delta=0.05)

    def test_seconds_find_the_same_scan(self):
        in_minutes = support.quantion.get_scans(self.file, rt=2.385)
        in_seconds = support.quantion.get_scans(self.file, rt=2.385 * 60, options={"time_unit": "SECONDS"})
        self.assertEqual(in_minutes[0]["rt"], in_seconds[0]["rt"])

    def test_a_scan_is_flat(self):
        scan = support.quantion.get_scans(self.file, rt=2.385)[0]
        self.assertEqual(
            list(scan),
            ["rt", "mz", "intensity", "level", "precursor_mz", "base_peak_mz", "base_peak_intensity",
             "total_ion_current", "polarity", "x", "y", "z"],
        )
        self.assertEqual(scan["level"], 1)

    def test_rt_and_mz_together_are_refused(self):
        with self.assertRaises(support.quantion.QuantionError) as caught:
            support.quantion.get_scans(self.file, rt=2.385, mz=89.0)
        self.assertEqual(caught.exception.argument, "mz")

    def test_a_range_without_to_is_refused(self):
        with self.assertRaises(support.quantion.QuantionError) as caught:
            support.quantion.get_scans(self.file, rt={"from": 2.3})
        self.assertEqual(caught.exception.kind, "missing_argument")
        self.assertEqual(caught.exception.argument, "rt.to")

    def test_the_old_tuple_names_are_gone(self):
        with self.assertRaises(TypeError):
            support.quantion.get_scans(self.file, rt_range=(2.3, 2.5))
