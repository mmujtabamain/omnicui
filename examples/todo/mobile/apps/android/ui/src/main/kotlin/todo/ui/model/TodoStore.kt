package todo.ui.model

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue

/**
 * The app's state: the tasks, the list's filter and the settings. Every change goes through here,
 * and each one that the user made is reported to [onEvent].
 */
class TodoStore(
    tasks: List<Task>,
    theme: ThemeChoice = ThemeChoice.System,
    showCompleted: Boolean = true,
    filter: Filter = Filter.All,
    private val onEvent: (TodoEvent) -> Unit = {},
) {
    private val _tasks = mutableStateListOf<Task>().apply { addAll(tasks) }
    val tasks: List<Task> get() = _tasks

    var filter by mutableStateOf(filter)
        private set
    var theme by mutableStateOf(theme)
        private set
    var showCompleted by mutableStateOf(showCompleted)
        private set

    // The highest task number ever used, so that a deleted task's id is never given again.
    private var highest = tasks.maxOfOrNull { it.id.removePrefix("t").toIntOrNull() ?: 0 } ?: 0

    /** The tasks the list shows: the filter's, and under All only the active ones when completed tasks are hidden. */
    val visible: List<Task>
        get() = when (filter) {
            Filter.All -> if (showCompleted) _tasks else _tasks.filter { !it.done }
            Filter.Active -> _tasks.filter { !it.done }
            Filter.Done -> _tasks.filter { it.done }
        }

    /** How many tasks are not done, whatever the filter. */
    val left: Int get() = _tasks.count { !it.done }

    fun task(id: String): Task? = _tasks.firstOrNull { it.id == id }

    /** Adds an active task at the end, with the title trimmed; a blank title adds nothing. */
    fun add(title: String): Task? {
        val trimmed = title.trim()
        if (trimmed.isEmpty()) return null
        highest += 1
        val task = Task("t$highest", trimmed)
        _tasks += task
        onEvent(TodoEvent.Added(task.id, task.title))
        return task
    }

    fun setDone(id: String, done: Boolean) = update(id, { it.done == done }, { it.copy(done = done) }) { TodoEvent.Toggled(id, done) }

    fun rename(id: String, title: String) = update(id, { it.title == title }, { it.copy(title = title) }) { TodoEvent.Renamed(id, title) }

    fun editNotes(id: String, notes: String) = update(id, { it.notes == notes }, { it.copy(notes = notes) }) { TodoEvent.NotesChanged(id, notes) }

    fun delete(id: String) {
        if (_tasks.removeAll { it.id == id }) onEvent(TodoEvent.Deleted(id))
    }

    fun select(filter: Filter) {
        if (filter == this.filter) return
        this.filter = filter
        onEvent(TodoEvent.FilterChanged(filter))
    }

    fun chooseTheme(theme: ThemeChoice) {
        if (theme == this.theme) return
        this.theme = theme
        onEvent(TodoEvent.ThemeChanged(theme))
    }

    fun showCompleted(show: Boolean) {
        if (show == showCompleted) return
        showCompleted = show
        onEvent(TodoEvent.ShowCompletedChanged(show))
    }

    /** Reports something the store doesn't hold itself (navigation), through the same channel. */
    fun report(event: TodoEvent) = onEvent(event)

    private inline fun update(id: String, unchanged: (Task) -> Boolean, change: (Task) -> Task, event: () -> TodoEvent) {
        val i = _tasks.indexOfFirst { it.id == id }
        if (i < 0 || unchanged(_tasks[i])) return
        _tasks[i] = change(_tasks[i])
        onEvent(event())
    }
}
