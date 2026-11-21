import java.nio.file.Files;
import java.nio.file.Path;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;
import net.eisber.lox.archive.*;

public final class ClockSmoke {
    static ClockClient client(String host, String scenario) throws Exception {
        return new ClockClient(host + "/" + scenario, ClockTestSupport.USER,
                ClockTestSupport.PASSWORD, ClockTestSupport.settings(scenario.endsWith("retry")));
    }

    public static void main(String[] args) throws Exception {
        String host = args[0];
        Path signals = Path.of(args[1]);
        for (String mode : new String[] {"normal", "number"}) {
            ClockClient client = client(host, "java/" + mode);
            try {
                ClockReading reading = client.readClock().get(5, TimeUnit.SECONDS);
                ClockTestSupport.require(reading.date().equals(mode.equals("normal") ? "2026-11-15" : "42")
                        && reading.time().equals(mode.equals("normal") ? "09:10:11" : "12.5"), "typed reading");
                if (mode.equals("normal")) {
                    ClockTestSupport.require(client.readClock().get(5, TimeUnit.SECONDS).equals(reading),
                            "same native client reuse");
                }
            } finally {
                client.close();
            }
            client.close();
            ClockTestSupport.expect(ClockRuntimeException.Closed.class, client.readClock());
        }
        for (String mode : new String[] {"json", "utf8", "missing", "status", "http"}) {
            try (ClockClient client = client(host, "java/" + mode)) {
                if (mode.equals("json") || mode.equals("utf8")) {
                    var error = ClockTestSupport.expect(ClockRuntimeException.Decode.class, client.readClock());
                    ClockTestSupport.require(error.endpoint() == ClockEndpoint.DATE, "decode endpoint");
                } else if (mode.equals("http")) {
                    var error = ClockTestSupport.expect(ClockRuntimeException.Http.class, client.readClock());
                    ClockTestSupport.require(error.endpoint() == ClockEndpoint.DATE && error.status() == 401, "HTTP context");
                } else {
                    var error = ClockTestSupport.expect(ClockRuntimeException.Validation.class, client.readClock());
                    ClockTestSupport.require(error.endpoint() == ClockEndpoint.DATE
                            && error.issue() == (mode.equals("missing")
                                ? ClockValidationIssue.MISSING_VALUE : ClockValidationIssue.REJECTED_CODE), "validation issue");
                }
                System.out.println("Java typed clock error: " + mode);
            }
        }
        for (String mode : new String[] {"blocked-date", "blocked-time", "retry"}) {
            String scenario = "java/" + mode;
            try (ClockClient client = client(host, scenario)) {
                CompletableFuture<ClockReading> read = client.readClock();
                ClockTestSupport.waitSignal(signals, scenario, mode.equals("retry") ? "retry-response" : "entered");
                ClockTestSupport.expect(ClockRuntimeException.Busy.class, client.readClock());
                ClockTestSupport.require(read.cancel(true) && read.isCancelled(), "future cancellation");
                if (!mode.equals("retry")) ClockTestSupport.waitSignal(signals, scenario, "eof");
                ClockTestSupport.waitFor(() -> client.activeOperations() == 0);
                ClockTestSupport.require(!client.isClosed(), "close must not mask cancellation");
                System.out.println("Java future cancellation -> Rust drain: " + mode);
            }
        }
        for (String mode : new String[] {"close-date", "close-time"}) {
            String scenario = "java/" + mode;
            ClockClient client = client(host, scenario);
            CompletableFuture<ClockReading> read = client.readClock();
            ClockTestSupport.waitSignal(signals, scenario, "entered");
            client.close();
            ClockTestSupport.expect(ClockRuntimeException.CancelledShutdown.class, read);
            ClockTestSupport.waitSignal(signals, scenario, "eof");
            client.close();
            ClockTestSupport.require(client.isClosed() && client.activeOperations() == 0, "shutdown state");
            ClockTestSupport.expect(ClockRuntimeException.Closed.class, client.readClock());
            System.out.println("Java shared shutdown/socket EOF/idempotence: " + mode);
        }
        try (ClockRuntime runtime = new ClockRuntime(host + "/java/unused", ClockTestSupport.USER,
                ClockTestSupport.PASSWORD, ClockTestSupport.settings(false));
                ClockCancellation cancellation = new ClockCancellation()) {
            cancellation.cancel();
            ClockTestSupport.require(cancellation.isCancelled(), "sticky cancellation handle");
            try {
                runtime.readClock(cancellation);
                throw new AssertionError("pre-cancel ignored");
            } catch (ClockRuntimeException.CancelledRequested expected) {
                runtime.shutdown();
                runtime.shutdown();
            }
        }
        CountDownLatch callbackEntered = new CountDownLatch(1);
        CountDownLatch release = new CountDownLatch(1);
        ClockClient callbackClient = client(host, "java/callback");
        try {
            callbackClient.readClock().thenRun(() -> {
                callbackEntered.countDown();
                try {
                    release.await();
                } catch (InterruptedException error) {
                    Thread.currentThread().interrupt();
                }
            });
            Files.writeString(ClockTestSupport.signal(signals, "java/callback", "release"), "release");
            ClockTestSupport.require(callbackEntered.await(5, TimeUnit.SECONDS), "callback barrier");
            try {
                callbackClient.close();
                throw new AssertionError("blocked caller continuation hidden");
            } catch (ClockDispatchTimeout expected) {
                ClockTestSupport.require(callbackClient.isClosed() && callbackClient.activeOperations() == 0,
                        "native shutdown must precede host timeout");
                System.out.println("Java honest host-dispatch timeout after native shutdown: PASS");
            }
        } finally {
            release.countDown();
            callbackClient.close();
        }
        NativeClockStatusProbe allocations = new NativeClockStatusProbe();
        for (int i = 0; i < 16; i++) {
            String scenario = "java-status/" + i + "/callback";
            ClockClient connection = client(host, scenario);
            try {
                var read = connection.readClock();
                ClockTestSupport.waitSignal(signals, scenario, "entered");
                var captured = read.thenRun(allocations::capture);
                Files.writeString(ClockTestSupport.signal(signals, scenario, "release"), "release");
                captured.get(5, TimeUnit.SECONDS);
            } finally {
                connection.close();
            }
            allocations.afterClose();
        }
        allocations.verify("Java");
        System.out.println("Java actual clock handles/futures/explicit disposal: PASS");
    }
}
