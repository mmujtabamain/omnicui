package todo.ui.settings

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.selection.toggleable
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.unit.dp
import todo.ui.Parts
import todo.ui.model.ThemeChoice
import todo.ui.model.TodoEvent
import todo.ui.model.TodoStore

/** Settings, wired to the store. [onLeave] is called after the up arrow is reported. */
@Composable
fun SettingsRoute(store: TodoStore, onLeave: () -> Unit = {}) {
    SettingsScreen(
        theme = store.theme,
        showCompleted = store.showCompleted,
        onTheme = store::chooseTheme,
        onShowCompleted = store::showCompleted,
        onBack = {
            store.report(TodoEvent.SettingsClosed)
            onLeave()
        },
    )
}

/** The theme (System, Light, Dark) and whether the list shows completed tasks under All. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsScreen(
    theme: ThemeChoice,
    showCompleted: Boolean,
    onTheme: (ThemeChoice) -> Unit,
    onShowCompleted: (Boolean) -> Unit,
    onBack: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Scaffold(
        modifier = modifier,
        topBar = {
            TopAppBar(
                title = { Text("Settings") },
                navigationIcon = {
                    IconButton(onClick = onBack, modifier = Modifier.testTag(Parts.Settings.BACK)) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "Back")
                    }
                },
            )
        },
    ) { padding ->
        Column(Modifier.padding(padding).fillMaxSize()) {
            Text(
                "Theme",
                style = MaterialTheme.typography.titleSmall,
                color = MaterialTheme.colorScheme.primary,
                modifier = Modifier.padding(start = 16.dp, end = 16.dp, top = 16.dp, bottom = 8.dp),
            )
            SingleChoiceSegmentedButtonRow(Modifier.fillMaxWidth().padding(horizontal = 16.dp)) {
                ThemeChoice.entries.forEachIndexed { i, choice ->
                    SegmentedButton(
                        selected = choice == theme,
                        onClick = { onTheme(choice) },
                        shape = SegmentedButtonDefaults.itemShape(i, ThemeChoice.entries.size),
                        modifier = Modifier.testTag(choice.tag),
                    ) { Text(choice.label) }
                }
            }
            Row(
                Modifier
                    .fillMaxWidth()
                    .padding(top = 16.dp)
                    .testTag(Parts.Settings.SHOW_COMPLETED)
                    .toggleable(value = showCompleted, role = Role.Switch, onValueChange = onShowCompleted)
                    .padding(horizontal = 16.dp, vertical = 12.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text("Show completed", style = MaterialTheme.typography.bodyLarge, modifier = Modifier.weight(1f))
                Switch(checked = showCompleted, onCheckedChange = null)
            }
        }
    }
}

private val ThemeChoice.label get() = when (this) {
    ThemeChoice.System -> "System"
    ThemeChoice.Light -> "Light"
    ThemeChoice.Dark -> "Dark"
}

private val ThemeChoice.tag get() = when (this) {
    ThemeChoice.System -> Parts.Settings.THEME_SYSTEM
    ThemeChoice.Light -> Parts.Settings.THEME_LIGHT
    ThemeChoice.Dark -> Parts.Settings.THEME_DARK
}
