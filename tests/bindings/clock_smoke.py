"""Installed native clock tests; orchestrates the same loopback fixture for JVM callers."""

import argparse
import asyncio
from datetime import datetime, timezone
import importlib.util
import json
from pathlib import Path
import subprocess
import sys

import lox_archive as native
from lox_archive_async import AsyncClockClient

SPEC = importlib.util.spec_from_file_location("clock_fixture", Path(__file__).with_name("clock_fixture.py"))
FIXTURE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(FIXTURE)


def settings(retry=False):
    value = native.default_clock_settings()
    value.use_system_proxy = False
    value.timeout_millis = 60000
    value.retry_delays_millis = [30000] if retry else []
    value.shutdown_timeout_millis = 2000
    return value


async def wait_for(predicate):
    deadline = asyncio.get_running_loop().time() + 5
    while not predicate():
        if asyncio.get_running_loop().time() > deadline:
            raise AssertionError("Clock fixture/resource condition timed out")
        await asyncio.sleep(0.01)


def signal(root, case, event):
    return root / (case.replace("/", "__") + "__" + event)


async def python_checks(fixture, signals):
    for mode, date, clock in (("normal", "2026-11-15", "09:10:11"), ("number", "42", "12.5")):
        client = AsyncClockClient(f"{fixture.host}/python/{mode}", FIXTURE.USER, FIXTURE.PASSWORD, settings())
        reading = await client.read_clock()
        assert reading.date == date and reading.time == clock
        if mode == "normal":
            again = await client.read_clock()
            assert again.date == date and again.time == clock
        await client.aclose()
        await client.aclose()
        assert client.is_closed and client.active_operations == 0
        try:
            await client.read_clock()
        except native.ClockRuntimeError.Closed:
            pass
        else:
            raise AssertionError("after-close read succeeded")
    expected = {
        "json": native.ClockRuntimeError.Decode,
        "utf8": native.ClockRuntimeError.Decode,
        "missing": native.ClockRuntimeError.Validation,
        "status": native.ClockRuntimeError.Validation,
        "http": native.ClockRuntimeError.Http,
    }
    for mode, error_type in expected.items():
        async with AsyncClockClient(f"{fixture.host}/python/{mode}", FIXTURE.USER, FIXTURE.PASSWORD, settings()) as client:
            try:
                await client.read_clock()
            except error_type as error:
                assert error.endpoint == native.ClockEndpoint.DATE
                if mode == "missing":
                    assert error.issue == native.ClockValidationIssue.MISSING_VALUE
                if mode == "status":
                    assert error.issue == native.ClockValidationIssue.REJECTED_CODE
                if mode == "http":
                    assert error.status == 401
                assert FIXTURE.PASSWORD not in str(error) and fixture.host not in str(error)
                print("Python typed clock error:", type(error).__qualname__)
            else:
                raise AssertionError(f"{mode} accepted")
    for mode in ("blocked-date", "blocked-time", "retry"):
        case = f"python/{mode}"
        client = AsyncClockClient(f"{fixture.host}/{case}", FIXTURE.USER, FIXTURE.PASSWORD, settings(mode == "retry"))
        task = asyncio.create_task(client.read_clock())
        await wait_for(lambda: signal(signals, case, "retry-response" if mode == "retry" else "entered").exists())
        try:
            await client.read_clock()
        except native.ClockRuntimeError.Busy:
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
            await wait_for(lambda: signal(signals, case, "eof").exists())
        await wait_for(lambda: client.active_operations == 0)
        # EOF/drain must precede close, otherwise close could mask abandoned I/O.
        await client.aclose()
        print("Python host cancellation -> Rust drain:", mode)
    for mode in ("close-date", "close-time"):
        case = f"python/{mode}"
        client = AsyncClockClient(f"{fixture.host}/{case}", FIXTURE.USER, FIXTURE.PASSWORD, settings())
        task = asyncio.create_task(client.read_clock())
        await wait_for(lambda: signal(signals, case, "entered").exists())
        await client.aclose()
        try:
            await task
        except native.ClockRuntimeError.CancelledShutdown:
            pass
        else:
            raise AssertionError("close did not produce native shutdown cancellation")
        await wait_for(lambda: signal(signals, case, "eof").exists())
        await client.aclose()
        assert client.is_closed and client.active_operations == 0
        print("Python shared shutdown/socket EOF/idempotence:", mode)
    case = "python/cancelled-close-waiter/close-date"
    client = AsyncClockClient(f"{fixture.host}/{case}", FIXTURE.USER, FIXTURE.PASSWORD, settings())
    read = asyncio.create_task(client.read_clock())
    await wait_for(lambda: signal(signals, case, "entered").exists())
    closing = asyncio.create_task(client.aclose())
    await asyncio.sleep(0)
    assert closing.cancel()
    try:
        await closing
    except asyncio.CancelledError:
        pass
    else:
        raise AssertionError("close waiter was not cancelled")
    # No second close until the first request independently closes Rust I/O/workers.
    await wait_for(lambda: signal(signals, case, "eof").exists())
    await wait_for(lambda: client.is_closed)
    try:
        await read
    except native.ClockRuntimeError.CancelledShutdown:
        pass
    else:
        raise AssertionError("cancelled close waiter abandoned native shutdown")
    await client.aclose()
    print("Python cancelled close-awaiter still completes native shutdown/socket EOF: PASS")


def sync_checks(fixture):
    client = native.ClockRuntime(
        f"{fixture.host}/python-sync/unused", FIXTURE.USER, FIXTURE.PASSWORD, settings(),
    )
    cancellation = native.ClockCancellation()
    cancellation.cancel()
    assert cancellation.is_cancelled()
    try:
        client.read_clock(cancellation)
    except native.ClockRuntimeError.CancelledRequested:
        pass
    else:
        raise AssertionError("pre-cancel ignored")
    client.shutdown()
    client.shutdown()
    assert client.is_closed() and client.active_operations() == 0
    try:
        client.read_clock(native.ClockCancellation())
    except native.ClockRuntimeError.Closed:
        pass
    else:
        raise AssertionError("native after-close accepted")
    try:
        native.ClockRuntime("bad host clock-secret", FIXTURE.USER, FIXTURE.PASSWORD, settings())
    except native.ClockRuntimeError.InvalidHost as error:
        assert "clock-secret" not in repr(error)
    else:
        raise AssertionError("invalid host accepted")
    print("Python generated native handles: sticky pre-cancel, shutdown and after-close PASS")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--signals", type=Path, required=True)
    parser.add_argument("--java")
    parser.add_argument("--api-jar")
    parser.add_argument("--classes")
    parser.add_argument("--kotlin-jar")
    parser.add_argument("--native-dir")
    args = parser.parse_args()
    assert Path(native.__file__).resolve().is_relative_to(Path(sys.prefix).resolve())
    with FIXTURE.ClockFixture(args.signals) as fixture:
        sync_checks(fixture)
        asyncio.run(python_checks(fixture, args.signals))
        commands = []
        if args.java:
            for name, main_class, extra in (
                ("java", "ClockSmoke", ""),
                ("kotlin", "ClockSmokeKt", ";" + args.kotlin_jar),
            ):
                command = [
                    args.java, "--enable-native-access=ALL-UNNAMED",
                    f"-Djava.library.path={args.native_dir}",
                    "-cp", args.api_jar + ";" + args.classes + extra,
                    main_class, fixture.host, str(args.signals),
                ]
                start = datetime.now(timezone.utc).isoformat()
                result = subprocess.run(command, capture_output=True, text=True, timeout=60)
                (args.signals.parent / f"clock-{name}.log").write_text(result.stdout + result.stderr)
                commands.append({"command": command, "started_utc": start,
                                 "ended_utc": datetime.now(timezone.utc).isoformat(),
                                 "exit_code": result.returncode})
                print(result.stdout, end="")
                if result.returncode:
                    raise AssertionError(result.stderr)
        evidence = {"commands": commands, "request_order": fixture.verify(),
                    "socket_eof_files": sorted(p.name for p in args.signals.glob("*__eof")),
                    "no_real_devices": True, "strict_default_policy": True}
        (args.signals.parent / "CLOCK-PROOF.json").write_text(json.dumps(evidence, indent=2) + "\n")
        print("REAL NATIVE CLOCK HEADS: typed values/errors, cancellation, bounded admission, shutdown PASS")


if __name__ == "__main__":
    main()
