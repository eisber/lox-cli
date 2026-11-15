import net.eisber.lox.archive.*

fun main() {
    val old = ConfigSummary(" raw Ω ", "", hashMapOf(
        " Same " to ConfigControlEntry("Old", "Switch", "r"),
        "A" to ConfigControlEntry("Twin", "Switch", ""),
        "stable" to ConfigControlEntry("Stable", "Switch", "r")),
        hashMapOf("r" to "", "gone" to "Twin"), hashMapOf("c" to "Old", "gone" to "Twin"),
        arrayListOf("", "Ω", "Ω"))
    val newer = ConfigSummary("new", "today", hashMapOf(
        " Same " to ConfigControlEntry("Neu Ω", "Dimmer", "missing"),
        "a" to ConfigControlEntry("Twin", "Switch", ""),
        "stable" to ConfigControlEntry("Stable", "Switch", "r")),
        hashMapOf("r" to "renamed", "added" to "Twin"), hashMapOf("c" to "New", "added" to "Twin"),
        arrayListOf("Ω", "Added", "Added"))
    val result = LoxArchive.diffConfigs(old, newer)
    check(result.hasChanges())
    check(result.versionOld() == " raw Ω " && result.versionNew() == "new")
    check(result.dateOld() == "" && result.dateNew() == "today")
    check(result.controlsAdded() == listOf(ConfigControlRecord("a", ConfigControlEntry("Twin", "Switch", ""))))
    check(result.controlsRemoved() == listOf(ConfigControlRecord("A", ConfigControlEntry("Twin", "Switch", ""))))
    check(result.controlsChanged() == listOf(
        ConfigControlChange(" Same ", "Neu Ω", ConfigControlFieldChange.Name("Old", "Neu Ω")),
        ConfigControlChange(" Same ", "Neu Ω", ConfigControlFieldChange.Type("Switch", "Dimmer")),
        ConfigControlChange(" Same ", "Neu Ω", ConfigControlFieldChange.Room(
            ConfigRoomReference("r", ""), ConfigRoomReference("missing", null)))))
    check(result.roomsAdded() == listOf(ConfigNamedEntry("added", "Twin")))
    check(result.roomsRemoved() == listOf(ConfigNamedEntry("gone", "Twin")))
    check(result.roomsRenamed() == listOf(ConfigNameChange("r", "", "renamed")))
    check(result.categoriesAdded() == listOf(ConfigNamedEntry("added", "Twin")))
    check(result.categoriesRemoved() == listOf(ConfigNamedEntry("gone", "Twin")))
    check(result.categoriesRenamed() == listOf(ConfigNameChange("c", "Old", "New")))
    check(result.usersAdded() == listOf("Added") && result.usersRemoved() == listOf(""))
    val reverse = LoxArchive.diffConfigs(newer, old)
    check(reverse.controlsChanged()[2].field() == ConfigControlFieldChange.Room(
        ConfigRoomReference("missing", null), ConfigRoomReference("r", "")))
    val metadata = ConfigSummary("new", "today", old.controls(), old.rooms(), old.categories(), arrayListOf("Ω", ""))
    val meta = LoxArchive.diffConfigs(old, metadata)
    check(!meta.hasChanges() && meta.versionNew() == "new" && meta.dateNew() == "today")
    val rooms = HashMap(old.rooms())
    rooms["r"] = "rename only"
    val renamed = LoxArchive.diffConfigs(old, ConfigSummary("v", "d", old.controls(), rooms, old.categories(), old.users()))
    check(renamed.hasChanges() && renamed.controlsChanged().isEmpty() && renamed.roomsRenamed().size == 1)
    old.controls().clear()
    newer.rooms().clear()
    newer.users().clear()
    check(result.roomsRenamed()[0]._new() == "renamed")
    val room = result.controlsChanged()[2].field() as ConfigControlFieldChange.Room
    check(room.old().name() == "" && room._new().name() == null)
    NativeConfigDiffProbe.verify("Kotlin") { a, b -> LoxArchive.diffConfigs(a, b) }
    println("Kotlin typed structural diff: SAME Java API/all variants/metadata/sets/raw UUIDs/Unicode/ownership PASS")
}
