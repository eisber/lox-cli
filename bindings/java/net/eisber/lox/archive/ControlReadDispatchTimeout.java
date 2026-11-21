package net.eisber.lox.archive;

/** Native shutdown completed, but a caller continuation still owns the wait lane. */
public final class ControlReadDispatchTimeout extends Exception {
    public ControlReadDispatchTimeout() {
        super("Control read host dispatch did not drain before its deadline");
    }
}
