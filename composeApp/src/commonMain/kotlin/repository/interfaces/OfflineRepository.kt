package repository.interfaces

import dataClasses.TrackInfo

interface OfflineRepository {
    suspend fun offlineSongList(): List<TrackInfo>
}