import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.util.Arrays;
import java.util.List;
import net.eisber.lox.archive.*;

public final class WeatherStateSmoke {
    public static final long[] BITS = {0, 0x8000000000000000L, 0x7ff0000000000000L,
            0xfff0000000000000L, 0x7ff0000000001234L, 0xfff8123456789abcL, -1, 1};

    public static void require(boolean value) {
        if (!value) throw new AssertionError("Weather payload contract");
    }

    public static void header(ByteBuffer data, int update, int count) {
        for (int j = 0; j < 16; j++) data.put((byte) j);
        data.putInt(update).putInt(count);
    }

    public static byte[] payload(int repeats) {
        ByteBuffer data = ByteBuffer.allocate(228 * BITS.length * repeats + 24)
                .order(ByteOrder.LITTLE_ENDIAN);
        for (int repeat = 0; repeat < repeats; repeat++) {
            for (int i = 0; i < BITS.length; i++) {
                header(data, -1 - i, 3);
                for (int j = 0; j < 3; j++) {
                    data.putInt(Integer.MIN_VALUE).putInt(-83).putInt(Integer.MAX_VALUE)
                            .putInt(-7).putInt(-1024);
                    for (int k = 0; k < 6; k++) data.putLong(BITS[(i + j + k) % BITS.length]);
                }
            }
        }
        header(data, Integer.MIN_VALUE, 0);
        return data.array();
    }

    public static void verify(List<WeatherState> states) {
        require(states.size() == BITS.length + 1);
        byte[] wire = new byte[16];
        for (int j = 0; j < 16; j++) wire[j] = (byte) j;
        for (int i = 0; i < BITS.length; i++) {
            WeatherState state = states.get(i);
            require(Arrays.equals(state.uuidWireBytes(), wire));
            require(state.uuidNative35().equals("03020100-0504-0706-08090a0b0c0d0e0f"));
            require(Integer.toUnsignedLong(state.lastUpdate()) == 0xffffffffL - i);
            require(state.entryCount() == 3 && state.entries().size() == 3);
            for (int j = 0; j < 3; j++) {
                WeatherEntry entry = state.entries().get(j);
                require(entry.timestamp() == Integer.MIN_VALUE && entry.weatherType() == -83
                        && entry.windDirection() == Integer.MAX_VALUE
                        && entry.solarRadiation() == -7 && entry.relativeHumidity() == -1024);
                long[] bits = {entry.temperatureBits(), entry.perceivedTemperatureBits(),
                        entry.dewPointBits(), entry.precipitationBits(), entry.windSpeedBits(),
                        entry.barometricPressureBits()};
                for (int k = 0; k < 6; k++) require(bits[k] == BITS[(i + j + k) % BITS.length]);
            }
        }
        require(states.getLast().entries().isEmpty() && states.getLast().entryCount() == 0);
        require(Integer.toUnsignedLong(states.getLast().lastUpdate()) == 0x80000000L);
    }

    public static void functional() throws Throwable {
        require(System.getenv("RUST_MIN_STACK") == null);
        require(LoxArchive.decodeWeatherStatePayload(new byte[0]).isEmpty());
        byte[] data = payload(1);
        byte[] before = data.clone();
        List<WeatherState> owned = LoxArchive.decodeWeatherStatePayload(data);
        require(Arrays.equals(data, before));
        verify(owned);
        for (int length = 1; length < 228; length++) {
            ByteBuffer malformed = ByteBuffer.allocate(24 + length).order(ByteOrder.LITTLE_ENDIAN);
            header(malformed, 0, 0);
            malformed.put(before, 0, length);
            try {
                LoxArchive.decodeWeatherStatePayload(malformed.array());
                throw new AssertionError("Atomic rejection required");
            } catch (WeatherStatePayloadException.TruncatedHeader failure) {
                require(length < 24 && failure.recordOffset() == 24 && failure.available() == length);
            } catch (WeatherStatePayloadException.TruncatedEntries failure) {
                require(length >= 24 && failure.recordOffset() == 24 && failure.declared() == 3
                        && failure.expected() == 204 && failure.available() == length - 24);
            }
        }
        for (int count : new int[] {-1, Integer.MIN_VALUE, Integer.MAX_VALUE}) {
            ByteBuffer malformed = ByteBuffer.allocate(48).order(ByteOrder.LITTLE_ENDIAN);
            header(malformed, 0, 0);
            header(malformed, 0, count);
            try {
                LoxArchive.decodeWeatherStatePayload(malformed.array());
                throw new AssertionError("Count rejection required");
            } catch (WeatherStatePayloadException.NegativeEntryCount failure) {
                require(count < 0 && failure.recordOffset() == 24 && failure.declared() == count);
            } catch (WeatherStatePayloadException.TruncatedEntries failure) {
                require(count == Integer.MAX_VALUE && failure.recordOffset() == 24
                        && failure.declared() == count && failure.expected() == count * 68L
                        && failure.available() == 0);
            }
        }
        Arrays.fill(data, (byte) 0);
        verify(owned);
    }

    public static void main(String[] args) throws Throwable {
        functional();
        NativeWeatherStateProbe.verify("Java", LoxArchive::decodeWeatherStatePayload);
        System.out.println("Java Weather unsigned/signed/sixIEEEbits/atomic/errors/ownership PASS");
    }
}
