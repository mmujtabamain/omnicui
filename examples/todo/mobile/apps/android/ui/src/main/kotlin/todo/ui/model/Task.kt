package todo.ui.model

/** One task. Ids are "t" and a number: the seeds use t1, t2, …; new tasks continue from the highest. */
data class Task(val id: String, val title: String, val notes: String = "", val done: Boolean = false)

/** Which tasks the list shows. */
enum class Filter(val key: String) { All("all"), Active("active"), Done("done") }

/** The app's theme setting. System follows the device's dark theme. */
enum class ThemeChoice(val key: String) { System("system"), Light("light"), Dark("dark") }
