package omnicui.javafx;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import java.io.File;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Map;
import java.util.concurrent.Callable;
import java.util.concurrent.CopyOnWriteArrayList;
import java.util.concurrent.TimeUnit;
import javafx.animation.AnimationTimer;
import javafx.embed.swing.SwingFXUtils;
import javafx.geometry.Bounds;
import javafx.geometry.Insets;
import javafx.geometry.Pos;
import javafx.scene.AccessibleAttribute;
import javafx.scene.Node;
import javafx.scene.Parent;
import javafx.scene.Scene;
import javafx.scene.control.Labeled;
import javafx.scene.control.Slider;
import javafx.scene.control.TextInputControl;
import javafx.scene.input.KeyCode;
import javafx.scene.layout.StackPane;
import javafx.stage.Stage;
import javax.imageio.ImageIO;
import org.testfx.api.FxRobot;
import org.testfx.api.FxToolkit;
import org.testfx.util.WaitForAsyncUtils;

// One method per construct of the fixture language, done the JavaFX way.
final class JavaFxTarget {
    // protocol: error codes
    static final int NO_COMPONENT = -32001, NO_PART = -32002, STEP_FAILED = -32003, NO_METHOD = -32601;
    // The largest viewport a contract may ask for; Adapter sizes Monocle's headless screen to it.
    static final int MAX_VIEWPORT = 4096;

    static final class RpcError extends Exception {
        final int code;

        RpcError(int code, String message) {
            super(message);
            this.code = code;
        }
    }

    private final ObjectMapper json = new ObjectMapper();
    private final List<Map<String, Object>> events = new CopyOnWriteArrayList<>();
    private final FxRobot robot = new FxRobot();
    private final Stage stage;
    private Scene scene;

    JavaFxTarget() throws Exception {
        stage = FxToolkit.registerPrimaryStage();
        app.components.Theme.loadFont(); // the app's font, DejaVu Sans
    }

    Object handle(String method, JsonNode p) throws Exception {
        String part = p.path("part").asText(null);
        switch (method) {
            // protocol: the first request of every run; the constructs (capabilities) this target supports
            case "advertise":
                return Map.of("protocol", "1", "can", List.of("mount", "press", "focus", "key", "type", "drag", "hover",
                        "read", "events", "screenshot", "trace", "record", "probe", "bounds", "role", "focused"));
            // protocol: the render check's picture, at the viewport asked for
            case "probe":
                return probe(p.path("viewport").path("width").asDouble(), p.path("viewport").path("height").asDouble(), p.path("path").asText());
            // protocol: where a part is on screen, in scene pixels
            case "bounds": {
                Node n = node(part);
                return onFx(() -> {
                    Bounds b = n.localToScene(n.getBoundsInLocal());
                    boolean shown = n.isVisible() && n.getScene() != null && n.getOpacity() > 0;
                    return Map.of("x", b.getMinX(), "y", b.getMinY(), "width", shown ? b.getWidth() : 0.0, "height", shown ? b.getHeight() : 0.0);
                });
            }
            // contract: tests[].props, viewports, colorSchemes; a fresh component for every test
            case "mount":
                mount(p);
                return null;
            // contract: tests[].steps[].action, on parts.<name>, sent as the part id "ui.counter.<name>"
            case "press": case "focus": case "key": case "type": case "drag": case "hover":
                step(method, p);
                return null;
            // contract: tests[].expect[] with part: (the role's fields: text, name, checked, disabled, value)
            case "read":
                return read(node(part), p.path("field").asText());
            // contract: tests[].expect[] with event:
            case "events": {
                var out = List.copyOf(events);
                events.clear();
                return out;
            }
            // contract: tests[].screenshot: true; the path is chosen by omnicui
            case "screenshot": {
                settle();
                String file = p.path("path").asText();
                return onFx(() -> screenshot(file));
            }
            // contract: tests[].trace[] (part and point); omnicui compares the samples
            case "trace": {
                Node n = node(part);
                String point = p.path("point").asText();
                return sample(p.path("steps"), () -> point(n.localToScene(n.getBoundsInLocal()), point));
            }
            // contract: tests[].record: true; frames of a clip, never compared
            case "record": {
                String dir = p.path("path").asText();
                new File(dir).mkdirs();
                int[] frame = {0};
                var frames = new ArrayList<Map<String, Object>>();
                var samples = sample(p.path("steps"), () -> screenshot(dir + "/" + (frame[0]++) + ".png"));
                for (var s : samples) frames.add(Map.of("ms", s.get("ms"), "file", s.get("value")));
                return frames;
            }
            default:
                throw new RpcError(NO_METHOD, "unknown method " + method);
        }
    }

    // mount: a fresh scene at the viewport's size, the colour scheme's stylesheet, the component at (24, 24)
    private void mount(JsonNode m) throws Exception {
        events.clear();
        String id = m.path("component").asText();
        Component component = Components.ALL.get(id); // contract id → registered component
        if (component == null) throw new RpcError(NO_COMPONENT, "no component " + id); // the test fails
        @SuppressWarnings("unchecked")
        Map<String, Object> props = json.convertValue(m.path("props"), Map.class);
        double w = m.path("viewport").path("width").asDouble(), h = m.path("viewport").path("height").asDouble();
        checkScreen(w, h);
        String scheme = m.path("colorScheme").asText("light");
        // Libraries print to System.out; this line stands for them. Adapter.java sends System.out
        // to standard error, which keeps standard output for replies only.
        System.out.println("[javafx] mounting " + id);
        FxToolkit.setupStage(s -> {
            // the emitter: whatever the component emits is kept for the next events()
            Parent root = component.create(props, (event, fields) -> events.add(Map.of("name", event, "fields", fields)));
            // Components keep their own size, at the top left, as on every target.
            if (!Adapter.defect("javafx.host.stretch") && root instanceof javafx.scene.layout.Region r) r.setMaxSize(javafx.scene.layout.Region.USE_PREF_SIZE, javafx.scene.layout.Region.USE_PREF_SIZE);
            var host = new StackPane(root);
            StackPane.setAlignment(root, Pos.TOP_LEFT);
            host.setPadding(new Insets(24));
            scene = new Scene(host, w, h);
            app.components.Theme.apply(scene, scheme.equals("dark")); // contract: colorSchemes, in the app's theme
            s.setScene(scene);
            s.setWidth(w);
            s.setHeight(h);
            s.show();
            s.toFront();
            s.requestFocus();
            host.requestFocus();
        });
        settle();
    }

    private void step(String method, JsonNode p) throws Exception {
        Node n = node(p.path("part").asText());
        switch (method) {
            case "press" -> robot.clickOn(n);
            case "hover" -> robot.moveTo(n);
            case "focus" -> onFx(() -> { n.requestFocus(); return null; });
            // the key goes wherever focus really is
            case "key" -> robot.push(keys(p.path("key").asText()));
            case "type" -> robot.write(p.path("text").asText());
            case "drag" -> robot.drag(n).moveBy(p.path("dx").asDouble(), p.path("dy").asDouble()).drop();
            default -> throw new RpcError(NO_METHOD, "unknown step " + method);
        }
        settle();
    }

    // read: what the JavaFX controls expose to assistive technology
    private Object read(Node n, String field) throws Exception {
        return onFx(() -> switch (field) {
            case "text" -> n instanceof Labeled l ? l.getText() : n instanceof TextInputControl t ? t.getText() : n.queryAccessibleAttribute(AccessibleAttribute.TEXT);
            case "name" -> accessibleName(n);
            case "checked" -> n.queryAccessibleAttribute(AccessibleAttribute.SELECTED);
            case "disabled" -> n.isDisabled();
            case "value" -> n instanceof Slider s ? s.getValue() : n.queryAccessibleAttribute(AccessibleAttribute.VALUE);
            case "focused" -> n.isFocused();
            case "role" -> ariaRole(n.getAccessibleRole());
            default -> throw new RpcError(STEP_FAILED, "unknown field " + field);
        });
    }

    /** JavaFX's accessible role as the WAI-ARIA role a contract names; other roles by their JavaFX name. */
    private static String ariaRole(javafx.scene.AccessibleRole r) {
        return switch (r) {
            // A toggle button is a button in WAI-ARIA (with aria-pressed); JavaFX has no switch role.
            case BUTTON, TOGGLE_BUTTON, HYPERLINK -> "button";
            case CHECK_BOX -> "checkbox";
            case SLIDER, SPINNER -> "slider";
            case TEXT_FIELD, TEXT_AREA, PASSWORD_FIELD -> "textbox";
            case NODE, PARENT -> "generic";
            default -> r.name().toLowerCase().replace("_", "");
        };
    }

    /** A viewport larger than the (headless) screen would overflow it; say so instead of crashing. */
    private static void checkScreen(double w, double h) throws RpcError {
        var screen = javafx.stage.Screen.getPrimary().getBounds();
        if (w > screen.getWidth() || h > screen.getHeight())
            throw new RpcError(STEP_FAILED, String.format("the viewport %.0f×%.0f is larger than the screen, %.0f×%.0f; set -Dheadless.geometry=<width>x<height>-32 in the adapter command",
                    w, h, screen.getWidth(), screen.getHeight()));
    }

    // probe: white, a black 32×32 square at 8,8, a red one 8 px from the bottom-right corner
    private String probe(double w, double h, String path) throws Exception {
        checkScreen(w, h);
        FxToolkit.setupStage(s -> {
            var black = new javafx.scene.shape.Rectangle(8, 8, 32, 32);
            var red = new javafx.scene.shape.Rectangle(w - 40, h - 40, 32, 32);
            red.setFill(javafx.scene.paint.Color.rgb(255, 0, 0));
            var pane = new javafx.scene.layout.Pane(black, red);
            pane.setStyle("-fx-background-color: white;");
            scene = new Scene(pane, w, h, javafx.scene.paint.Color.WHITE);
            s.setScene(scene);
            s.setWidth(w);
            s.setHeight(h);
            s.show();
        });
        settle();
        String out = onFx(() -> screenshot(path));
        scene = null;
        return out;
    }

    /** The name a screen reader announces: the node's own accessible text, or the Label that labels it. */
    private static Object accessibleName(Node n) {
        Object text = n.queryAccessibleAttribute(AccessibleAttribute.TEXT);
        if (text instanceof String s && !s.isEmpty()) return s;
        Object by = n.queryAccessibleAttribute(AccessibleAttribute.LABELED_BY);
        if (by instanceof Labeled l) return l.getText();
        return text;
    }

    private String screenshot(String path) throws Exception {
        var file = new File(path);
        file.getParentFile().mkdirs();
        ImageIO.write(SwingFXUtils.fromFXImage(scene.snapshot(null), null), "png", file);
        return path;
    }

    // trace and record: run the steps while a per-frame timer samples something, with the time in ms
    private List<Map<String, Object>> sample(JsonNode steps, Callable<Object> what) throws Exception {
        var samples = new CopyOnWriteArrayList<Map<String, Object>>();
        var timer = new AnimationTimer() {
            long start = -1;

            @Override
            public void handle(long now) {
                if (start < 0) start = now;
                try {
                    samples.add(Map.of("ms", (now - start) / 1e6, "value", what.call()));
                } catch (Exception e) {
                    throw new RuntimeException(e);
                }
            }
        };
        onFx(() -> { timer.start(); return null; });
        Thread.sleep(100);
        for (JsonNode s : steps) step(s.path("method").asText(), s);
        Thread.sleep(150);
        onFx(() -> { timer.stop(); return null; });
        if (samples.isEmpty() || !(samples.get(0).get("value") instanceof Map)) return samples;
        var out = new ArrayList<Map<String, Object>>();
        for (var s : samples) {
            @SuppressWarnings("unchecked")
            var pt = (Map<String, Double>) s.get("value");
            out.add(Map.of("ms", s.get("ms"), "x", pt.get("x"), "y", pt.get("y")));
        }
        return out;
    }

    /**
     * Waits until JavaFX has processed every event and nothing on screen has moved, resized or
     * faded for three frames in a row (so transitions have finished), for up to five seconds.
     */
    private void settle() throws Exception {
        WaitForAsyncUtils.waitForFxEvents();
        if (scene == null) return;
        // The component may be waiting for something outside the UI (Component.BUSY).
        long busyDeadline = System.nanoTime() + TimeUnit.SECONDS.toNanos(10);
        while (onFx(this::busy)) {
            if (System.nanoTime() > busyDeadline) throw new RpcError(STEP_FAILED, "the component was still busy after 10 s");
            Thread.sleep(10);
        }
        WaitForAsyncUtils.waitForFxEvents();
        long deadline = System.nanoTime() + TimeUnit.SECONDS.toNanos(5);
        String last = null;
        int still = 0;
        while (System.nanoTime() < deadline && still < 3) {
            Thread.sleep(16);
            String now = onFx(() -> fingerprint(scene.getRoot()));
            still = now.equals(last) ? still + 1 : 0;
            last = now;
        }
        WaitForAsyncUtils.waitForFxEvents();
    }

    private boolean busy() {
        if (scene == null || !(scene.getRoot() instanceof StackPane host) || host.getChildren().isEmpty()) return false;
        return host.getChildren().get(0).getProperties().get(Component.BUSY) instanceof java.util.function.BooleanSupplier b && b.getAsBoolean();
    }

    private static String fingerprint(Node n) {
        var sb = new StringBuilder();
        Bounds b = n.localToScene(n.getBoundsInLocal());
        sb.append(b.getMinX()).append(',').append(b.getMinY()).append(',').append(b.getWidth()).append(',').append(b.getHeight())
                .append(',').append(n.getOpacity()).append(';');
        if (n instanceof Parent p) for (Node c : p.getChildrenUnmodifiable()) sb.append(fingerprint(c));
        return sb.toString();
    }

    // the interface with components: a part is the node whose id is "<contract id>.<part>", set by the component
    private Node node(String id) throws Exception {
        Node n = onFx(() -> lookup(scene.getRoot(), id));
        if (n == null) throw new RpcError(NO_PART, "no part " + id); // an error reply, so the step fails
        return n;
    }

    private static Node lookup(Node n, String id) {
        if (id.equals(n.getId())) return n;
        if (n instanceof Parent p) {
            for (Node c : p.getChildrenUnmodifiable()) {
                Node r = lookup(c, id);
                if (r != null) return r;
            }
        }
        return null;
    }

    private static Map<String, Double> point(Bounds b, String where) {
        return switch (where) {
            case "center" -> Map.of("x", b.getCenterX(), "y", b.getCenterY());
            case "topLeft" -> Map.of("x", b.getMinX(), "y", b.getMinY());
            default -> throw new IllegalArgumentException("unknown point " + where);
        };
    }

    // "Enter" → ENTER, "Space" → SPACE, "Shift+Tab" → SHIFT + TAB
    private static KeyCode[] keys(String combo) throws RpcError {
        var codes = Arrays.stream(combo.split("\\+")).map(KeyCode::getKeyCode).toArray(KeyCode[]::new);
        for (int i = 0; i < codes.length; i++) if (codes[i] == null) throw new RpcError(STEP_FAILED, "unknown key " + combo);
        return codes;
    }

    private static <T> T onFx(Callable<T> c) throws Exception {
        try {
            return WaitForAsyncUtils.asyncFx(c).get(10, TimeUnit.SECONDS);
        } catch (java.util.concurrent.ExecutionException e) {
            if (e.getCause() instanceof Exception x) throw x;
            throw e;
        }
    }

    void close() throws Exception {
        FxToolkit.cleanupStages();
    }
}
