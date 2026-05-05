package repository.interfaces

import dataClasses.TrackInfo
import kotlinx.coroutines.flow.StateFlow

interface SongsRepository {
    fun play(trackInfo: TrackInfo)
    fun nextTrack()
    fun prevTrack()
    fun startSongService()
    fun seekTo(percentage: Float)
    fun shuffle()
    fun repeat()
    val songList: StateFlow<List<TrackInfo>>
    val isPlaying : StateFlow<Boolean>
    val currentPlay : StateFlow<TrackInfo?>
    val equalizerArray : StateFlow<List<Double>>

    val currentTrackLength: StateFlow<Long>
    val currentPositionMs: StateFlow<Long>
    val shuffle: StateFlow<Boolean>
    val repeat: StateFlow<Boolean>
}