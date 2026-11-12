package todo.app

import android.app.Application
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.viewModels
import androidx.lifecycle.AndroidViewModel
import todo.ui.app.TodoApp
import todo.ui.model.Seeds
import todo.ui.model.TodoStore

/** Keeps the tasks across configuration changes. The app starts with the "three" fixture tasks. */
class TodoViewModel(app: Application) : AndroidViewModel(app) {
    val store = TodoStore(Seeds.load(app, "three"))
}

class MainActivity : ComponentActivity() {
    private val model: TodoViewModel by viewModels()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent { TodoApp(model.store) }
    }
}
