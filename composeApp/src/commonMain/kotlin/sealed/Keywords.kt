package sealed

sealed class Keywords(val value: String) {
    data object ConfigEndpoint: Keywords("/config")
    data object SSESub: Keywords("/sse/subscribe")
    data object Refresh: Keywords("refresh")
}