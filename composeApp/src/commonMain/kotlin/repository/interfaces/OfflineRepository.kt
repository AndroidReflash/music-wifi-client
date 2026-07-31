package repository.interfaces

import dataClasses.TrackInfo
import kotlinx.coroutines.flow.StateFlow

interface OfflineRepository {
    val offlineSongsFlow: StateFlow<List<TrackInfo>>
}