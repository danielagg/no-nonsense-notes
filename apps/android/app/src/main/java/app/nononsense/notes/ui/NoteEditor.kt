package app.nononsense.notes.ui

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material.icons.filled.Checklist
import androidx.compose.material.icons.filled.Description
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.OffsetMapping
import androidx.compose.ui.text.input.TransformedText
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import app.nononsense.notes.core.NoteKind
import app.nononsense.notes.core.NoteRecord
import kotlinx.coroutines.delay

@Composable
internal fun NoteEditor(note: NoteRecord, dark: Boolean, toggleTheme: () -> Unit, back: () -> Unit, saveMarkdown: (NoteRecord, String, String) -> Unit, saveList: (NoteRecord, String, List<String>) -> Unit) {
    var title by remember(note.id) { mutableStateOf(note.title) }
    var content by remember(note.id) { mutableStateOf(note.content) }
    var listItems by remember(note.id) { mutableStateOf(note.items) }
    var initialized by remember(note.id) { mutableStateOf(false) }
    LaunchedEffect(title, content, listItems) {
        if (!initialized) { initialized = true; return@LaunchedEffect }
        delay(650)
        if (note.kind == NoteKind.MARKDOWN) saveMarkdown(note, title, content) else saveList(note, title, listItems)
    }
    BackHandler(onBack = back)
    Column(Modifier.fillMaxSize()) {
        Row(Modifier.fillMaxWidth().height(66.dp).padding(horizontal = 8.dp), verticalAlignment = Alignment.CenterVertically) {
            IconButton(back) { Icon(Icons.Default.ArrowBack, "Back") }; Brand(compact = true)
            Spacer(Modifier.weight(1f)); Text("SAVED LOCALLY", fontFamily = Mono, color = MaterialTheme.colorScheme.primary, fontSize = 9.sp); ThemeButton(dark, toggleTheme)
        }
        Column(Modifier.fillMaxSize().padding(horizontal = 20.dp, vertical = 12.dp)) {
            BasicTextField(title, { title = it }, Modifier.fillMaxWidth(), textStyle = TextStyle(color = MaterialTheme.colorScheme.onBackground, fontFamily = Mono, fontWeight = FontWeight.Bold, fontSize = 30.sp, letterSpacing = (-1).sp), cursorBrush = SolidColor(MaterialTheme.colorScheme.primary), singleLine = true)
            Row(Modifier.padding(top = 10.dp, bottom = 18.dp), verticalAlignment = Alignment.CenterVertically) { Icon(if (note.kind == NoteKind.MARKDOWN) Icons.Default.Description else Icons.Default.Checklist, null, tint = MaterialTheme.colorScheme.primary, modifier = Modifier.size(15.dp)); Spacer(Modifier.width(6.dp)); Label(if (note.kind == NoteKind.MARKDOWN) "MARKDOWN // LIVE" else "CHECKLIST // LIVE") }
            HorizontalDivider(color = MaterialTheme.colorScheme.primary.copy(alpha = .18f))
            if (note.kind == NoteKind.MARKDOWN) {
                BasicTextField(content, { content = it }, Modifier.fillMaxSize().padding(top = 18.dp), textStyle = TextStyle(color = MaterialTheme.colorScheme.onBackground, fontSize = 16.sp, lineHeight = 25.sp, fontFamily = FontFamily.SansSerif), cursorBrush = SolidColor(MaterialTheme.colorScheme.primary), visualTransformation = MarkdownTransformation(MaterialTheme.colorScheme.primary, MaterialTheme.colorScheme.onSurfaceVariant), decorationBox = { inner -> if (content.isEmpty()) Text("Start writing…", color = MaterialTheme.colorScheme.onSurfaceVariant) else inner() })
            } else ChecklistEditor(listItems) { listItems = it }
        }
    }
}

private class MarkdownTransformation(private val accent: androidx.compose.ui.graphics.Color, private val muted: androidx.compose.ui.graphics.Color) : VisualTransformation {
    override fun filter(text: AnnotatedString): TransformedText {
        val styled = AnnotatedString.Builder(text)
        fun apply(regex: Regex, style: SpanStyle, group: Int = 0) = regex.findAll(text.text).forEach { match -> val range = match.groups[group]?.range ?: return@forEach; styled.addStyle(style, range.first, range.last + 1) }
        apply(Regex("(?m)^#{1,6} .+$"), SpanStyle(color = accent, fontFamily = Mono, fontWeight = FontWeight.Bold))
        apply(Regex("\\*\\*([^*]+)\\*\\*"), SpanStyle(fontWeight = FontWeight.Bold), 1)
        apply(Regex("`([^`]+)`"), SpanStyle(color = accent, background = accent.copy(alpha = .08f), fontFamily = Mono), 1)
        apply(Regex("(?m)^> .+$"), SpanStyle(color = muted))
        apply(Regex("(?m)^\\|.*\\|$"), SpanStyle(fontFamily = Mono, color = accent))
        return TransformedText(styled.toAnnotatedString(), OffsetMapping.Identity)
    }
}
