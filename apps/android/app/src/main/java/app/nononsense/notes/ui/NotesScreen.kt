package app.nononsense.notes.ui

import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import app.nononsense.notes.core.NoteKind
import app.nononsense.notes.core.NoteRecord
import app.nononsense.notes.core.SyncStatus

@Composable
internal fun NotesScreen(notes: List<NoteRecord>, query: String, syncStatus: SyncStatus, dark: Boolean, toggleTheme: () -> Unit, setQuery: (String) -> Unit, open: (NoteRecord) -> Unit, create: (NoteKind) -> Unit, delete: (String) -> Unit, logout: () -> Unit) {
    Column(Modifier.fillMaxSize().padding(top = if (syncStatus == SyncStatus.CONNECTED) 0.dp else 40.dp)) {
        Row(Modifier.fillMaxWidth().height(66.dp).padding(horizontal = 16.dp).border(width = 0.dp, color = Color.Transparent), verticalAlignment = Alignment.CenterVertically) {
            Brand(); Spacer(Modifier.weight(1f)); ThemeButton(dark, toggleTheme); IconButton(logout) { Icon(Icons.Default.Logout, "Log out") }
        }
        Column(Modifier.fillMaxSize().padding(horizontal = 16.dp)) {
            Text("All notes", Modifier.padding(top = 24.dp), fontFamily = Mono, fontSize = 34.sp, fontWeight = FontWeight.Bold, letterSpacing = (-1.5).sp)
            Text(if (notes.isEmpty()) "A quiet space, ready for your next idea." else "${notes.size} note${if (notes.size == 1) "" else "s"} in your workspace.", color = MaterialTheme.colorScheme.onSurfaceVariant, fontSize = 14.sp)
            OutlinedTextField(query, setQuery, Modifier.fillMaxWidth().padding(top = 18.dp), leadingIcon = { Icon(Icons.Default.Search, null) }, placeholder = { Text("Search notes", fontFamily = Mono) }, singleLine = true, shape = Corner)
            Row(Modifier.fillMaxWidth().padding(vertical = 14.dp), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                OutlinedButton({ create(NoteKind.LIST) }, Modifier.weight(1f), shape = Corner) { Icon(Icons.Default.Checklist, null); Spacer(Modifier.width(7.dp)); Text("NEW LIST", fontFamily = Mono, fontSize = 11.sp) }
                Button({ create(NoteKind.MARKDOWN) }, Modifier.weight(1f), shape = Corner) { Icon(Icons.Default.Add, null); Spacer(Modifier.width(7.dp)); Text("NEW NOTE", fontFamily = Mono, fontSize = 11.sp) }
            }
            if (notes.isEmpty()) EmptyState({ create(NoteKind.MARKDOWN) }, { create(NoteKind.LIST) }) else LazyVerticalGrid(GridCells.Adaptive(270.dp), verticalArrangement = Arrangement.spacedBy(12.dp), horizontalArrangement = Arrangement.spacedBy(12.dp), contentPadding = PaddingValues(bottom = 30.dp)) {
                items(notes, key = { it.id }) { note -> NoteCard(note, { open(note) }, { delete(note.id) }) }
            }
        }
    }
}

@Composable
private fun EmptyState(note: () -> Unit, list: () -> Unit) {
    Column(Modifier.fillMaxWidth().padding(top = 18.dp).border(1.dp, MaterialTheme.colorScheme.primary.copy(alpha = .25f), Corner).padding(vertical = 54.dp, horizontal = 24.dp), horizontalAlignment = Alignment.CenterHorizontally) {
        Icon(Icons.Default.EditNote, null, tint = MaterialTheme.colorScheme.primary, modifier = Modifier.size(42.dp)); Label("BUFFER EMPTY")
        Text("Start with a blank note", Modifier.padding(top = 10.dp), fontFamily = Mono, fontWeight = FontWeight.Bold, fontSize = 18.sp)
        Text("Capture an idea in markdown, or make a list you can check off as you go.", Modifier.padding(top = 8.dp), color = MaterialTheme.colorScheme.onSurfaceVariant)
        Row(Modifier.padding(top = 20.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) { OutlinedButton(list) { Text("NEW LIST") }; Button(note) { Text("NEW NOTE") } }
    }
}

@Composable
private fun NoteCard(note: NoteRecord, open: () -> Unit, delete: () -> Unit) {
    TerminalCard(Modifier.fillMaxWidth().heightIn(min = 190.dp).clickable(onClick = open)) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Text(note.title.ifBlank { "Untitled note" }, Modifier.weight(1f), maxLines = 1, overflow = TextOverflow.Ellipsis, fontFamily = Mono, fontWeight = FontWeight.Bold, fontSize = 17.sp)
            IconButton(delete, Modifier.size(34.dp)) { Icon(Icons.Default.DeleteOutline, "Delete", tint = MaterialTheme.colorScheme.onSurfaceVariant) }
        }
        Text(if (note.kind == NoteKind.MARKDOWN) note.content.replace(Regex("[#*_`]"), "").ifBlank { "No content yet." } else "${note.items.size} items", Modifier.padding(top = 10.dp).weight(1f, false), maxLines = 3, overflow = TextOverflow.Ellipsis, color = MaterialTheme.colorScheme.onSurfaceVariant, lineHeight = 22.sp)
        Text(if (note.kind == NoteKind.MARKDOWN) "MARKDOWN" else "CHECKLIST", Modifier.padding(top = 22.dp), fontFamily = Mono, fontSize = 9.sp, letterSpacing = 1.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}
