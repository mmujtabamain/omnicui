package todo.ui.app

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.navigation.NavBackStackEntry
import androidx.navigation.NavDestination.Companion.hasRoute
import androidx.navigation.NavHostController
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import androidx.navigation.toRoute
import kotlinx.coroutines.flow.drop
import kotlinx.serialization.Serializable
import todo.ui.Parts
import todo.ui.detail.TaskDetailRoute
import todo.ui.list.TaskListRoute
import todo.ui.model.TodoEvent
import todo.ui.model.TodoStore
import todo.ui.settings.SettingsRoute
import todo.ui.theme.TodoTheme

@Serializable object TaskList
@Serializable data class TaskDetail(val id: String)
@Serializable object Settings

/**
 * The whole app: the task list, a task's detail and settings, with Navigation Compose. The up
 * arrows and the system back return to the list. Each change of screen after the first is
 * reported as [TodoEvent.Navigated].
 */
@Composable
fun TodoApp(store: TodoStore, navController: NavHostController = rememberNavController()) {
    LaunchedEffect(navController) {
        navController.currentBackStackEntryFlow.drop(1).collect { store.report(it.toNavigated()) }
    }
    TodoTheme(store.theme) {
        NavHost(navController, startDestination = TaskList) {
            composable<TaskList> {
                TaskListRoute(
                    store,
                    onOpen = { navController.navigate(TaskDetail(it)) { launchSingleTop = true } },
                    actions = {
                        IconButton(
                            onClick = { navController.navigate(Settings) { launchSingleTop = true } },
                            modifier = Modifier.testTag(Parts.App.OPEN_SETTINGS),
                        ) { Icon(Icons.Filled.Settings, contentDescription = "Settings") }
                    },
                )
            }
            composable<TaskDetail> { entry ->
                TaskDetailRoute(store, entry.toRoute<TaskDetail>().id, onLeave = { navController.popBackStack(TaskList, inclusive = false) })
            }
            composable<Settings> {
                SettingsRoute(store, onLeave = { navController.popBackStack(TaskList, inclusive = false) })
            }
        }
    }
}

private fun NavBackStackEntry.toNavigated(): TodoEvent.Navigated = when {
    destination.hasRoute<TaskDetail>() -> TodoEvent.Navigated("detail", toRoute<TaskDetail>().id)
    destination.hasRoute<Settings>() -> TodoEvent.Navigated("settings", "")
    else -> TodoEvent.Navigated("list", "")
}
