package net.eisber.lox.archive;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;
import java.nio.charset.StandardCharsets;
import java.util.HexFormat;

public final class AuthHashSmoke {
    private static final SymbolLookup PROBE = SymbolLookup.libraryLookup(
            System.getProperty("lox.archive.heapProbe"), Arena.global());
    private static final MethodHandle LIVE = Linker.nativeLinker().downcallHandle(
            PROBE.find("lox_test_heap_live").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS));
    private static final MethodHandle COUNT = Linker.nativeLinker().downcallHandle(
            PROBE.find("lox_test_heap_auth_error_count").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.JAVA_INT, ValueLayout.ADDRESS));
    private static final MethodHandle HEX_COUNT = Linker.nativeLinker().downcallHandle(
            PROBE.find("lox_test_heap_auth_hex_count").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS));
    private static final String TOKEN_HEX =
            "4b59c58e38a81ab3dbe92b052651a678a6f5a175175ba01f1f1853849b25da89";

    private static void require(boolean condition) {
        if (!condition) throw new AssertionError("Auth assertion");
    }

    private static long count(int tag) throws Throwable {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment result = arena.allocate(ValueLayout.JAVA_LONG);
            int code = tag == 0 ? (int) HEX_COUNT.invokeExact(result)
                    : (int) COUNT.invokeExact(tag, result);
            require(code == 0);
            return result.get(ValueLayout.JAVA_LONG, 0);
        }
    }

    private static boolean live(long handle) throws Throwable {
        return (int) LIVE.invokeExact(MemorySegment.ofAddress(handle - 16)) != 0;
    }

    private static void check(AuthenticationHash value, String expected) throws Exception {
        try (value) {
            require(value.toHex().equals(expected));
            require(!value.toString().contains(expected));
            value.dispose();
            value.dispose();
            try {
                value.toHex();
                throw new AssertionError("Disposed secret readable");
            } catch (AuthenticationHashException.Closed failure) {
                require(failure.message().equals("authentication hash is closed"));
            }
        }
        value.close();
        try {
            value.toHex();
            throw new AssertionError("Closed handle readable");
        } catch (IllegalStateException expectedFailure) {
            require(!expectedFailure.toString().contains(expected));
        }
    }

    public static void verify() throws Throwable {
        byte[] key = HexFormat.of().parseHex("001122aaccee");
        String app = "88bfb35cc1a3ce183b6fcd66b3b5fe3acbdaba620c3ce041d866ce6d055e2bb2";
        check(LoxArchive.hashAppCredentials("us:ér", "päss:word", key), app);
        check(LoxArchive.hashAppCredentialsHex("us:ér", "päss:word", "001122aaccee"), app);
        for (WsxPasswordDigest digest : WsxPasswordDigest.values()) {
            String expected = digest == WsxPasswordDigest.SHA1
                    ? "9d063ec0b204dd0a2368b4be99e052704579ada03e5b755e2fa4452646473790"
                    : "dffd58fcc0395555f0ea22c20b7f613cf1f2cb9103864f0f340c99bb3776a79c";
            check(LoxArchive.hashWsxCredentials("us:ér", "päss:word", "s:älт", digest, key), expected);
            check(LoxArchive.hashWsxCredentialsHex(
                    "us:ér", "päss:word", "s:älт", digest, "001122aaccee"), expected);
        }
        byte[] unicodeKey = "clé:key".getBytes(StandardCharsets.UTF_8);
        check(LoxArchive.hashAppToken("jwt:тøken", unicodeKey), TOKEN_HEX);
        check(LoxArchive.hashWsxAutht("jwt:тøken", unicodeKey), TOKEN_HEX.toUpperCase(java.util.Locale.ROOT));
        check(LoxArchive.hashAppTokenHex("jwt:тøken", HexFormat.of().formatHex(unicodeKey)), TOKEN_HEX);
        check(LoxArchive.hashWsxAuthtHex("jwt:тøken", HexFormat.of().formatHex(unicodeKey)),
                TOKEN_HEX.toUpperCase(java.util.Locale.ROOT));
        check(LoxArchive.hashAppTokenHex("tok", ""),
                "3d6cb1ab5aa9e075eeb3a608a885b15452f8f9c50a7d33aef07acfd8fdfdc6e1");
        try (AuthenticationHash raw = LoxArchive.hashAppToken("tok", new byte[]{102, 102});
             AuthenticationHash decoded = LoxArchive.hashAppTokenHex("tok", "ff")) {
            require(!raw.toHex().equals(decoded.toHex()));
        }
        for (String invalid : new String[]{"a", "🌸"}) {
            try {
                LoxArchive.hashAppTokenHex("synthetic-secret-token", invalid);
                throw new AssertionError("Invalid hex accepted");
            } catch (AuthenticationHashException.InvalidHexKey failure) {
                require(failure.message().equals("invalid hex authentication key"));
            }
        }
        try {
            LoxArchive.hashWsxAutht("synthetic-secret-token", new byte[]{(byte) 255});
            throw new AssertionError("Non-UTF8 key accepted");
        } catch (AuthenticationHashException.NonUtf8WsxKey failure) {
            require(!failure.toString().contains("synthetic-secret"));
        }
        try (AuthenticationHash composed = LoxArchive.hashAppToken("é", new byte[]{1});
             AuthenticationHash decomposed = LoxArchive.hashAppToken("e\u0301", new byte[]{1})) {
            require(!composed.toHex().equals(decomposed.toHex()));
        }
        for (int tag : new int[]{1, 2}) {
            String keyInput = tag == 1 ? "synthetic-secret-key" : "ff";
            try (Arena arena = Arena.ofConfined()) {
                MemorySegment status = UniffiRustCallStatus.allocate(arena);
                MemorySegment token = FfiConverterString.INSTANCE.lower("synthetic-secret-token");
                MemorySegment encodedKey = FfiConverterString.INSTANCE.lower(keyInput);
                long result = tag == 1
                        ? UniffiLib.uniffi_lox_archive_fn_func_hash_app_token_hex(token, encodedKey, status)
                        : UniffiLib.uniffi_lox_archive_fn_func_hash_wsx_autht_hex(token, encodedKey, status);
                require(result == 0 && UniffiRustCallStatus.isError(status));
                require(count(tag) == 1);
                RustBuffer.free(UniffiRustCallStatus.getErrorBuf(status));
                require(count(tag) == 0);
            }
            for (int i = 0; i < 64; i++) {
                try {
                    if (tag == 1) LoxArchive.hashAppTokenHex("synthetic-secret-token", keyInput);
                    else LoxArchive.hashWsxAuthtHex("synthetic-secret-token", keyInput);
                    throw new AssertionError("Expected typed error");
                } catch (AuthenticationHashException failure) {
                    require(tag == 1 ? failure instanceof AuthenticationHashException.InvalidHexKey
                            : failure instanceof AuthenticationHashException.NonUtf8WsxKey);
                    require(!failure.toString().contains("synthetic-secret"));
                }
            }
            require(count(tag) == 0);
            System.out.println("JVM auth error tag=" + tag
                    + " actual_live=1 freed=0 calls=64 retained=0 GC_requested=false");
        }
        for (int i = 0; i < 64; i++) {
            AuthenticationHash value = LoxArchive.hashAppToken("synthetic-secret-token", new byte[]{1, 2, 3});
            long handle = value.handle;
            require(live(handle));
            value.dispose();
            value.close();
            value.close();
            require(!live(handle));
        }
        System.out.println("JVM auth actual object live=1 freed=0 calls=64 retained=0 GC_requested=false");
        try (AuthenticationHash value = LoxArchive.hashAppToken("jwt:тøken", unicodeKey)) {
            for (boolean closed : new boolean[]{false, true}) {
                if (closed) value.dispose();
                int tag = closed ? 3 : 0;
                try (Arena arena = Arena.ofConfined()) {
                    MemorySegment status = UniffiRustCallStatus.allocate(arena);
                    MemorySegment result = UniffiLib.uniffi_lox_archive_fn_method_authenticationhash_to_hex(
                            arena, value.uniffiCloneHandle(), status);
                    require(UniffiRustCallStatus.isError(status) == closed);
                    require(count(tag) == 1);
                    if (closed) RustBuffer.free(UniffiRustCallStatus.getErrorBuf(status));
                    RustBuffer.free(result);
                    require(count(tag) == 0);
                }
                for (int i = 0; i < 64; i++) {
                    try {
                        require(value.toHex().equals(TOKEN_HEX) && !closed);
                    } catch (AuthenticationHashException.Closed failure) {
                        require(closed && failure.message().equals("authentication hash is closed"));
                    }
                }
                require(count(tag) == 0);
                System.out.println("JVM auth reveal closed=" + closed
                        + " actual_live=1 freed=0 calls=64 retained=0 GC_requested=false");
            }
        }
        System.out.println("JVM auth profiles/algorithms/UTF8/hex/redaction/disposal PASS");
    }

    public static void main(String[] args) throws Throwable {
        verify();
    }
}
