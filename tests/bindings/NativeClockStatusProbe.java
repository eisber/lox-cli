package net.eisber.lox.archive;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;
import java.lang.reflect.Field;
import java.util.HashSet;
import java.util.Set;

/** Test-only Windows process-heap evidence, never reads a released allocation. */
public final class NativeClockStatusProbe {
    private static final MethodHandle LIVE = Linker.nativeLinker().downcallHandle(
            SymbolLookup.libraryLookup(System.getProperty("lox.archive.heapProbe"), Arena.global())
                    .find("lox_test_heap_live").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS));
    private static final MethodHandle COUNT = Linker.nativeLinker().downcallHandle(
            SymbolLookup.libraryLookup(System.getProperty("lox.archive.heapProbe"), Arena.global())
                    .find("lox_test_heap_status_count").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));
    private final Set<Long> lanes = new HashSet<>();
    private int captured;
    private int retained;
    private int openScopes;
    private MemorySegment status;
    private long marker;
    private final String threadName;

    private static boolean live(long address) {
        try {
            return (int) LIVE.invokeExact(MemorySegment.ofAddress(address)) != 0;
        } catch (Throwable error) {
            throw new AssertionError(error);
        }
    }

    public NativeClockStatusProbe() {
        this("lox-clock-ffi");
    }

    public NativeClockStatusProbe(String threadName) {
        this.threadName = threadName;
        long calibrationMarker = 0x4c4f58434c4f4300L;
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment calibration = arena.allocate(32, 8);
            mark(calibration, calibrationMarker);
            if (!live(calibration.address()) || matching(calibrationMarker) != 1) {
                throw new AssertionError("32-byte heap calibration not live");
            }
        }
        if (matching(calibrationMarker) != 0) throw new AssertionError("32-byte calibration not released");
        System.out.println("Clock status heap calibration: live=true after_close=false");
    }

    private static void mark(MemorySegment segment, long value) {
        segment.set(ValueLayout.JAVA_LONG, 0, value);
        segment.set(ValueLayout.JAVA_LONG, 8, ~value);
        segment.set(ValueLayout.JAVA_LONG, 16, value ^ 0x5a17c93eb46d208fL);
        segment.set(ValueLayout.JAVA_LONG, 24, value ^ 0x8c26f1a05b73de49L);
    }

    private static long matching(long value) {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment count = arena.allocate(ValueLayout.JAVA_LONG);
            int error = (int) COUNT.invokeExact(value, count);
            if (error != 0) throw new AssertionError("HeapWalk error: " + error);
            return count.get(ValueLayout.JAVA_LONG, 0);
        } catch (Throwable error) {
            throw new AssertionError(error);
        }
    }

    public void capture() {
        try {
            Field field = UniffiHelpers.class.getDeclaredField("REUSABLE_STATUS");
            field.setAccessible(true);
            status = (MemorySegment) ((ThreadLocal<?>) field.get(null)).get();
            if (!Thread.currentThread().getName().equals(threadName)
                    || status == null || status.byteSize() != 32 || !live(status.address())) {
                throw new AssertionError("Expected actual owned wait-lane live call status");
            }
            lanes.add(Thread.currentThread().threadId());
            captured++;
            marker = 0x4c4f58434c4f4300L + captured;
            // Only after this dispatch's final FFI call; no subsequent status decoding.
            mark(status, marker);
            if (matching(marker) != 1) throw new AssertionError("Live status marker not found");
        } catch (ReflectiveOperationException error) {
            throw new AssertionError(error);
        }
    }

    public void afterClose() {
        // HeapWalk reads only busy heap entries; no freed address is dereferenced.
        if (matching(marker) != 0) retained++;
        if (status.scope().isAlive()) openScopes++;
        status = null;
    }

    public void verify(String language) {
        System.out.println(language + " explicitly_closed_clients=" + captured
                + " distinct_owned_lanes=" + lanes.size()
                + " retained_status_allocations=" + retained + " open_status_scopes=" + openScopes
                + " GC_requested=false");
        if (captured != 16 || lanes.size() != 16 || retained != 0 || openScopes != 0) {
            throw new AssertionError("Owned clock status lifetime leak");
        }
        verifyNestedExit();
    }

    private static void verifyNestedExit() {
        try {
            var owned = UniffiHelpers.class.getDeclaredMethod("uniffiRunWithOwnedStatus", Runnable.class);
            owned.setAccessible(true);
            Field field = UniffiHelpers.class.getDeclaredField("REUSABLE_STATUS");
            field.setAccessible(true);
            ThreadLocal<?> local = (ThreadLocal<?>) field.get(null);
            MemorySegment previous = (MemorySegment) local.get();
            MemorySegment[] scopes = new MemorySegment[2];
            RuntimeException sentinel = new RuntimeException("scope-exit");
            Runnable nested = () -> {
                scopes[1] = (MemorySegment) local.get();
                throw sentinel;
            };
            Runnable outer = () -> {
                scopes[0] = (MemorySegment) local.get();
                try {
                    owned.invoke(null, nested);
                    throw new AssertionError("exception lost");
                } catch (java.lang.reflect.InvocationTargetException error) {
                    if (error.getCause() != sentinel) throw new AssertionError(error);
                } catch (ReflectiveOperationException error) {
                    throw new AssertionError(error);
                }
                if (local.get() != scopes[0] || !scopes[0].scope().isAlive()
                        || scopes[1].scope().isAlive()) {
                    throw new AssertionError("Nested exceptional status scope not restored/closed");
                }
            };
            owned.invoke(null, outer);
            if (local.get() != previous || scopes[0].scope().isAlive()) {
                throw new AssertionError("Outer status scope not restored/closed");
            }
            System.out.println("Nested/exceptional owned status restoration: PASS");
        } catch (ReflectiveOperationException error) {
            throw new AssertionError(error);
        }
    }
}
