package dispatchers

import kotlinx.coroutines.flow.SharedFlow

interface FolderPickerDispatcher {
    fun requestFolder()
    val events: SharedFlow<Unit>
}