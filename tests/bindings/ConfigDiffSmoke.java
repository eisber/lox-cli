import java.util.*;
import net.eisber.lox.archive.*;

public final class ConfigDiffSmoke {
    private static void require(boolean value) {
        if (!value) throw new AssertionError("Typed diff mismatch");
    }

    public static void main(String[] args) throws Throwable {
        var old = new ConfigSummary(" raw Ω ", "", new HashMap<>(Map.of(
                " Same ", new ConfigControlEntry("Old", "Switch", "r"),
                "A", new ConfigControlEntry("Twin", "Switch", ""),
                "stable", new ConfigControlEntry("Stable", "Switch", "r"))),
                new HashMap<>(Map.of("r", "", "gone", "Twin")),
                new HashMap<>(Map.of("c", "Old", "gone", "Twin")),
                new ArrayList<>(List.of("", "Ω", "Ω")));
        var newer = new ConfigSummary("new", "today", new HashMap<>(Map.of(
                " Same ", new ConfigControlEntry("Neu Ω", "Dimmer", "missing"),
                "a", new ConfigControlEntry("Twin", "Switch", ""),
                "stable", new ConfigControlEntry("Stable", "Switch", "r"))),
                new HashMap<>(Map.of("r", "renamed", "added", "Twin")),
                new HashMap<>(Map.of("c", "New", "added", "Twin")),
                new ArrayList<>(List.of("Ω", "Added", "Added")));
        ConfigDiff result = LoxArchive.diffConfigs(old, newer);
        require(result.hasChanges());
        require(result.versionOld().equals(" raw Ω ") && result.versionNew().equals("new"));
        require(result.dateOld().isEmpty() && result.dateNew().equals("today"));
        require(result.controlsAdded().equals(List.of(new ConfigControlRecord("a",
                new ConfigControlEntry("Twin", "Switch", "")))));
        require(result.controlsRemoved().equals(List.of(new ConfigControlRecord("A",
                new ConfigControlEntry("Twin", "Switch", "")))));
        require(result.controlsChanged().equals(List.of(
                new ConfigControlChange(" Same ", "Neu Ω", new ConfigControlFieldChange.Name("Old", "Neu Ω")),
                new ConfigControlChange(" Same ", "Neu Ω", new ConfigControlFieldChange.Type("Switch", "Dimmer")),
                new ConfigControlChange(" Same ", "Neu Ω", new ConfigControlFieldChange.Room(
                        new ConfigRoomReference("r", ""), new ConfigRoomReference("missing", null))))));
        require(result.roomsAdded().equals(List.of(new ConfigNamedEntry("added", "Twin"))));
        require(result.roomsRemoved().equals(List.of(new ConfigNamedEntry("gone", "Twin"))));
        require(result.roomsRenamed().equals(List.of(new ConfigNameChange("r", "", "renamed"))));
        require(result.categoriesAdded().equals(List.of(new ConfigNamedEntry("added", "Twin"))));
        require(result.categoriesRemoved().equals(List.of(new ConfigNamedEntry("gone", "Twin"))));
        require(result.categoriesRenamed().equals(List.of(new ConfigNameChange("c", "Old", "New"))));
        require(result.usersAdded().equals(List.of("Added")) && result.usersRemoved().equals(List.of("")));
        var reverse = LoxArchive.diffConfigs(newer, old);
        require(reverse.controlsChanged().get(2).field().equals(new ConfigControlFieldChange.Room(
                new ConfigRoomReference("missing", null), new ConfigRoomReference("r", ""))));
        var metadata = new ConfigSummary("new", "today", old.controls(), old.rooms(), old.categories(),
                new ArrayList<>(List.of("Ω", "")));
        var meta = LoxArchive.diffConfigs(old, metadata);
        require(!meta.hasChanges() && meta.versionNew().equals("new") && meta.dateNew().equals("today"));
        var renamedRooms = new HashMap<>(old.rooms());
        renamedRooms.put("r", "rename only");
        var renamed = LoxArchive.diffConfigs(old, new ConfigSummary("v", "d", old.controls(),
                renamedRooms, old.categories(), old.users()));
        require(renamed.hasChanges() && renamed.controlsChanged().isEmpty() && renamed.roomsRenamed().size() == 1);
        old.controls().clear();
        newer.rooms().clear();
        newer.users().clear();
        require(result.roomsRenamed().get(0)._new().equals("renamed"));
        require(result.controlsChanged().get(2).field() instanceof ConfigControlFieldChange.Room room
                && room.old().name().isEmpty() && room._new().name() == null);
        NativeConfigDiffProbe.verify("Java", LoxArchive::diffConfigs);
        System.out.println("Java typed structural diff: all variants/metadata/sets/raw UUIDs/Unicode/ownership PASS");
    }
}
