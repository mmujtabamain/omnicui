package app.components;

import java.util.function.Consumer;
import javafx.animation.Interpolator;
import javafx.animation.TranslateTransition;
import javafx.scene.AccessibleRole;
import javafx.scene.control.Label;
import javafx.scene.control.ToggleButton;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Pane;
import javafx.scene.layout.Region;
import javafx.util.Duration;

// contract: omnicui/contracts/switch.contract.yaml (id: ui.switch). A ToggleButton drawn as a track with a
// sliding thumb; the visible Label names it (labelFor), as a screen reader would read it.
public final class SwitchControl extends HBox {
    public static final String SWITCH = "ui.switch.switch", THUMB = "ui.switch.thumb";

    private Consumer<Boolean> onToggled = on -> {};

    public SwitchControl() {
        this(false, "Dark mode"); // contract: props.on, default false; props.label, default "Dark mode"
    }

    public SwitchControl(boolean on, String text) {

        var label = new Label(text);
        var thumb = new Region();
        thumb.setId(THUMB);
        thumb.getStyleClass().add("thumb");
        thumb.setLayoutX(3);
        thumb.setLayoutY(3);
        thumb.resize(18, 18);
        var track = new Pane(thumb);
        track.getStyleClass().add("track");

        var toggle = new ToggleButton();
        toggle.setId(SWITCH);
        toggle.setAccessibleRole(AccessibleRole.TOGGLE_BUTTON);
        toggle.setGraphic(track);
        toggle.getStyleClass().add("switch-toggle");
        toggle.setSelected(on);
        // The visible label names the toggle for assistive technology. Forgetting this is an easy
        // mistake: the switch then has no name, and the contract's first test catches it.
        if (!Defects.on("javafx.switch.no-label")) label.setLabelFor(toggle);
        thumb.setTranslateX(on ? 20 : 0);

        boolean linear = Defects.on("javafx.switch.linear");
        var slide = new TranslateTransition(Duration.millis(linear ? 300 : 200), thumb);
        // CSS ease-out is cubic-bezier(0, 0, 0.58, 1)
        slide.setInterpolator(linear ? Interpolator.LINEAR : Interpolator.SPLINE(0, 0, 0.58, 1));
        toggle.selectedProperty().addListener((obs, was, now) -> {
            slide.stop();
            slide.setFromX(thumb.getTranslateX());
            slide.setToX(now ? 20 : 0);
            slide.playFromStart();
            // javafx.switch.dim: the thumb also fades to half opacity when on, slowly (600 ms,
            // linear), which only this app does.
            if (Defects.on("javafx.switch.dim")) {
                var fade = new javafx.animation.FadeTransition(Duration.millis(600), thumb);
                fade.setInterpolator(Interpolator.LINEAR);
                fade.setToValue(now ? 0.5 : 1);
                fade.playFromStart();
            }
            onToggled.accept(now); // contract: events.toggled { on }
        });

        getChildren().addAll(label, toggle);
        getStyleClass().add("switch");
        getStylesheets().add(SwitchControl.class.getResource("switch.css").toExternalForm());
    }

    // contract: events.toggled, with whether it is now on
    public void setOnToggled(Consumer<Boolean> handler) {
        onToggled = handler;
    }
}
