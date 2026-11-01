package io.github.aghorbani84.aurex.storage

import android.app.Activity
import android.content.Intent
import android.net.Uri
import android.os.CancellationSignal
import android.provider.DocumentsContract
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.activity.ComponentActivity
import androidx.activity.result.ActivityResultLauncher
import androidx.activity.result.contract.ActivityResultContracts
import org.json.JSONArray
import org.json.JSONObject
import java.io.ByteArrayOutputStream
import java.lang.ref.WeakReference
import java.nio.ByteBuffer
import java.nio.charset.CodingErrorAction
import java.util.Locale
import java.util.concurrent.ScheduledThreadPoolExecutor
import java.util.concurrent.TimeUnit

/** Read-only SAF boundary. Only provider-discovered identities may reach ContentResolver. */
class StoragePlugin(private val activity: Activity) {
    private data class DocumentKind(val directory: Boolean, val virtual: Boolean)
    companion object {
        private const val PREFS = "aurex.saf.grants.v1"
        private const val MAX_KNOWN = 4096
        private const val MAX_PAGE = 500
        private const val MAX_PREVIEW = 256 * 1024
        private val lock = Any()
        private var rootTree: Uri? = null
        private var initialized = false
        private var generation = 0L
        private var picking = false
        private var lastError: String? = null
        private val known = HashMap<String, DocumentKind>()
        private var launcher: ActivityResultLauncher<Intent>? = null
        private var launcherActivity: WeakReference<ComponentActivity>? = null
        private val timers = ScheduledThreadPoolExecutor(1) { runnable ->
            Thread(runnable, "aurex-saf-timeout").apply { isDaemon = true }
        }.apply { removeOnCancelPolicy = true }
    }
    private val resolver get() = activity.contentResolver
    private val preferences get() = activity.getSharedPreferences(PREFS, Activity.MODE_PRIVATE)
    private fun documentRoot(tree: Uri): Uri = DocumentsContract.buildDocumentUriUsingTree(
        tree, DocumentsContract.getTreeDocumentId(tree))
    private fun readable(tree: Uri): Boolean = resolver.persistedUriPermissions.any {
        it.uri == tree && it.isReadPermission
    }
    private fun reset(tree: Uri?) {
        check(generation < Long.MAX_VALUE) { "Storage generation exhausted; restart app" }
        generation += 1
        rootTree = tree
        known.clear()
        if (tree != null) known[documentRoot(tree).toString()] = DocumentKind(true, false)
    }
    private fun grant(): Uri = synchronized(lock) {
        if (!initialized) {
            initialized = true
            val saved = preferences.getString("tree", null)
            if (saved != null) {
                val uri = Uri.parse(saved)
                if (uri.scheme == "content" && DocumentsContract.isTreeUri(uri) && readable(uri)) reset(uri)
                else preferences.edit().remove("tree").apply()
            }
        }
        val tree = rootTree ?: throw SecurityException("Choose a folder first")
        if (!readable(tree)) {
            reset(null)
            preferences.edit().remove("tree").apply()
            throw SecurityException("Folder access was revoked. Choose the folder again.")
        }
        tree
    }
    private fun authorize(tree: Uri, value: String, directory: Boolean): Uri = synchronized(lock) {
        check(rootTree == tree) { "Folder grant changed; retry the current view" }
        val kind = known[value]
        require(value.length <= 16384 && kind != null && kind.directory == directory && !kind.virtual) { "Document was not discovered in this folder grant" }
        val uri = Uri.parse(value)
        require(uri.scheme == "content" && uri.authority == tree.authority &&
            DocumentsContract.getTreeDocumentId(uri) == DocumentsContract.getTreeDocumentId(tree)) {
            "Foreign document URI"
        }
        uri
    }
    private inline fun response(block: () -> JSONObject): String = try {
        block().toString()
    } catch (error: Exception) {
        JSONObject().put("error", error.message ?: "Storage provider refused the request").toString()
    }
    fun stateJson(): String = response {
        // No grant is a valid initial state, not a fabricated empty directory.
        try { grant() } catch (error: SecurityException) {
            synchronized(lock) { if (error.message != "Choose a folder first") lastError = error.message }
        }
        synchronized(lock) {
            JSONObject().put("root", rootTree?.let { documentRoot(it).toString() } ?: JSONObject.NULL)
                .put("generation", generation).put("picking", picking).put("notice", lastError ?: JSONObject.NULL)
        }
    }
    fun chooseJson(): String = response {
        synchronized(lock) { check(!picking) { "Folder picker is already open" }; picking = true; lastError = null }
        activity.runOnUiThread {
            try {
                val owner = activity as? ComponentActivity ?: error("Activity result registry is unavailable")
                if (launcherActivity?.get() !== owner) {
                    launcher?.unregister()
                    launcherActivity = WeakReference(owner)
                    owner.lifecycle.addObserver(LifecycleEventObserver { _, event ->
                        if (event == Lifecycle.Event.ON_DESTROY && launcherActivity?.get() === owner) {
                            launcher?.unregister(); launcher = null; launcherActivity = null
                            synchronized(lock) { picking = false }
                        }
                    })
                    launcher = owner.activityResultRegistry.register("aurex.saf.tree.v1",
                        ActivityResultContracts.StartActivityForResult()) { result ->
                        try {
                            val data = result.data
                            val uri = data?.data
                            if (result.resultCode != Activity.RESULT_OK || uri == null) {
                                synchronized(lock) { lastError = "Folder selection cancelled" }
                            } else {
                                require(uri.scheme == "content" && DocumentsContract.isTreeUri(uri)) { "Not a document tree" }
                                require((data.flags and Intent.FLAG_GRANT_READ_URI_PERMISSION) != 0) { "Provider did not grant read access" }
                                // Persist READ only, even if a provider offers WRITE as well.
                                resolver.takePersistableUriPermission(uri, Intent.FLAG_GRANT_READ_URI_PERMISSION)
                                synchronized(lock) {
                                    // Release only this app's previous selected grant, never user data.
                                    val previous = rootTree
                                    if (previous != null && previous != uri && readable(previous)) {
                                        resolver.releasePersistableUriPermission(previous, Intent.FLAG_GRANT_READ_URI_PERMISSION)
                                    }
                                    reset(uri); initialized = true; lastError = null
                                    preferences.edit().putString("tree", uri.toString()).apply()
                                }
                            }
                        } catch (error: Exception) {
                            synchronized(lock) { lastError = error.message ?: "Folder permission failed" }
                        } finally { synchronized(lock) { picking = false } }
                    }
                }
                val intent = Intent(Intent.ACTION_OPEN_DOCUMENT_TREE).addFlags(
                    Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION or Intent.FLAG_GRANT_PREFIX_URI_PERMISSION)
                if (android.os.Build.VERSION.SDK_INT >= 26) {
                    // A picker hint, not an access grant. The OS still asks the user.
                    intent.putExtra(DocumentsContract.EXTRA_INITIAL_URI, DocumentsContract.buildDocumentUri(
                        "com.android.externalstorage.documents", "primary:Documents"))
                }
                launcher!!.launch(intent)
            } catch (error: Exception) {
                synchronized(lock) { picking = false; lastError = error.message ?: "Cannot open folder picker" }
            }
        }
        JSONObject().put("requested", true)
    }
    fun forgetJson(): String = response {
        synchronized(lock) {
            check(!picking) { "Close the folder picker first" }
            val previous = rootTree
            if (previous != null && readable(previous)) {
                resolver.releasePersistableUriPermission(previous, Intent.FLAG_GRANT_READ_URI_PERMISSION)
            }
            reset(null); initialized = true; lastError = null
            preferences.edit().remove("tree").apply()
        }
        JSONObject().put("forgotten", true)
    }
    fun listJson(folder: String): String = response {
        val tree = grant()
        val epoch = synchronized(lock) { generation }
        val parent = authorize(tree, folder, true)
        val children = DocumentsContract.buildChildDocumentsUriUsingTree(tree, DocumentsContract.getDocumentId(parent))
        val signal = CancellationSignal()
        val timeout = timers.schedule({ signal.cancel() }, 10, TimeUnit.SECONDS)
        val entries = ArrayList<JSONObject>()
        var truncated = false
        try {
            val columns = arrayOf(DocumentsContract.Document.COLUMN_DOCUMENT_ID,
                DocumentsContract.Document.COLUMN_DISPLAY_NAME, DocumentsContract.Document.COLUMN_MIME_TYPE,
                DocumentsContract.Document.COLUMN_SIZE, DocumentsContract.Document.COLUMN_FLAGS)
            val cursor = resolver.query(children, columns, null, null, null, signal)
                ?: error("Provider returned no directory cursor")
            cursor.use {
                while (it.moveToNext()) {
                    if (entries.size == MAX_PAGE) { truncated = true; break }
                    val id = it.getString(0) ?: error("Provider returned no document identity")
                    val name = it.getString(1) ?: "Unnamed document"
                    require(id.length <= 4096 && name.length <= 4096) { "Provider metadata exceeds limits" }
                    val directory = it.getString(2) == DocumentsContract.Document.MIME_TYPE_DIR
                    val uri = DocumentsContract.buildDocumentUriUsingTree(tree, id).toString()
                    val virtual = (it.getInt(4) and DocumentsContract.Document.FLAG_VIRTUAL_DOCUMENT) != 0
                    entries.add(JSONObject().put("uri", uri).put("name", name).put("directory", directory)
                        .put("virtual", virtual).put("size", if (it.isNull(3)) JSONObject.NULL else it.getLong(3)))
                }
            }
        } finally { timeout.cancel(false) }
        entries.sortWith(compareBy<JSONObject>({ !it.getBoolean("directory") },
            { it.getString("name").lowercase(Locale.ROOT) }, { it.getString("uri") }))
        synchronized(lock) {
            check(rootTree == tree && generation == epoch && readable(tree)) { "Folder grant changed or was revoked" }
            val additions = entries.map { it.getString("uri") }
            require(additions.toSet().size == additions.size) { "Provider returned duplicate document identities" }
            require(known.size + additions.count { !known.containsKey(it) } <= MAX_KNOWN) { "Session document limit; reselect folder" }
            val discovered = entries.associate { entry -> entry.getString("uri") to
                DocumentKind(entry.getBoolean("directory"), entry.getBoolean("virtual")) }
            require(discovered.all { (uri, kind) -> known[uri] == null || known[uri] == kind }) {
                "Document kind changed; reselect folder"
            }
            known.putAll(discovered)
        }
        JSONObject().put("documents", JSONArray(entries)).put("truncated", truncated).put("generation", epoch)
    }
    fun previewJson(document: String): String = response {
        val tree = grant()
        val epoch = synchronized(lock) { generation }
        val uri = authorize(tree, document, false)
        val signal = CancellationSignal()
        val openingTimeout = timers.schedule({ signal.cancel() }, 10, TimeUnit.SECONDS)
        val descriptor = try {
            resolver.openAssetFileDescriptor(uri, "r", signal) ?: error("Provider returned no readable document")
        } finally { openingTimeout.cancel(false) }
        val text = descriptor.use { asset ->
            asset.createInputStream().use { input ->
                val timeout = timers.schedule({ try { input.close() } catch (_: Exception) { } }, 10, TimeUnit.SECONDS)
                try {
                    val output = ByteArrayOutputStream()
                    val buffer = ByteArray(16384)
                    while (true) {
                        val count = input.read(buffer, 0, minOf(buffer.size, MAX_PREVIEW + 1 - output.size()))
                        if (count == -1) break
                        check(count > 0) { "Provider made no read progress" }
                        output.write(buffer, 0, count)
                        require(output.size() <= MAX_PREVIEW) { "Preview exceeds 256 KiB" }
                    }
                    Charsets.UTF_8.newDecoder().onMalformedInput(CodingErrorAction.REPORT)
                        .onUnmappableCharacter(CodingErrorAction.REPORT).decode(ByteBuffer.wrap(output.toByteArray())).toString()
                } finally { timeout.cancel(false) }
            }
        }
        require(text.none { (it.code < 32 && it !in "\t\r\n") || it.code in 127..159 }) { "Binary/control data is not previewed" }
        synchronized(lock) { check(rootTree == tree && generation == epoch && readable(tree)) { "Folder grant changed or was revoked" } }
        JSONObject().put("text", text).put("generation", epoch)
    }
}
