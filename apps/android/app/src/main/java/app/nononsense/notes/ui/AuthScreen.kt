package app.nononsense.notes.ui

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

@Composable
internal fun AuthScreen(loading: Boolean, error: String?, dark: Boolean, toggleTheme: () -> Unit, authenticate: (Boolean, String, String) -> Unit) {
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

@Composable
private fun AuthTab(label: String, selected: Boolean, modifier: Modifier, onClick: () -> Unit) {
    TextButton(onClick, modifier, colors = ButtonDefaults.textButtonColors(containerColor = if (selected) MaterialTheme.colorScheme.background else Color.Transparent), shape = Corner) { Text(label, fontFamily = Mono, fontSize = 12.sp) }
}

@Composable
private fun FormField(label: String, value: String, change: (String) -> Unit, placeholder: String, password: Boolean) {
    Column {
        Text(label, fontFamily = Mono, fontWeight = FontWeight.Bold, fontSize = 10.sp, letterSpacing = 1.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
        OutlinedTextField(value, change, Modifier.fillMaxWidth().padding(top = 7.dp), placeholder = { Text(placeholder, fontFamily = Mono) }, singleLine = true, shape = Corner, textStyle = TextStyle(fontFamily = Mono), visualTransformation = if (password) PasswordVisualTransformation() else VisualTransformation.None, keyboardOptions = androidx.compose.foundation.text.KeyboardOptions(keyboardType = if (password) KeyboardType.Password else KeyboardType.Email))
    }
}
