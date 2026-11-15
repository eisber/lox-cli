import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.function.Consumer;
import net.eisber.lox.archive.Control;
import net.eisber.lox.archive.ControlResolutionException;
import net.eisber.lox.archive.IdentifierException;
import net.eisber.lox.archive.LoxArchive;
import net.eisber.lox.archive.PreparedControlQuery;
import net.eisber.lox.archive.ResolvedControl;

public final class ControlSmoke {
    @FunctionalInterface
    interface Operation { void run() throws Exception; }

    static void require(boolean condition, String context) {
        if (!condition) throw new AssertionError(context);
    }

    static <T extends Exception> void expect(
            Class<T> type, Operation operation, Consumer<T> check) throws Exception {
        try {
            operation.run();
        } catch (Exception error) {
            if (!type.isInstance(error)) throw error;
            check.accept(type.cast(error));
            return;
        }
        throw new AssertionError("Expected " + type.getName());
    }

    static List<Control> catalog() {
        return List.of(
                new Control("Licht Extra", "opaque-first", "Switch", "KÜCHE", "Indoor", true, false),
                new Control("Licht", "opaque-exact", "LightController2", null, null, false, true),
                new Control("Panel [West]", "nested", "Display", "KÜCHE", "", false, false),
                new Control("Licht", "spaced", "Dimmer", " Küche ", null, true, true),
                new Control("Other", "roomless", "State", null, null, false, false),
                new Control("Open [", "unclosed", "Switch", null, null, false, false),
                new Control("Straße", "unicode", "Sensor", "ROOM", "x", false, true),
                new Control("İSTANBUL", "dotted", "Sensor", "ROOM", "x", false, true),
                new Control("Literal [tag] ", "trailing", "Switch", null, null, false, false));
    }

    static ResolvedControl resolve(String query, String room, List<Control> controls)
            throws ControlResolutionException {
        return LoxArchive.resolveControlQuery(
                LoxArchive.prepareControlQuery(query, room, Map.of()), controls);
    }

    public static void main(String[] args) throws Exception {
        require(System.getenv("RUST_MIN_STACK") == null, "native control caller must use stock stack");
        for (String id : List.of("-".repeat(21), "-".repeat(51),
                "1FBC668C-005C-7471-FFFFED57184A04D2")) {
            var prepared = LoxArchive.prepareControlQuery(id, "ignored", Map.of(id, "must-not-win"));
            require(prepared instanceof PreparedControlQuery.Resolved, "identifier must bypass catalog");
            require(LoxArchive.resolveControlQuery(prepared, List.of()).uuid().equals(id), "identifier");
        }
        Map<String, String> aliases = new HashMap<>(Map.of(
                "Alias [Room]", "../raw-alias", "../raw-alias", "must-not-recurse", "", ""));
        var alias = LoxArchive.prepareControlQuery("Alias [Room]", "ignored", aliases);
        aliases.put("Alias [Room]", "changed-after-prepare");
        for (int i = 0; i < 2; i++) {
            require(LoxArchive.resolveControlQuery(alias, List.of()).uuid().equals("../raw-alias"),
                    "raw/nonrecursive/owned alias");
        }
        require(LoxArchive.resolveControlQuery(
                LoxArchive.prepareControlQuery("", null, aliases), List.of()).uuid().isEmpty(),
                "empty alias");
        require(LoxArchive.prepareControlQuery("alias [Room]", null, aliases)
                instanceof PreparedControlQuery.Catalog, "case-sensitive raw alias");
        require(LoxArchive.resolveControlQuery(LoxArchive.prepareControlQuery(
                "other", null, Map.of("Other", "alias")), catalog()).uuid().equals("roomless"),
                "case mismatch must use catalog");
        System.out.println("Java control identifiers/owned aliases: PASS");

        String query = " Panel [West] [küche]";
        var prepared = LoxArchive.prepareControlQuery(query, "ignored", Map.of());
        require(prepared instanceof PreparedControlQuery.Catalog, "catalog stage");
        var snapshot = (PreparedControlQuery.Catalog) prepared;
        require(snapshot.query().equals(query) && snapshot.roomFilter().equals("ignored"),
                "original request snapshot");
        for (int i = 0; i < 2; i++) {
            ResolvedControl result = LoxArchive.resolveControlQuery(prepared, catalog());
            require(result.uuid().equals("nested"), "reusable prepared query");
        }
        List<Control> controls = new ArrayList<>(catalog());
        List<Control> expected = List.of(controls.get(0), controls.get(1), controls.get(3));
        expect(ControlResolutionException.Ambiguous.class,
                () -> resolve("Licht", null, controls), error -> {
                    require(error.query().equals("Licht"), "original ambiguous query");
                    List<Control> candidates = error.candidates();
                    require(candidates.equals(expected), "ordered seven-field candidate records");
                    controls.clear();
                    require(candidates.equals(expected), "native error collection ownership");
                });
        var other = catalog().get(4);
        expect(ControlResolutionException.Ambiguous.class,
                () -> resolve("Other", null, List.of(other, other)),
                error -> require(error.candidates().equals(List.of(other, other)), "no deduplication"));
        for (String missing : List.of(" Missing [ Küche ]", " Licht ",
                "Panel [West] [küche", " Licht [küche] ")) {
            expect(ControlResolutionException.NotFound.class,
                    () -> resolve(missing, null, catalog()),
                    error -> require(error.query().equals(missing), "unmodified missing query"));
        }
        System.out.println("Java control typed success/ordered ambiguity/not-found: PASS");

        String[][] rooms = {
            {" Licht [ küche ]", "missing", "spaced"},
            {" Panel [West] [küche]", "missing", "nested"},
            {"Other []", "missing", "roomless"},
            {"", " küche ", "spaced"},
            {"Open [", null, "unclosed"},
            {"Literal [tag] ", null, "trailing"},
        };
        for (String[] row : rooms) {
            require(resolve(row[0], row[1], catalog()).uuid().equals(row[2]), "literal room " + row[0]);
        }
        expect(ControlResolutionException.NotFound.class,
                () -> resolve("Licht [ küche  ]", "küche", catalog()), error -> {});
        for (String room : new String[] {null, ""}) {
            expect(ControlResolutionException.Ambiguous.class,
                    () -> resolve("", room, catalog()),
                    error -> require(error.candidates().equals(catalog()), "empty substring/order"));
        }
        require(resolve("straße", null, catalog()).uuid().equals("unicode"), "Unicode lowercase");
        require(resolve("i\u0307stan", null, catalog()).uuid().equals("dotted"), "expanding lowercase");
        for (String missing : List.of("STRASSE", "ISTANBUL")) {
            expect(ControlResolutionException.NotFound.class,
                    () -> resolve(missing, null, catalog()), error -> {});
        }
        System.out.println("Java control room/rightmost bracket/Unicode/empty query: PASS");

        for (String id : List.of("a", "-", "-".repeat(20), "a".repeat(30), "é".repeat(30) + "-")) {
            require(!LoxArchive.isUuid(id), "loose identifier rejection");
        }
        for (String id : List.of("-".repeat(21), "-".repeat(51))) {
            require(LoxArchive.isUuid(id), "loose identifier acceptance");
        }
        for (String id : List.of("a", "-", "-".repeat(50))) LoxArchive.validateUuid(id);
        for (String id : List.of("", "-".repeat(51), "é".repeat(26), "/".repeat(51))) {
            expect(IdentifierException.InvalidLength.class, () -> LoxArchive.validateUuid(id), e -> {});
        }
        for (String id : List.of("..", "a/b", "a\\b", "a%b")) {
            expect(IdentifierException.InvalidCharacters.class, () -> LoxArchive.validateUuid(id), e -> {});
        }
        for (String id : List.of("a.b", "é".repeat(25), "opaque-id")) {
            expect(IdentifierException.NonHex.class, () -> LoxArchive.validateUuid(id), e -> {});
        }
        System.out.println("Java control distinct byte-length/path identifier errors: PASS");
    }
}
