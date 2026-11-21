"""Bounded asyncio adapter over the generated, production-backed clock driver."""

import asyncio
from concurrent.futures import ThreadPoolExecutor

from lox_archive import (
    ClockCancellation,
    ClockRuntime,
    ClockRuntimeError,
    default_clock_settings,
    ControlReadRuntime,
    ControlReadError,
)


class AsyncClockClient:
    """One native read plus one close lane; no HTTP, retry, or protocol logic."""

    def __init__(self, host, user, password, settings=None):
        self._loop = asyncio.get_running_loop()
        self._native = ClockRuntime(
            host, user, password,
            default_clock_settings() if settings is None else settings,
        )
        self._executor = ThreadPoolExecutor(max_workers=2, thread_name_prefix="lox-clock-ffi")
        self._reading = None
        self._close_task = None
        self._closing = False
        self._closed = False

    def _check_loop(self):
        if asyncio.get_running_loop() is not self._loop:
            raise RuntimeError("AsyncClockClient belongs to its creating event loop")

    @property
    def active_operations(self):
        return 0 if self._native is None else self._native.active_operations()

    @property
    def is_closed(self):
        return self._closed

    @staticmethod
    def _consume(future):
        if not future.cancelled():
            future.exception()

    async def read_clock(self):
        self._check_loop()
        if self._closing:
            raise ClockRuntimeError.Closed()
        if self._reading is not None:
            raise ClockRuntimeError.Busy()
        cancellation = ClockCancellation()
        work = self._loop.run_in_executor(
            self._executor, self._native.read_clock, cancellation,
        )
        self._reading = work

        def finished(future):
            self._consume(future)
            if self._reading is future:
                self._reading = None

        work.add_done_callback(finished)
        try:
            return await asyncio.shield(work)
        except asyncio.CancelledError:
            # Cancelling an asyncio executor Future alone does not cancel Rust I/O.
            cancellation.cancel()
            raise

    async def _finish_close(self):
        try:
            await self._loop.run_in_executor(self._executor, self._native.shutdown)
            reading = self._reading
            if reading is not None:
                try:
                    await asyncio.shield(reading)
                except Exception:
                    pass
            # Both bounded native waits have returned; join only our own idle workers.
            self._executor.shutdown(wait=True)
            self._native = None
            self._closed = True
        except BaseException:
            self._close_task = None
            raise

    async def aclose(self):
        self._check_loop()
        if self._closed:
            return
        self._closing = True
        if self._close_task is None:
            self._close_task = self._loop.create_task(self._finish_close())
            self._close_task.add_done_callback(self._consume)
        # Host cancellation must not abandon an already requested native shutdown.
        await asyncio.shield(self._close_task)

    async def __aenter__(self):
        return self

    async def __aexit__(self, *_):
        await self.aclose()


class AsyncControlReadClient:
    """Bounded waits for Core-validated control reads; identifiers are explicit."""

    def __init__(self, host, user, password, settings=None):
        self._loop = asyncio.get_running_loop()
        self._native = ControlReadRuntime(
            host, user, password,
            default_clock_settings() if settings is None else settings,
        )
        self._executor = ThreadPoolExecutor(max_workers=2, thread_name_prefix="lox-control-read-ffi")
        self._reading = None
        self._close_task = None
        self._closing = False
        self._closed = False

    def _check_loop(self):
        if asyncio.get_running_loop() is not self._loop:
            raise RuntimeError("AsyncControlReadClient belongs to its creating event loop")

    @property
    def active_operations(self):
        return 0 if self._native is None else self._native.active_operations()

    @property
    def is_closed(self):
        return self._closed

    async def read_control(self, identifier):
        self._check_loop()
        if self._closing:
            raise ControlReadError.Closed()
        if self._reading is not None:
            raise ControlReadError.Busy()
        cancellation = ClockCancellation()
        work = self._loop.run_in_executor(
            self._executor, self._native.read_control, identifier, cancellation,
        )
        self._reading = work

        def finished(future):
            AsyncClockClient._consume(future)
            if self._reading is future:
                self._reading = None

        work.add_done_callback(finished)
        try:
            return await asyncio.shield(work)
        except asyncio.CancelledError:
            cancellation.cancel()
            raise

    async def _finish_close(self):
        try:
            await self._loop.run_in_executor(self._executor, self._native.shutdown)
            reading = self._reading
            if reading is not None:
                try:
                    await asyncio.shield(reading)
                except Exception:
                    pass
            self._executor.shutdown(wait=True)
            self._native = None
            self._closed = True
        except BaseException:
            self._close_task = None
            raise

    async def aclose(self):
        self._check_loop()
        if self._closed:
            return
        self._closing = True
        if self._close_task is None:
            self._close_task = self._loop.create_task(self._finish_close())
            self._close_task.add_done_callback(AsyncClockClient._consume)
        await asyncio.shield(self._close_task)

    async def __aenter__(self):
        return self

    async def __aexit__(self, *_):
        await self.aclose()
