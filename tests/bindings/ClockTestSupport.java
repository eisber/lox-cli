import java.nio.file.Files;
import java.nio.file.Path;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;
import java.util.function.BooleanSupplier;
import net.eisber.lox.archive.*;

public final class ClockTestSupport {
    public static final String USER = "clock-user";
    public static final String PASSWORD = "clock-secret";

    public static ClockSettings settings(boolean retry) {
        ClockSettings settings = LoxArchive.defaultClockSettings();
        settings.setUseSystemProxy(false);
        settings.setTimeoutMillis(60000);
        settings.setRetryDelaysMillis(retry ? new long[] {30000} : new long[0]);
        settings.setShutdownTimeoutMillis(2000);
        return settings;
    }

    public static void require(boolean condition, String message) {
        if (!condition) throw new AssertionError(message);
    }

    public static void waitFor(BooleanSupplier condition) throws Exception {
        long deadline = System.nanoTime() + TimeUnit.SECONDS.toNanos(5);
        while (!condition.getAsBoolean()) {
            if (System.nanoTime() > deadline) throw new AssertionError("Fixture/resource deadline");
            Thread.sleep(10);
        }
    }

    public static Path signal(Path root, String scenario, String event) {
        return root.resolve(scenario.replace("/", "__") + "__" + event);
    }

    public static void waitSignal(Path root, String scenario, String event) throws Exception {
        waitFor(() -> Files.exists(signal(root, scenario, event)));
    }

    public static <T extends Throwable> T expect(Class<T> type, CompletableFuture<?> future) throws Exception {
        try {
            future.get(5, TimeUnit.SECONDS);
        } catch (ExecutionException error) {
            if (!type.isInstance(error.getCause())) throw error;
            return type.cast(error.getCause());
        }
        throw new AssertionError("Expected " + type.getSimpleName());
    }
}
