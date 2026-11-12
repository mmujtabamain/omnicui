package todo.ui

import java.io.File
import java.security.MessageDigest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import todo.ui.model.Filter
import todo.ui.model.Seeds
import todo.ui.model.ThemeChoice
import todo.ui.model.TodoEvent
import todo.ui.model.TodoStore

class TodoStoreTest {
    private val json = File("src/main/res/raw/seeds.json").readText()
    private val events = mutableListOf<TodoEvent>()
    private fun store(seed: String = "three", showCompleted: Boolean = true) =
        TodoStore(Seeds.parse(json, seed), showCompleted = showCompleted, onEvent = { events += it })

    @Test fun `the seeds are the shared fixture file, byte for byte`() {
        val sha = MessageDigest.getInstance("SHA-256").digest(File("src/main/res/raw/seeds.json").readBytes()).joinToString("") { "%02x".format(it) }
        assertEquals("c1ace19804fea36d7f4a6e1b96c2e183c518741010339f09c058615992a51d2e", sha)
        assertEquals(listOf(0, 3, 20), Seeds.names.map { Seeds.parse(json, it).size })
    }

    @Test fun `adding trims the title, continues the ids and reports it`() {
        val s = store()
        assertEquals("t4", s.add("  Water the plants ")!!.id)
        assertEquals("Water the plants", s.tasks.last().title)
        assertEquals(listOf(TodoEvent.Added("t4", "Water the plants")), events)
        assertNull(s.add("   "))
        s.delete("t4")
        assertEquals("t5", s.add("Again")!!.id) // a deleted task's id is never given again
    }

    @Test fun `filters, the count left and show completed`() {
        val s = store()
        assertEquals(2, s.left)
        s.select(Filter.Done)
        assertEquals(listOf("t2"), s.visible.map { it.id })
        s.select(Filter.Done) // already chosen: nothing to report
        assertEquals(listOf<TodoEvent>(TodoEvent.FilterChanged(Filter.Done)), events)
        s.select(Filter.Active)
        assertEquals(listOf("t1", "t3"), s.visible.map { it.id })
        val hidden = store(showCompleted = false)
        assertEquals(listOf("t1", "t3"), hidden.visible.map { it.id })
    }

    @Test fun `toggling, renaming and deleting report only real changes`() {
        val s = store()
        s.setDone("t1", true)
        s.setDone("t1", true)
        s.rename("t1", "Buy milk")
        s.rename("t1", "Buy oat milk")
        s.delete("t9")
        s.delete("t2")
        s.chooseTheme(ThemeChoice.Dark)
        assertEquals(
            listOf(TodoEvent.Toggled("t1", true), TodoEvent.Renamed("t1", "Buy oat milk"), TodoEvent.Deleted("t2"), TodoEvent.ThemeChanged(ThemeChoice.Dark)),
            events,
        )
        assertEquals(1, s.left)
    }
}
