package todo.ui.list

import androidx.compose.foundation.clickable
import androidx.compose.material3.Checkbox
import androidx.compose.material3.ListItem
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.style.TextDecoration
import todo.ui.Parts
import todo.ui.model.Task

/**
 * One task: a checkbox that marks it done, and its title. Pressing the row (anywhere but the
 * checkbox) opens the task. [position] is the row's place in the list, 1 at the top.
 */
@Composable
fun TaskRow(task: Task, position: Int, onToggle: (Boolean) -> Unit, onOpen: () -> Unit, modifier: Modifier = Modifier) {
    ListItem(
        modifier = modifier
            .testTag(Parts.List.row(position))
            .clickable(role = Role.Button, onClickLabel = "Open", onClick = onOpen),
        leadingContent = {
            Checkbox(
                checked = task.done,
                onCheckedChange = onToggle,
                modifier = Modifier.testTag(Parts.List.rowCheck(position)).semantics { contentDescription = task.title },
            )
        },
        headlineContent = {
            Text(
                task.title,
                color = if (task.done) MaterialTheme.colorScheme.onSurfaceVariant else MaterialTheme.colorScheme.onSurface,
                textDecoration = if (task.done) TextDecoration.LineThrough else null,
            )
        },
    )
}
