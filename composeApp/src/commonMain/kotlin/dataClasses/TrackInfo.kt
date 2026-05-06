package dataClasses

data class TrackInfo(
    val trackUrl: String,
    val trackName: String,
    val trackIndex: Int,
    val isLocal: Boolean
)