import java.nio.file.Files;
import java.nio.file.Path;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;
import net.eisber.lox.archive.*;

public final class ControlReadTestSupport {
    public static final String USER = "control-user";
    public static final String PASSWORD = "control-secret";
    public static final String ID = "1fbc668c-005c-7471-ffffed57184a04d2";
    public static final String VALUE = "control-owned-value-" + "x".repeat(65536);

    public interface Client extends AutoCloseable {
        CompletableFuture<ControlReading> read(String identifier);
        long active();
        boolean closed();
        void close() throws Exception;
    }

    public interface Factory {
        Client open(String host, ClockSettings settings) throws Exception;
    }

    public static ClockSettings settings(String mode) {
        ClockSettings result = LoxArchive.defaultClockSettings();
        result.setUseSystemProxy(false);
        result.setTimeoutMillis(mode.equals("timeout") ? 150 : 60000);
        result.setRetryDelaysMillis(mode.equals("retry") ? new long[] {30000} : new long[0]);
        result.setShutdownTimeoutMillis(2000);
        return result;
    }

    public static void checkError(String mode, Throwable error) {
        ClockTestSupport.require(!error.toString().contains(PASSWORD)
                && !error.toString().contains("127.0.0.1")
                && !error.toString().contains("private"), "sanitized native exception");
        ControlValidationIssue issue = switch (mode) {
            case "envelope" -> ControlValidationIssue.ENVELOPE;
            case "missing-code" -> ControlValidationIssue.MISSING_CODE;
            case "unsupported-code" -> ControlValidationIssue.UNSUPPORTED_CODE;
            case "status", "conflict" -> ControlValidationIssue.REJECTED_CODE;
            case "missing" -> ControlValidationIssue.MISSING_VALUE;
            case "nontext" -> ControlValidationIssue.UNSUPPORTED_VALUE;
            default -> null;
        };
        if (issue != null) {
            ClockTestSupport.require(error instanceof ControlReadException.Validation e
                    && e.issue() == issue, "validation distinction: " + mode);
        } else if (mode.equals("http")) {
            ClockTestSupport.require(error instanceof ControlReadException.Http e
                    && e.status() == 401, "HTTP context");
        } else if (mode.equals("connect") || mode.equals("timeout") || mode.equals("body")) {
            ClockTestSupport.require(error instanceof ControlReadException.Transport e
                    && e.kind() == switch (mode) {
                        case "connect" -> ClockTransportFailure.CONNECT;
                        case "body" -> ClockTransportFailure.BODY;
                        default -> ClockTransportFailure.TIMEOUT;
                    }, "source context");
        } else {
            ClockTestSupport.require(error instanceof ControlReadException.Decode, "strict decode");
        }
    }

    public static void run(String language, String host, Path signals, String refused, Factory factory)
            throws Exception {
        for (String mode : new String[] {"normal", "number", "empty", "lower"}) {
            Client client = factory.open(host + "/" + language + "/" + mode, settings(""));
            ControlReading reading;
            try (client) {
                reading = client.read(ID).get(5, TimeUnit.SECONDS);
                String expected = switch (mode) {
                    case "number" -> "-0";
                    case "empty" -> "";
                    case "lower" -> "lowercase";
                    default -> VALUE;
                };
                ClockTestSupport.require(reading.value().equals(expected) && reading.code().equals("200"),
                        "Core-validated reading");
                ClockTestSupport.require(client.read(ID).get(5, TimeUnit.SECONDS).equals(reading), "reuse");
            }
            client.close();
            ClockTestSupport.require(client.closed() && client.active() == 0, "drained close");
            ClockTestSupport.expect(ControlReadException.Closed.class, client.read(ID));
            ClockTestSupport.require(reading.code().equals("200"), "owned reading after close");
        }
        for (String mode : new String[] {"envelope", "missing-code", "unsupported-code", "status",
                "conflict", "missing", "nontext", "http", "json", "utf8", "body", "connect", "timeout"}) {
            String scenario = language + "/" + mode;
            try (Client client = factory.open(mode.equals("connect") ? refused : host + "/" + scenario,
                    settings(mode))) {
                checkError(mode, ClockTestSupport.expect(ControlReadException.class, client.read(ID)));
            }
            if (mode.equals("timeout")) ClockTestSupport.waitSignal(signals, scenario, "eof");
        }
        for (String mode : new String[] {"blocked", "retry"}) {
            String scenario = language + "/" + mode;
            try (Client client = factory.open(host + "/" + scenario, settings(mode))) {
                var read = client.read(ID);
                ClockTestSupport.waitSignal(signals, scenario, mode.equals("retry") ? "retry-response" : "entered");
                ClockTestSupport.expect(ControlReadException.Busy.class, client.read(ID));
                ClockTestSupport.require(read.cancel(true) && read.isCancelled(), "future cancellation");
                if (!mode.equals("retry")) ClockTestSupport.waitSignal(signals, scenario, "eof");
                ClockTestSupport.waitFor(() -> client.active() == 0);
                ClockTestSupport.require(!client.closed(), "close must not mask cancellation");
            }
        }
        String scenario = language + "/close";
        Client client = factory.open(host + "/" + scenario, settings(""));
        var read = client.read(ID);
        ClockTestSupport.waitSignal(signals, scenario, "entered");
        client.close();
        ClockTestSupport.expect(ControlReadException.CancelledShutdown.class, read);
        ClockTestSupport.waitSignal(signals, scenario, "eof");
        client.close();
        ClockTestSupport.expect(ControlReadException.Closed.class, client.read(ID));

        try (ControlReadRuntime runtime = new ControlReadRuntime(host + "/" + language + "/unused",
                USER, PASSWORD, settings("")); ClockCancellation cancellation = new ClockCancellation()) {
            String[] identifiers = {"", "../private", "not-hex"};
            ControlIdentifierIssue[] issues = {ControlIdentifierIssue.INVALID_LENGTH,
                    ControlIdentifierIssue.INVALID_CHARACTERS, ControlIdentifierIssue.NON_HEX};
            for (int i = 0; i < identifiers.length; i++) {
                try {
                    runtime.readControl(identifiers[i], cancellation);
                    throw new AssertionError("invalid identifier admitted");
                } catch (ControlReadException.InvalidTarget e) {
                    ClockTestSupport.require(e.issue() == issues[i], "identifier issue");
                }
            }
            cancellation.cancel();
            for (int i = 0; i < 2; i++) {
                try {
                    runtime.readControl(ID, cancellation);
                    throw new AssertionError("pre-cancel ignored");
                } catch (ControlReadException.CancelledRequested expected) {
                    ClockTestSupport.require(cancellation.isCancelled(), "sticky cancellation");
                }
            }
            runtime.shutdown();
            runtime.shutdown();
            try {
                runtime.readControl(ID, cancellation);
                throw new AssertionError("native closed read accepted");
            } catch (ControlReadException.Closed expected) {}
            runtime.close();
            try {
                runtime.activeOperations();
                throw new AssertionError("disposed generated handle accepted");
            } catch (IllegalStateException expected) {}
        }
        try {
            new ControlReadRuntime("bad host control-secret", USER, PASSWORD, settings(""));
            throw new AssertionError("invalid host admitted");
        } catch (ControlReadException.InvalidHost expected) {
            ClockTestSupport.require(!expected.toString().contains(PASSWORD), "sanitized host error");
        }
        CountDownLatch callbackEntered = new CountDownLatch(1);
        CountDownLatch release = new CountDownLatch(1);
        Client callbackClient = factory.open(host + "/" + language + "/callback", settings(""));
        try {
            callbackClient.read(ID).thenRun(() -> {
                callbackEntered.countDown();
                try { release.await(); }
                catch (InterruptedException e) { Thread.currentThread().interrupt(); }
            });
            Files.writeString(ClockTestSupport.signal(signals, language + "/callback", "release"), "release");
            ClockTestSupport.require(callbackEntered.await(5, TimeUnit.SECONDS), "callback barrier");
            try {
                callbackClient.close();
                throw new AssertionError("blocked caller continuation hidden");
            } catch (ControlReadDispatchTimeout expected) {
                ClockTestSupport.require(callbackClient.closed() && callbackClient.active() == 0,
                        "native shutdown precedes host dispatch timeout");
            }
        } finally {
            release.countDown();
            callbackClient.close();
        }
        NativeClockStatusProbe allocations = new NativeClockStatusProbe("lox-control-read-ffi");
        for (int i = 0; i < 16; i++) {
            String statusCase = language + "-control-status/" + i + "/callback";
            try (Client connection = factory.open(host + "/" + statusCase, settings(""))) {
                var pending = connection.read(ID);
                ClockTestSupport.waitSignal(signals, statusCase, "entered");
                var captured = pending.thenRun(allocations::capture);
                Files.writeString(ClockTestSupport.signal(signals, statusCase, "release"), "release");
                captured.get(5, TimeUnit.SECONDS);
            }
            allocations.afterClose();
        }
        allocations.verify(language);
        ClockTestSupport.require(Thread.getAllStackTraces().keySet().stream()
                .noneMatch(t -> t.isAlive() && t.getName().equals("lox-control-read-ffi")), "host lanes joined");
        System.out.println(language + " validated control states/errors/cancel/EOF/busy/disposal: PASS");
    }
}
