package repository.interfaces

import dataClasses.TrackInfo
import kotlinx.coroutines.flow.StateFlow

interface SongsRepository {
    fun play(trackInfo: TrackInfo)
    fun nextTrack()
    fun prevTrack()
    fun seekTo(percentage: Float)
    fun shuffle()
    fun repeat()
    val isPlaying : StateFlow<Boolean>
    val currentPlay : StateFlow<TrackInfo?>
    val equalizerArray : StateFlow<List<Double>>

    val currentTrackLength: StateFlow<Long>
    val currentPositionMs: StateFlow<Long>
    val shuffle: StateFlow<Boolean>
    val repeat: StateFlow<Boolean>
    val songListOffline: StateFlow<List<TrackInfo>>
    val songListOnline: StateFlow<List<TrackInfo>>
    val isLocal : StateFlow<Boolean>
    fun onTrackSelectedManually(index: Int)
}