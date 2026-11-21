import java.io.ByteArrayOutputStream;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;
import java.util.Arrays;
import java.util.List;
import net.eisber.lox.archive.*;

public final class TextStateSmoke {
    public static final String[] TEXTS = {"", "A", "é", "😀\0", "é e\u0301 😀\0tail\0", "é"};

    public static void require(boolean value) {
        if (!value) throw new AssertionError("Text payload contract");
    }

    public static byte[] header(int declared) {
        ByteBuffer data = ByteBuffer.allocate(36).order(ByteOrder.LITTLE_ENDIAN);
        for (int i = 0; i < 16; i++) data.put((byte) i);
        for (int i = 240; i < 256; i++) data.put((byte) i);
        return data.putInt(declared).array();
    }

    public static byte[] concat(byte[]... parts) {
        ByteArrayOutputStream data = new ByteArrayOutputStream();
        for (byte[] part : parts) data.writeBytes(part);
        return data.toByteArray();
    }

    public static byte[] record(byte[] text) {
        byte[] padding = new byte[(4 - text.length % 4) % 4];
        Arrays.fill(padding, (byte) 0xa5);
        return concat(header(text.length), text, padding);
    }

    public static byte[] payload(int repeats) {
        ByteArrayOutputStream data = new ByteArrayOutputStream();
        for (int repeat = 0; repeat < repeats; repeat++) {
            for (String text : TEXTS) data.writeBytes(record(text.getBytes(StandardCharsets.UTF_8)));
        }
        return data.toByteArray();
    }

    public static void verify(List<TextState> states) {
        require(states.size() == TEXTS.length);
        for (int i = 0; i < TEXTS.length; i++) {
            TextState state = states.get(i);
            byte[] wire = Arrays.copyOfRange(header(0), 0, 16);
            byte[] icon = Arrays.copyOfRange(header(0), 16, 32);
            require(Arrays.equals(state.uuidWireBytes(), wire));
            require(Arrays.equals(state.iconUuidWireBytes(), icon));
            require(state.uuidNative35().equals("03020100-0504-0706-08090a0b0c0d0e0f"));
            require(state.iconUuidNative35().equals("f3f2f1f0-f5f4-f7f6-f8f9fafbfcfdfeff"));
            require(state.text().equals(TEXTS[i]));
            byte[] exact = TEXTS[i].getBytes(StandardCharsets.UTF_8);
            require(Arrays.equals(state.text().getBytes(StandardCharsets.UTF_8), exact));
            require(Integer.toUnsignedLong(state.textByteLen()) == exact.length);
        }
    }

    public record ErrorCase(int tag, long first, long second, int serializedLength, byte[] input) {
        public void check(TextStatePayloadException failure) {
            if (tag == 1 && failure instanceof TextStatePayloadException.TruncatedHeader f) {
                require(f.recordOffset() == 65536L && f.available() == first);
            } else if (tag == 3 && failure instanceof TextStatePayloadException.TruncatedText f) {
                require(f.recordOffset() == 65536L
                        && Integer.toUnsignedLong(f.declared()) == first && f.available() == second);
            } else if (tag == 4 && failure instanceof TextStatePayloadException.TruncatedPadding f) {
                require(f.recordOffset() == 65536L && f.expected() == first && f.available() == second);
            } else if (tag == 5 && failure instanceof TextStatePayloadException.InvalidUtf8 f) {
                require(f.recordOffset() == 65536L && f.validUpTo() == first);
                require(second == -1L ? f.errorLen() == null : Long.valueOf(second).equals(f.errorLen()));
            } else {
                throw new AssertionError("Wrong typed failure", failure);
            }
            require(!failure.toString().contains("private") && !failure.toString().contains("pppp"));
        }
    }

    public static List<ErrorCase> errors() {
        byte[] text = new byte[65500];
        Arrays.fill(text, (byte) 'p');
        byte[] prefix = record(text);
        require(prefix.length == 65536);
        byte[] shortHeader = new byte[17];
        Arrays.fill(shortHeader, (byte) 0xa5);
        byte[] secret = "private-text".getBytes(StandardCharsets.UTF_8);
        return List.of(
                new ErrorCase(1, 17, 0, 20, concat(prefix, shortHeader)),
                new ErrorCase(3, 0xffffffffL, 7, 24,
                        concat(prefix, header(-1), "private".getBytes(StandardCharsets.UTF_8))),
                new ErrorCase(4, 3, 1, 28, concat(prefix, header(1), new byte[] {'x', (byte) 0xa5})),
                new ErrorCase(5, 12, 1, 29, concat(prefix, record(concat(secret, new byte[] {(byte) 0xff})))),
                new ErrorCase(5, 12, -1, 21,
                        concat(prefix, record(concat(secret, new byte[] {(byte) 0xf0, (byte) 0x9f})))));
    }

    public static void main(String[] args) throws Throwable {
        require(System.getenv("RUST_MIN_STACK") == null);
        require(LoxArchive.decodeTextStatePayload(new byte[0]).isEmpty());
        byte[] data = payload(1);
        byte[] before = data.clone();
        List<TextState> states = LoxArchive.decodeTextStatePayload(data);
        require(Arrays.equals(data, before));
        verify(states);
        Arrays.fill(data, (byte) 0);
        verify(states);
        for (ErrorCase error : errors()) {
            try {
                LoxArchive.decodeTextStatePayload(error.input());
                throw new AssertionError("Expected complete-input rejection, not prefix results");
            } catch (TextStatePayloadException failure) {
                error.check(failure);
            }
            verify(LoxArchive.decodeTextStatePayload(before));
        }
        for (int length = 1; length < 36; length++) {
            try {
                LoxArchive.decodeTextStatePayload(new byte[length]);
                throw new AssertionError("Expected short-header failure");
            } catch (TextStatePayloadException.TruncatedHeader failure) {
                require(failure.recordOffset() == 0 && failure.available() == length);
            }
        }
        NativeTextStateProbe.verifyPath("Java", LoxArchive::decodeTextStatePayload,
                payload(512), 0, 0, 0, 0);
        for (ErrorCase error : errors()) {
            NativeTextStateProbe.verifyPath("Java", LoxArchive::decodeTextStatePayload,
                    error.input(), error.tag(), error.first(), error.second(), error.serializedLength());
        }
        System.out.println("Java text UUID/icon/order/duplicate/UTF8/bytecount/NUL/atomic errors/ownership PASS");
    }
}
