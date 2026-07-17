package app.nononsense.notes.ui

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.KeyboardArrowUp
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.input.key.*
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

@Composable
internal fun ChecklistEditor(items: List<String>, change: (List<String>) -> Unit) {
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
