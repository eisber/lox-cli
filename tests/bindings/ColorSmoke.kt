import net.eisber.lox.archive.*

fun main(args: Array<String>) {
    ColorSmoke.verify("Kotlin", object : ColorSmoke.Api {
        override fun encode(input: ColorEncodeInput, brightness: Double?): ColorEncoded =
            LoxArchive.encodeColor(input, brightness)
        override fun decode(value: String): ColorDecoded = LoxArchive.decodeColorLegacy(value)
        override fun mood(value: String): ColorMood = LoxArchive.parseMoodColorLegacy(value)
        override fun actor(value: Long): ColorActor = LoxArchive.observeActorColorLegacy(value)
        override fun number(value: Double): String = LoxArchive.colorFormatNumber(value)
    }, args[0])
}
