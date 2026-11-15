"""Installed-wheel color values and calibrated native release, never CLI/JSON."""

import ctypes
import importlib
import math
import os
from pathlib import Path
import sys

import lox_archive as lox


assert "RUST_MIN_STACK" not in os.environ
assert Path(lox.__file__).resolve().is_relative_to(Path(sys.prefix).resolve())
for line in Path(sys.argv[2]).read_text().splitlines():
    spec, value, r, g, b, ar, ag, ab = line.split("\t")
    mood = lox.parse_mood_color_legacy(spec)
    assert mood.value == int(value)
    assert mood.percent == lox.ColorPercentRgb(red=int(r), green=int(g), blue=int(b))
    assert mood.approximate_rgb == lox.ColorRgb(red=int(ar), green=int(ag), blue=int(ab))


def fail(kind, call, **fields):
    try:
        call()
    except kind as error:
        for field, value in fields.items():
            assert getattr(error, field) == value, (field, error)
        return error
    raise AssertionError(f"Expected {kind}")


red = lox.encode_color(lox.ColorEncodeInput.RGB(255.0, 0.0, 0.0), None)
assert isinstance(red, lox.ColorEncoded.RGB)
assert (red.rgb.red, red.rgb.green, red.rgb.blue) == (255, 0, 0)
assert red.actor.value == 255 and red.actor.rgb == red.rgb
assert red.mood.value == 1610612836
assert red.mood.percent == lox.ColorPercentRgb(red=100, green=0, blue=0)
assert red.mood.approximate_rgb == red.rgb
assert red.hsv == lox.ColorHsv(hue=0.0, saturation=100.0, value=100.0)
assert red.hsv_command == "hsv(0,100,100)"
assert lox.encode_color(lox.ColorEncodeInput.RGB(0.5, 0.0, 0.0), 50.0).rgb.red == 1
assert lox.encode_color(lox.ColorEncodeInput.HSV(lox.ColorHsv(hue=360, saturation=100, value=100)), None).rgb == red.rgb
temp = lox.encode_color(lox.ColorEncodeInput.TEMPERATURE(2700), None)
assert (temp.kelvin, temp.brightness, temp.command) == (2700, 100.0, "temp(100,2700)")
assert lox.encode_color(lox.ColorEncodeInput.TEMPERATURE(15000), 25.5).command == "temp(25.5,15000)"
zero = lox.encode_color(lox.ColorEncodeInput.TEMPERATURE(1000), -0.0)
assert math.copysign(1, zero.brightness) == -1 and zero.command == "temp(0,1000)"
assert lox.decode_color_legacy("255").actor == red.actor
assert lox.decode_color_legacy("1610612836").mood == red.mood
wrapped = lox.decode_color_legacy("hsv(-360,100,100)")
assert wrapped.hsv.hue == -360.0 and wrapped.rgb == red.rgb
nonfinite = lox.decode_color_legacy("hsv(NaN,inf,-inf)").hsv
assert math.isnan(nonfinite.hue) and nonfinite.saturation == math.inf and nonfinite.value == -math.inf
legacy = lox.decode_color_legacy("lumitech(-0,NaN)")
assert math.copysign(1, legacy.brightness) == -1 and math.isnan(legacy.kelvin)
legacy = lox.decode_color_legacy("temp(inf,-inf)")
assert legacy.brightness == math.inf and legacy.kelvin == -math.inf
for spec in ("rgb(255,0,0)", "hsv(360,100,100)", "255"):
    assert lox.parse_mood_color_legacy(spec) == red.mood
high = 6610612736
assert lox.parse_mood_color_legacy(str(high)).value == high
assert lox.decode_color_legacy(str(high)).mood.value == high
assert lox.observe_actor_color_legacy(-1000000000) == lox.ColorActor(value=-1000000000, rgb=lox.ColorRgb(red=0, green=0, blue=0))
assert lox.observe_actor_color_legacy(-9000000000000000000).value == -9000000000000000000
fail(ValueError, lambda: lox.observe_actor_color_legacy(2**63))
fail(ValueError, lambda: lox.encode_color(lox.ColorEncodeInput.TEMPERATURE(2**63), None))
assert lox.parse_mood_color_legacy("-1000000000").percent == lox.ColorPercentRgb(red=0, green=0, blue=0)
assert [lox.color_format_number(v) for v in (-0.0, math.nan, math.inf, 25.5)] == ["0", "NaN", "inf", "25.5"]
fail(lox.ColorError.RgbChannel, lambda: lox.encode_color(lox.ColorEncodeInput.RGB(0, 256, 0), None),
     channel=lox.ColorChannel.GREEN, value=256.0)
error = fail(lox.ColorError.RgbChannel, lambda: lox.encode_color(lox.ColorEncodeInput.RGB(math.nan, 0, 0), None),
             channel=lox.ColorChannel.RED)
assert math.isnan(error.value)
fail(lox.ColorError.HsvRange, lambda: lox.encode_color(lox.ColorEncodeInput.HSV(lox.ColorHsv(hue=361, saturation=2, value=3)), None),
     value=lox.ColorHsv(hue=361, saturation=2, value=3))
error = fail(lox.ColorError.HsvRange, lambda: lox.encode_color(lox.ColorEncodeInput.HSV(lox.ColorHsv(hue=math.nan, saturation=2, value=3)), None))
assert math.isnan(error.value.hue) and error.value.saturation == 2 and error.value.value == 3
fail(lox.ColorError.Brightness, lambda: lox.encode_color(lox.ColorEncodeInput.TEMPERATURE(2700), 101), value=101)
assert math.isnan(fail(lox.ColorError.Brightness, lambda: lox.encode_color(lox.ColorEncodeInput.TEMPERATURE(2700), math.nan)).value)
fail(lox.ColorError.Kelvin, lambda: lox.encode_color(lox.ColorEncodeInput.TEMPERATURE(999), None), value=999)
for value in (-1, 2**63 - 1, 2**32):
    fail(lox.ColorError.KelvinRepresentation, lambda: lox.encode_color(lox.ColorEncodeInput.TEMPERATURE(value), None), value=value)
fail(lox.ColorError.Arity, lambda: lox.decode_color_legacy("hsv(1,2)"),
     value="1,2", notation=lox.ColorNotation.HSV_COMMAND, expected=3)
fail(lox.ColorError.Arity, lambda: lox.decode_color_legacy("temp(1)"),
     value="1", notation=lox.ColorNotation.TEMPERATURE, expected=2)
fail(lox.ColorError.Arity, lambda: lox.parse_mood_color_legacy("rgb(1,2)"),
     value="1,2", notation=lox.ColorNotation.RGB, expected=3)
fail(lox.ColorError.Number, lambda: lox.decode_color_legacy("hsv(oops,2,3)"),
     value="oops", notation=lox.ColorNotation.HSV_COMMAND)
fail(lox.ColorError.ActorChannel, lambda: lox.observe_actor_color_legacy(256),
     value=256, channel=lox.ColorChannel.RED, channel_value=256)
fail(lox.ColorError.ActorChannel, lambda: lox.observe_actor_color_legacy(-2**63),
     value=-2**63, channel=lox.ColorChannel.RED, channel_value=-808)
fail(lox.ColorError.MoodChannel, lambda: lox.parse_mood_color_legacy("1610612837"),
     value=1610612837, channel=lox.ColorChannel.RED, channel_value=101)
fail(lox.ColorError.MoodChannel, lambda: lox.parse_mood_color_legacy(str(2**63 - 1)),
     value=2**63 - 1, channel=lox.ColorChannel.GREEN, channel_value=163)
for call, profile in ((lox.decode_color_legacy, lox.ColorIntegerProfile.DECODE),
                      (lox.parse_mood_color_legacy, lox.ColorIntegerProfile.MOOD)):
    fail(lox.ColorError.InvalidValue, lambda: call("secret"), value="secret", profile=profile)
fail(lox.ColorError.ActorRange, lambda: lox.decode_color_legacy("-1000000000"), value=-1000000000)
assert red.rgb.red == 255 and red.hsv_command == "hsv(0,100,100)"
print("Python color typed branches/errors/profiles/i64/nonfinite/host ownership PASS")

# Use the existing native HeapWalk helper; never request GC.
native = importlib.import_module("lox_archive.lox_archive")
heap = ctypes.CDLL(sys.argv[1]).lox_test_heap_count
heap.argtypes = [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t)]
heap.restype = ctypes.c_uint32


def matching(prefix, length):
    count = ctypes.c_size_t()
    assert heap(prefix, len(prefix), length, ctypes.byref(count)) == 0
    return count.value


input_value = lox.ColorEncodeInput.RGB(137.0, 29.0, 211.0)
bad = "color-buffer-probe-" + "x" * 65536
paths = [
    ("encode_color", (input_value, None), None),
    ("encode_color", (input_value, 101.25), lox.ColorError.Brightness),
    ("decode_color_legacy", ("hsv(137.25,29.5,81.75)",), None),
    ("decode_color_legacy", (bad,), lox.ColorError.InvalidValue),
    ("parse_mood_color_legacy", ("rgb(137,29,211)",), None),
    ("parse_mood_color_legacy", (bad,), lox.ColorError.InvalidValue),
    ("observe_actor_color_legacy", (-1000000000,), None),
    ("observe_actor_color_legacy", (-123456789,), lox.ColorError.ActorChannel),
    ("color_format_number", (137.251937,), None),
]
for path, (name, args, expected_error) in enumerate(paths):
    converters = {
        "encode_color": (native._UniffiFfiConverterTypeColorEncodeInput, native._UniffiFfiConverterOptionalFloat64),
        "decode_color_legacy": (native._UniffiFfiConverterString,),
        "parse_mood_color_legacy": (native._UniffiFfiConverterString,),
        "observe_actor_color_legacy": (native._UniffiFfiConverterInt64,),
        "color_format_number": (native._UniffiFfiConverterFloat64,),
    }
    status = native._UniffiRustCallStatus.default()
    result = getattr(native._UniffiLib, "uniffi_lox_archive_fn_func_" + name)(
        *(converter.lower(value) for converter, value in zip(converters[name], args)),
        ctypes.byref(status))
    assert status.code == (1 if expected_error else 0)
    buffer = status.error_buf if expected_error else result
    length = buffer.len
    prefix = ctypes.string_at(buffer.data, min(length, 128))
    assert matching(prefix, length) == 1, ("live", path)
    if expected_error:
        buffer.free()
    result.free()
    assert matching(prefix, length) == 0, ("released", path)
    for _ in range(64):
        if expected_error:
            fail(expected_error, lambda: getattr(lox, name)(*args))
        else:
            assert getattr(lox, name)(*args) is not None
    retained = matching(prefix, length)
    print(f"Python color path={path} calls=64 calibrated_live=1 calibrated_released=0 retained_buffers={retained} GC_requested=false")
    assert retained == 0
