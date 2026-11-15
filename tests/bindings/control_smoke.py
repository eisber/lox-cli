"""Installed native wheel control selection; no transport or catalog discovery."""

from pathlib import Path
import os
import sys
import unittest

import lox_archive as lox


def catalog():
    return [
        lox.Control(name="Licht Extra", uuid="opaque-first", typ="Switch",
                    room="KÜCHE", cat="Indoor", is_favorite=True, is_secured=False),
        lox.Control(name="Licht", uuid="opaque-exact", typ="LightController2",
                    room=None, cat=None, is_favorite=False, is_secured=True),
        lox.Control(name="Panel [West]", uuid="nested", typ="Display",
                    room="KÜCHE", cat="", is_favorite=False, is_secured=False),
        lox.Control(name="Licht", uuid="spaced", typ="Dimmer",
                    room=" Küche ", cat=None, is_favorite=True, is_secured=True),
        lox.Control(name="Other", uuid="roomless", typ="State",
                    room=None, cat=None, is_favorite=False, is_secured=False),
        lox.Control(name="Open [", uuid="unclosed", typ="Switch",
                    room=None, cat=None, is_favorite=False, is_secured=False),
        lox.Control(name="Straße", uuid="unicode", typ="Sensor",
                    room="ROOM", cat="x", is_favorite=False, is_secured=True),
        lox.Control(name="İSTANBUL", uuid="dotted", typ="Sensor",
                    room="ROOM", cat="x", is_favorite=False, is_secured=True),
        lox.Control(name="Literal [tag] ", uuid="trailing", typ="Switch",
                    room=None, cat=None, is_favorite=False, is_secured=False),
    ]


def resolve(query, controls, room=None, aliases=None):
    prepared = lox.prepare_control_query(query, room, aliases or {})
    return lox.resolve_control_query(prepared, controls)


class ControlSmoke(unittest.TestCase):
    def test_identifiers_win_before_aliases_without_catalog_or_length_cap(self):
        for value in ("-" * 21, "-" * 51, "1FBC668C-005C-7471-FFFFED57184A04D2"):
            with self.subTest(value=value):
                prepared = lox.prepare_control_query(value, "ignored", {value: "must-not-win"})
                self.assertIsInstance(prepared, lox.PreparedControlQuery.RESOLVED)
                self.assertIsInstance(prepared.control, lox.ResolvedControl)
                self.assertEqual(prepared.control.uuid, value)
                self.assertEqual(lox.resolve_control_query(prepared, []).uuid, value)

    def test_aliases_are_exact_raw_nonrecursive_and_not_path_validated(self):
        aliases = {"Alias [Room]": "../raw-alias", "../raw-alias": "must-not-recurse", "": ""}
        prepared = lox.prepare_control_query("Alias [Room]", "ignored", aliases)
        aliases["Alias [Room]"] = "changed-after-prepare"
        self.assertIsInstance(prepared, lox.PreparedControlQuery.RESOLVED)
        self.assertEqual(lox.resolve_control_query(prepared, []).uuid, "../raw-alias")
        self.assertEqual(lox.resolve_control_query(prepared, []).uuid, "../raw-alias")
        self.assertEqual(resolve("", [], aliases=aliases).uuid, "")
        self.assertIsInstance(
            lox.prepare_control_query("alias [Room]", None, aliases),
            lox.PreparedControlQuery.CATALOG,
        )
        self.assertEqual(resolve("other", catalog(), aliases={"Other": "alias"}).uuid, "roomless")

    def test_prepare_retains_an_owned_reusable_request_and_success_is_typed(self):
        query = " Panel [West] [küche]"
        prepared = lox.prepare_control_query(query, "ignored", {})
        self.assertIsInstance(prepared, lox.PreparedControlQuery.CATALOG)
        self.assertEqual((prepared.query, prepared.room_filter), (query, "ignored"))
        for _ in range(2):
            result = lox.resolve_control_query(prepared, catalog())
            self.assertIsInstance(result, lox.ResolvedControl)
            self.assertEqual(result.uuid, "nested")

    def test_ambiguity_keeps_order_all_seven_fields_and_has_no_exact_name_bonus(self):
        controls = catalog()
        with self.assertRaises(lox.ControlResolutionError.Ambiguous) as caught:
            resolve("Licht", controls)
        error = caught.exception
        self.assertIsInstance(error, lox.ControlResolutionError)
        self.assertEqual(error.query, "Licht")
        self.assertIsInstance(error.candidates, list)
        self.assertTrue(all(isinstance(value, lox.Control) for value in error.candidates))
        self.assertEqual(error.candidates, [controls[0], controls[1], controls[3]])
        controls[0].name = "changed-after-error"
        self.assertEqual(error.candidates[0].name, "Licht Extra")
        with self.assertRaises(lox.ControlResolutionError.Ambiguous) as duplicated:
            resolve("Other", [controls[4], controls[4]])
        self.assertEqual(duplicated.exception.candidates, [controls[4], controls[4]])

    def test_not_found_retains_original_not_trimmed_or_parsed_query(self):
        for query in (" Missing [ Küche ]", " Licht ", "Panel [West] [küche", " Licht [küche] "):
            with self.subTest(query=query):
                with self.assertRaises(lox.ControlResolutionError.NotFound) as caught:
                    resolve(query, catalog())
                self.assertIsInstance(caught.exception, lox.ControlResolutionError)
                self.assertEqual(caught.exception.query, query)

    def test_rightmost_room_literal_whitespace_empty_override_and_open_bracket(self):
        for query, room, expected in (
            (" Licht [ küche ]", "missing", "spaced"),
            (" Panel [West] [küche]", "missing", "nested"),
            ("Other []", "missing", "roomless"),
            ("", " küche ", "spaced"),
            ("Open [", None, "unclosed"),
            ("Literal [tag] ", None, "trailing"),
        ):
            with self.subTest(query=query, room=room):
                self.assertEqual(resolve(query, catalog(), room).uuid, expected)
        with self.assertRaises(lox.ControlResolutionError.NotFound):
            resolve("Licht [ küche  ]", catalog(), "küche")

    def test_empty_query_room_and_unicode_lowercase_are_not_ranking_or_casefold(self):
        controls = catalog()
        for room in (None, ""):
            with self.assertRaises(lox.ControlResolutionError.Ambiguous) as caught:
                resolve("", controls, room)
            self.assertEqual(caught.exception.candidates, controls)
        self.assertEqual(resolve("straße", controls).uuid, "unicode")
        self.assertEqual(resolve("i\u0307stan", controls).uuid, "dotted")
        for query in ("STRASSE", "ISTANBUL"):
            with self.assertRaises(lox.ControlResolutionError.NotFound):
                resolve(query, controls)

    def test_identifier_detection_is_not_path_validation_and_errors_stay_typed(self):
        for value, expected in (
            ("a", False), ("-", False), ("-" * 20, False), ("-" * 21, True),
            ("-" * 51, True), ("a" * 30, False), ("é" * 30 + "-", False),
        ):
            self.assertIs(lox.is_uuid(value), expected)
        for value in ("a", "-", "-" * 50):
            self.assertIsNone(lox.validate_uuid(value))
        for value, error_type in (
            ("", lox.IdentifierError.InvalidLength),
            ("-" * 51, lox.IdentifierError.InvalidLength),
            ("é" * 26, lox.IdentifierError.InvalidLength),
            ("/" * 51, lox.IdentifierError.InvalidLength),
            ("..", lox.IdentifierError.InvalidCharacters),
            ("a/b", lox.IdentifierError.InvalidCharacters),
            ("a\\b", lox.IdentifierError.InvalidCharacters),
            ("a%b", lox.IdentifierError.InvalidCharacters),
            ("a.b", lox.IdentifierError.NonHex),
            ("é" * 25, lox.IdentifierError.NonHex),
            ("opaque-id", lox.IdentifierError.NonHex),
        ):
            with self.subTest(value=value):
                with self.assertRaises(error_type) as caught:
                    lox.validate_uuid(value)
                self.assertIsInstance(caught.exception, lox.IdentifierError)


if __name__ == "__main__":
    module = Path(lox.__file__).resolve()
    if not module.is_relative_to(Path(sys.prefix).resolve()):
        raise AssertionError(f"not the installed native package: {module}")
    print(f"Python native control API: {module}; RUST_MIN_STACK absent")
    assert "RUST_MIN_STACK" not in os.environ
    unittest.main(verbosity=2)
