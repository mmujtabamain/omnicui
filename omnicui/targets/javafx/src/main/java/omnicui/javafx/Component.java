package omnicui.javafx;

import java.util.Map;
import javafx.scene.Parent;

// How the adapter mounts a component: its props in, its events out. Components.java adapts each
// of the app's components to it.
public interface Component {
    // The key, in the created root's getProperties(), of a java.util.function.BooleanSupplier that
    // says whether the component is still waiting for something outside the UI (a network reply,
    // a file). The adapter answers no request until it returns false. Optional: a component that
    // never waits doesn't set it. Components.java sets it, so the app needn't know about omnicui.
    String BUSY = "omnicui.busy";

    // contract: props arrive here as a map; the adapter's mount calls this.
    Parent create(Map<String, Object> props, Emitter emit);

    // contract: events.<name> with its fields; the adapter keeps them for events().
    interface Emitter {
        void emit(String event, Map<String, Object> fields);
    }
}
