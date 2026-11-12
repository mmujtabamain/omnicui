package omnicui.android

import java.io.FileDescriptor
import java.io.FileInputStream
import java.io.FileOutputStream
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

/**
 * The Robolectric sandbox the adapter runs in, for the whole conversation: Android 16 (API 36),
 * real (native) graphics, and an mdpi screen so that 1 dp is 1 px. Every mount sets the screen's
 * size and night mode from the viewport and the colour scheme ([AndroidTarget.mount]).
 *
 * Run by [Main]. Under Gradle (`./gradlew testDebugUnitTest`) standard input is empty, so it
 * returns at once.
 */
@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = "w360dp-h640dp-port-notnight-mdpi")
class Serve {
    @Test
    fun serve() {
        // FileDescriptor.out, not System.out: Main points System.out at standard error.
        AndroidTarget(Rpc(FileInputStream(FileDescriptor.`in`), FileOutputStream(FileDescriptor.out))).run()
    }
}
