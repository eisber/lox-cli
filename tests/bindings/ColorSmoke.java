package net.eisber.lox.archive;

import java.nio.file.Files;
import java.nio.file.Path;

public final class ColorSmoke {
    public interface Api {
        ColorEncoded encode(ColorEncodeInput input, Double brightness) throws ColorException;
        ColorDecoded decode(String value) throws ColorException;
        ColorMood mood(String value) throws ColorException;
        ColorActor actor(long value) throws ColorException;
        String number(double value);
    }

    public static final Api JAVA = new Api() {
        public ColorEncoded encode(ColorEncodeInput i, Double b) throws ColorException { return LoxArchive.encodeColor(i, b); }
        public ColorDecoded decode(String v) throws ColorException { return LoxArchive.decodeColorLegacy(v); }
        public ColorMood mood(String v) throws ColorException { return LoxArchive.parseMoodColorLegacy(v); }
        public ColorActor actor(long v) throws ColorException { return LoxArchive.observeActorColorLegacy(v); }
        public String number(double v) { return LoxArchive.colorFormatNumber(v); }
    };

    interface Failure { void run() throws ColorException; }
    static ColorException failure(Failure call) throws Exception {
        try { call.run(); } catch (ColorException error) { return error; }
        throw new AssertionError("Expected typed color error");
    }
    static void require(boolean value) { if (!value) throw new AssertionError("Color field mismatch"); }

    public static void verify(String language, Api api, String fixture) throws Throwable {
        require(System.getenv("RUST_MIN_STACK") == null);
        for (String line : Files.readAllLines(Path.of(fixture))) {
            String[] fields = line.split("\t");
            ColorMood value = api.mood(fields[0]);
            require(value.value() == Long.parseLong(fields[1]));
            require(value.percent().equals(new ColorPercentRgb(Integer.valueOf(fields[2]), Integer.valueOf(fields[3]), Integer.valueOf(fields[4]))));
            require(value.approximateRgb().equals(new ColorRgb(Integer.valueOf(fields[5]), Integer.valueOf(fields[6]), Integer.valueOf(fields[7]))));
        }
        ColorEncoded.Rgb red = (ColorEncoded.Rgb) api.encode(new ColorEncodeInput.Rgb(255.0, 0.0, 0.0), null);
        require(red.rgb().equals(new ColorRgb(255, 0, 0)));
        require(red.actor().equals(new ColorActor(255L, red.rgb())));
        require(red.mood().value() == 1610612836L);
        require(red.mood().percent().equals(new ColorPercentRgb(100, 0, 0)));
        require(red.mood().approximateRgb().equals(red.rgb()));
        require(red.hsv().equals(new ColorHsv(0.0, 100.0, 100.0)) && red.hsvCommand().equals("hsv(0,100,100)"));
        ColorEncoded.Rgb rounded = (ColorEncoded.Rgb) api.encode(new ColorEncodeInput.Rgb(0.5, 0.0, 0.0), 50.0);
        require(rounded.rgb().red() == 1);
        require(((ColorEncoded.Rgb) api.encode(new ColorEncodeInput.Hsv(new ColorHsv(360.0, 100.0, 100.0)), null)).rgb().equals(red.rgb()));
        ColorEncoded.Temperature temp = (ColorEncoded.Temperature) api.encode(new ColorEncodeInput.Temperature(2700L), null);
        require(temp.kelvin() == 2700L && temp.brightness() == 100.0 && temp.command().equals("temp(100,2700)"));
        require(((ColorEncoded.Temperature) api.encode(new ColorEncodeInput.Temperature(15000L), 25.5)).command().equals("temp(25.5,15000)"));
        ColorEncoded.Temperature zero = (ColorEncoded.Temperature) api.encode(new ColorEncodeInput.Temperature(1000L), -0.0);
        require(Double.doubleToRawLongBits(zero.brightness()) == Long.MIN_VALUE && zero.command().equals("temp(0,1000)"));
        require(((ColorDecoded.Rgb) api.decode("255")).actor().equals(red.actor()));
        require(((ColorDecoded.Mood) api.decode("1610612836")).mood().equals(red.mood()));
        ColorDecoded.Hsv wrapped = (ColorDecoded.Hsv) api.decode("hsv(-360,100,100)");
        require(wrapped.hsv().hue() == -360.0 && wrapped.rgb().equals(red.rgb()));
        ColorDecoded.Hsv nonfinite = (ColorDecoded.Hsv) api.decode("hsv(NaN,inf,-inf)");
        require(Double.isNaN(nonfinite.hsv().hue()) && nonfinite.hsv().saturation() == Double.POSITIVE_INFINITY
                && nonfinite.hsv().value() == Double.NEGATIVE_INFINITY);
        ColorDecoded.Temperature legacy = (ColorDecoded.Temperature) api.decode("lumitech(-0,NaN)");
        require(Double.doubleToRawLongBits(legacy.brightness()) == Long.MIN_VALUE && Double.isNaN(legacy.kelvin()));
        legacy = (ColorDecoded.Temperature) api.decode("temp(inf,-inf)");
        require(legacy.brightness() == Double.POSITIVE_INFINITY && legacy.kelvin() == Double.NEGATIVE_INFINITY);
        require(api.mood("rgb(255,0,0)").equals(red.mood()) && api.mood("hsv(360,100,100)").equals(red.mood()));
        require(api.mood("255").equals(red.mood()));
        long high = 6610612736L;
        require(api.mood(Long.toString(high)).value() == high);
        require(((ColorDecoded.Mood) api.decode(Long.toString(high))).mood().value() == high);
        require(api.actor(-1000000000L).equals(new ColorActor(-1000000000L, new ColorRgb(0, 0, 0))));
        require(api.actor(-9000000000000000000L).value() == -9000000000000000000L);
        require(api.mood("-1000000000").percent().equals(new ColorPercentRgb(0, 0, 0)));
        require(api.number(-0.0).equals("0") && api.number(Double.NaN).equals("NaN")
                && api.number(Double.POSITIVE_INFINITY).equals("inf") && api.number(25.5).equals("25.5"));
        ColorException.RgbChannel rgb = (ColorException.RgbChannel) failure(() -> api.encode(new ColorEncodeInput.Rgb(0.0, 256.0, 0.0), null));
        require(rgb.channel() == ColorChannel.GREEN && rgb.value() == 256.0);
        rgb = (ColorException.RgbChannel) failure(() -> api.encode(new ColorEncodeInput.Rgb(Double.NaN, 0.0, 0.0), null));
        require(rgb.channel() == ColorChannel.RED && Double.isNaN(rgb.value()));
        ColorException.HsvRange hsv = (ColorException.HsvRange) failure(() -> api.encode(new ColorEncodeInput.Hsv(new ColorHsv(361.0, 2.0, 3.0)), null));
        require(hsv.value().equals(new ColorHsv(361.0, 2.0, 3.0)));
        hsv = (ColorException.HsvRange) failure(() -> api.encode(new ColorEncodeInput.Hsv(new ColorHsv(Double.NaN, 2.0, 3.0)), null));
        require(Double.isNaN(hsv.value().hue()) && hsv.value().saturation() == 2.0 && hsv.value().value() == 3.0);
        require(((ColorException.Brightness) failure(() -> api.encode(new ColorEncodeInput.Temperature(2700L), 101.0))).value() == 101.0);
        require(Double.isNaN(((ColorException.Brightness) failure(() -> api.encode(new ColorEncodeInput.Temperature(2700L), Double.NaN))).value()));
        require(((ColorException.Kelvin) failure(() -> api.encode(new ColorEncodeInput.Temperature(999L), null))).value() == 999L);
        for (long value : new long[] {-1L, Long.MAX_VALUE, 4294967296L}) {
            require(((ColorException.KelvinRepresentation) failure(() -> api.encode(new ColorEncodeInput.Temperature(value), null))).value() == value);
        }
        ColorException.Arity arity = (ColorException.Arity) failure(() -> api.decode("hsv(1,2)"));
        require(arity.value().equals("1,2") && arity.notation() == ColorNotation.HSV_COMMAND && arity.expected() == 3L);
        arity = (ColorException.Arity) failure(() -> api.decode("temp(1)"));
        require(arity.notation() == ColorNotation.TEMPERATURE && arity.expected() == 2L);
        arity = (ColorException.Arity) failure(() -> api.mood("rgb(1,2)"));
        require(arity.notation() == ColorNotation.RGB);
        ColorException.Number number = (ColorException.Number) failure(() -> api.decode("hsv(oops,2,3)"));
        require(number.value().equals("oops") && number.notation() == ColorNotation.HSV_COMMAND);
        ColorException.ActorChannel actor = (ColorException.ActorChannel) failure(() -> api.actor(256L));
        require(actor.value() == 256L && actor.channel() == ColorChannel.RED && actor.channelValue() == 256L);
        actor = (ColorException.ActorChannel) failure(() -> api.actor(Long.MIN_VALUE));
        require(actor.value() == Long.MIN_VALUE && actor.channel() == ColorChannel.RED && actor.channelValue() == -808L);
        ColorException.MoodChannel mood = (ColorException.MoodChannel) failure(() -> api.mood("1610612837"));
        require(mood.value() == 1610612837L && mood.channel() == ColorChannel.RED && mood.channelValue() == 101L);
        mood = (ColorException.MoodChannel) failure(() -> api.mood(Long.toString(Long.MAX_VALUE)));
        require(mood.value() == Long.MAX_VALUE && mood.channel() == ColorChannel.GREEN && mood.channelValue() == 163L);
        for (boolean decode : new boolean[] {false, true}) {
            ColorException.InvalidValue error = (ColorException.InvalidValue) failure(() -> { if (decode) api.decode("secret"); else api.mood("secret"); });
            require(error.value().equals("secret") && error.profile() == (decode ? ColorIntegerProfile.DECODE : ColorIntegerProfile.MOOD));
        }
        require(((ColorException.ActorRange) failure(() -> api.decode("-1000000000"))).value() == -1000000000L);
        require(red.rgb().red() == 255 && red.hsvCommand().equals("hsv(0,100,100)"));
        NativeColorProbe.verify(language, api);
        System.out.println(language + " color typed branches/errors/profiles/i64/nonfinite/host ownership PASS");
    }

    public static void main(String[] args) throws Throwable { verify("Java", JAVA, args[0]); }
}
