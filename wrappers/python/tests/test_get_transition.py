import unittest

from tests import support

ALANINE = 89.04768
SERINE = 90.0555


@unittest.skipIf(support.library_problem, support.library_problem)
class GetTransition(unittest.TestCase):
    def setUp(self):
        self.file = support.quantion.parse_ion(str(support.SRM_FIXTURE))
        self.addCleanup(self.file.dispose)

    def read(self, query):
        return support.quantion.get_transition(self.file, query)

    def assert_refused(self, query, argument):
        with self.assertRaises(support.quantion.QuantionError) as caught:
            self.read(query)
        self.assertEqual(caught.exception.kind, "invalid_argument")
        self.assertEqual(caught.exception.argument, argument)

    def test_an_index_reads_each_stored_chromatogram(self):
        for index in (0, 1):
            trace = self.read(index)
            self.assertGreater(len(trace["x"]), 0)
            self.assertEqual(len(trace["x"]), len(trace["y"]))

    def test_an_index_past_the_end_gives_an_empty_chromatogram(self):
        trace = self.read(2)
        self.assertEqual(len(trace["x"]), 0)
        self.assertEqual(len(trace["y"]), 0)

    def test_a_transition_reads_the_same_chromatogram_as_its_index(self):
        trace = self.read({"precursor": ALANINE, "product": ALANINE})
        by_index = [list(self.read(index)["y"]) for index in (0, 1)]
        self.assertGreater(len(trace["y"]), 0)
        self.assertIn(list(trace["y"]), by_index)

    def test_a_transition_matches_within_the_tolerance(self):
        near = self.read({"precursor": 89.0526, "product": 89.0436})
        exact = self.read({"precursor": ALANINE, "product": ALANINE})
        self.assertEqual(list(near["y"]), list(exact["y"]))

    def test_a_wrong_product_gives_an_empty_chromatogram(self):
        trace = self.read({"precursor": ALANINE, "product": SERINE})
        self.assertEqual(len(trace["x"]), 0)

    def test_a_negative_index_is_refused(self):
        self.assert_refused(-1, "query")

    def test_a_query_without_a_product_is_refused(self):
        self.assert_refused({"precursor": ALANINE}, "query")

    def test_a_text_query_is_refused(self):
        self.assert_refused("0", "query")

    def test_a_negative_precursor_is_refused(self):
        self.assert_refused({"precursor": -1, "product": ALANINE}, "precursor")


@unittest.skipIf(support.library_problem, support.library_problem)
class GetTransitionMetadata(unittest.TestCase):
    def setUp(self):
        self.file = support.quantion.parse_ion(str(support.SRM_FIXTURE))
        self.addCleanup(self.file.dispose)

    def read(self, query, options=None):
        return support.quantion.get_transition_metadata(self.file, query, options)

    def assert_refused(self, query, argument, options=None):
        with self.assertRaises(support.quantion.QuantionError) as caught:
            self.read(query, options)
        self.assertEqual(caught.exception.kind, "invalid_argument")
        self.assertEqual(caught.exception.argument, argument)

    def test_an_index_reads_every_row_of_its_chromatogram(self):
        rows = self.read(0)
        self.assertEqual(len(rows), 11)
        self.assertEqual(
            [(row["path"], row["name"], row["value"]) for row in rows[:5]],
            [
                ("chromatogram", "index", "0"),
                ("chromatogram", "id", "SRM SIC Q1=89.04768 Q3=89.04768"),
                ("chromatogram", "defaultArrayLength", "1035"),
                ("chromatogram/precursor/isolationWindow", "isolation window target m/z", "89.04768"),
                ("chromatogram/product/isolationWindow", "isolation window target m/z", "89.04768"),
            ],
        )
        self.assertEqual(rows[3]["kind"], support.quantion.ScanParamKind.CV)
        self.assertEqual(rows[3]["accession"], "MS:1000827")
        self.assertIsNone(rows[0]["accession"])

    def test_a_transition_reads_the_same_rows_as_its_index(self):
        self.assertEqual(self.read({"precursor": SERINE, "product": SERINE}), self.read(1))
        self.assertEqual(self.read({"precursor": 89.0526, "product": 89.0436}), self.read(0))

    def test_tags_keep_only_their_rows(self):
        self.assertEqual(len(self.read(0, {"tags": ["chromatogram"]})), 3)
        windows = self.read(0, {"tags": ["isolationWindow"]})
        self.assertEqual(
            [row["path"] for row in windows],
            ["chromatogram/precursor/isolationWindow", "chromatogram/product/isolationWindow"],
        )

    def test_no_matching_chromatogram_gives_no_rows(self):
        self.assertEqual(self.read(2), [])
        self.assertEqual(self.read({"precursor": ALANINE, "product": SERINE}), [])

    def test_an_unknown_tag_is_refused(self):
        self.assert_refused(0, "tags", {"tags": ["scan"]})

    def test_a_negative_index_is_refused(self):
        self.assert_refused(-1, "query")

    def test_a_negative_product_is_refused(self):
        self.assert_refused({"precursor": ALANINE, "product": -1}, "product")


if __name__ == "__main__":
    unittest.main()
