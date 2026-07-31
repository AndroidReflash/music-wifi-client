//! Картинка обложки из метаданных дорожки.

use std::fs::File;
use std::path::Path;

use anyhow::{Context, Result};
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::{MediaSourceStream, MediaSourceStreamOptions};
use symphonia::core::meta::{MetadataOptions, StandardVisualKey, Visual};
use symphonia::default::get_probe;

/// Байты вложенной картинки дорожки, как они лежат в файле, — JPEG или PNG, разбирать их здесь
/// нечем и незачем.
///
/// `None` означает «картинки нет», и это обычное дело, а не беда: в сыром `.aac` метаданным негде
/// лежать вовсе, а у многих файлов обложку просто не прописали. Ошибкой считается только то, что
/// файл не открылся или контейнер не распознан.
///
/// Берётся первая картинка, помеченная как передняя обложка; если такой пометки нет ни у одной —
/// просто первая. Пометка необязательна, а вторая картинка в файле обычно задник или буклет, то
/// есть точно не то, что показывают.
///
/// Файл открывается заново, а не берётся у играющего декодера: спрашивают обложку и у дорожки,
/// которая сейчас не играет, а разбор контейнера читает только заголовок — для локального файла
/// это дёшево.
pub fn cover_of(path: &Path) -> Result<Option<Vec<u8>>> {
    let file = File::open(path).with_context(|| format!("не открыть {}", path.display()))?;
    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or_default();

    let stream = MediaSourceStream::new(Box::new(file), MediaSourceStreamOptions::default());
    let mut hint = Hint::new();
    if !extension.is_empty() {
        hint.with_extension(extension);
    }

    let mut format = get_probe()
        .probe(
            &hint,
            stream,
            FormatOptions::default(),
            MetadataOptions::default(),
        )
        .context("контейнер не распознан")?;

    let Some(revision) = format.metadata().current().cloned() else {
        return Ok(None);
    };
    let visuals = revision.media.visuals;

    let front = visuals
        .iter()
        .find(|visual| visual.usage == Some(StandardVisualKey::FrontCover));

    Ok(front
        .or_else(|| visuals.first())
        .map(|visual: &Visual| visual.data.to_vec()))
}
