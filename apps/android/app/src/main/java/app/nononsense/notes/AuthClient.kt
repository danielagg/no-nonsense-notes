package app.nononsense.notes

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.json.JSONObject
import java.net.HttpURLConnection
import java.net.URL

data class Session(val token: String, val accountId: String)

object AuthClient {
    suspend fun authenticate(baseUrl: String, create: Boolean, email: String, password: String): Session = withContext(Dispatchers.IO) {
        val endpoint = if (create) "signup" else "signin"
        val connection = (URL("${baseUrl.trimEnd('/')}/auth/$endpoint").openConnection() as HttpURLConnection).apply {
            requestMethod = "POST"
            connectTimeout = 10_000
            readTimeout = 15_000
            doOutput = true
            setRequestProperty("Content-Type", "application/json")
        }
        connection.outputStream.bufferedWriter().use {
            it.write(JSONObject().put("email", email).put("password", password).toString())
        }
        val body = (if (connection.responseCode in 200..299) connection.inputStream else connection.errorStream)
            ?.bufferedReader()?.use { it.readText() }.orEmpty()
        if (connection.responseCode !in 200..299) {
            val message = runCatching { JSONObject(body).optString("error") }.getOrNull().orEmpty()
            error(message.ifBlank { "Authentication failed (${connection.responseCode})" })
        }
        val json = JSONObject(body)
        Session(json.getString("token"), json.getString("account_id"))
    }
}

