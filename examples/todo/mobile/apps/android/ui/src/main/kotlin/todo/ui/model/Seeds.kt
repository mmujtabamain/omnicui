package todo.ui.model

import android.content.Context
import org.json.JSONObject
import todo.ui.R

/**
 * The fixture task lists, by name: "empty", "three" and "many". They are res/raw/seeds.json, which
 * is byte for byte the same file as every other version of the app uses (SPEC-APP.md, Fixtures).
 */
object Seeds {
    val names = listOf("empty", "three", "many")

    fun load(context: Context, name: String): List<Task> =
        parse(context.resources.openRawResource(R.raw.seeds).bufferedReader().use { it.readText() }, name)

    fun parse(json: String, name: String): List<Task> {
        val all = JSONObject(json)
        require(all.has(name)) { "no seed \"$name\"; the seeds are ${names.joinToString()}" }
        val list = all.getJSONArray(name)
        return (0 until list.length()).map { i ->
            val t = list.getJSONObject(i)
            Task(t.getString("id"), t.getString("title"), t.getString("notes"), t.getBoolean("done"))
        }
    }
}
