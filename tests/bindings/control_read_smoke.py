"""Installed native control-read proof, also driving Java and Kotlin callers."""

import argparse
import asyncio
from concurrent.futures import ThreadPoolExecutor
from contextlib import contextmanager
import ctypes
from datetime import datetime, timezone
import gc
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import threading
import time

import lox_archive as lox
from lox_archive import lox_archive as native
from lox_archive_async import AsyncControlReadClient

SPEC = importlib.util.spec_from_file_location("control_read_fixture", Path(__file__).with_name("control_read_fixture.py"))
FIXTURE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(FIXTURE)
ID = FIXTURE.IDENTIFIER
ERRORS = {
    "envelope": lox.ControlValidationIssue.ENVELOPE,
    "missing-code": lox.ControlValidationIssue.MISSING_CODE,
    "unsupported-code": lox.ControlValidationIssue.UNSUPPORTED_CODE,
    "status": lox.ControlValidationIssue.REJECTED_CODE,
    "conflict": lox.ControlValidationIssue.REJECTED_CODE,
    "missing": lox.ControlValidationIssue.MISSING_VALUE,
    "nontext": lox.ControlValidationIssue.UNSUPPORTED_VALUE,
}


def settings(mode=""):
    value = lox.default_clock_settings()
    value.use_system_proxy = False
    value.timeout_millis = 150 if mode == "timeout" else 60000
    value.retry_delays_millis = [30000] if mode == "retry" else []
    value.shutdown_timeout_millis = 2000
    return value


async def wait_for(predicate):
    deadline = asyncio.get_running_loop().time() + 5
    while not predicate():
        if asyncio.get_running_loop().time() > deadline:
            raise AssertionError("Control fixture/resource deadline")
        await asyncio.sleep(0.01)


def wait_sync(predicate):
    deadline = time.monotonic() + 5
    while not predicate():
        if time.monotonic() > deadline:
            raise AssertionError("Control fixture/resource deadline")
        time.sleep(0.005)


def check_error(mode, error):
    assert not any(secret in repr(error) + str(error) for secret in (
        FIXTURE.PASSWORD, FIXTURE.USER, "127.0.0.1", "private-backend-body", "private"))
    if mode in ERRORS:
        assert isinstance(error, lox.ControlReadError.Validation) and error.issue == ERRORS[mode]
    elif mode in ("json", "utf8"):
        assert isinstance(error, lox.ControlReadError.Decode)
    elif mode == "http":
        assert isinstance(error, lox.ControlReadError.Http) and error.status == 401
    elif mode in ("connect", "timeout", "body"):
        assert isinstance(error, lox.ControlReadError.Transport)
        assert error.kind == {"connect": lox.ClockTransportFailure.CONNECT,
                              "timeout": lox.ClockTransportFailure.TIMEOUT,
                              "body": lox.ClockTransportFailure.BODY}[mode]
    else:
        raise AssertionError(mode)


async def async_checks(fixture):
    for mode, value in (("normal", FIXTURE.VALUE), ("number", "-0"), ("empty", ""), ("lower", "lowercase")):
        client = AsyncControlReadClient(f"{fixture.host}/python/{mode}", FIXTURE.USER, FIXTURE.PASSWORD, settings())
        reading = await client.read_control(ID)
        assert (reading.value, reading.code) == (value, "200")
        assert (await client.read_control(ID)) == reading
        await client.aclose()
        await client.aclose()
        assert client.is_closed and client.active_operations == 0
        assert reading.value == value
        try:
            await client.read_control(ID)
        except lox.ControlReadError.Closed:
            pass
        else:
            raise AssertionError("after-close read accepted")
    for mode in list(ERRORS) + ["json", "utf8", "body", "http", "timeout", "connect"]:
        case = f"python/{mode}"
        host = fixture.refused_host if mode == "connect" else f"{fixture.host}/{case}"
        async with AsyncControlReadClient(host, FIXTURE.USER, FIXTURE.PASSWORD, settings(mode)) as client:
            try:
                await client.read_control(ID)
            except Exception as error:
                check_error(mode, error)
            else:
                raise AssertionError(f"{mode} accepted as validated state")
        if mode == "timeout":
            await wait_for(lambda: fixture.signal(case, "eof").exists())
    for mode in ("blocked", "retry"):
        case = f"python/{mode}"
        async with AsyncControlReadClient(f"{fixture.host}/{case}", FIXTURE.USER, FIXTURE.PASSWORD, settings(mode)) as client:
            task = asyncio.create_task(client.read_control(ID))
            await wait_for(lambda: fixture.signal(case, "retry-response" if mode == "retry" else "entered").exists())
            try:
                await client.read_control(ID)
            except lox.ControlReadError.Busy:
                pass
            else:
                raise AssertionError("unbounded second read admitted")
            task.cancel()
            try:
                await task
            except asyncio.CancelledError:
                pass
            else:
                raise AssertionError("async cancellation lost")
            if mode != "retry":
                await wait_for(lambda: fixture.signal(case, "eof").exists())
            await wait_for(lambda: client.active_operations == 0)
            assert not client.is_closed
    for cancel_close_waiter in (False, True):
        case = f"python/close-{cancel_close_waiter}/close"
        client = AsyncControlReadClient(f"{fixture.host}/{case}", FIXTURE.USER, FIXTURE.PASSWORD, settings())
        task = asyncio.create_task(client.read_control(ID))
        await wait_for(lambda: fixture.signal(case, "entered").exists())
        closing = asyncio.create_task(client.aclose())
        if cancel_close_waiter:
            await asyncio.sleep(0)
            assert closing.cancel()
        try:
            await closing
        except asyncio.CancelledError:
            assert cancel_close_waiter
        await wait_for(lambda: fixture.signal(case, "eof").exists())
        await wait_for(lambda: client.is_closed)
        try:
            await task
        except lox.ControlReadError.CancelledShutdown:
            pass
        else:
            raise AssertionError("shutdown cancellation lost")
        await client.aclose()
        assert client.active_operations == 0
    assert not any(t.name.startswith("lox-control-read-ffi") for t in threading.enumerate())
    print("Python validated controls, typed failures, bounded admission, cancellation/EOF, joined host lanes: PASS")


def sync_checks(fixture):
    client = lox.ControlReadRuntime(f"{fixture.host}/python-sync/unused", FIXTURE.USER, FIXTURE.PASSWORD, settings())
    for identifier, issue in (("", lox.ControlIdentifierIssue.INVALID_LENGTH),
                              ("../private", lox.ControlIdentifierIssue.INVALID_CHARACTERS),
                              ("not-hex", lox.ControlIdentifierIssue.NON_HEX)):
        try:
            client.read_control(identifier, lox.ClockCancellation())
        except lox.ControlReadError.InvalidTarget as error:
            assert error.issue == issue
            if identifier:
                assert identifier not in repr(error)
        else:
            raise AssertionError("unsafe identifier admitted")
    cancellation = lox.ClockCancellation()
    cancellation.cancel()
    for _ in range(2):
        try:
            client.read_control(ID, cancellation)
        except lox.ControlReadError.CancelledRequested:
            pass
        else:
            raise AssertionError("sticky pre-cancel ignored")
    client.shutdown()
    client.shutdown()
    assert client.is_closed() and client.active_operations() == 0
    try:
        client.read_control(ID, lox.ClockCancellation())
    except lox.ControlReadError.Closed:
        pass
    else:
        raise AssertionError("native after-close accepted")
    try:
        lox.ControlReadRuntime("bad host control-secret", FIXTURE.USER, FIXTURE.PASSWORD, settings())
    except lox.ControlReadError.InvalidHost as error:
        assert "control-secret" not in repr(error)
    else:
        raise AssertionError("invalid host accepted")
    print("Python generated handles: strict zero-I/O IDs, sticky pre-cancel, idle close/use-after-close: PASS")


def buffer_checks(fixture, heap_probe):
    probe = ctypes.CDLL(heap_probe)
    probe.lox_test_heap_count.argtypes = (
        ctypes.c_void_p, ctypes.c_size_t, ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t))
    probe.lox_test_heap_control_error_count.argtypes = (
        ctypes.c_uint64, ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t))
    serial = 0

    @contextmanager
    def operation(mode):
        nonlocal serial
        serial += 1
        case = f"python-buffers/{serial}/{mode}"
        host = fixture.refused_host if mode == "connect" else f"{fixture.host}/{case}"
        runtime = lox.ControlReadRuntime(host, FIXTURE.USER, FIXTURE.PASSWORD, settings(mode))
        cancellation = lox.ClockCancellation()
        identifier = {"length": "", "characters": "../private", "nonhex": "secret"}.get(mode, ID)
        if mode == "closed":
            runtime.shutdown()
        if mode == "precancel":
            cancellation.cancel()
        pool = None
        pending = None
        if mode == "busy":
            pool = ThreadPoolExecutor(max_workers=1)
            # Use a blocked endpoint for the first native call.
            runtime.shutdown()
            runtime = lox.ControlReadRuntime(f"{fixture.host}/{case}/blocked", FIXTURE.USER, FIXTURE.PASSWORD, settings())
            pending = pool.submit(runtime.read_control, ID, cancellation)
            wait_sync(lambda: fixture.signal(case + "/blocked", "entered").exists())
        if mode == "shutdown":
            pool = ThreadPoolExecutor(max_workers=1)
            runtime.shutdown()
            runtime = lox.ControlReadRuntime(f"{fixture.host}/{case}/close", FIXTURE.USER, FIXTURE.PASSWORD, settings())

            def close_entered():
                wait_sync(lambda: fixture.signal(case + "/close", "entered").exists())
                runtime.shutdown()

            pending = pool.submit(close_entered)
        try:
            yield runtime, identifier, lox.ClockCancellation() if mode == "busy" else cancellation
        finally:
            runtime.shutdown()
            if pending:
                try:
                    pending.result(timeout=5)
                except lox.ControlReadError.CancelledShutdown:
                    pass
                wait_sync(lambda: fixture.signal(case + ("/blocked" if mode == "busy" else "/close"), "eof").exists())
            if pool:
                pool.shutdown()
            if mode == "timeout":
                wait_sync(lambda: fixture.signal(case, "eof").exists())

    gc.disable()
    modes = ["normal", "length", "characters", "nonhex", *ERRORS,
             "json", "utf8", "body", "http", "connect", "timeout", "precancel", "closed", "busy", "shutdown"]
    for mode in modes:
        with operation(mode) as (runtime, identifier, cancellation):
            status = native._UniffiRustCallStatus.default()
            result = native._UniffiLib.uniffi_lox_archive_fn_method_controlreadruntime_read_control(
                runtime._uniffi_clone_handle(),
                native._UniffiFfiConverterString.lower(identifier),
                native._UniffiFfiConverterTypeClockCancellation.lower(cancellation),
                ctypes.byref(status))
            error_path = mode != "normal"
            assert status.code == int(error_path), (mode, status.code)
            buffer = status.error_buf if error_path else result
            length = buffer.len
            marker = int.from_bytes(ctypes.string_at(buffer.data, length), "little") if error_path else None
            prefix = None if error_path else ctypes.string_at(buffer.data, 128)

            def matching():
                count = ctypes.c_size_t()
                code = (probe.lox_test_heap_control_error_count(marker, length, ctypes.byref(count))
                        if error_path else probe.lox_test_heap_count(prefix, len(prefix), length, ctypes.byref(count)))
                assert code == 0
                return count.value

            assert matching() == 1, ("calibration live", mode, length, matching())
            if error_path:
                buffer.free()
            result.free()
            assert matching() == 0, ("calibration freed", mode, matching())
        for _ in range(64):
            with operation(mode) as (runtime, identifier, cancellation):
                try:
                    value = runtime.read_control(identifier, cancellation)
                    assert not error_path and value.value == FIXTURE.VALUE
                except lox.ControlReadError as error:
                    assert error_path
                    if mode in ERRORS or mode in ("json", "utf8", "body", "http", "connect", "timeout"):
                        check_error(mode, error)
        retained = matching()
        assert retained == 0, ("retained buffer", mode, retained)
        print(f"Python control path={mode} calls=64 calibrated_live=1 calibrated_freed=0 retained={retained} GC_requested=false")
    status = native._UniffiRustCallStatus.default()
    handle = native._UniffiLib.uniffi_lox_archive_fn_constructor_controlreadruntime_new(
        native._UniffiFfiConverterString.lower("bad host control-secret"),
        native._UniffiFfiConverterString.lower(FIXTURE.USER),
        native._UniffiFfiConverterString.lower(FIXTURE.PASSWORD),
        native._UniffiFfiConverterTypeClockSettings.lower(settings()), ctypes.byref(status))
    assert status.code == 1 and handle == 0
    length = status.error_buf.len
    marker = int.from_bytes(ctypes.string_at(status.error_buf.data, length), "little")
    count = ctypes.c_size_t()

    def host_matching():
        assert probe.lox_test_heap_control_error_count(marker, length, ctypes.byref(count)) == 0
        return count.value

    assert host_matching() == 1
    status.error_buf.free()
    assert host_matching() == 0
    for _ in range(64):
        try:
            lox.ControlReadRuntime("bad host control-secret", FIXTURE.USER, FIXTURE.PASSWORD, settings())
        except lox.ControlReadError.InvalidHost:
            pass
        else:
            raise AssertionError("invalid host accepted")
    assert host_matching() == 0
    print("Python control path=constructor-error calls=64 calibrated_live=1 calibrated_freed=0 retained=0 GC_requested=false")


def main():
    parser = argparse.ArgumentParser()
    for name in ("signals", "java", "api-jar", "classes", "kotlin-jar", "native-dir", "heap-probe"):
        parser.add_argument("--" + name, required=True)
    args = parser.parse_args()
    args.signals = Path(args.signals)
    assert "RUST_MIN_STACK" not in os.environ
    assert Path(lox.__file__).resolve().is_relative_to(Path(sys.prefix).resolve())
    with FIXTURE.ControlFixture(args.signals) as fixture:
        sync_checks(fixture)
        asyncio.run(async_checks(fixture))
        buffer_checks(fixture, args.heap_probe)
        commands = []
        for name, main_class, extra in (
            ("java", "ControlReadSmoke", ""),
            ("kotlin", "ControlReadSmokeKt", ";" + args.kotlin_jar),
        ):
            command = [
                args.java, "--enable-native-access=ALL-UNNAMED",
                f"-Djava.library.path={args.native_dir}",
                f"-Dlox.archive.heapProbe={args.heap_probe}",
                "-cp", args.api_jar + ";" + args.classes + extra,
                main_class, fixture.host, str(args.signals), fixture.refused_host,
            ]
            start = datetime.now(timezone.utc).isoformat()
            result = subprocess.run(command, capture_output=True, text=True, timeout=180)
            (args.signals.parent / f"control-read-{name}.log").write_text(result.stdout + result.stderr)
            commands.append({"command": command, "started_utc": start,
                             "ended_utc": datetime.now(timezone.utc).isoformat(),
                             "exit_code": result.returncode})
            print(result.stdout, end="")
            if result.returncode:
                raise AssertionError(result.stderr)
        evidence = {"commands": commands, "requests": fixture.verify(),
                    "socket_eof_files": sorted(p.name for p in args.signals.glob("*__eof")),
                    "strict_default_policy": True, "validated_not_legacy_projection": True,
                    "no_real_devices": True, "native_config_credit": 0}
        (args.signals.parent / "CONTROL-READ-PROOF.json").write_text(json.dumps(evidence, indent=2) + "\n")
    print("REAL NATIVE CONTROL READ HEADS: PASS")


if __name__ == "__main__":
    main()
