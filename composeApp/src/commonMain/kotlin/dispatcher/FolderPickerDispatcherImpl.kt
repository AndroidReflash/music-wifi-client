package dispatcher

import dispatchers.FolderPickerDispatcher
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow

class FolderPickerDispatcherImpl: FolderPickerDispatcher {
    private val _events = MutableSharedFlow<Unit>(extraBufferCapacity = 1)
    override val events = _events.asSharedFlow()

    override fun requestFolder() {
        _events.tryEmit(Unit)
    }
}