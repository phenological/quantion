import unittest

from tests import support

MASS = 89.04768
TARGET_RT = 2.385


@unittest.skipIf(support.library_problem, support.library_problem)
@unittest.skipIf(support.fixture_problem, support.fixture_problem)
class FitAndDraw(unittest.TestCase):
    def setUp(self):
        file = support.quantion.parse_ion(str(support.ION_FIXTURE))
        self.addCleanup(file.dispose)
        self.eic = support.quantion.calculate_eic(file, MASS, {"from": 0.0, "to": 5.0})
        self.peak = support.quantion.get_peak(self.eic["x"], self.eic["y"], TARGET_RT, 0.1)

    def fit(self, shape):
        return support.quantion.fit_peak(
            self.eic["x"], self.eic["y"], self.peak["rt"], self.peak["intensity"], shape
        )

    def test_the_fit_names_its_shape(self):
        self.assertEqual(self.fit("emg")["shape"], "emg")
        self.assertEqual(self.fit("gaussian")["shape"], "gaussian")

    def test_a_fit_can_be_drawn(self):
        curve = support.quantion.draw_peak(self.eic["x"], self.fit("emg"))
        self.assertEqual(len(curve), len(self.eic["x"]))
