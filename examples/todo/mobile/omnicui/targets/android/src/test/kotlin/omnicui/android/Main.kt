package omnicui.android

import kotlin.system.exitProcess
import org.junit.runner.JUnitCore

/**
 * The adapter's entry point: `java @build/adapter.args omnicui.android.Main` (target.yaml).
 * omnicui writes one JSON-RPC request per line on standard input and reads one reply per line
 * from standard output. The conversation itself runs inside a Robolectric sandbox, which a JUnit
 * runner sets up, so this runs the one "test" that serves it ([Serve]) and exits with its result.
 */
object Main {
    @JvmStatic
    fun main(args: Array<String>) {
        // Standard output carries replies only. Robolectric, JUnit and the app may print; all of
        // that goes to standard error. Serve opens the real standard output for the replies.
        System.setOut(System.err)
        val started = System.nanoTime()
        val result = JUnitCore().run(Serve::class.java)
        for (f in result.failures) System.err.println("[android] the adapter stopped: ${f.trace}")
        System.err.println("[android] served for %.1f s".format((System.nanoTime() - started) / 1e9))
        exitProcess(if (result.wasSuccessful()) 0 else 1)
    }
}
