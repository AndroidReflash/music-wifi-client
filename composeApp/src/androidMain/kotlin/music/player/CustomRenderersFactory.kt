package music.player

import android.content.Context
import android.os.Handler
import androidx.media3.common.audio.AudioProcessor
import androidx.media3.common.util.UnstableApi
import androidx.media3.exoplayer.DefaultRenderersFactory
import androidx.media3.exoplayer.Renderer
import androidx.media3.exoplayer.audio.AudioRendererEventListener
import androidx.media3.exoplayer.audio.DefaultAudioSink
import androidx.media3.exoplayer.audio.MediaCodecAudioRenderer
import androidx.media3.exoplayer.mediacodec.MediaCodecSelector
import androidx.media3.exoplayer.metadata.MetadataOutput
import androidx.media3.exoplayer.text.TextOutput
import androidx.media3.exoplayer.video.VideoRendererEventListener


//нельзя просто так взять и засунуть аудиопроцессор в exoPlayer, нужна "обвязка"
//данный класс явялется этой самой "обвязкой"
@UnstableApi
class CustomRenderersFactory(
    private val context: Context,
    private val audioProcessor: AudioProcessor
) : DefaultRenderersFactory(context) {

    override fun createRenderers(
        eventHandler: Handler,
        videoRendererEventListener: VideoRendererEventListener,
        audioRendererEventListener: AudioRendererEventListener,
        textOutput: TextOutput,
        metadataOutput: MetadataOutput
    ): Array<Renderer> {
        val audioSink = DefaultAudioSink.Builder()
            .setAudioProcessors(arrayOf(audioProcessor))
            .build()

        val mediaCodecSelector = MediaCodecSelector.DEFAULT
        val renderers = mutableListOf<Renderer>()

        renderers.add(
            MediaCodecAudioRenderer(
                context,
                mediaCodecSelector,
                true,
                eventHandler,
                audioRendererEventListener,
                audioSink
            )
        )
        return renderers.toTypedArray()
    }
}