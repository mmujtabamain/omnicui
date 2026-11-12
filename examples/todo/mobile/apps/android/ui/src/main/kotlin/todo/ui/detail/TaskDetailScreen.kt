package todo.ui.detail

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.selection.toggleable
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.text.input.TextFieldLineLimits
import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.input.KeyboardCapitalization
import androidx.compose.ui.unit.dp
import todo.ui.Parts
import todo.ui.model.Task
import todo.ui.model.TodoEvent
import todo.ui.model.TodoStore

/**
 * One task's detail, wired to the store. [onLeave] is called when the user leaves it: with the
 * back arrow (reported as Closed) or by deleting the task.
 */
@Composable
fun TaskDetailRoute(store: TodoStore, id: String, onLeave: () -> Unit = {}) {
    val task = store.task(id) ?: return
    TaskDetailScreen(
        task = task,
        onRename = { store.rename(id, it) },
        onEditNotes = { store.editNotes(id, it) },
        onDone = { store.setDone(id, it) },
        onDelete = {
            store.delete(id)
            onLeave()
        },
        onBack = {
            store.report(TodoEvent.Closed(id))
            onLeave()
        },
    )
}

/** The title and the notes (edited in place), the Done switch, Delete, and the up arrow. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun TaskDetailScreen(
    task: Task,
    onRename: (String) -> Unit,
    onEditNotes: (String) -> Unit,
    onDone: (Boolean) -> Unit,
    onDelete: () -> Unit,
    onBack: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Scaffold(
        modifier = modifier,
        topBar = {
            TopAppBar(
                title = { Text("Task") },
                navigationIcon = {
                    IconButton(onClick = onBack, modifier = Modifier.testTag(Parts.Detail.BACK)) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "Back")
                    }
                },
                actions = {
                    IconButton(onClick = onDelete, modifier = Modifier.testTag(Parts.Detail.DELETE)) {
                        Icon(Icons.Filled.Delete, contentDescription = "Delete")
                    }
                },
            )
        },
    ) { padding ->
        Column(
            Modifier.padding(padding).padding(horizontal = 16.dp, vertical = 8.dp).fillMaxSize(),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            // The fields keep their own text while it is edited; every change is reported.
            val title = rememberTextFieldState(task.title)
            val notes = rememberTextFieldState(task.notes)
            LaunchedEffect(title) { snapshotFlow { title.text.toString() }.collect(onRename) }
            LaunchedEffect(notes) { snapshotFlow { notes.text.toString() }.collect(onEditNotes) }
            OutlinedTextField(
                state = title,
                label = { Text("Title") },
                lineLimits = TextFieldLineLimits.SingleLine,
                keyboardOptions = KeyboardOptions(capitalization = KeyboardCapitalization.Sentences),
                modifier = Modifier.fillMaxWidth().testTag(Parts.Detail.TITLE),
            )
            OutlinedTextField(
                state = notes,
                label = { Text("Notes") },
                lineLimits = TextFieldLineLimits.MultiLine(minHeightInLines = 3, maxHeightInLines = 8),
                keyboardOptions = KeyboardOptions(capitalization = KeyboardCapitalization.Sentences),
                modifier = Modifier.fillMaxWidth().testTag(Parts.Detail.NOTES),
            )
            Row(
                Modifier
                    .fillMaxWidth()
                    .testTag(Parts.Detail.DONE)
                    .toggleable(value = task.done, role = Role.Switch, onValueChange = onDone)
                    .padding(vertical = 8.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text("Done", style = MaterialTheme.typography.bodyLarge, modifier = Modifier.weight(1f))
                Switch(checked = task.done, onCheckedChange = null)
            }
        }
    }
}
