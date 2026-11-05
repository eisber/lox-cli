"""Fail-closed, file-only OCR for the bounded native-output pilot."""

import csv
import hashlib
import io
import json
import math
import os
from pathlib import Path
import platform
import re
import subprocess


class OCRBlocked(ValueError):
    """The actual image cannot be admitted as a native output observation."""


def _require(condition, message):
    if not condition:
        raise OCRBlocked(message)


def _digest(path):
    result = hashlib.sha256()
    with Path(path).open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            result.update(chunk)
    return result.hexdigest()


def grayscale_pixel(rgb):
    red, green, blue = rgb
    return 255 if green > red + 30 and green > blue + 30 else min(rgb)


def _parse_words(text, image_size):
    reader = csv.DictReader(io.StringIO(text), delimiter="\t")
    required = {"level", "left", "top", "width", "height", "conf", "text"}
    _require(reader.fieldnames is not None and required <= set(reader.fieldnames)
             and len(reader.fieldnames) == len(set(reader.fieldnames)), "Malformed OCR TSV header.")
    words = []
    try:
        for row in reader:
            _require(None not in row and all(row.get(key) is not None for key in required),
                     "Malformed OCR TSV row.")
            if row["text"].strip():
                _require(int(row["level"]) == 5, "OCR text is not a word-level observation.")
                box = [int(row[key]) for key in ("left", "top", "width", "height")]
                left, top, width, height = box
                _require(left >= 0 and top >= 0 and width > 0 and height > 0
                         and left + width <= image_size[0] and top + height <= image_size[1],
                         "OCR word lies outside the processed image.")
                confidence = float(row["conf"])
                _require(math.isfinite(confidence) and 0 <= confidence <= 100, "Invalid OCR confidence.")
                words.append({"text": row["text"].strip(), "confidence": confidence, "box": box})
    except (TypeError, ValueError) as error:
        if isinstance(error, OCRBlocked):
            raise
        raise OCRBlocked("Malformed OCR numeric field.") from error
    return words


def parse_literal_tsv(text, image_size):
    words = _parse_words(text, image_size)
    _require(len(words) == 1, "Expected exactly one native output word.")
    word = words[0]
    _require(word["text"] in ("Ein", "Aus") and word["confidence"] >= 90,
             "Native output word is unknown or below the confidence threshold.")
    return {**word, "value": {"Ein": 1, "Aus": 0}[word["text"]]}


def parse_header_tsv(text, image_size):
    words = _parse_words(text, image_size)
    _require([word["text"] for word in words] == ["Energy", "Manager"]
             and all(word["confidence"] >= 90 for word in words),
             "Native header is not the exact high-confidence Energy Manager title.")
    return {"text": "Energy Manager", "confidence": min(word["confidence"] for word in words), "words": words}


def parse_clock_tsv(text, image_size):
    words = _parse_words(text, image_size)
    _require(len(words) == 1 and words[0]["confidence"] >= 90
             and re.fullmatch(r"(?:[01][0-9]|2[0-3]):[0-5][0-9]:[0-5][0-9]", words[0]["text"]) is not None,
             "Native clock is not one exact high-confidence HH:MM:SS value.")
    hours, minutes, seconds = (int(value) for value in words[0]["text"].split(":"))
    return {**words[0], "seconds_since_midnight": hours * 3600 + minutes * 60 + seconds}


def _prepare_image(image_path, box):
    try:
        from PIL import Image, ImageOps, __version__ as pillow_version
    except ImportError as error:
        raise OCRBlocked("File-only native OCR requires Pillow; install the documented optional dependency.") from error
    _require(len(box) == 4 and all(type(value) is int for value in box), "Expected an integer image rectangle.")
    with Image.open(image_path) as source:
        _require(source.format == "PNG", "Native OCR requires the original PNG capture.")
        _require(0 < source.width * source.height <= 16_000_000, "Native PNG exceeds the bounded image size.")
        left, top, right, bottom = box
        _require(0 <= left < right <= source.width and 0 <= top < bottom <= source.height,
                 "Native output rectangle is outside the actual bitmap.")
        _require((right - left) * (bottom - top) <= 100_000, "Native output rectangle is too large.")
        crop = source.crop(tuple(box)).convert("RGB")
        gray = Image.new("L", crop.size)
        gray.putdata([grayscale_pixel(pixel) for pixel in crop.getdata()])
        processed = ImageOps.expand(gray.resize(
            (gray.width * 4, gray.height * 4), Image.Resampling.BICUBIC
        ), border=24, fill="white")
        return processed, list(source.size), pillow_version


def _pinned_path(profile, prefix):
    path = profile.get(prefix + "_path")
    expected = profile.get(prefix + "_sha256")
    _require(isinstance(path, str) and path and isinstance(expected, str) and len(expected) == 64,
             f"Missing pinned OCR {prefix}.")
    path = Path(path).resolve(strict=True)
    _require(path.is_file() and _digest(path) == expected, f"OCR {prefix} hash mismatch.")
    return path, expected


def _recognize(image_path, box, profile, output_dir, language, model_prefix, parse_result, kind):
    _require(isinstance(profile, dict) and type(profile.get("schema_version")) is int
             and profile["schema_version"] == 1
             and profile.get("kind") == "native-tesseract-ocr-profile"
             and profile.get("language") == "deu", "Unsupported native OCR profile.")
    executable, executable_hash = _pinned_path(profile, "executable")
    model, model_hash = _pinned_path(profile, model_prefix)
    _require(model.name == language + ".traineddata", "Expected the pinned OCR language model.")
    source = Path(image_path).resolve(strict=True)
    _require(source.is_file() and source.stat().st_size <= 64 * 1024 * 1024, "Native image is missing or oversized.")
    source_hash = _digest(source)
    processed, original_size, pillow_version = _prepare_image(source, box)
    output = Path(output_dir).resolve()
    output.mkdir(parents=True, exist_ok=False)
    image = output / "processed.png"
    processed.save(image)
    allowed = {"systemroot", "windir", "path", "comspec", "pathext", "systemdrive"}
    environment = {key: value for key, value in os.environ.items() if key.lower() in allowed}
    environment.update({"TMP": str(output), "TEMP": str(output)})
    command = [
        str(executable), str(image), "stdout", "--tessdata-dir", str(model.parent),
        "-l", language, "--psm", "7", "tsv",
    ]
    try:
        result = subprocess.run(
            command, cwd=output, env=environment, capture_output=True, timeout=15, check=False,
            creationflags=subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0,
        )
    except subprocess.TimeoutExpired as error:
        raise OCRBlocked("Native OCR timed out; no output admitted.") from error
    stdout = output / "recognizer.tsv"
    stderr = output / "recognizer.stderr.txt"
    stdout.write_bytes(result.stdout)
    stderr.write_bytes(result.stderr)
    _require(result.returncode == 0, f"Native OCR exited {result.returncode}; see {stderr}.")
    _require(_digest(source) == source_hash, "Native source image changed during recognition.")
    word = parse_result(result.stdout.decode("utf-8-sig"), processed.size)
    observation = {
        "schema_version": 1, "kind": kind,
        "source": {"path": str(source), "sha256": source_hash, "image_size": original_size, "box": list(box)},
        "engine": {"name": "tesseract", "executable_sha256": executable_hash, "model_sha256": model_hash,
                   "language": language, "recipe": "minimum-rgb-green-wire-cleanup-4x-bicubic-pad24-psm7-v1",
                   "pillow_version": pillow_version, "python_version": platform.python_version(),
                   "observer_source_sha256": _digest(__file__)},
        **word,
        "artifacts": [
            {"path": str(path), "sha256": _digest(path)} for path in (image, stdout, stderr)
        ],
    }
    record = output / "observation.json"
    with record.open("x", encoding="utf-8") as file:
        json.dump(observation, file, indent=2, allow_nan=False)
        file.write("\n")
    return {**observation, "record_path": str(record), "record_sha256": _digest(record)}


def recognize_literal(image_path, box, profile, output_dir):
    """Return actual recognized text/value, never an expected scenario value."""
    return _recognize(image_path, box, profile, output_dir, "deu", "model",
                      parse_literal_tsv, "native-output-ocr")


def recognize_header(image_path, candidate_box, profile, output_dir):
    """Verify a proposed title crop independently; never correct a fuzzy OCR label."""
    _require(len(candidate_box) == 4 and all(type(value) is int for value in candidate_box),
             "Expected an integer header candidate rectangle.")
    left, top, right, bottom = candidate_box
    box = (left - 4, top - 4, right + 4, bottom + 4)
    return _recognize(image_path, box, profile, output_dir, "eng", "header_model",
                      parse_header_tsv, "native-header-ocr")


def recognize_clock(image_path, box, profile, output_dir):
    """Read the actual native clock; no expected time or inferred progress."""
    return _recognize(image_path, box, profile, output_dir, "eng", "header_model",
                      parse_clock_tsv, "native-clock-ocr")
