package platform

import constants.PlayerConstantsFloats
import constants.PlayerConstantsLongs
import enums.HotkeyAction
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import platform.interfaces.HotkeyActions
import repository.interfaces.SettingsRepository
import repository.interfaces.SongsRepository

/**
 * Громкость идёт не в плеер, а в настройки — как у ползунка: плеер слушает `settings.volume`, и
 * ползунок сдвинется вслед за клавишей.
 */
class HotkeyActionsImpl(
    private val songs: SongsRepository,
    private val settings: SettingsRepository,
    private val scope: CoroutineScope,
) : HotkeyActions {

    override fun perform(action: HotkeyAction) {
        when (action) {
            HotkeyAction.SHUFFLE -> songs.shuffle()
            HotkeyAction.PREVIOUS -> songs.prevTrack()
            HotkeyAction.PLAY_PAUSE -> songs.playOrPause()
            HotkeyAction.NEXT -> songs.nextTrack()
            HotkeyAction.REPEAT -> songs.repeat()
            HotkeyAction.REWIND -> songs.seekBy(-PlayerConstantsLongs.SeekStepMs.value)
            HotkeyAction.FORWARD -> songs.seekBy(PlayerConstantsLongs.SeekStepMs.value)
            HotkeyAction.VOLUME_DOWN -> changeVolumeBy(-PlayerConstantsFloats.VolumeStep.value)
            HotkeyAction.VOLUME_UP -> changeVolumeBy(PlayerConstantsFloats.VolumeStep.value)
        }
    }

    private fun changeVolumeBy(delta: Float) {
        scope.launch(Dispatchers.IO) { settings.changeVolumeBy(delta) }
    }
}
