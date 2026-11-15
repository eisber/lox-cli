import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.util.Arrays;
import java.util.List;
import net.eisber.lox.archive.*;

public final class NumericStateSmoke {
    public static final long[] BITS = {0L, 1L, 0x3ff0000000000000L, 0xbff0000000000000L,
            0x8000000000000001L, 0x8000000000000000L, 0x7ff0000000000000L,
            0xfff0000000000000L, 0x7ff0000000000001L, 0xfff8123456789abcL,
            0xffffffffffffffffL};

    public static void require(boolean value) {
        if (!value) throw new AssertionError("Numeric payload contract");
    }

    public static byte[] payload(int repeats) {
        ByteBuffer data = ByteBuffer.allocate(24 * BITS.length * repeats).order(ByteOrder.LITTLE_ENDIAN);
        for (int repeat = 0; repeat < repeats; repeat++) {
            for (int i = 0; i < BITS.length; i++) {
                for (int j = 0; j < 16; j++) data.put((byte) (i % 2 == 0 ? j : 15 - j));
                data.putLong(BITS[i]);
            }
        }
        return data.array();
    }

    public static void verify(List<NumericState> states) {
        require(states.size() == BITS.length);
        for (int i = 0; i < BITS.length; i++) {
            NumericState state = states.get(i);
            byte[] wire = new byte[16];
            for (int j = 0; j < 16; j++) wire[j] = (byte) (i % 2 == 0 ? j : 15 - j);
            require(Arrays.equals(state.uuidWireBytes(), wire));
            require(state.uuidNative35().equals(i % 2 == 0
                    ? "03020100-0504-0706-08090a0b0c0d0e0f" : "0c0d0e0f-0a0b-0809-0706050403020100"));
            // Java long is signed, but retains all u64 bits, including bit 63.
            require(state.valueBits() == BITS[i]);
            if (Double.isNaN(Double.longBitsToDouble(BITS[i]))) {
                require(Double.isNaN(state.value()));
            } else {
                require(Double.doubleToRawLongBits(state.value()) == BITS[i]);
            }
        }
    }

    public static void main(String[] args) throws Throwable {
        require(System.getenv("RUST_MIN_STACK") == null);
        require(LoxArchive.decodeValueStatePayload(new byte[0]).isEmpty());
        byte[] data = payload(1);
        byte[] before = data.clone();
        List<NumericState> states = LoxArchive.decodeValueStatePayload(data);
        require(Arrays.equals(data, before));
        verify(states);
        for (int length = 1; length <= 72; length++) {
            if (length % 24 == 0) continue;
            byte[] malformed = new byte[length];
            Arrays.fill(malformed, (byte) 0xa5);
            try {
                LoxArchive.decodeValueStatePayload(malformed);
                throw new AssertionError("Expected atomic typed length failure");
            } catch (NumericStatePayloadException.InvalidLength failure) {
                require(failure.actual() == length && failure.trailing() == length % 24);
                require(!failure.toString().contains("a5"));
            }
            verify(LoxArchive.decodeValueStatePayload(before));
        }
        Arrays.fill(data, (byte) 0);
        verify(states);
        NativeNumericStateProbe.verify("Java", LoxArchive::decodeValueStatePayload);
        System.out.println("Java numeric payload UUID/order/duplicate/IEEEbits/errors/host ownership PASS");
    }
}
