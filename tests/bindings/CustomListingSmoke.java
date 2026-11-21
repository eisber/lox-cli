import java.util.ArrayList;
import java.util.List;
import net.eisber.lox.archive.*;

public final class CustomListingSmoke {
    static void require(boolean value) {
        if (!value) throw new AssertionError("Strict custom listing contract");
    }

    public static void values(NativeCustomListingProbe.Operation operation) throws Throwable {
        List<String> rows = new ArrayList<>(List.of(
                "d ignored/path.txt", "l ignored\\path.txt", "- false.ℓoxcc", "- false.loxccé",
                "- z.loXcC", "- A.LoxCC", "- z.loXcC", "D upper.loxcc", "L upper-link.loxcc",
                " d leading.loxcc", " l leading-link.loxcc", "nonnumeric\u2003unicode.loxcc",
                "- nul\0.loxcc", "- not-a-uuid.loxcc", "- first.loxcc last-token",
                "- \ud800\udc00.loxcc", "- \ue000.loxcc"));
        List<String> before = List.copyOf(rows);
        List<String> expected = List.of(
                "A.LoxCC", "leading-link.loxcc", "leading.loxcc", "not-a-uuid.loxcc",
                "nul\0.loxcc", "unicode.loxcc", "upper-link.loxcc", "upper.loxcc",
                "z.loXcC", "z.loXcC", "\ue000.loxcc", "\ud800\udc00.loxcc");
        List<String> owned = operation.run(rows);
        require(owned.equals(expected) && rows.equals(before));
        require(operation.run(List.of()).isEmpty());
        List<CustomListingException> retained = new ArrayList<>();
        for (String bad : List.of("", "\u2003\t ", "d private.loxcc", "l private.loxcc",
                "- private/path.loxcc", "- private\\path.loxcc")) {
            try {
                operation.run(List.of("- valid.loxcc", "- unrelated", bad, ""));
                throw new AssertionError("Atomic first-error rejection required");
            } catch (CustomListingException.Malformed failure) {
                require((bad.isEmpty() || bad.equals("\u2003\t ")) && failure.row() == 2);
                require(!failure.toString().contains("private"));
                retained.add(failure);
            } catch (CustomListingException.Unsupported failure) {
                require(bad.contains("private") && failure.row() == 2);
                require(!failure.toString().contains("private"));
                retained.add(failure);
            }
        }
        try {
            operation.run(List.of("", "- x/y.loxcc"));
            throw new AssertionError("First malformed row required");
        } catch (CustomListingException.Malformed failure) {
            require(failure.row() == 0);
        }
        rows.clear();
        require(owned.equals(expected));
        for (CustomListingException failure : retained) {
            if (failure instanceof CustomListingException.Malformed value) require(value.row() == 2);
            else if (failure instanceof CustomListingException.Unsupported value) require(value.row() == 2);
            else throw new AssertionError("Typed error");
        }
    }

    public static void main(String[] args) throws Throwable {
        values(LoxArchive::parseCustomListing);
        NativeCustomListingProbe.verify("Java", LoxArchive::parseCustomListing);
        System.out.println("Java custom-listing raw/sorted/duplicates/quirks/typed-atomic-errors/owned PASS");
    }
}
