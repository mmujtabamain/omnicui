package omnicui.javafx;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ObjectNode;
import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.io.PrintStream;
import java.nio.charset.StandardCharsets;

// omnicui's javafx target, the adapter: the conversation with omnicui. omnicui starts it once per check
// (`adapter:` in target.yaml), writes one JSON-RPC 2.0 request per line on standard input, and
// reads exactly one reply per request, one line each, from standard output.
public final class Adapter {
    // The target's own deliberate defects (the harness's, not the app's), with OMNICUI_DEFECTS.
    static boolean defect(String name) {
        return java.util.List.of(System.getenv().getOrDefault("OMNICUI_DEFECTS", "").split(",")).contains(name);
    }

    public static void main(String[] args) throws Exception {
        // Run JavaFX without a screen: Monocle's headless platform, software rendering.
        System.setProperty("testfx.robot", "glass");
        System.setProperty("testfx.headless", "true");
        System.setProperty("glass.platform", "Monocle");
        System.setProperty("monocle.platform", "Headless");
        System.setProperty("prism.order", "sw");
        System.setProperty("prism.text", "t2k");
        System.setProperty("java.awt.headless", "true");
        // Monocle's headless screen is 1280×800 unless told otherwise, and a scene larger than the
        // screen overflows its framebuffer (BufferOverflowException at mount). Make it as large as
        // the largest viewport a contract may ask for (4096×4096), unless it is set already.
        if (System.getProperty("headless.geometry") == null) System.setProperty("headless.geometry", JavaFxTarget.MAX_VIEWPORT + "x" + JavaFxTarget.MAX_VIEWPORT + "-32");

        // protocol: standard output carries replies only. Anything else that prints there
        // (a library, a stray println) would break the conversation, so it all goes to stderr.
        PrintStream replies = new PrintStream(new java.io.FileOutputStream(java.io.FileDescriptor.out), true, StandardCharsets.UTF_8);
        if (!defect("javafx.adapter.stdout")) System.setOut(System.err);

        var json = new ObjectMapper();
        var target = new JavaFxTarget();
        var in = new BufferedReader(new InputStreamReader(System.in, StandardCharsets.UTF_8));
        for (String line; (line = in.readLine()) != null; ) {
            if (line.isBlank()) continue;
            ObjectNode reply = json.createObjectNode().put("jsonrpc", "2.0");
            JsonNode request;
            try {
                request = json.readTree(line);
            } catch (Exception e) {
                reply.putNull("id");
                reply.putObject("error").put("code", -32700).put("message", "parse error");
                replies.println(json.writeValueAsString(reply));
                continue;
            }
            reply.set("id", request.get("id"));
            try {
                reply.set("result", json.valueToTree(target.handle(request.path("method").asText(), request.path("params"))));
            } catch (JavaFxTarget.RpcError e) {
                reply.putObject("error").put("code", e.code).put("message", e.getMessage());
            } catch (Exception e) {
                System.err.println("[javafx] " + request.path("method").asText() + " failed: " + e);
                reply.putObject("error").put("code", JavaFxTarget.STEP_FAILED).put("message", String.valueOf(e.getMessage()));
            }
            // protocol: one reply per request, on standard output
            replies.println(json.writeValueAsString(reply));
        }
        target.close();
        System.exit(0);
    }
}
