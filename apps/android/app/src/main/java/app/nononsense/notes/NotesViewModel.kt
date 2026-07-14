package app.nononsense.notes

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.viewModelScope
import app.nononsense.notes.core.NoteKind
import app.nononsense.notes.core.NoteRecord
import app.nononsense.notes.core.NotesStore
import app.nononsense.notes.core.SyncDelegate
import app.nononsense.notes.core.SyncSession
import app.nononsense.notes.core.SyncStatus
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

data class NotesUiState(
    val authenticated: Boolean = false,
    val loading: Boolean = false,
    val authError: String? = null,
    val notes: List<NoteRecord> = emptyList(),
    val query: String = "",
    val selected: NoteRecord? = null,
    val syncStatus: SyncStatus = SyncStatus.DISCONNECTED,
    val syncDetail: String? = null,
)

class NotesViewModel(application: Application) : AndroidViewModel(application) {
    private val preferences = application.getSharedPreferences("session", 0)
    private val _state = MutableStateFlow(NotesUiState())
    val state = _state.asStateFlow()
    private var store: NotesStore? = null
    private var sync: SyncSession? = null

    init {
        val token = preferences.getString("token", null)
        val account = preferences.getString("account", null)
        if (token != null && account != null) openWorkspace(token, account)
    }

    fun authenticate(create: Boolean, email: String, password: String) {
        if (email.isBlank() || password.isBlank()) return
        _state.value = _state.value.copy(loading = true, authError = null)
        viewModelScope.launch {
            runCatching { AuthClient.authenticate(BuildConfig.API_URL, create, email, password) }
                .onSuccess { session ->
                    preferences.edit().putString("token", session.token).putString("account", session.accountId).apply()
                    openWorkspace(session.token, session.accountId)
                }
                .onFailure { error -> _state.value = _state.value.copy(loading = false, authError = error.message) }
        }
    }

    private fun openWorkspace(token: String, account: String) {
        _state.value = _state.value.copy(
            loading = true,
            authError = null,
            syncStatus = SyncStatus.CONNECTING,
            syncDetail = null,
        )
        viewModelScope.launch(Dispatchers.IO) {
            runCatching {
                val safeAccount = account.replace(Regex("[^A-Za-z0-9_-]"), "_")
                val opened = NotesStore.open(getApplication<Application>().getDatabasePath("notes-$safeAccount.db").absolutePath)
                val delegate = object : SyncDelegate {
                    override fun stateChanged(status: SyncStatus, detail: String?) {
                        viewModelScope.launch(Dispatchers.Main.immediate) {
                            _state.value = _state.value.copy(syncStatus = status, syncDetail = detail)
                        }
                    }
                    override fun notesChanged() { viewModelScope.launch { refresh() } }
                }
                val wsUrl = BuildConfig.API_URL.replaceFirst("http", "ws").trimEnd('/') + "/sync"
                Triple(opened, SyncSession.start(opened, wsUrl, token, delegate), opened.listNotes())
            }.onSuccess { (opened, session, notes) ->
                store = opened
                sync = session
                _state.value = _state.value.copy(authenticated = true, loading = false, notes = notes)
            }.onFailure { error ->
                _state.value = _state.value.copy(loading = false, authError = error.message)
            }
        }
    }

    fun logout() {
        sync?.stop(); sync?.close(); sync = null
        store?.close(); store = null
        preferences.edit().clear().apply()
        _state.value = NotesUiState()
    }

    fun setQuery(query: String) {
        _state.value = _state.value.copy(query = query)
        viewModelScope.launch { refresh() }
    }

    fun select(note: NoteRecord?) { _state.value = _state.value.copy(selected = note) }

    fun create(kind: NoteKind) = mutate { opened ->
        val note = opened.createNote(kind)
        withContext(Dispatchers.Main) { _state.value = _state.value.copy(selected = note) }
    }

    fun delete(id: String) = mutate { opened -> opened.deleteNote(id) }

    fun saveMarkdown(note: NoteRecord, title: String, content: String) = mutate(refreshAfter = false) { opened ->
        val updated = opened.updateMarkdown(note.id, content, title)
        withContext(Dispatchers.Main) { _state.value = _state.value.copy(selected = updated) }
    }

    fun saveList(note: NoteRecord, title: String, items: List<String>) = mutate(refreshAfter = false) { opened ->
        val updated = opened.updateList(note.id, items, title)
        withContext(Dispatchers.Main) { _state.value = _state.value.copy(selected = updated) }
    }

    private fun mutate(refreshAfter: Boolean = true, operation: suspend (NotesStore) -> Unit) {
        val opened = store ?: return
        viewModelScope.launch(Dispatchers.IO) {
            runCatching { operation(opened); sync?.wake() }
                .onSuccess { if (refreshAfter) refresh() }
                .onFailure { error -> _state.value = _state.value.copy(syncDetail = error.message) }
        }
    }

    private suspend fun refresh() = withContext(Dispatchers.IO) {
        val opened = store ?: return@withContext
        val notes = if (_state.value.query.isBlank()) opened.listNotes() else opened.searchNotes(_state.value.query)
        val selectedId = _state.value.selected?.id
        _state.value = _state.value.copy(notes = notes, selected = selectedId?.let { id -> notes.find { it.id == id } })
    }

    override fun onCleared() { sync?.stop(); sync?.close(); store?.close(); super.onCleared() }

    companion object {
        fun factory(application: Application) = object : ViewModelProvider.Factory {
            @Suppress("UNCHECKED_CAST")
            override fun <T : ViewModel> create(modelClass: Class<T>): T = NotesViewModel(application) as T
        }
    }
}
