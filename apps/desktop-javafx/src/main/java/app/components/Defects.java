package app.components;

import java.util.Set;

// The app's deliberate defects, switched on with OMNICUI_DEFECTS=name,name… so that every failure can be
// caused on purpose. With none set, the components are correct.
public final class Defects {
    private static final Set<String> ACTIVE = Set.of(System.getenv().getOrDefault("OMNICUI_DEFECTS", "").split(","));

    public static boolean on(String name) {
        return ACTIVE.contains(name);
    }

    private Defects() {}
}
