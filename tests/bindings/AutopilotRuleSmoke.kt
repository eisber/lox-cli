import net.eisber.lox.archive.*

fun main() {
    val operation = NativeAutopilotRuleProbe.Operation { rows, query ->
        LoxArchive.resolveAutopilotRule(rows, query)
    }
    val selected: AutopilotRule = operation.run(
        listOf(AutopilotRule("Night", "raw\u0000ID", "changed", "history")), "night")
    check(selected.uuid() == "raw\u0000ID")
    AutopilotRuleSmoke.values(operation)
    NativeAutopilotRuleProbe.verify("Kotlin", operation)
    println("Kotlin SAME Java supplied typed Autopilot selection/owned values/errors PASS")
}
