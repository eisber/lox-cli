"""Installed selected direct permissions, owned results and rejected XML lowering."""

import ctypes
import gc
import os
from pathlib import Path
import platform
import sys
import sysconfig

import lox_archive as lox
from lox_archive import lox_archive as native


assert sys.platform == "win32" and platform.machine() == "AMD64"
assert sys.implementation.name == "cpython" and sys.version_info[:2] == (3, 13)
assert not sysconfig.get_config_var("Py_GIL_DISABLED") and sys._is_gil_enabled()

USER = lox.PrincipalKind.USER
GROUP = lox.PrincipalKind.USER_GROUP
ERROR = lox.PrincipalPermissionsError
MALFORMED = b"<C A='1' A='2'/>"
TITLES = ["Duplicate", "Empty", "Sämple & 灯", "duplicate"]


def outcome(data, selector, kind):
    try:
        return lox.principal_permissions(
            lox.PrincipalPermissionsRequest(bytes=data, selector=selector, kind=kind))
    except ERROR as failure:
        return failure


def check(value, branch, expected):
    if branch == "success":
        assert isinstance(value, lox.PrincipalPermissions)
        assert isinstance(value.principal, lox.PermissionPrincipal)
        identity = value.principal
        rows = []
        for row in value.permissions:
            assert isinstance(row, lox.DirectPermission)
            rows.append((row.reference, row.title, row.element_type, row.act, row.tim))
        assert ((identity.kind, identity.title, identity.uuid), rows) == expected
    elif branch == "load":
        assert isinstance(value, ERROR.Load)
        assert value.message == "Configuration load failed"
        assert isinstance(value.source_message, str) and value.source_message
    elif branch == "not-found":
        assert isinstance(value, ERROR.NotFound)
        assert (value.kind, value.selector, value.titles) == expected
    elif branch == "ambiguous":
        assert isinstance(value, ERROR.Ambiguous)
        assert (value.kind, value.selector, value.uuids) == expected
    else:
        raise AssertionError(f"Unknown branch: {branch}")


xml = (Path(__file__).resolve().parents[2] / "crates" / "lox-bindings"
       / "tests" / "fixtures" / "principal_permissions.xml").read_bytes()
expected_user = ((USER, "Sämple & 灯", "user-01"), [
    ("target", "Erste & Küche", "Button", "0007", "opaque & 01"),
    ("target", "Erste & Küche", "Button", "", ""),
    ("Target", None, None, "text", None),
    ("", "", "", None, None),
    (None, "", "", None, None),
])
for selector in ("SÄMPLE & 灯", "uuid:user-01"):
    check(outcome(xml, selector, USER), "success", expected_user)
check(outcome(xml, "sämple & 灯", GROUP), "success",
      ((GROUP, "Sämple & 灯", "group-01"), [("target", "Erste & Küche", "Button", "opaque", "01")]))
check(outcome(xml, "Empty", USER), "success", ((USER, "Empty", ""), []))
check(outcome(xml, "uuid:titleless", USER), "success", ((USER, "", "titleless"), []))
check(outcome(xml, "uuid:USER-01", USER), "not-found", (USER, "uuid:USER-01", TITLES))
check(outcome(xml, "Sämple", USER), "not-found", (USER, "Sämple", TITLES))
check(outcome(b"<Root/>", "Sämple", GROUP), "not-found", (GROUP, "Sämple", []))
check(outcome(xml, "dUpLiCaTe", USER), "ambiguous", (USER, "dUpLiCaTe", ["z", "", "z"]))
check(outcome(xml, "uuid:z", USER), "ambiguous", (USER, "uuid:z", ["z", "z"]))
check(outcome(MALFORMED, "missing", GROUP), "load", None)

probe = ctypes.CDLL(sys.argv[1])
probe.lox_test_heap_count.argtypes = (
    ctypes.c_void_p, ctypes.c_size_t, ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t))
probe.lox_test_heap_count.restype = ctypes.c_uint32
probe.lox_test_heap_live.argtypes = (ctypes.c_void_p,)
probe.lox_test_heap_live.restype = ctypes.c_int


def matching(prefix, minimum):
    count = ctypes.c_size_t()
    assert probe.lox_test_heap_count(prefix, len(prefix), minimum, ctypes.byref(count)) == 0
    return count.value


def calibrate(branch, data, selector, kind):
    status = native._UniffiRustCallStatus.default()
    result = native._UniffiLib.uniffi_lox_archive_fn_func_principal_permissions(
        native._UniffiFfiConverterTypePrincipalPermissionsRequest.lower(
            lox.PrincipalPermissionsRequest(bytes=data, selector=selector, kind=kind)),
        ctypes.byref(status))
    error = branch != "success"
    assert status.code == (1 if error else 0)
    buffer = status.error_buf if error else result
    try:
        length = buffer.len
        prefix_length, minimum = (16, length) if branch == "load" else (128, 64000)
        assert length >= max(2 * prefix_length, minimum)
        address = ctypes.cast(buffer.data, ctypes.c_void_p).value
        prefix = ctypes.string_at(address, prefix_length)
        assert probe.lox_test_heap_live(address) and matching(prefix, minimum) == 1
    finally:
        if error:
            buffer.free()
        result.free()
    assert not probe.lox_test_heap_live(address) and matching(prefix, minimum) == 0
    return prefix, minimum, length


def rejected_selector_releases_input():
    data = b"<!--principal-permissions-rejected-selector-input:" + b"q" * 65536 + b"--><Root/>"
    minimum = 4 + len(data)
    converter = native._UniffiFfiConverterTypePrincipalPermissionsRequest
    control = converter.lower(lox.PrincipalPermissionsRequest(bytes=data, selector="valid", kind=USER))
    try:
        assert control.len >= minimum
        address = ctypes.cast(control.data, ctypes.c_void_p).value
        prefix = ctypes.string_at(address, 128)
        assert prefix == len(data).to_bytes(4, "big") + data[:124]
        assert probe.lox_test_heap_live(address) and matching(prefix, minimum) == 1
    finally:
        control.free()
    assert not probe.lox_test_heap_live(address) and matching(prefix, minimum) == 0

    request = lox.PrincipalPermissionsRequest(bytes=data, selector="\ud800", kind=USER)
    for _ in range(64):
        try:
            lox.principal_permissions(request)
        except UnicodeEncodeError as failure:
            assert failure.encoding == "utf-8" and failure.object == request.selector
            assert (failure.start, failure.end) == (0, 1)
        else:
            raise AssertionError("Expected strict host selector-encoding refusal before calling Core")
    assert matching(prefix, minimum) == 0, "Rejected selector retained a marked XML request buffer"
    print("Python principal-permissions invalid selector: input_live=1 input_free=0 "
          "rejections=64 input_native_retained=0 GC_requested=false")


marker = "principal-direct-success-Καλημέρα-" + "x" * 65536
missing = "principal-missing-selector-" + "y" * 65536
long_uuid = "principal-ambiguous-uuid-" + "z" * 65536
trials = [
    ("success", (f'<C Type="User" U="owner" Title="{marker}">'
                 '<PH><PRM REF="unresolved" ACT="0002" TIM=""/></PH></C>').encode(),
     "uuid:owner", USER, ((USER, marker, "owner"), [("unresolved", None, None, "0002", "")])),
    ("load", MALFORMED, "missing", GROUP, None),
    ("not-found", xml, missing, GROUP, (GROUP, missing, ["Sämple & 灯"])),
    ("ambiguous", (f'<Root><C Type="User" Title="Same" U="{long_uuid}"/>'
                   '<C Type="User" Title="Same"/></Root>').encode(),
     "sAmE", USER, (USER, "sAmE", [long_uuid, ""])),
]
assert "RUST_MIN_STACK" not in os.environ
gc.disable()
for branch, data, selector, kind, expected in trials:
    prefix, minimum, length = calibrate(branch, data, selector, kind)
    retained = [outcome(data, selector, kind) for _ in range(64)]
    assert len({id(value) for value in retained}) == 64
    assert matching(prefix, minimum) == 0, f"{branch} retained a native result buffer"
    for value in retained:
        check(value, branch, expected)
    if branch == "load":
        assert all(value.source_message == retained[0].source_message for value in retained)
    print(f"Python principal-permissions branch={branch} actual_bytes={length} live=1 free=0 "
          "calls=64 host_retained=64 native_retained=0 GC_requested=false")
rejected_selector_releases_input()
print("Python principal-permissions raw/order/optional/typed-errors PASS")
