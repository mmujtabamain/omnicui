package todo.ui.list

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import todo.ui.Parts
import todo.ui.model.Filter
import todo.ui.model.Task
import todo.ui.model.TodoEvent
import todo.ui.model.TodoStore

/** The task list, wired to the store. [onOpen] is called after the store has reported it. */
@Composable
fun TaskListRoute(store: TodoStore, onOpen: (String) -> Unit = {}, actions: @Composable RowScope.() -> Unit = {}) {
    TaskListScreen(
        tasks = store.visible,
        filter = store.filter,
        left = store.left,
        anyTasks = store.tasks.isNotEmpty(),
        onFilter = store::select,
        onToggle = store::setDone,
        onOpen = { id ->
            store.report(TodoEvent.Opened(id))
            onOpen(id)
        },
        onAdd = { store.add(it) },
        actions = actions,
    )
}

/**
 * The task list: the top app bar ("Tasks", and [actions]), the filter, how many are left, the
 * tasks, and the button that opens [AddTaskDialog].
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun TaskListScreen(
    tasks: List<Task>,
    filter: Filter,
    left: Int,
    anyTasks: Boolean,
    onFilter: (Filter) -> Unit,
    onToggle: (String, Boolean) -> Unit,
    onOpen: (String) -> Unit,
    onAdd: (String) -> Unit,
    modifier: Modifier = Modifier,
    actions: @Composable RowScope.() -> Unit = {},
) {
    var adding by rememberSaveable { mutableStateOf(false) }
    Scaffold(
        modifier = modifier,
        topBar = {
            TopAppBar(
                title = { Text("Tasks", Modifier.testTag(Parts.List.HEADING).semantics { heading() }) },
                actions = actions,
            )
        },
        floatingActionButton = {
            FloatingActionButton(onClick = { adding = true }, modifier = Modifier.testTag(Parts.List.ADD)) {
                Icon(Icons.Filled.Add, contentDescription = "Add task")
            }
        },
    ) { padding ->
        Column(Modifier.padding(padding).fillMaxSize()) {
            FilterBar(filter, onFilter, Modifier.padding(horizontal = 16.dp, vertical = 8.dp))
            Text(
                "$left left",
                style = MaterialTheme.typography.labelLarge,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier
                    .padding(horizontal = 16.dp, vertical = 4.dp)
                    .testTag(Parts.List.LEFT)
                    .semantics { liveRegion = LiveRegionMode.Polite },
            )
            if (tasks.isEmpty()) {
                EmptyState(emptyMessage(filter, anyTasks))
            } else {
                LazyColumn(contentPadding = PaddingValues(bottom = 88.dp)) {
                    itemsIndexed(tasks, key = { _, t -> t.id }) { i, task ->
                        TaskRow(task, position = i + 1, onToggle = { onToggle(task.id, it) }, onOpen = { onOpen(task.id) })
                    }
                }
            }
        }
    }
    if (adding) {
        AddTaskDialog(
            onAdd = {
                onAdd(it)
                adding = false
            },
            onDismiss = { adding = false },
        )
    }
}

/** What the list says when it shows no task (SPEC-APP.md, Empty states). */
fun emptyMessage(filter: Filter, anyTasks: Boolean): String = when {
    filter == Filter.Done -> "Nothing done yet"
    !anyTasks -> "No tasks yet"
    else -> "Nothing left to do"
}

@Composable
private fun EmptyState(message: String) {
    Box(Modifier.fillMaxWidth().padding(top = 48.dp), contentAlignment = Alignment.TopCenter) {
        Text(
            message,
            style = MaterialTheme.typography.bodyLarge,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.testTag(Parts.List.EMPTY).semantics { liveRegion = LiveRegionMode.Polite },
        )
    }
}
