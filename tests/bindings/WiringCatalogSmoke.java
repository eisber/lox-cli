import net.eisber.lox.archive.*;

public final class WiringCatalogSmoke {
    public static void main(String[] args) throws Throwable {
        NativeWiringCatalogProbe.verify("Java", LoxArchive::listConnectorWiring);
        System.out.println("Java selected wiring catalog PASS");
    }
}
