package keywords

sealed class Keywords (val value: String) {
    data object PlayerId: Keywords("player")
    data object PlayerNextItem: Keywords("next_item")
    data object PlayerPrevItem: Keywords("prev_item")
    data object PlayerPrevURL: Keywords("emptyURLPrev")
    data object PlayerNextURL: Keywords("emptyURLNext")
    data object DefaultAction: Keywords("default_action")
}