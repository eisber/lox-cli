package net.eisber.lox.archive;

/** Native shutdown completed, but the bounded head-worker drain did not. */
public final class ClockDispatchTimeout extends Exception {
    public ClockDispatchTimeout() {
        super("Clock host dispatch did not drain; release caller continuations and retry close");
    }
}
