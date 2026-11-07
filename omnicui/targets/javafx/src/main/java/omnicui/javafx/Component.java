package omnicui.javafx;

import java.util.Map;
import javafx.scene.Parent;

// How the adapter mounts a component: its props in, its events out. Components.java adapts each
// of the app's components to it.
public interface Component {
    // contract: props arrive here as a map; the adapter's mount calls this.
    Parent create(Map<String, Object> props, Emitter emit);

    // contract: events.<name> with its fields; the adapter keeps them for events().
    interface Emitter {
        void emit(String event, Map<String, Object> fields);
    }
}
