package omnicui.javafx;

import app.components.Counter;
import app.components.SwitchControl;
import java.util.Map;

// The registry: which of the app's components answers to which contract id, and how the test's
// props and the component's events become the contract's. One entry per component.
final class Components {
    // contract: the key is the contract's id (counter.contract.yaml → id: ui.counter).
    static final Map<String, Component> ALL = Map.of(
        "ui.counter", (props, emit) -> {
            var c = new Counter(((Number) props.getOrDefault("start", 0)).intValue());
            c.setOnChanged(count -> emit.emit("changed", Map.of("count", count)));
            return c;
        },
        "ui.switch", (props, emit) -> {
            var s = new SwitchControl((Boolean) props.getOrDefault("on", false), (String) props.getOrDefault("label", "Dark mode"));
            s.setOnToggled(on -> emit.emit("toggled", Map.of("on", on)));
            return s;
        }
    );

    private Components() {}
}
