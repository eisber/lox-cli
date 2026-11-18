import net.eisber.lox.archive.*;
import java.nio.file.*;
import java.util.*;

public final class AutopilotRuleSmoke {
    static void require(boolean value) {
        if (!value) throw new AssertionError("Autopilot selector values");
    }

    static AutopilotRule row(String uuid, String name) {
        return new AutopilotRule(name, uuid, "raw/\0", "<history>\r\n");
    }

    public static void values(NativeAutopilotRuleProbe.Operation operation) throws Throwable {
        String uuid = "abcdef01-2345-6789-abcdef0123456789";
        List<AutopilotRule> catalog = LoxArchive.listLoxapp3AutopilotRules(Files.readAllBytes(
                Path.of("crates", "lox-bindings", "tests", "fixtures", "autopilot_catalog.json")));
        String catalogUuid = "01234567-89ab-cdef-0123456789abcdef";
        require(operation.run(catalog, catalogUuid).equals(
                catalog.stream().filter(value -> value.uuid().equals(catalogUuid)).findFirst().orElseThrow()));
        List<AutopilotRule> rows = new ArrayList<>(List.of(row(uuid, "first"), row(uuid, "second")));
        AutopilotRule selected = operation.run(rows, uuid);
        require(selected.equals(rows.get(0)));
        rows.clear();
        require(selected.stateChanged().equals("raw/\0") && selected.stateHistory().equals("<history>\r\n"));
        for (String query : List.of(uuid.toUpperCase(Locale.ROOT), "-".repeat(21))) {
            try {
                operation.run(List.of(row(uuid, query)), query);
                throw new AssertionError("Expected raw UUID failure");
            } catch (RuleResolutionException.UuidNotFound failure) {
                require(failure.query().equals(query));
            }
        }
        for (String query : List.of("missing", "missing\0query\r\n", " Kitchen", "Kitchen [Room]", "opaque")) {
            try {
                operation.run(List.of(row("opaque", "Kitchen")), query);
                throw new AssertionError("Expected name failure");
            } catch (RuleResolutionException.NotFound failure) {
                require(failure.query().equals(query));
            }
        }
        rows.addAll(List.of(row("same", "Kitchen Extra"), row("same", "Kitchen"), row("same", "Kitchen")));
        try {
            operation.run(rows, "Kitchen");
            throw new AssertionError("Exact names and duplicate IDs must remain ambiguous");
        } catch (RuleResolutionException.Ambiguous failure) {
            require(failure.query().equals("Kitchen") && failure.candidates().equals(rows));
            rows.clear();
            require(failure.candidates().stream().map(AutopilotRule::name).toList()
                    .equals(List.of("Kitchen Extra", "Kitchen", "Kitchen")));
            require(failure.candidates().get(0).stateHistory().equals("<history>\r\n"));
        }
        AutopilotRule unicode = row("raw/\0ID", "Ü İ Σ Straße é");
        for (String query : List.of("", "ü", "i\u0307", "σ", "straße", "é")) {
            require(operation.run(List.of(unicode), query).equals(unicode));
        }
        for (String query : List.of("-".repeat(20), "abcdef0123456789abcdef0123456789",
                "gggggggg-gggg-gggg-gggggggggggggggg")) {
            AutopilotRule value = row("opaque", query);
            require(operation.run(List.of(value), query).equals(value));
        }
        for (String query : List.of("STRASSE", "e\u0301", " ü", "ς")) {
            try {
                operation.run(List.of(unicode), query);
                throw new AssertionError("Only lowercase substring semantics apply");
            } catch (RuleResolutionException.NotFound failure) {
                require(failure.query().equals(query));
            }
        }
        try {
            operation.run(List.of(), "");
            throw new AssertionError("Empty catalog");
        } catch (RuleResolutionException.NotFound failure) {
            require(failure.query().isEmpty());
        }
        try {
            operation.run(List.of(row("a", ""), row("b", "?")), "");
            throw new AssertionError("Empty substring matches every supplied row");
        } catch (RuleResolutionException.Ambiguous failure) {
            require(failure.candidates().stream().map(AutopilotRule::uuid).toList().equals(List.of("a", "b")));
        }
    }

    public static void main(String[] args) throws Throwable {
        values(LoxArchive::resolveAutopilotRule);
        NativeAutopilotRuleProbe.verify("Java", LoxArchive::resolveAutopilotRule);
        System.out.println("Java supplied typed Autopilot selection/owned values/errors PASS");
    }
}
