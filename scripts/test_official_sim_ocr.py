import hashlib
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from official_sim_ocr import OCRBlocked, grayscale_pixel, literal_consensus, parse_clock_tsv, parse_header_tsv, parse_literal_tsv, recognize_literal


HEADER = "level\tleft\ttop\twidth\theight\tconf\ttext\n"


def word(text, confidence=95):
    return f"5\t2\t3\t20\t10\t{confidence}\t{text}\n"


class NativeOCRTests(unittest.TestCase):
    def test_exact_native_words_not_expected_scenario_values(self):
        for text, value in [("Ein", 1), ("Aus", 0)]:
            actual = parse_literal_tsv(HEADER + word(text), (100, 100))
            self.assertEqual(actual["value"], value)
            self.assertEqual(actual["text"], text)

    def test_unknown_low_confidence_or_conflicting_text_is_blocked(self):
        for rows in [word("En"), word("Ein", 89), word("Aus", 73), word("Ein") + word("Aus"), ""]:
            with self.subTest(rows=rows), self.assertRaises(OCRBlocked):
                parse_literal_tsv(HEADER + rows, (100, 100))

    def test_invalid_numeric_fields_and_bounds_are_not_admitted(self):
        for rows in [word("Ein", "nan"), word("Ein", "inf"), word("Ein", 101),
                     "5\t-1\t3\t20\t10\t95\tEin\n", "5\t99\t3\t20\t10\t95\tEin\n",
                     "5\tbad\t3\t20\t10\t95\tEin\n", "5\t2\t3\t20\t10\t95\tEin\textra\n"]:
            with self.subTest(rows=rows), self.assertRaises(OCRBlocked):
                parse_literal_tsv(HEADER + rows, (100, 100))

    def test_malformed_header_or_nonword_text_is_blocked(self):
        for text in ["confidence\ttext\n95\tEin\n", HEADER + "4\t2\t3\t20\t10\t95\tEin\n",
                     HEADER.rstrip("\n") + "\ttext\n" + word("Ein").rstrip("\n") + "\tEin\n"]:
            with self.assertRaises(OCRBlocked):
                parse_literal_tsv(text, (100, 100))

    def test_fixed_pixel_rule_removes_green_wire_not_gray_text(self):
        self.assertEqual(grayscale_pixel((10, 180, 20)), 255)
        self.assertEqual(grayscale_pixel((20, 20, 20)), 20)
        self.assertEqual(grayscale_pixel((100, 80, 200)), 80)

    def test_header_requires_two_actual_exact_high_confidence_words(self):
        actual = parse_header_tsv(HEADER + word("Energy", 95) + word("Manager", 96), (100, 100))
        self.assertEqual(actual["text"], "Energy Manager")
        for rows in [word("Energy") + word("Manage"), word("Energy") + word("Manager", 89),
                     word("Energy Manager"), word("Energy") + word("Mar-eger")]:
            with self.subTest(rows=rows), self.assertRaises(OCRBlocked):
                parse_header_tsv(HEADER + rows, (100, 100))

    def test_native_clock_is_strict_without_time_guessing(self):
        actual = parse_clock_tsv(HEADER + word("20:00:12"), (100, 100))
        self.assertEqual(actual["seconds_since_midnight"], 72012)
        for rows in [word("24:00:00"), word("20:60:00"), word("20:00:60"),
                     word("2:00:12"), word("20:00:12", 89), word("20:00:12") + word("PM"), ""]:
            with self.subTest(rows=rows), self.assertRaises(OCRBlocked):
                parse_clock_tsv(HEADER + rows, (100, 100))

    def test_wrong_tool_identity_fails_before_image_processing_or_execution(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            executable = root / "ocr.exe"
            executable.write_bytes(b"independent fake executable")
            model = root / "deu.traineddata"
            model.write_bytes(b"independent fake model")
            profile = {
                "schema_version": 1, "kind": "native-tesseract-ocr-profile", "language": "deu",
                "executable_path": str(executable), "executable_sha256": "0" * 64,
                "model_path": str(model), "model_sha256": hashlib.sha256(model.read_bytes()).hexdigest(),
            }
            with patch("official_sim_ocr.subprocess.run") as run:
                with self.assertRaisesRegex(OCRBlocked, "hash mismatch"):
                    recognize_literal(root / "absent.png", (0, 0, 1, 1), profile, root / "result")
                run.assert_not_called()
                self.assertFalse((root / "result").exists())

    def test_fixed_candidates_admit_both_states_without_lowering_threshold(self):
        on = literal_consensus([
            {"words": [{"text": "Ein", "confidence": 95}]},
            {"words": [{"text": "In", "confidence": 43}]},
        ])
        off = literal_consensus([
            {"words": [{"text": "Aus", "confidence": 73}]},
            {"words": [{"text": "Aus", "confidence": 96}]},
        ])
        self.assertEqual((on["value"], off["value"]), (1, 0))
        self.assertEqual(off["confidence"], 96)

    def test_fixed_candidates_block_disagreement_high_unknown_and_extra_words(self):
        known = {"words": [{"text": "Ein", "confidence": 95}]}
        for other in [
            {"words": [{"text": "Aus", "confidence": 95}]},
            {"words": [{"text": "Aus", "confidence": 12}]},
            {"words": [{"text": "Energy", "confidence": 95}]},
            {"words": [{"text": "Ein", "confidence": 95}, {"text": "extra", "confidence": 1}]},
        ]:
            with self.subTest(other=other), self.assertRaises(OCRBlocked):
                literal_consensus([known, other])
        for candidates in [[{"words": []}, {"words": []}],
                           [{"words": [{"text": "Aus", "confidence": 89}]}, {"words": []}]]:
            with self.assertRaises(OCRBlocked):
                literal_consensus(candidates)
        for confidence in (float("nan"), float("inf"), -1, 101, True):
            with self.subTest(confidence=confidence), self.assertRaises(OCRBlocked):
                literal_consensus([known, {"words": [{"text": "Ein", "confidence": confidence}]}])

    def test_literal_v2_does_not_skip_the_second_candidate_after_a_positive(self):
        with tempfile.TemporaryDirectory() as temporary:
            profile = {"literal_recipe": "dual-gray-tight-v2"}
            source = {"path": "independent.png", "sha256": "a" * 64}
            candidates = [
                {"source": source, "words": [{"text": "Ein", "confidence": 95}]},
                {"source": source, "words": [{"text": "Aus", "confidence": 95}]},
            ]
            with patch("official_sim_ocr._pinned_path"), \
                    patch("official_sim_ocr._recognize", side_effect=candidates) as run:
                with self.assertRaisesRegex(OCRBlocked, "disagree"):
                    recognize_literal("independent.png", (0, 0, 25, 17), profile, Path(temporary) / "read")
                self.assertEqual(run.call_count, 2)

    def test_literal_v2_rejects_clipped_binding_before_running_ocr(self):
        with patch("official_sim_ocr._recognize") as run:
            with self.assertRaisesRegex(OCRBlocked, "calibrated"):
                recognize_literal("independent.png", (0, 0, 8, 8),
                                  {"literal_recipe": "dual-gray-tight-v2"}, "unused")
            run.assert_not_called()


if __name__ == "__main__":
    unittest.main()
