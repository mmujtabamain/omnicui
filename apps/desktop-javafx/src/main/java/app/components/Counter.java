package app.components;

import java.util.function.IntConsumer;
import javafx.beans.property.IntegerProperty;
import javafx.beans.property.SimpleIntegerProperty;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.layout.HBox;

// contract: omnicui/contracts/counter.contract.yaml (id: ui.counter). The omnicui javafx target
// registers it in its Components.java.
// Written the JavaFX way: an IntegerProperty, a Label bound to it, and a Button.
public final class Counter extends HBox {
    // contract: parts.value and parts.increment. The omnicui target finds these nodes by id.
    public static final String VALUE = "ui.counter.value", INCREMENT = "ui.counter.increment";

    private final IntegerProperty count = new SimpleIntegerProperty();
    private IntConsumer onChanged = n -> {};

    public Counter() {
        this(0); // contract: props.start, default 0
    }

    public Counter(int start) {
        count.set(start); // contract: props.start

        var value = new Label();
        value.setId(VALUE);
        value.textProperty().bind(count.asString());

        var increment = new Button("+");
        increment.setId(INCREMENT);
        increment.setAccessibleText("Increment"); // what read(increment, "name") answers
        increment.setOnAction(e -> {
            count.set(count.get() + 1);
            onChanged.accept(count.get()); // contract: events.changed { count }
        });
        // A focused JavaFX Button already fires on Enter and on Space, as a browser's does.
        // javafx.counter.no-enter: a control that ignores Enter (as a custom control built from a
        // Region or a Label does, unless it handles keys itself).
        if (Defects.on("javafx.counter.no-enter"))
            increment.addEventFilter(javafx.scene.input.KeyEvent.KEY_PRESSED, e -> {
                if (e.getCode() == javafx.scene.input.KeyCode.ENTER) e.consume();
            });

        getChildren().addAll(value, increment);
        getStyleClass().add("counter");
        getStylesheets().add(Counter.class.getResource("counter.css").toExternalForm());
    }

    // contract: events.changed, with the new count
    public void setOnChanged(IntConsumer handler) {
        onChanged = handler;
    }
}
