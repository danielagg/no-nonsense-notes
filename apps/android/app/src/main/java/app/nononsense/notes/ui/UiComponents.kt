package app.nononsense.notes.ui

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.DarkMode
import androidx.compose.material.icons.filled.Description
import androidx.compose.material.icons.filled.LightMode
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.luminance
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import app.nononsense.notes.core.SyncStatus
import app.nononsense.notes.ui.theme.Neon

internal val Mono = FontFamily.Monospace
internal val Corner = RoundedCornerShape(6.dp)

@Composable
internal fun TerminalBackground(content: @Composable BoxScope.() -> Unit) {
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
internal fun Brand(compact: Boolean = false) {
    val isDark = MaterialTheme.colorScheme.background.luminance() < .5f
    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
        Box(Modifier.size(36.dp).clip(Corner).background(if (isDark) Color(0xFF071009) else MaterialTheme.colorScheme.primary).border(1.dp, MaterialTheme.colorScheme.primary.copy(alpha = .7f), Corner), contentAlignment = Alignment.Center) {
            Icon(Icons.Default.Description, null, tint = if (isDark) Neon else MaterialTheme.colorScheme.onPrimary, modifier = Modifier.size(18.dp))
        }
        if (!compact) Text("NO NONSENSE / NOTES", fontFamily = Mono, fontWeight = FontWeight.Bold, fontSize = 13.sp, letterSpacing = 1.sp)
    }
}

@Composable
internal fun TerminalCard(modifier: Modifier = Modifier, content: @Composable ColumnScope.() -> Unit) {
    Card(modifier, shape = Corner, colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface.copy(alpha = .91f), contentColor = MaterialTheme.colorScheme.onSurface), border = BorderStroke(1.dp, MaterialTheme.colorScheme.primary.copy(alpha = .2f))) { Column(Modifier.padding(24.dp), content = content) }
}

@Composable
internal fun Label(text: String) { Text(text, color = MaterialTheme.colorScheme.primary, fontFamily = Mono, fontWeight = FontWeight.Bold, fontSize = 10.sp, letterSpacing = 1.sp) }

@Composable
internal fun ThemeButton(dark: Boolean, toggle: () -> Unit) { IconButton(toggle) { Icon(if (dark) Icons.Default.LightMode else Icons.Default.DarkMode, if (dark) "Light mode" else "Dark mode") } }

@Composable
internal fun SyncBanner(status: SyncStatus, modifier: Modifier = Modifier) {
    if (status == SyncStatus.CONNECTED) return
    val (label, color) = when (status) {
        SyncStatus.CONNECTING -> "CONNECTING TO SYNC…" to MaterialTheme.colorScheme.primary
        SyncStatus.ERROR -> "SYNC ISSUE — CHANGES ARE SAFE LOCALLY" to MaterialTheme.colorScheme.error
        else -> "YOU'RE OFFLINE — CHANGES ARE SAVED LOCALLY" to Color(0xFFFFC65C)
    }
    Surface(modifier.fillMaxWidth().height(40.dp), color = MaterialTheme.colorScheme.background.copy(alpha = .98f), border = BorderStroke(1.dp, color.copy(alpha = .4f))) { Box(contentAlignment = Alignment.Center) { Text(label, color = color, fontFamily = Mono, fontWeight = FontWeight.Bold, fontSize = 9.sp, letterSpacing = .7.sp) } }
}
