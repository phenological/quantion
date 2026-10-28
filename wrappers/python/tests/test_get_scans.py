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

    def test_one_rt_gives_the_closest_scan(self):
        scan = support.quantion.get_scans(self.file, rt=2.385)
        self.assertAlmostEqual(scan["rt"], 2.385, delta=0.05)

    def test_rt_and_mz_together_are_refused(self):
        with self.assertRaisesRegex(ValueError, "exactly one of rt or mz"):
            support.quantion.get_scans(self.file, rt=2.385, mz=89.0)

    def test_a_range_without_to_is_refused(self):
        with self.assertRaisesRegex(ValueError, "rt must be a dict"):
            support.quantion.get_scans(self.file, rt={"from": 2.3})

    def test_the_old_tuple_names_are_gone(self):
        with self.assertRaises(TypeError):
            support.quantion.get_scans(self.file, rt_range=(2.3, 2.5))
