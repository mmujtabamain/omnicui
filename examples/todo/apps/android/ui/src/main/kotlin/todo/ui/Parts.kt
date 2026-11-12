package todo.ui

/**
 * The tags on the widgets a test can find (Modifier.testTag), one per part of the contracts in
 * examples/todo/omnicui/contracts. A tag is "<component>.<part>"; SPEC-APP.md lists them all.
 */
object Parts {
    object List {
        const val HEADING = "todo.list.tasksHeading"
        const val FILTER_ALL = "todo.list.filterAll"
        const val FILTER_ACTIVE = "todo.list.filterActive"
        const val FILTER_DONE = "todo.list.filterDone"
        const val LEFT = "todo.list.left"
        const val ADD = "todo.list.add"
        const val EMPTY = "todo.list.empty"
        const val DIALOG = "todo.list.newTaskDialog"
        const val NEW_TITLE = "todo.list.newTitle"
        const val CONFIRM_ADD = "todo.list.confirmAdd"
        const val CANCEL_ADD = "todo.list.cancelAdd"

        /** The row shown at [position] (1 = the top one), whichever task it holds. */
        fun row(position: Int) = "todo.list.row$position"
        fun rowCheck(position: Int) = "todo.list.row${position}Check"
    }

    object Detail {
        const val TITLE = "todo.detail.title"
        const val NOTES = "todo.detail.notes"
        const val DONE = "todo.detail.done"
        const val DELETE = "todo.detail.delete"
        const val BACK = "todo.detail.detailBack"
    }

    object Settings {
        const val THEME_SYSTEM = "todo.settings.themeSystem"
        const val THEME_LIGHT = "todo.settings.themeLight"
        const val THEME_DARK = "todo.settings.themeDark"
        const val SHOW_COMPLETED = "todo.settings.showCompleted"
        const val BACK = "todo.settings.settingsBack"
    }

    object App {
        const val OPEN_SETTINGS = "todo.app.openSettings"
    }
}
