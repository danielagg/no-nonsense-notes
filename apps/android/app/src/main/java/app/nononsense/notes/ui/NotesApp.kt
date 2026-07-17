package app.nononsense.notes.ui

import android.app.Activity
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.SideEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalView
import androidx.core.view.WindowCompat
import app.nononsense.notes.NotesViewModel
import app.nononsense.notes.ui.theme.NoNonsenseTheme

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
