package omnicui.android

import android.graphics.Bitmap
import android.view.KeyEvent
import android.view.WindowManager
import androidx.activity.ComponentActivity
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.InputMode
import androidx.compose.ui.input.InputModeManager
import androidx.compose.ui.platform.LocalInputModeManager
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.SemanticsActions
import androidx.compose.ui.semantics.SemanticsNode
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.state.ToggleableState
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.SemanticsMatcher
import androidx.compose.ui.test.SemanticsNodeInteraction
import androidx.compose.ui.test.AndroidComposeUiTest
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.isFocused
import androidx.compose.ui.test.isRoot
import androidx.compose.ui.test.onFirst
import androidx.compose.ui.test.onLast
import androidx.compose.ui.test.performKeyInput
import androidx.compose.ui.test.performMouseInput
import androidx.compose.ui.test.performSemanticsAction
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.test.performTouchInput
import androidx.compose.ui.test.pressKey
import androidx.compose.ui.test.swipe
import androidx.compose.ui.test.runAndroidComposeUiTest
import androidx.compose.ui.test.withKeysDown
import java.io.File
import org.json.JSONArray
import org.json.JSONObject
import org.robolectric.RuntimeEnvironment
import org.robolectric.Shadows.shadowOf
import android.os.Looper
import java.time.Duration

/**
 * One method per construct of omnicui's fixture language, done the Compose way: parts are found
 * by testTag in the semantics tree, actions are injected as touch and key input, and every reply
 * waits until Compose is idle on its test clock (animations included).
 */
@OptIn(ExperimentalTestApi::class, androidx.compose.ui.ExperimentalComposeUiApi::class)
class AndroidTarget(private val rpc: Rpc) {
    private val events = mutableListOf<Map<String, Any>>()
    private var inputMode: InputModeManager? = null

    fun run() {
        var req = rpc.next()
        while (req != null) {
            val r: Request = req
            req = when (r.method) {
                "mount" -> mount(r)
                "probe" -> {
                    answer(r) { probe(r.params) }
                    rpc.next()
                }
                else -> {
                    answer(r) {
                        when (r.method) {
                            "advertise" -> advertise()
                            "events" -> emptyList<Any>()
                            else -> throw RpcError(if (r.method in METHODS) STEP_FAILED else NO_METHOD,
                                if (r.method in METHODS) "${r.method}: nothing is mounted" else "unknown method ${r.method}")
                        }
                    }
                    rpc.next()
                }
            }
        }
    }

    private fun answer(r: Request, body: () -> Any?) {
        try {
            rpc.reply(r.id, body())
        } catch (e: RpcError) {
            rpc.error(r.id, e.code, e.message ?: "")
        } catch (e: Throwable) {
            log("${r.method} failed: ${e.stackTraceToString().lines().take(12).joinToString("\n")}")
            rpc.error(r.id, STEP_FAILED, (e.message ?: e.toString()))
        }
    }

    // protocol: the first request; what this target does, and why it doesn't do the rest.
    private fun advertise() = mapOf(
        "protocol" to "1",
        "can" to listOf(
            "mount", "press", "focus", "key", "type", "drag", "hover", "read", "events", "screenshot", "trace", "trace.scale",
            "record", "probe", "bounds", "role", "focused", "selected", "do.back",
        ),
        "cannot" to mapOf(
            "trace.opacity" to "Compose semantics don't expose the opacity a node is drawn with (a graphicsLayer's alpha can't be read from outside the composition)",
            "do.scroll" to "not implemented in this target yet",
            "do.swipe" to "not implemented in this target yet",
            "do.refresh" to "the app has no pull to refresh",
            "do.pick" to "the app has no picker",
        ),
        "platform" to mapOf("os" to "android"),
    )

    private fun qualifiers(width: Int, height: Int, dark: Boolean) =
        "w${width}dp-h${height}dp-${if (width > height) "land" else "port"}-${if (dark) "night" else "notnight"}-mdpi"

    // contract: tests[].props, viewports, colorSchemes. A fresh activity for every mount, with the
    // screen at the viewport's size (mdpi: 1 dp = 1 px) and night mode for the dark scheme.
    private fun mount(req: Request): Request? {
        val p = req.params
        val component = p.optString("component")
        val create = Components.find(component)
        if (create == null) {
            rpc.error(req.id, NO_COMPONENT, "no component $component")
            return rpc.next()
        }
        val width = p.getJSONObject("viewport").getInt("width")
        val height = p.getJSONObject("viewport").getInt("height")
        val props = (p.optJSONObject("props") ?: JSONObject()).toMap()
        RuntimeEnvironment.setQualifiers(qualifiers(width, height, p.optString("colorScheme") == "dark"))
        events.clear()
        var replied = false
        var next: Request? = null
        var ended = false
        try {
            runAndroidComposeUiTest<ComponentActivity> {
                val content = create(props, activity!!) { name, fields -> events += mapOf("name" to name, "fields" to fields) }
                setContent {
                    inputMode = LocalInputModeManager.current
                    content()
                }
                waitForIdle()
                rpc.reply(req.id, null)
                replied = true
                val session = Session(this, width, height)
                while (true) {
                    val r = rpc.next()
                    if (r == null || r.method == "mount" || r.method == "probe") {
                        next = r
                        ended = true
                        break
                    }
                    answer(r) { session.handle(r) }
                }
            }
        } catch (e: Throwable) {
            log("mount of $component: $e")
            if (!replied) {
                rpc.error(req.id, STEP_FAILED, "mount failed: ${e.message ?: e}")
                return rpc.next()
            }
        }
        return if (ended) next else rpc.next()
    }

    // protocol: the render check's picture: white, a black 32×32 square at 8,8, a red one 8 px
    // from the bottom-right corner, drawn by Compose at the size asked for.
    private fun probe(p: JSONObject): String {
        val width = p.getJSONObject("viewport").getInt("width")
        val height = p.getJSONObject("viewport").getInt("height")
        val path = p.getString("path")
        RuntimeEnvironment.setQualifiers(qualifiers(width, height, false))
        runAndroidComposeUiTest<ComponentActivity> {
            setContent {
                Canvas(Modifier.fillMaxSize()) {
                    drawRect(Color.White)
                    drawRect(Color.Black, topLeft = Offset(8f, 8f), size = Size(32f, 32f))
                    drawRect(Color.Red, topLeft = Offset(size.width - 40f, size.height - 40f), size = Size(32f, 32f))
                }
            }
            waitForIdle()
            save(Windows(this, width, height).capture(), path)
        }
        return path
    }

    /** What a mounted component answers. */
    private inner class Session(private val ui: AndroidComposeUiTest<ComponentActivity>, private val width: Int, private val height: Int) {
        private val windows = Windows(ui, width, height)

        fun handle(r: Request): Any? {
            val p = r.params
            return when (r.method) {
                "advertise" -> advertise()
                "press", "focus", "key", "type", "drag", "hover" -> step(r.method, p, settle = true).let { null }
                "do" -> act(p).let { null }
                "read" -> read(p.getString("part"), p.getString("field"))
                "events" -> ArrayList(events).also { events.clear() }
                "bounds" -> bounds(p.getString("part"))
                "screenshot" -> {
                    ui.waitForIdle()
                    save(windows.capture(), p.getString("path"))
                    p.getString("path")
                }
                "trace" -> trace(p)
                "record" -> record(p)
                else -> throw RpcError(NO_METHOD, "unknown method ${r.method}")
            }
        }

        // --- parts ------------------------------------------------------------------------------

        private fun matcher(part: String) = SemanticsMatcher("part $part") { n ->
            val tag = n.config.getOrNull(SemanticsProperties.TestTag)
            tag != null && Components.tagMatches(part, tag)
        }

        /** The part's node in the merged tree (what assistive technology sees), shown ones first. */
        private fun find(part: String): Pair<SemanticsNodeInteraction, SemanticsNode>? {
            for (unmerged in listOf(false, true)) {
                val all = ui.onAllNodes(matcher(part), useUnmergedTree = unmerged)
                val nodes = all.fetchSemanticsNodes(atLeastOneRootRequired = false)
                if (nodes.isEmpty()) continue
                val i = nodes.indexOfFirst { it.layoutInfo.isPlaced && it.boundsInRoot.width > 0 && it.boundsInRoot.height > 0 }.coerceAtLeast(0)
                return all[i] to nodes[i]
            }
            return null
        }

        private fun node(part: String) = find(part) ?: throw RpcError(NO_PART, "no part $part")

        // --- steps ------------------------------------------------------------------------------

        fun step(method: String, p: JSONObject, settle: Boolean) {
            // A key with no part goes to whatever has focus (or the top window), as a keyboard's does.
            if (method == "key" && !p.has("part")) {
                val focused = ui.onAllNodes(isFocused())
                val n = if (focused.fetchSemanticsNodes(atLeastOneRootRequired = false).isNotEmpty()) focused.onFirst() else ui.onAllNodes(isRoot()).onLast()
                key(n, p.getString("key"))
                if (settle) ui.waitForIdle()
                return
            }
            val (n, node) = node(p.getString("part"))
            when (method) {
                "press" -> n.performClick()
                "hover" -> n.performMouseInput { moveTo(center) }
                "focus" -> focus(n, node)
                // The key goes wherever focus is, as a hardware keyboard's does.
                "key" -> key(n, p.getString("key"))
                "type" -> {
                    if (node.config.getOrNull(SemanticsProperties.Focused) != true) focus(n, node)
                    n.performTextInput(p.getString("text"))
                }
                "drag" -> n.performTouchInput {
                    val from = center
                    swipe(from, from + Offset(p.getDouble("dx").toFloat(), p.getDouble("dy").toFloat()), durationMillis = 200)
                }
                else -> throw RpcError(NO_METHOD, "unknown step $method")
            }
            if (settle) ui.waitForIdle()
        }

        /**
         * Keyboard focus, as a hardware keyboard user gets it. Compose doesn't let a clickable take
         * focus in touch mode. On a device the keyboard's first key leaves touch mode; a key
         * injected under Robolectric doesn't (measured), so this asks Compose's InputModeManager
         * for keyboard mode, which is the state that first key leaves it in, and then for focus.
         */
        private fun focus(n: SemanticsNodeInteraction, node: SemanticsNode) {
            val tag = node.config.getOrNull(SemanticsProperties.TestTag)
            if (!node.config.contains(SemanticsActions.RequestFocus)) throw RpcError(STEP_FAILED, "$tag can't take focus (no RequestFocus action)")
            if (inputMode?.inputMode != InputMode.Keyboard) ui.runOnUiThread { inputMode?.requestInputMode(InputMode.Keyboard) }
            n.performSemanticsAction(SemanticsActions.RequestFocus)
            ui.waitForIdle()
            if (n.fetchSemanticsNode().config.getOrNull(SemanticsProperties.Focused) != true) throw RpcError(STEP_FAILED, "$tag didn't take focus")
        }

        private fun key(n: SemanticsNodeInteraction, combo: String) {
            val names = combo.split("+")
            val keys = names.map { KEYS[it] ?: KEYS[it.uppercase()] ?: throw RpcError(STEP_FAILED, "unknown key $it in $combo") }
            n.performKeyInput { withKeysDown(keys.dropLast(1)) { pressKey(keys.last()) } }
        }

        // protocol: do. back is the system back: a dialog on top gets the back key, as on a
        // phone; otherwise the activity's OnBackPressedDispatcher (Navigation Compose pops).
        private fun act(p: JSONObject) {
            when (val name = p.optString("name")) {
                "back" -> {
                    val top = windows.all().lastOrNull()
                    ui.runOnUiThread {
                        if (top != null && top.params.type != WindowManager.LayoutParams.TYPE_BASE_APPLICATION) {
                            top.decor.dispatchKeyEvent(KeyEvent(KeyEvent.ACTION_DOWN, KeyEvent.KEYCODE_BACK))
                            top.decor.dispatchKeyEvent(KeyEvent(KeyEvent.ACTION_UP, KeyEvent.KEYCODE_BACK))
                        } else {
                            ui.activity!!.onBackPressedDispatcher.onBackPressed()
                        }
                    }
                    ui.waitForIdle()
                }
                else -> throw RpcError(NO_METHOD, "do.$name is not something this target does")
            }
        }

        // --- reading ----------------------------------------------------------------------------

        fun read(part: String, field: String): Any? {
            val c = node(part).second.config
            val texts = c.getOrNull(SemanticsProperties.Text)?.joinToString(" ") { it.text }
            val editable = c.getOrNull(SemanticsProperties.EditableText)?.text
            return when (field) {
                // What the part shows: a text box's text, else the text its node and children draw.
                "text" -> editable ?: texts ?: ""
                // What a screen reader announces: the content description, else (a text box's
                // label, or) the text.
                "name" -> c.getOrNull(SemanticsProperties.ContentDescription)?.joinToString(" ") ?: texts
                "checked" -> c.getOrNull(SemanticsProperties.ToggleableState)?.let { it == ToggleableState.On }
                "disabled" -> c.contains(SemanticsProperties.Disabled)
                "selected" -> c.getOrNull(SemanticsProperties.Selected) ?: false
                "focused" -> c.getOrNull(SemanticsProperties.Focused) ?: false
                "value" -> c.getOrNull(SemanticsProperties.ProgressBarRangeInfo)?.current?.toDouble()
                "role" -> role(c)
                else -> throw RpcError(STEP_FAILED, "unknown field $field")
            }
        }

        /** Compose's role as the WAI-ARIA role a contract names. */
        private fun role(c: androidx.compose.ui.semantics.SemanticsConfiguration): String = when (c.getOrNull(SemanticsProperties.Role)) {
            Role.Button -> "button"
            Role.Checkbox -> "checkbox"
            Role.Switch -> "switch"
            Role.RadioButton -> "radio"
            Role.Tab -> "tab"
            Role.Image -> "img"
            Role.DropdownList -> "combobox"
            else -> when {
                c.contains(SemanticsProperties.EditableText) -> "textbox"
                c.contains(SemanticsProperties.Heading) -> "heading"
                c.contains(SemanticsProperties.LiveRegion) -> "status"
                c.contains(SemanticsProperties.ProgressBarRangeInfo) -> "slider"
                c.contains(SemanticsActions.OnClick) -> "button"
                else -> "generic"
            }
        }

        // protocol: where a part is on the screen; 0×0 when it isn't placed.
        fun bounds(part: String): Map<String, Double> {
            val node = node(part).second
            if (!node.layoutInfo.isPlaced) return mapOf("x" to 0.0, "y" to 0.0, "width" to 0.0, "height" to 0.0)
            val b = node.boundsInRoot
            val (dx, dy) = windows.offsetOf(node)
            return mapOf("x" to (b.left + dx).toDouble(), "y" to (b.top + dy).toDouble(), "width" to b.width.toDouble(), "height" to b.height.toDouble())
        }

        // --- motion -----------------------------------------------------------------------------

        /**
         * Runs the steps with Compose's test clock under our control, one 16 ms frame at a time,
         * and calls [sample] at every frame (with the ms since the first). It stops once nothing
         * has changed for 20 frames after the last step (at least 300 ms after it), or after 5 s.
         */
        private fun frames(steps: JSONArray, inputs: MutableList<Double>?, sample: (ms: Double) -> Any?): Int {
            val clock = ui.mainClock
            clock.autoAdvance = false
            try {
                val start = clock.currentTime
                var last: Any? = null
                var still = 0
                fun frame(): Boolean {
                    val v = sample((clock.currentTime - start).toDouble())
                    still = if (v == last) still + 1 else 0
                    last = v
                    clock.advanceTimeByFrame()
                    // Compose's test clock drives the composition; the views' own frame (a new
                    // dialog window's layout, its first draw) runs on the main looper's clock.
                    shadowOf(Looper.getMainLooper()).idleFor(Duration.ofMillis(16))
                    return true
                }
                repeat(6) { frame() }
                for (i in 0 until steps.length()) {
                    val s = steps.getJSONObject(i)
                    inputs?.add((clock.currentTime - start).toDouble())
                    step(s.getString("method"), s, settle = false)
                }
                val after = clock.currentTime
                still = 0
                while (clock.currentTime - after < 5000) {
                    frame()
                    if (still >= 20 && clock.currentTime - after >= 300) break
                }
                return 0
            } finally {
                clock.autoAdvance = true
                ui.waitForIdle()
            }
        }

        // contract: trace[]. A point of the part's box (boundsInRoot, so a graphicsLayer's scale
        // counts), or with property: scale its width as drawn over its width as laid out. The part
        // is looked up at every frame, so a dialog that appears during the steps is traced from
        // its first frame.
        fun trace(p: JSONObject): List<Map<String, Double>> {
            val part = p.getString("part")
            val property = p.optString("property", "position")
            if (property == "opacity") throw RpcError(STEP_FAILED, "this target can't read opacity")
            val point = p.optString("point", "center")
            val samples = mutableListOf<Map<String, Double>>()
            frames(p.getJSONArray("steps"), null) { ms ->
                val node = find(part)?.second?.takeIf { it.layoutInfo.isPlaced }
                if (node == null) null else {
                    val b = node.boundsInRoot
                    val (dx, dy) = windows.offsetOf(node)
                    val s = when (property) {
                        "scale" -> if (node.size.width > 0) mapOf("ms" to ms, "x" to 0.0, "y" to 0.0, "value" to (b.width / node.size.width).toDouble()) else null
                        else -> if (point == "center") mapOf("ms" to ms, "x" to (b.center.x + dx).toDouble(), "y" to (b.center.y + dy).toDouble())
                        else mapOf("ms" to ms, "x" to (b.left + dx).toDouble(), "y" to (b.top + dy).toDouble())
                    }
                    s?.also { samples += it }
                    s?.minus("ms")
                }
            }
            if (samples.isEmpty()) throw RpcError(NO_PART, "no part $part (not on screen at any frame)")
            val t0 = samples.first()["ms"]!!
            return samples.map { it + ("ms" to it["ms"]!! - t0) }
        }

        // contract: record: true. Every 16 ms frame of the test clock, as the screen shows it.
        // held: false, since a frame is written for every pulse, changed or not.
        fun record(p: JSONObject): Map<String, Any> {
            val dir = File(p.getString("path")).also { it.mkdirs() }
            val frames = mutableListOf<Map<String, Any>>()
            val inputs = mutableListOf<Double>()
            frames(p.getJSONArray("steps"), inputs) { ms ->
                val image = windows.capture()
                val file = File(dir, "${frames.size}.png")
                save(image, file.path)
                frames += mapOf("ms" to ms, "file" to file.path)
                image.sameAs(previous).also { previous = image }.let { if (it) "same" else ms }
            }
            previous = null
            return mapOf("frames" to frames, "inputs" to inputs, "held" to false)
        }

        private var previous: Bitmap? = null
    }

    companion object {
        private val METHODS = setOf("press", "focus", "key", "type", "drag", "hover", "do", "read", "bounds", "screenshot", "trace", "record")

        // protocol: key names, as Playwright names them, to Compose's keys
        private val KEYS: Map<String, Key> = buildMap {
            put("Enter", Key.Enter); put("Space", Key.Spacebar); put("Tab", Key.Tab); put("Escape", Key.Escape)
            put("Backspace", Key.Backspace); put("Delete", Key.Delete); put("Home", Key.MoveHome); put("End", Key.MoveEnd)
            put("ArrowUp", Key.DirectionUp); put("ArrowDown", Key.DirectionDown); put("ArrowLeft", Key.DirectionLeft); put("ArrowRight", Key.DirectionRight)
            put("Shift", Key.ShiftLeft); put("Control", Key.CtrlLeft); put("Alt", Key.AltLeft); put("Meta", Key.MetaLeft)
            ('A'..'Z').forEach { put(it.toString(), Key(android.view.KeyEvent.KEYCODE_A + (it - 'A'))) }
            ('0'..'9').forEach { put(it.toString(), Key(android.view.KeyEvent.KEYCODE_0 + (it - '0'))) }
        }

        fun save(image: Bitmap, path: String) {
            val file = File(path)
            file.parentFile?.mkdirs()
            file.outputStream().use { image.compress(Bitmap.CompressFormat.PNG, 100, it) }
        }
    }
}
