package todo.ui.list

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.CubicBezierEasing
import androidx.compose.animation.core.tween
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.text.input.TextFieldLineLimits
import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardCapitalization
import todo.ui.Parts

/** The dialog's entrance: it fades in and grows from 90% over 200 ms, easing out (CSS ease-out). */
internal val DialogEnter = tween<Float>(durationMillis = 200, easing = CubicBezierEasing(0f, 0f, 0.58f, 1f))

/**
 * "New task": a title field, Add (enabled once the title isn't blank) and Cancel. Enter in the
 * field adds too. [onAdd] gets the title as typed; the store trims it.
 */
@Composable
fun AddTaskDialog(onAdd: (String) -> Unit, onDismiss: () -> Unit) {
    val title = rememberTextFieldState()
    val canAdd by remember { derivedStateOf { title.text.isNotBlank() } }
    val add = { if (canAdd) onAdd(title.text.toString()) }
    val focus = remember { FocusRequester() }
    val appear = remember { Animatable(0f) }
    LaunchedEffect(Unit) {
        focus.requestFocus()
        appear.animateTo(1f, DialogEnter)
    }
    AlertDialog(
        onDismissRequest = onDismiss,
        // The layer first, so that the part's box is the box as drawn, scale included.
        modifier = Modifier
            .graphicsLayer {
                alpha = appear.value
                scaleX = 0.9f + 0.1f * appear.value
                scaleY = scaleX
            }
            .testTag(Parts.List.DIALOG),
        title = { Text("New task") },
        text = {
            OutlinedTextField(
                state = title,
                label = { Text("Title") },
                lineLimits = TextFieldLineLimits.SingleLine,
                keyboardOptions = KeyboardOptions(capitalization = KeyboardCapitalization.Sentences, imeAction = ImeAction.Done),
                onKeyboardAction = { add() },
                modifier = Modifier.testTag(Parts.List.NEW_TITLE).focusRequester(focus),
            )
        },
        confirmButton = {
            TextButton(onClick = add, enabled = canAdd, modifier = Modifier.testTag(Parts.List.CONFIRM_ADD)) { Text("Add") }
        },
        dismissButton = {
            TextButton(onClick = onDismiss, modifier = Modifier.testTag(Parts.List.CANCEL_ADD)) { Text("Cancel") }
        },
    )
}
