package constants

/** Величины плеера, у которых тип `Float`. */
sealed class PlayerConstantsFloats(val value: Float) {

    /** Шаг громкости горячей клавишей — доля от полной: двадцать шагов от тишины до полной. */
    data object VolumeStep : PlayerConstantsFloats(0.05f)
}
