package omnicui.android

import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.Rect
import android.view.Gravity
import android.view.View
import android.view.ViewGroup
import android.view.WindowManager
import androidx.compose.ui.platform.ViewRootForTest
import androidx.compose.ui.semantics.SemanticsNode
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.AndroidComposeUiTest
import kotlin.math.roundToInt

/**
 * The app's windows, as the screen shows them: the activity's, and a dialog's above it. Robolectric
 * has no window manager or compositor, so this does their part: it places each window by its
 * gravity, dims what is behind a window that asks for it (FLAG_DIM_BEHIND, dimAmount), and draws
 * the windows bottom first.
 *
 * Each window is drawn by its own views (View.draw) on a Canvas backed by Robolectric's native
 * graphics, which is real Skia. Compose's captureToImage() was the first choice, but under
 * Robolectric it timed out on every call (it waits for a frame that the paused main looper never
 * draws), and it sees one window only. Software drawing leaves out elevation shadows, which only
 * a hardware renderer draws (the FAB's shadow).
 */
private var dimLogged = false

@OptIn(ExperimentalTestApi::class)
class Windows(private val ui: AndroidComposeUiTest<*>, private val width: Int, private val height: Int) {
    class Window(val decor: View, val params: WindowManager.LayoutParams, val x: Int, val y: Int, val compose: ViewRootForTest?)

    /** Every window, bottom first. */
    fun all(): List<Window> {
        val global = Class.forName("android.view.WindowManagerGlobal").getMethod("getInstance").invoke(null)
        fun <T> field(name: String): List<T> {
            val f = global.javaClass.getDeclaredField(name)
            f.isAccessible = true
            @Suppress("UNCHECKED_CAST")
            return ArrayList(f.get(global) as List<T>)
        }
        val views = field<View>("mViews")
        val params = field<WindowManager.LayoutParams>("mParams")
        return views.indices.filter { views[it].isAttachedToWindow && views[it].visibility == View.VISIBLE }.map { i ->
            val v = views[i]
            val p = params[i]
            val frame = Rect()
            if (p.type == WindowManager.LayoutParams.TYPE_BASE_APPLICATION) {
                frame.set(0, 0, v.width, v.height)
            } else {
                Gravity.apply(p.gravity, v.width, v.height, Rect(0, 0, width, height), p.x, p.y, frame)
            }
            Window(v, p, frame.left, frame.top, findCompose(v))
        }
    }

    /** Where a semantics node's window is on the screen. */
    fun offsetOf(node: SemanticsNode): Pair<Int, Int> {
        var top = node
        while (true) top = top.parent ?: break
        val root = top.id
        val w = all().firstOrNull { it.compose?.semanticsOwner?.unmergedRootSemanticsNode?.id == root } ?: return 0 to 0
        val loc = IntArray(2)
        w.compose!!.view.getLocationInWindow(loc)
        return (w.x + loc[0]) to (w.y + loc[1])
    }

    /** The screen as it is now: each window drawn, placed and dimmed. */
    fun capture(): Bitmap {
        val out = Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888)
        val canvas = Canvas(out)
        canvas.drawColor(Color.BLACK)
        for (w in all()) {
            if (w.params.flags and WindowManager.LayoutParams.FLAG_DIM_BEHIND != 0) {
                canvas.drawColor(Color.argb((w.params.dimAmount * 255).roundToInt(), 0, 0, 0))
                if (!dimLogged) log("a window dims what is behind it by ${w.params.dimAmount}").also { dimLogged = true }
            }
            canvas.save()
            canvas.translate(w.x.toFloat(), w.y.toFloat())
            w.decor.draw(canvas)
            canvas.restore()
        }
        return out
    }

    private fun findCompose(v: View): ViewRootForTest? {
        if (v is ViewRootForTest) return v
        if (v is ViewGroup) for (i in 0 until v.childCount) findCompose(v.getChildAt(i))?.let { return it }
        return null
    }
}
