package todo.ui.model

/**
 * What the user did, as the app reports it to whoever hosts its screens (analytics, a test). The
 * screens themselves only take callbacks; [TodoStore] and [todo.ui.app.TodoApp] report these.
 */
sealed interface TodoEvent {
    data class Added(val id: String, val title: String) : TodoEvent
    data class Toggled(val id: String, val done: Boolean) : TodoEvent
    data class Opened(val id: String) : TodoEvent
    data class FilterChanged(val filter: Filter) : TodoEvent
    data class Renamed(val id: String, val title: String) : TodoEvent
    data class NotesChanged(val id: String, val notes: String) : TodoEvent
    data class Deleted(val id: String) : TodoEvent
    data class Closed(val id: String) : TodoEvent
    data class ThemeChanged(val theme: ThemeChoice) : TodoEvent
    data class ShowCompletedChanged(val show: Boolean) : TodoEvent
    data object SettingsClosed : TodoEvent
    /** The screen shown changed: "list", "detail" (with the task's id) or "settings". */
    data class Navigated(val screen: String, val id: String) : TodoEvent
}
