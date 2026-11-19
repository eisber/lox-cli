import java.util.concurrent.Executors;
import java.util.concurrent.TimeUnit;
import net.eisber.lox.archive.*;

public final class BootstrapPreflightSmoke {
    private static final String PATH = "/jdev/cfg/api";
    private static final String MARKER = "private-bootstrap-marker";

    private static void require(boolean value) {
        if (!value) throw new AssertionError("Bootstrap preflight contract");
    }

    private static <T extends BootstrapException> T fails(
            Class<T> type, BootstrapRequest request) throws BootstrapException {
        try {
            LoxArchive.bootstrapPreflight(request);
        } catch (BootstrapException failure) {
            if (!type.isInstance(failure)) throw failure;
            String text = failure.toString();
            require(!text.contains(MARKER) && !text.contains("127.0.0.1") && !text.contains(PATH));
            require(failure.getCause() == null);
            return type.cast(failure);
        }
        throw new AssertionError("Expected " + type.getSimpleName());
    }

    private static void check(String http, String sink, String ws) throws BootstrapException {
        var direct = BootstrapProxyPolicy.DIRECT;
        var request = new BootstrapRequest(http + PATH, ws, direct);
        var expected = new BootstrapCapabilities[] {
            new BootstrapCapabilities(BootstrapEventSlots.REPORTED_AVAILABLE, BootstrapTlsStatus.REPORTED_AVAILABLE),
            new BootstrapCapabilities(BootstrapEventSlots.REPORTED_UNAVAILABLE, BootstrapTlsStatus.REPORTED_CERTIFICATE_EXPIRED),
            new BootstrapCapabilities(BootstrapEventSlots.MISSING, BootstrapTlsStatus.UNKNOWN),
            new BootstrapCapabilities(BootstrapEventSlots.UNKNOWN, BootstrapTlsStatus.MISSING),
        };
        for (var pair : expected) {
            var observed = LoxArchive.bootstrapPreflight(request);
            require(observed.eventSlots() == pair.eventSlots() && observed.tlsStatus() == pair.tlsStatus());
            require(!observed.toString().contains(MARKER));
        }
        require(fails(BootstrapException.HttpStatus.class, request).status() == 503);
        require(fails(BootstrapException.CommandStatus.class, request).status() == 403);
        require(fails(BootstrapException.Payload.class, request).issue() == BootstrapPayloadIssue.INVALID_ENVELOPE);
        fails(BootstrapException.WrongRoute.class, new BootstrapRequest(sink + "/" + MARKER, ws, direct));
        fails(BootstrapException.UnsupportedProxyPolicy.class,
                new BootstrapRequest(sink + PATH, ws, BootstrapProxyPolicy.SYSTEM));
        var invalid = fails(BootstrapException.InvalidEndpoint.class,
                new BootstrapRequest(sink + PATH, MARKER, direct));
        require(invalid.endpoint() == BootstrapEndpointKind.WEB_SOCKET
                && invalid.issue() == BootstrapEndpointIssue.SCHEME);
    }

    public static void main(String[] args) throws Exception {
        try (var worker = Executors.newSingleThreadExecutor()) {
            worker.submit(() -> {
                check(args[0], args[1], args[2]);
                return null;
            }).get(30, TimeUnit.SECONDS);
        }
        System.out.println("Java generated preflight PASS");
    }
}
