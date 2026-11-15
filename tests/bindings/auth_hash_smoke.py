"""Synthetic explicit profiles, redacted objects and actual native ownership."""

import ctypes
import gc
import os
import struct
import sys
import threading

threads_before_import = {thread.ident for thread in threading.enumerate()}
import lox_archive as lox
assert {thread.ident for thread in threading.enumerate()} == threads_before_import

native = getattr(lox, "lox_archive", lox)
assert "RUST_MIN_STACK" not in os.environ
probe = ctypes.CDLL(sys.argv[1])
probe.lox_test_heap_count.argtypes = (
    ctypes.c_void_p, ctypes.c_size_t, ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t))
probe.lox_test_heap_live.argtypes = (ctypes.c_void_p,)
probe.lox_test_heap_live.restype = ctypes.c_int
probe.lox_test_heap_auth_error_count.argtypes = (
    ctypes.c_uint32, ctypes.POINTER(ctypes.c_size_t))
probe.lox_test_heap_auth_hex_count.argtypes = (ctypes.POINTER(ctypes.c_size_t),)


def matching(tag):
    count = ctypes.c_size_t()
    assert probe.lox_test_heap_auth_error_count(tag, ctypes.byref(count)) == 0
    return count.value


def check(value, expected):
    assert isinstance(value, lox.AuthenticationHash)
    assert value.to_hex() == expected
    assert expected not in repr(value) and expected not in str(value)
    value.dispose()
    value.dispose()
    try:
        value.to_hex()
    except lox.AuthenticationHashError.Closed as failure:
        assert failure.message == "authentication hash is closed"
    else:
        raise AssertionError("Disposed hash remained readable")


gc.disable()
app = "88bfb35cc1a3ce183b6fcd66b3b5fe3acbdaba620c3ce041d866ce6d055e2bb2"
key = bytes.fromhex("001122aaccee")
check(lox.hash_app_credentials("us:ér", "päss:word", key), app)
check(lox.hash_app_credentials_hex("us:ér", "päss:word", "001122aaccee"), app)
for digest, expected in [
    (lox.WsxPasswordDigest.SHA1, "9d063ec0b204dd0a2368b4be99e052704579ada03e5b755e2fa4452646473790"),
    (lox.WsxPasswordDigest.SHA256, "dffd58fcc0395555f0ea22c20b7f613cf1f2cb9103864f0f340c99bb3776a79c"),
]:
    check(lox.hash_wsx_credentials("us:ér", "päss:word", "s:älт", digest, key), expected)
    check(lox.hash_wsx_credentials_hex(
        "us:ér", "päss:word", "s:älт", digest, "001122aaccee"), expected)
token = "4b59c58e38a81ab3dbe92b052651a678a6f5a175175ba01f1f1853849b25da89"
check(lox.hash_app_token("jwt:тøken", "clé:key".encode()), token)
check(lox.hash_wsx_autht("jwt:тøken", "clé:key".encode()), token.upper())
check(lox.hash_app_token_hex("jwt:тøken", "clé:key".encode().hex()), token)
check(lox.hash_wsx_autht_hex("jwt:тøken", "clé:key".encode().hex()), token.upper())
check(lox.hash_app_token_hex("tok", ""),
      "3d6cb1ab5aa9e075eeb3a608a885b15452f8f9c50a7d33aef07acfd8fdfdc6e1")
check(lox.hash_app_token_hex("tok", "ff"),
      "ed4646e109c4890bf51f8bed06aa54b6a79626c69a60ae2d23e6277d940b8b1a")
raw = lox.hash_app_token("tok", b"ff")
decoded = lox.hash_app_token_hex("tok", "ff")
assert raw.to_hex() != decoded.to_hex()
del raw, decoded
for invalid in ["a", "🌸"]:
    try:
        lox.hash_app_token_hex("synthetic-secret-token", invalid)
    except lox.AuthenticationHashError.InvalidHexKey as failure:
        assert failure.message == "invalid hex authentication key"
        assert "synthetic-secret" not in str(failure)
    else:
        raise AssertionError("Invalid hex accepted")
try:
    lox.hash_wsx_autht("synthetic-secret-token", b"\xff")
except lox.AuthenticationHashError.NonUtf8WsxKey:
    pass
else:
    raise AssertionError("Non-UTF8 key accepted for CONFIG autht")
composed = lox.hash_app_token("é", b"key")
decomposed = lox.hash_app_token("e\u0301", b"key")
assert composed.to_hex() != decomposed.to_hex()
del composed, decomposed

# These are actual exported error RustBuffers, not a helper allocation.
for tag, key_input, operation, variant, message in [
    (1, "synthetic-secret-key", lox.hash_app_token_hex,
     lox.AuthenticationHashError.InvalidHexKey, "invalid hex authentication key"),
    (2, "ff", lox.hash_wsx_autht_hex,
     lox.AuthenticationHashError.NonUtf8WsxKey, "CONFIG wsx authentication key is not UTF-8"),
]:
    ffi = (native._UniffiLib.uniffi_lox_archive_fn_func_hash_app_token_hex if tag == 1
           else native._UniffiLib.uniffi_lox_archive_fn_func_hash_wsx_autht_hex)
    status = native._UniffiRustCallStatus.default()
    result = ffi(native._UniffiFfiConverterString.lower("synthetic-secret-token"),
                 native._UniffiFfiConverterString.lower(key_input), ctypes.byref(status))
    assert result == 0 and status.code == 1
    expected = struct.pack(">II", tag, len(message)) + message.encode()
    assert ctypes.string_at(status.error_buf.data, status.error_buf.len) == expected
    assert matching(tag) == 1
    status.error_buf.free()
    assert matching(tag) == 0
    for _ in range(64):
        try:
            operation("synthetic-secret-token", key_input)
        except variant as failure:
            assert failure.message == message
            assert "synthetic-secret" not in str(failure) + repr(failure)
        else:
            raise AssertionError("Expected typed error")
    assert matching(tag) == 0
    print(f"Python auth error tag={tag} actual_live=1 freed=0 calls=64 retained=0 GC_requested=false")

# UniFFI 0.32.1 Rust handles are Arc::into_raw pointers. On this qualified
# Windows AMD64 build the heap allocation begins at its two usize counters.
ffi = native._UniffiLib.uniffi_lox_archive_fn_func_hash_app_token
status = native._UniffiRustCallStatus.default()
handle = ffi(native._UniffiFfiConverterString.lower("synthetic-secret-token"),
             native._UniffiFfiConverterBytes.lower(b"synthetic-key"), ctypes.byref(status))
assert status.code == 0 and probe.lox_test_heap_live(handle - 16) == 1
free = native._UniffiLib.uniffi_lox_archive_fn_free_authenticationhash
native._uniffi_rust_call(free, handle)
assert probe.lox_test_heap_live(handle - 16) == 0
for _ in range(64):
    value = lox.hash_app_token("synthetic-secret-token", b"synthetic-key")
    address = value._handle - 16
    assert probe.lox_test_heap_live(address) == 1
    value.dispose()
    value.dispose()
    del value
    assert probe.lox_test_heap_live(address) == 0
print("Python auth actual object live=1 freed=0 calls=64 retained=0 GC_requested=false")

value = lox.hash_app_token("jwt:тøken", "clé:key".encode())
for closed in [False, True]:
    if closed:
        value.dispose()
    status = native._UniffiRustCallStatus.default()
    result = native._UniffiLib.uniffi_lox_archive_fn_method_authenticationhash_to_hex(
        value._uniffi_clone_handle(), ctypes.byref(status))
    count = ctypes.c_size_t()

    def retained():
        if closed:
            return matching(3)
        assert probe.lox_test_heap_auth_hex_count(ctypes.byref(count)) == 0
        return count.value

    assert status.code == int(closed)
    assert retained() == 1
    if closed:
        status.error_buf.free()
    result.free()
    assert retained() == 0
    for _ in range(64):
        try:
            assert value.to_hex() == token and not closed
        except lox.AuthenticationHashError.Closed as failure:
            assert closed and failure.message == "authentication hash is closed"
    assert retained() == 0
    print(f"Python auth reveal closed={closed} actual_live=1 freed=0 calls=64 retained=0 GC_requested=false")
del value
print("Python auth profiles/algorithms/UTF8/hex/redaction/disposal PASS")
