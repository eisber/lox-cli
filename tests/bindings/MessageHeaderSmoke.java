import net.eisber.lox.archive.*;
import java.util.Arrays;

public final class MessageHeaderSmoke {
    public static void values(NativeMessageHeaderProbe.Operation operation) throws Throwable {
        MessageType[] known = MessageType.values();
        for (int kind : new int[] {0, 1, 2, 3, 4, 5, 6, 7, 0xe7, 0xff}) {
            for (int flags : new int[] {0x80, 0xff}) {
                byte[] wire = {3, (byte)kind, (byte)flags, (byte)0x91,
                        0x76, 0x54, 0x32, (byte)0xf0};
                if (!Arrays.equals(wire, LoxArchive.encodeMessageHeader(
                        (byte)kind, (byte)flags, (byte)0x91, 0xf0325476)))
                    throw new AssertionError("Exact Core encoding");
                MessageHeader h = operation.run(wire);
                if (Byte.toUnsignedInt(h.messageType()) != kind
                        || Byte.toUnsignedInt(h.infoFlags()) != flags
                        || Byte.toUnsignedInt(h.reserved()) != 0x91
                        || Integer.toUnsignedLong(h.payloadSize()) != 0xf0325476L
                        || h.knownType() != (kind < 8 ? known[kind] : null)
                        || h.sizeIsEstimated() != ((flags & 1) != 0)
                        || !Arrays.equals(h.wireBytes(), wire))
                    throw new AssertionError("Owned unsigned observation");
                Arrays.fill(wire, (byte)0);
                if (h.wireBytes()[0] != 3) throw new AssertionError("Caller ownership");
            }
        }
        for (int length : new int[] {0, 1, 7, 9, 32}) {
            try {
                operation.run(new byte[length]);
                throw new AssertionError("Expected Core length failure");
            } catch (MessageHeaderException.InvalidLength failure) {
                if (failure.actual() != length) throw failure;
            }
        }
        try {
            operation.run(new byte[] {(byte)0x83, 0, 0, 0, 0, 0, 0, 0});
            throw new AssertionError("Expected Core magic failure");
        } catch (MessageHeaderException.InvalidMagic failure) {
            if (Byte.toUnsignedInt(failure.actual()) != 0x83) throw failure;
        }
    }

    public static void main(String[] args) throws Throwable {
        values(LoxArchive::decodeMessageHeader);
        System.out.println("Java actual native header values/unsigned bits/roundtrip/errors PASS");
        NativeMessageHeaderProbe.verify("Java", LoxArchive::decodeMessageHeader);
    }
}
