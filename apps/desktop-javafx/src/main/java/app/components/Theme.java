package app.components;

import java.io.InputStream;
import javafx.scene.Scene;
import javafx.scene.text.Font;

// The app's look: DejaVu Sans 16px and its colours, light or dark, the same on every app.
public final class Theme {
    // Loads DejaVu Sans, which the build copies from shared/fonts into the jar.
    public static void loadFont() {
        try (InputStream in = Theme.class.getResourceAsStream("DejaVuSans.ttf")) {
            if (in == null || Font.loadFont(in, 16) == null) throw new IllegalStateException("could not load DejaVuSans.ttf");
        } catch (java.io.IOException e) {
            throw new IllegalStateException(e);
        }
    }

    public static void apply(Scene scene, boolean dark) {
        scene.getStylesheets().setAll(Theme.class.getResource(dark ? "theme-dark.css" : "theme-light.css").toExternalForm());
    }

    private Theme() {}
}
