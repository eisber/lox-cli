import java.nio.file.Path;
import java.util.concurrent.CompletableFuture;
import net.eisber.lox.archive.*;

public final class ControlReadSmoke {
    public static void main(String[] args) throws Throwable {
        ControlReadTestSupport.run("java", args[0], Path.of(args[1]), args[2], (host, settings) -> {
            ControlReadClient client = new ControlReadClient(host, ControlReadTestSupport.USER,
                    ControlReadTestSupport.PASSWORD, settings);
            return new ControlReadTestSupport.Client() {
                public CompletableFuture<ControlReading> read(String identifier) { return client.readControl(identifier); }
                public long active() { return client.activeOperations(); }
                public boolean closed() { return client.isClosed(); }
                public void close() throws Exception { client.close(); }
            };
        });
        NativeControlReadProbe.verify("Java", args[0], args[2], Path.of(args[1]));
    }
}
