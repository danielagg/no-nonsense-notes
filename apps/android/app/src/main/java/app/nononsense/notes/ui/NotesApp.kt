package app.nononsense.notes.ui

import android.app.Activity
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.luminance
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.OffsetMapping
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.TransformedText
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.core.view.WindowCompat
import app.nononsense.notes.NotesViewModel
import app.nononsense.notes.core.NoteKind
import app.nononsense.notes.core.NoteRecord
import app.nononsense.notes.core.SyncStatus
import app.nononsense.notes.ui.theme.Neon
import app.nononsense.notes.ui.theme.NoNonsenseTheme
import kotlinx.coroutines.delay

private val Mono = FontFamily.Monospace
private val Corner = RoundedCornerShape(6.dp)

@Composable
fun NotesApp(viewModel: NotesViewModel) {
    val state by viewModel.state.collectAsState()
    val systemDark = isSystemInDarkTheme()
    var dark by rememberSaveable { mutableStateOf(systemDark) }
    val view = LocalView.current
    if (!view.isInEditMode) {
        SideEffect {
            WindowCompat.getInsetsController((view.context as Activity).window, view).apply {
                isAppearanceLightStatusBars = !dark
                isAppearanceLightNavigationBars = !dark
            }
        }
    }
    NoNonsenseTheme(dark) {
        Surface(
            modifier = Modifier.fillMaxSize(),
            color = MaterialTheme.colorScheme.background,
            contentColor = MaterialTheme.colorScheme.onBackground,
        ) {
            TerminalBackground {
                Box(Modifier.fillMaxSize().statusBarsPadding().navigationBarsPadding()) {
                    when {
                        !state.authenticated -> AuthScreen(state.loading, state.authError, dark, { dark = !dark }, viewModel::authenticate)
                        state.selected != null -> NoteEditor(state.selected!!, dark, { dark = !dark }, { viewModel.select(null) }, viewModel::saveMarkdown, viewModel::saveList)
                        else -> NotesScreen(state.notes, state.query, state.syncStatus, dark, { dark = !dark }, viewModel::setQuery, viewModel::select, viewModel::create, viewModel::delete, viewModel::logout)
                    }
                    if (state.authenticated) {
                        SyncBanner(state.syncStatus, Modifier.align(Alignment.TopCenter))
                    }
                }
            }
        }
    }
}

@Composable
private fun TerminalBackground(content: @Composable BoxScope.() -> Unit) {
    val background = MaterialTheme.colorScheme.background
    val line = MaterialTheme.colorScheme.primary.copy(alpha = .035f)
    Box(Modifier.fillMaxSize().background(background)) {
        Canvas(Modifier.matchParentSize()) {
            val step = 32.dp.toPx()
            var x = 0f
            while (x < size.width) { drawLine(line, Offset(x, 0f), Offset(x, size.height)); x += step }
            var y = 0f
            while (y < size.height) { drawLine(line, Offset(0f, y), Offset(size.width, y)); y += step }
            drawRect(Brush.radialGradient(listOf(Color.Transparent, Color.Black.copy(alpha = .32f)), center = center, radius = size.maxDimension * .72f))
        }
        content()
    }
}

@Composable
private fun Brand(compact: Boolean = false) {
    val isDark = MaterialTheme.colorScheme.background.luminance() < .5f
    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
        Box(Modifier.size(36.dp).clip(Corner).background(if (isDark) Color(0xFF071009) else MaterialTheme.colorScheme.primary).border(1.dp, MaterialTheme.colorScheme.primary.copy(alpha = .7f), Corner), contentAlignment = Alignment.Center) {
            Icon(Icons.Default.Description, null, tint = if (isDark) Neon else MaterialTheme.colorScheme.onPrimary, modifier = Modifier.size(18.dp))
        }
        if (!compact) Text("NO NONSENSE / NOTES", fontFamily = Mono, fontWeight = FontWeight.Bold, fontSize = 13.sp, letterSpacing = 1.sp)
    }
}

@Composable
private fun AuthScreen(loading: Boolean, error: String?, dark: Boolean, toggleTheme: () -> Unit, authenticate: (Boolean, String, String) -> Unit) {
    var create by rememberSaveable { mutableStateOf(false) }
    var email by rememberSaveable { mutableStateOf("") }
    var password by rememberSaveable { mutableStateOf("") }
    LazyColumn(Modifier.fillMaxSize().padding(horizontal = 20.dp), contentPadding = PaddingValues(bottom = 40.dp)) {
        item {
            Row(Modifier.fillMaxWidth().height(72.dp), verticalAlignment = Alignment.CenterVertically) {
                Brand(); Spacer(Modifier.weight(1f)); ThemeButton(dark, toggleTheme)
            }
            Column(Modifier.padding(top = 28.dp, bottom = 32.dp)) {
                Label("◉  LOCAL FIRST. END-TO-END ENCRYPTED. FAST BY DESIGN.")
                Spacer(Modifier.height(26.dp))
                Text("Just notes.", fontFamily = Mono, fontSize = 42.sp, fontWeight = FontWeight.Bold, letterSpacing = (-2).sp)
                Text("Fast. Local. Yours.", color = MaterialTheme.colorScheme.primary, fontFamily = Mono, fontSize = 36.sp, fontWeight = FontWeight.Bold, letterSpacing = (-2).sp)
                Text("No wikis, workflows, or AI bolted on. Just notes and lists, built around local data and kept deliberately small.", Modifier.padding(top = 20.dp).border(BorderStroke(0.dp, Color.Transparent)).padding(start = 12.dp), color = MaterialTheme.colorScheme.onSurfaceVariant, lineHeight = 24.sp)
            }
            TerminalCard(Modifier.fillMaxWidth()) {
                Label("AUTH // WORKSPACE ACCESS")
                Text("Identify yourself", Modifier.padding(top = 14.dp), fontFamily = Mono, fontSize = 25.sp, fontWeight = FontWeight.Bold)
                Text("Sign in or initialize a new account.", Modifier.padding(top = 6.dp), color = MaterialTheme.colorScheme.onSurfaceVariant)
                Row(Modifier.fillMaxWidth().padding(top = 22.dp).background(MaterialTheme.colorScheme.surfaceVariant, Corner)) {
                    AuthTab("Sign in", !create, Modifier.weight(1f)) { create = false }
                    AuthTab("Create account", create, Modifier.weight(1f)) { create = true }
                }
                Spacer(Modifier.height(22.dp))
                FormField("EMAIL ADDRESS", email, { email = it }, "you@company.com", false)
                Spacer(Modifier.height(16.dp))
                FormField("PASSWORD", password, { password = it }, if (create) "Choose a secure password" else "Enter your password", true)
                if (error != null) Text(error, Modifier.fillMaxWidth().padding(top = 14.dp).background(MaterialTheme.colorScheme.error.copy(alpha = .1f), Corner).border(1.dp, MaterialTheme.colorScheme.error.copy(alpha = .35f), Corner).padding(12.dp), color = MaterialTheme.colorScheme.error, fontFamily = Mono, fontSize = 12.sp)
                Button(onClick = { authenticate(create, email, password) }, enabled = !loading, modifier = Modifier.fillMaxWidth().padding(top = 20.dp).height(48.dp), shape = Corner) {
                    Text(if (loading) "WORKING…" else if (create) "CREATE ACCOUNT  →" else "SIGN IN  →", fontFamily = Mono, fontWeight = FontWeight.Bold)
                }
                Text("ENCRYPTED TRANSPORT // LOCAL-FIRST DATA", Modifier.align(Alignment.CenterHorizontally).padding(top = 18.dp), color = MaterialTheme.colorScheme.onSurfaceVariant, fontFamily = Mono, fontSize = 9.sp, letterSpacing = 1.sp)
            }
        }
    }
}

@Composable private fun AuthTab(label: String, selected: Boolean, modifier: Modifier, onClick: () -> Unit) {
    TextButton(onClick, modifier, colors = ButtonDefaults.textButtonColors(containerColor = if (selected) MaterialTheme.colorScheme.background else Color.Transparent), shape = Corner) { Text(label, fontFamily = Mono, fontSize = 12.sp) }
}

@Composable private fun FormField(label: String, value: String, change: (String) -> Unit, placeholder: String, password: Boolean) {
    Column {
        Text(label, fontFamily = Mono, fontWeight = FontWeight.Bold, fontSize = 10.sp, letterSpacing = 1.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
        OutlinedTextField(value, change, Modifier.fillMaxWidth().padding(top = 7.dp), placeholder = { Text(placeholder, fontFamily = Mono) }, singleLine = true, shape = Corner, textStyle = TextStyle(fontFamily = Mono), visualTransformation = if (password) PasswordVisualTransformation() else VisualTransformation.None, keyboardOptions = KeyboardOptions(keyboardType = if (password) KeyboardType.Password else KeyboardType.Email))
    }
}

@Composable
private fun NotesScreen(notes: List<NoteRecord>, query: String, syncStatus: SyncStatus, dark: Boolean, toggleTheme: () -> Unit, setQuery: (String) -> Unit, open: (NoteRecord) -> Unit, create: (NoteKind) -> Unit, delete: (String) -> Unit, logout: () -> Unit) {
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

@Composable private fun EmptyState(note: () -> Unit, list: () -> Unit) {
    Column(Modifier.fillMaxWidth().padding(top = 18.dp).border(1.dp, MaterialTheme.colorScheme.primary.copy(alpha = .25f), Corner).padding(vertical = 54.dp, horizontal = 24.dp), horizontalAlignment = Alignment.CenterHorizontally) {
        Icon(Icons.Default.EditNote, null, tint = MaterialTheme.colorScheme.primary, modifier = Modifier.size(42.dp)); Label("BUFFER EMPTY")
        Text("Start with a blank note", Modifier.padding(top = 10.dp), fontFamily = Mono, fontWeight = FontWeight.Bold, fontSize = 18.sp)
        Text("Capture an idea in markdown, or make a list you can check off as you go.", Modifier.padding(top = 8.dp), color = MaterialTheme.colorScheme.onSurfaceVariant)
        Row(Modifier.padding(top = 20.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) { OutlinedButton(list) { Text("NEW LIST") }; Button(note) { Text("NEW NOTE") } }
    }
}

@Composable private fun NoteCard(note: NoteRecord, open: () -> Unit, delete: () -> Unit) {
    TerminalCard(Modifier.fillMaxWidth().heightIn(min = 190.dp).clickable(onClick = open)) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Text(note.title.ifBlank { "Untitled note" }, Modifier.weight(1f), maxLines = 1, overflow = TextOverflow.Ellipsis, fontFamily = Mono, fontWeight = FontWeight.Bold, fontSize = 17.sp)
            IconButton(delete, Modifier.size(34.dp)) { Icon(Icons.Default.DeleteOutline, "Delete", tint = MaterialTheme.colorScheme.onSurfaceVariant) }
        }
        Text(if (note.kind == NoteKind.MARKDOWN) note.content.replace(Regex("[#*_`]"), "").ifBlank { "No content yet." } else "${note.items.size} items", Modifier.padding(top = 10.dp).weight(1f, false), maxLines = 3, overflow = TextOverflow.Ellipsis, color = MaterialTheme.colorScheme.onSurfaceVariant, lineHeight = 22.sp)
        Text(if (note.kind == NoteKind.MARKDOWN) "MARKDOWN" else "CHECKLIST", Modifier.padding(top = 22.dp), fontFamily = Mono, fontSize = 9.sp, letterSpacing = 1.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}

@Composable
private fun NoteEditor(note: NoteRecord, dark: Boolean, toggleTheme: () -> Unit, back: () -> Unit, saveMarkdown: (NoteRecord, String, String) -> Unit, saveList: (NoteRecord, String, List<String>) -> Unit) {
    var title by remember(note.id) { mutableStateOf(note.title) }
    var content by remember(note.id) { mutableStateOf(note.content) }
    var listItems by remember(note.id) { mutableStateOf(note.items) }
    var initialized by remember(note.id) { mutableStateOf(false) }
    LaunchedEffect(title, content, listItems) {
        if (!initialized) { initialized = true; return@LaunchedEffect }
        delay(650)
        if (note.kind == NoteKind.MARKDOWN) saveMarkdown(note, title, content) else saveList(note, title, listItems)
    }
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

@Composable private fun ChecklistEditor(items: List<String>, change: (List<String>) -> Unit) {
    var focusNewItemAt by remember { mutableStateOf<Int?>(null) }
    LazyColumn(Modifier.fillMaxSize().padding(top = 12.dp), contentPadding = PaddingValues(bottom = 80.dp)) {
        itemsIndexed(items) { index, raw ->
            val checked = raw.startsWith("[x] ")
            val text = raw.removePrefix("[x] ").removePrefix("[ ] ")
            val focusRequester = remember { FocusRequester() }
            val insertItemAfter = {
                change(items.toMutableList().also { it.add(index + 1, "[ ] ") })
                focusNewItemAt = index + 1
            }
            LaunchedEffect(focusNewItemAt == index) {
                if (focusNewItemAt == index) {
                    focusRequester.requestFocus()
                    focusNewItemAt = null
                }
            }
            Row(Modifier.fillMaxWidth().padding(vertical = 5.dp), verticalAlignment = Alignment.CenterVertically) {
                Checkbox(checked, { value -> change(items.toMutableList().also { it[index] = (if (value) "[x] " else "[ ] ") + text }) })
                BasicTextField(
                    text,
                    { value -> change(items.toMutableList().also { it[index] = (if (checked) "[x] " else "[ ] ") + value }) },
                    Modifier.weight(1f).padding(8.dp).focusRequester(focusRequester).onPreviewKeyEvent { event ->
                        if (event.type == KeyEventType.KeyDown && event.key == Key.Enter) {
                            insertItemAfter()
                            true
                        } else false
                    },
                    textStyle = TextStyle(color = MaterialTheme.colorScheme.onBackground, fontSize = 16.sp),
                    cursorBrush = SolidColor(MaterialTheme.colorScheme.primary),
                    singleLine = true,
                    keyboardOptions = KeyboardOptions(imeAction = ImeAction.Next),
                    keyboardActions = KeyboardActions(onNext = { insertItemAfter() }),
                )
                IconButton({ if (index > 0) change(items.toMutableList().also { list -> val item = list.removeAt(index); list.add(index - 1, item) }) }, enabled = index > 0, modifier = Modifier.size(32.dp)) { Icon(Icons.Default.KeyboardArrowUp, "Move up") }
                IconButton({ change(items.toMutableList().also { list -> list.removeAt(index) }) }, modifier = Modifier.size(32.dp)) { Icon(Icons.Default.Close, "Remove") }
            }
        }
        item { OutlinedButton({ change(items + "[ ] ") }, Modifier.padding(top = 12.dp), shape = Corner) { Icon(Icons.Default.Add, null); Text("ADD ITEM", fontFamily = Mono, fontSize = 11.sp) } }
    }
}

private class MarkdownTransformation(private val accent: Color, private val muted: Color) : VisualTransformation {
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

@Composable private fun TerminalCard(modifier: Modifier = Modifier, content: @Composable ColumnScope.() -> Unit) {
    Card(modifier, shape = Corner, colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface.copy(alpha = .91f), contentColor = MaterialTheme.colorScheme.onSurface), border = BorderStroke(1.dp, MaterialTheme.colorScheme.primary.copy(alpha = .2f))) { Column(Modifier.padding(24.dp), content = content) }
}
@Composable private fun Label(text: String) { Text(text, color = MaterialTheme.colorScheme.primary, fontFamily = Mono, fontWeight = FontWeight.Bold, fontSize = 10.sp, letterSpacing = 1.sp) }
@Composable private fun ThemeButton(dark: Boolean, toggle: () -> Unit) { IconButton(toggle) { Icon(if (dark) Icons.Default.LightMode else Icons.Default.DarkMode, if (dark) "Light mode" else "Dark mode") } }

@Composable private fun SyncBanner(status: SyncStatus, modifier: Modifier = Modifier) {
    if (status == SyncStatus.CONNECTED) return
    val (label, color) = when (status) {
        SyncStatus.CONNECTING -> "CONNECTING TO SYNC…" to MaterialTheme.colorScheme.primary
        SyncStatus.ERROR -> "SYNC ISSUE — CHANGES ARE SAFE LOCALLY" to MaterialTheme.colorScheme.error
        else -> "YOU'RE OFFLINE — CHANGES ARE SAVED LOCALLY" to Color(0xFFFFC65C)
    }
    Surface(modifier.fillMaxWidth().height(40.dp), color = MaterialTheme.colorScheme.background.copy(alpha = .98f), border = BorderStroke(1.dp, color.copy(alpha = .4f))) { Box(contentAlignment = Alignment.Center) { Text(label, color = color, fontFamily = Mono, fontWeight = FontWeight.Bold, fontSize = 9.sp, letterSpacing = .7.sp) } }
}
