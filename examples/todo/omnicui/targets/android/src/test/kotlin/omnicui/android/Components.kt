package omnicui.android

import android.content.Context
import androidx.compose.runtime.Composable
import todo.ui.app.TodoApp
import todo.ui.detail.TaskDetailRoute
import todo.ui.list.TaskListRoute
import todo.ui.model.Seeds
import todo.ui.model.ThemeChoice
import todo.ui.model.TodoEvent
import todo.ui.model.TodoStore
import todo.ui.settings.SettingsRoute
import todo.ui.theme.TodoTheme

typealias Emit = (name: String, fields: Map<String, Any>) -> Unit

/**
 * The registry: for each contract id, how to build the app's screen from a test's props, and how
 * the app's events become the contract's. Props a test leaves out take the contract's defaults.
 */
object Components {
    private val all: Map<String, (Map<String, Any?>, Context, Emit) -> @Composable () -> Unit> = mapOf(
        "todo.list" to { props, context, emit ->
            val store = store(props, context, emit)
            ({ TodoTheme(store.theme) { TaskListRoute(store) } })
        },
        "todo.detail" to { props, context, emit ->
            val store = store(props, context, emit)
            val id = props["taskId"] as? String ?: "t1"
            ({ TodoTheme(store.theme) { TaskDetailRoute(store, id) } })
        },
        "todo.settings" to { props, context, emit ->
            val store = store(props, context, emit)
            ({ TodoTheme(store.theme) { SettingsRoute(store) } })
        },
        "todo.app" to { props, context, emit ->
            val store = store(props, context, emit)
            ({ TodoApp(store) })
        },
    )

    fun find(component: String) = all[component]

    /**
     * The tags a part may carry. A screen's part is tagged "<component>.<part>". `todo.app` shows
     * the screens, so its parts are its own ("todo.app.openSettings") or the shown screen's part
     * of that name ("todo.list.row1"); part names are unique across the screens (SPEC-APP.md).
     */
    fun tagMatches(part: String, tag: String): Boolean {
        if (tag == part) return true
        if (!part.startsWith("todo.app.")) return false
        val name = part.removePrefix("todo.app.")
        return tag.startsWith("todo.") && tag.count { it == '.' } == 2 && tag.substringAfterLast('.') == name
    }

    private fun store(props: Map<String, Any?>, context: Context, emit: Emit) = TodoStore(
        tasks = Seeds.load(context, props["seed"] as? String ?: "three"),
        theme = ThemeChoice.entries.first { it.key == (props["theme"] as? String ?: "system") },
        showCompleted = props["showCompleted"] as? Boolean ?: true,
        onEvent = { e -> contract(e)?.let { (name, fields) -> emit(name, fields) } },
    )

    /** The app's event as the contracts name it. */
    private fun contract(e: TodoEvent): Pair<String, Map<String, Any>>? = when (e) {
        is TodoEvent.Added -> "added" to mapOf("id" to e.id, "title" to e.title)
        is TodoEvent.Toggled -> "toggled" to mapOf("id" to e.id, "done" to e.done)
        is TodoEvent.Opened -> "opened" to mapOf("id" to e.id)
        is TodoEvent.FilterChanged -> "filterChanged" to mapOf("filter" to e.filter.key)
        is TodoEvent.Renamed -> "renamed" to mapOf("id" to e.id, "title" to e.title)
        is TodoEvent.NotesChanged -> "notesChanged" to mapOf("id" to e.id, "notes" to e.notes)
        is TodoEvent.Deleted -> "deleted" to mapOf("id" to e.id)
        is TodoEvent.Closed -> "closed" to mapOf("id" to e.id)
        is TodoEvent.ThemeChanged -> "themeChanged" to mapOf("theme" to e.theme.key)
        is TodoEvent.ShowCompletedChanged -> "showCompletedChanged" to mapOf("show" to e.show)
        is TodoEvent.SettingsClosed -> "closed" to emptyMap()
        is TodoEvent.Navigated -> "navigated" to mapOf("screen" to e.screen, "id" to e.id)
    }
}
