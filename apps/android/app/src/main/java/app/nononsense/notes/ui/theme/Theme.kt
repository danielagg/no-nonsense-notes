package app.nononsense.notes.ui.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

val Neon = Color(0xFF8CFF66)
val Ink = Color(0xFF071009)
val TerminalCard = Color(0xFF0D170F)
val TerminalText = Color(0xFFD1DDCE)
val MutedText = Color(0xFF8A9987)
val TerminalBorder = Color(0xFF31442E)

private val DarkColors = darkColorScheme(
    primary = Neon, onPrimary = Ink, background = Ink, onBackground = TerminalText,
    surface = TerminalCard, onSurface = TerminalText, surfaceVariant = Color(0xFF142118),
    onSurfaceVariant = MutedText, outline = TerminalBorder, error = Color(0xFFFF7A72),
)
private val LightColors = lightColorScheme(
    primary = Color(0xFF6C6A20), onPrimary = Color(0xFFFFF9DF), background = Color(0xFFF2EBCD),
    onBackground = Color(0xFF3B3528), surface = Color(0xFFFFF8DF), onSurface = Color(0xFF3B3528),
    surfaceVariant = Color(0xFFE4D9B9), onSurfaceVariant = Color(0xFF6A604C), outline = Color(0xFFA89A72),
)

@Composable
fun NoNonsenseTheme(dark: Boolean = isSystemInDarkTheme(), content: @Composable () -> Unit) {
    MaterialTheme(colorScheme = if (dark) DarkColors else LightColors, content = content)
}

