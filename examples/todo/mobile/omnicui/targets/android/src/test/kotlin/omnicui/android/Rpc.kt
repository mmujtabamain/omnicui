package omnicui.android

import java.io.InputStream
import java.io.OutputStream
import java.io.PrintStream
import org.json.JSONArray
import org.json.JSONObject

// protocol: error codes
const val NO_COMPONENT = -32001
const val NO_PART = -32002
const val STEP_FAILED = -32003
const val NO_METHOD = -32601

class RpcError(val code: Int, message: String) : Exception(message)

class Request(val id: Any?, val method: String, val params: JSONObject)

fun log(message: String) = System.err.println("[android] $message")

/** JSON-RPC 2.0, one request per line in, one reply per line out. */
class Rpc(input: InputStream, output: OutputStream) {
    private val reader = input.bufferedReader(Charsets.UTF_8)
    private val out = PrintStream(output, true, "UTF-8")

    /** The next request, or null when omnicui has closed standard input. */
    fun next(): Request? {
        while (true) {
            val line = reader.readLine() ?: return null
            if (line.isBlank()) continue
            try {
                val o = JSONObject(line)
                return Request(o.opt("id"), o.optString("method"), o.optJSONObject("params") ?: JSONObject())
            } catch (e: Exception) {
                send(JSONObject().put("id", JSONObject.NULL).put("error", JSONObject().put("code", -32700).put("message", "parse error")))
            }
        }
    }

    fun reply(id: Any?, result: Any?) = send(JSONObject().put("id", id ?: JSONObject.NULL).put("result", json(result)))

    fun error(id: Any?, code: Int, message: String) =
        send(JSONObject().put("id", id ?: JSONObject.NULL).put("error", JSONObject().put("code", code).put("message", message.lineSequence().first())))

    private fun send(o: JSONObject) {
        o.put("jsonrpc", "2.0")
        out.println(o.toString())
    }

    private fun json(v: Any?): Any = when (v) {
        null -> JSONObject.NULL
        is Map<*, *> -> JSONObject().also { o -> v.forEach { (k, x) -> o.put(k.toString(), json(x)) } }
        is List<*> -> JSONArray().also { a -> v.forEach { a.put(json(it)) } }
        is Float -> v.toDouble()
        else -> v
    }
}

/** A JSON object as a Kotlin map of strings, booleans, numbers, maps and lists. */
fun JSONObject.toMap(): Map<String, Any?> = keys().asSequence().associateWith { k ->
    when (val v = opt(k)) {
        JSONObject.NULL -> null
        is JSONObject -> v.toMap()
        is JSONArray -> (0 until v.length()).map { v.opt(it) }
        else -> v
    }
}
