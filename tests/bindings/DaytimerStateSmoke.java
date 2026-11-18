import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.util.Arrays;
import java.util.List;
import net.eisber.lox.archive.*;

public final class DaytimerStateSmoke {
    public static final long[] BITS = {0, 0x8000000000000000L, 0x7ff0000000000000L,
            0xfff0000000000000L, 0x7ff0000000001234L, 0xfff8123456789abcL, -1, 1};

    public static void require(boolean value) {
        if (!value) throw new AssertionError("Daytimer payload contract");
    }

    public static void header(ByteBuffer data, long bits, int count) {
        for (int j = 0; j < 16; j++) data.put((byte) j);
        data.putLong(bits).putInt(count);
    }

    public static byte[] payload(int repeats) {
        ByteBuffer data = ByteBuffer.allocate(100 * BITS.length * repeats + 28)
                .order(ByteOrder.LITTLE_ENDIAN);
        for (int repeat = 0; repeat < repeats; repeat++) {
            for (long bits : BITS) {
                header(data, bits, 3);
                for (int word : new int[] {0, -7, -7}) {
                    data.putInt(Integer.MIN_VALUE).putInt(-83).putInt(Integer.MAX_VALUE)
                            .putInt(word).putLong(bits);
                }
            }
        }
        header(data, 1, 0);
        return data.array();
    }

    public static void verify(List<DaytimerState> states) {
        require(states.size() == BITS.length + 1);
        byte[] wire = new byte[16];
        for (int j = 0; j < 16; j++) wire[j] = (byte) j;
        for (int i = 0; i < BITS.length; i++) {
            DaytimerState state = states.get(i);
            require(Arrays.equals(state.uuidWireBytes(), wire));
            require(state.uuidNative35().equals("03020100-0504-0706-08090a0b0c0d0e0f"));
            require(state.defaultValueBits() == BITS[i] && state.entries().size() == 3);
            for (int j = 0; j < 3; j++) {
                DaytimerEntry entry = state.entries().get(j);
                require(entry.mode() == Integer.MIN_VALUE && entry.fromMinutes() == -83
                        && entry.toMinutes() == Integer.MAX_VALUE);
                require(entry.activationWord() == (j == 0 ? 0 : -7)
                        && entry.needsActivation() == (j != 0) && entry.valueBits() == BITS[i]);
            }
        }
        require(states.getLast().entries().isEmpty() && states.getLast().defaultValueBits() == 1);
    }

    public static void functional() throws Throwable {
        require(System.getenv("RUST_MIN_STACK") == null);
        require(LoxArchive.decodeDaytimerStatePayload(new byte[0]).isEmpty());
        byte[] data = payload(1);
        byte[] before = data.clone();
        List<DaytimerState> owned = LoxArchive.decodeDaytimerStatePayload(data);
        require(Arrays.equals(data, before));
        verify(owned);
        for (int length = 1; length < 100; length++) {
            ByteBuffer malformed = ByteBuffer.allocate(28 + length).order(ByteOrder.LITTLE_ENDIAN);
            header(malformed, 0, 0);
            malformed.put(before, 0, length);
            try {
                LoxArchive.decodeDaytimerStatePayload(malformed.array());
                throw new AssertionError("Atomic rejection required");
            } catch (DaytimerStatePayloadException.TruncatedHeader failure) {
                require(length < 28 && failure.recordOffset() == 28 && failure.available() == length);
            } catch (DaytimerStatePayloadException.TruncatedEntries failure) {
                require(length >= 28 && failure.recordOffset() == 28
                        && failure.declared() == 3 && failure.available() == length - 28);
            }
        }
        for (int count : new int[] {-1, Integer.MIN_VALUE, Integer.MAX_VALUE}) {
            ByteBuffer malformed = ByteBuffer.allocate(56).order(ByteOrder.LITTLE_ENDIAN);
            header(malformed, 0, 0);
            header(malformed, 0, count);
            try {
                LoxArchive.decodeDaytimerStatePayload(malformed.array());
                throw new AssertionError("Count rejection required");
            } catch (DaytimerStatePayloadException.NegativeEntryCount failure) {
                require(count < 0 && failure.recordOffset() == 28 && failure.declared() == count);
            } catch (DaytimerStatePayloadException.TruncatedEntries failure) {
                require(count == Integer.MAX_VALUE && failure.recordOffset() == 28
                        && failure.declared() == count && failure.available() == 0);
            }
        }
        Arrays.fill(data, (byte) 0);
        verify(owned);
    }

    public static void main(String[] args) throws Throwable {
        functional();
        NativeDaytimerStateProbe.verify("Java", LoxArchive::decodeDaytimerStatePayload);
        System.out.println("Java Daytimer ordered/duplicate/signed/IEEEbits/atomic/errors/ownership PASS");
    }
}
