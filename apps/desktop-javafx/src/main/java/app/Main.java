package app;


import javafx.application.Application;
import javafx.geometry.Insets;
import javafx.scene.Scene;
import javafx.scene.control.CheckBox;
import javafx.scene.control.Label;
import javafx.scene.control.ListView;
import javafx.scene.control.Separator;
import javafx.scene.layout.Region;
import javafx.scene.layout.VBox;
import javafx.stage.Stage;
import app.components.Counter;
import app.components.SwitchControl;
import app.components.Theme;

// The JavaFX desktop app: the counter and the switch, a dark colour scheme toggle, and the events
// they report. `java -jar target/desktop-javafx.jar` (or `run.sh app desktop-javafx`).
// OMNICUI_DEFECTS=… switches on the components' deliberate defects.
public final class Main {
    public static void main(String[] args) {
        // A main class that is not an Application, so that JavaFX on the class path starts.
        Application.launch(Window.class, args);
    }

    public static final class Window extends Application {
        @Override
        public void start(Stage stage) throws Exception {
            Theme.loadFont();
            var events = new ListView<String>();
            events.setPrefHeight(150);

            var counter = new Counter();
            counter.setOnChanged(n -> events.getItems().add(0, "changed: count " + n));
            var toggle = new SwitchControl();
            toggle.setOnToggled(on -> events.getItems().add(0, "toggled: on " + on));
            for (Region c : new Region[] {counter, toggle}) c.setMaxSize(Region.USE_PREF_SIZE, Region.USE_PREF_SIZE);

            var dark = new CheckBox("Dark colour scheme");
            dark.setStyle("-fx-text-fill: -omni-fg;");
            events.setStyle("-fx-control-inner-background: -omni-bg; -fx-control-inner-background-alt: -omni-bg;");
            var root = new VBox(8, caption("Counter"), counter, caption("Switch"), toggle, new Separator(),
                dark, new Label("Events (newest first):"), events);
            VBox.setMargin(toggle, new Insets(0, 0, 16, 0));
            root.setPadding(new Insets(24));
            var scene = new Scene(root, 420, 420);
            Theme.apply(scene, false);
            dark.selectedProperty().addListener((o, was, now) -> Theme.apply(scene, now));

            stage.setTitle("Desktop app (JavaFX)");
            stage.setScene(scene);
            stage.show();
        }

        private static Label caption(String text) {
            var l = new Label(text);
            l.setStyle("-fx-font-size: 12px; -fx-opacity: 0.6;");
            return l;
        }
    }
}
