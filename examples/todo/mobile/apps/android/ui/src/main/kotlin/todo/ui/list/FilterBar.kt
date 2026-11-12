package todo.ui.list

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import todo.ui.Parts
import todo.ui.model.Filter

/** All, Active, Done: a single-choice segmented button. */
@Composable
fun FilterBar(filter: Filter, onFilter: (Filter) -> Unit, modifier: Modifier = Modifier) {
    SingleChoiceSegmentedButtonRow(modifier.fillMaxWidth()) {
        Filter.entries.forEachIndexed { i, f ->
            SegmentedButton(
                selected = f == filter,
                onClick = { onFilter(f) },
                shape = SegmentedButtonDefaults.itemShape(i, Filter.entries.size),
                modifier = Modifier.testTag(f.tag),
            ) { Text(f.label) }
        }
    }
}

private val Filter.label get() = when (this) {
    Filter.All -> "All"
    Filter.Active -> "Active"
    Filter.Done -> "Done"
}

private val Filter.tag get() = when (this) {
    Filter.All -> Parts.List.FILTER_ALL
    Filter.Active -> Parts.List.FILTER_ACTIVE
    Filter.Done -> Parts.List.FILTER_DONE
}
