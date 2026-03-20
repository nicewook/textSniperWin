use windows::Globalization::Language;
use windows::Graphics::Imaging::{BitmapPixelFormat, SoftwareBitmap};
use windows::Media::Ocr::OcrEngine;
use windows::Storage::Streams::{DataWriter, InMemoryRandomAccessStream};
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Threading::{CreateEventW, SetEvent, WaitForSingleObject, INFINITE};

/// 사용 가능한 OCR 언어 확인 (prefix 매칭: "en"은 "en-US" 등과 매칭)
pub fn is_language_available(lang_tag: &str) -> bool {
    find_matching_tag(lang_tag).is_some()
}

/// 사용 가능한 언어 목록 반환
pub fn available_languages() -> Vec<String> {
    OcrEngine::AvailableRecognizerLanguages()
        .map(|langs| {
            langs
                .into_iter()
                .filter_map(|l| l.LanguageTag().ok().map(|t| t.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

/// prefix 매칭으로 실제 사용 가능한 전체 언어 태그를 반환
fn find_matching_tag(lang_tag: &str) -> Option<String> {
    let lower = lang_tag.to_ascii_lowercase();
    available_languages().into_iter().find(|t| {
        let tl = t.to_ascii_lowercase();
        tl == lower || tl.starts_with(&format!("{}-", lower))
    })
}

/// RGBA 바이트 배열로 SoftwareBitmap 생성
pub fn create_bitmap_from_rgba(
    data: &[u8],
    width: u32,
    height: u32,
) -> Result<SoftwareBitmap, String> {
    let expected_len = (width * height * 4) as usize;
    if data.len() != expected_len {
        return Err(format!(
            "Data length mismatch: expected {}, got {}",
            expected_len,
            data.len()
        ));
    }

    // RGBA -> BGRA 변환 (SoftwareBitmap은 BGRA 사용)
    let mut bgra = data.to_vec();
    for chunk in bgra.chunks_exact_mut(4) {
        chunk.swap(0, 2);
    }

    // DataWriter를 통해 바이트를 IBuffer에 기록한 뒤 CopyFromBuffer 호출
    let stream =
        InMemoryRandomAccessStream::new().map_err(|e| e.message().to_string())?;
    let writer =
        DataWriter::CreateDataWriter(&stream).map_err(|e| e.message().to_string())?;
    writer
        .WriteBytes(&bgra)
        .map_err(|e| e.message().to_string())?;
    let buffer = writer
        .DetachBuffer()
        .map_err(|e| e.message().to_string())?;

    let bitmap = SoftwareBitmap::Create(BitmapPixelFormat::Bgra8, width as i32, height as i32)
        .map_err(|e| e.message().to_string())?;

    bitmap
        .CopyFromBuffer(&buffer)
        .map_err(|e| e.message().to_string())?;

    Ok(bitmap)
}

fn wait_for_async_operation<T: windows::core::RuntimeType + 'static>(
    async_op: &windows_future::IAsyncOperation<T>,
) -> Result<T, String> {
    unsafe {
        let event =
            CreateEventW(None, true, false, None).map_err(|e| e.message().to_string())?;

        let event_ptr = event.0 as isize;
        async_op
            .SetCompleted(&windows_future::AsyncOperationCompletedHandler::new(
                move |_, _| {
                    let _ = SetEvent(windows::Win32::Foundation::HANDLE(event_ptr as *mut _));
                    Ok(())
                },
            ))
            .map_err(|e| e.message().to_string())?;

        let _ = WaitForSingleObject(event, INFINITE);
        let _ = CloseHandle(event);
    }

    async_op
        .GetResults()
        .map_err(|e| e.message().to_string())
}

fn recognize_with_language(bitmap: &SoftwareBitmap, lang_tag: &str) -> Result<String, String> {
    let hstring: windows::core::HSTRING = lang_tag.into();
    let lang =
        Language::CreateLanguage(&hstring).map_err(|e| e.message().to_string())?;

    let engine =
        OcrEngine::TryCreateFromLanguage(&lang).map_err(|e| e.message().to_string())?;

    let async_op = engine
        .RecognizeAsync(bitmap)
        .map_err(|e| e.message().to_string())?;

    let result: windows::Media::Ocr::OcrResult = wait_for_async_operation(&async_op)?;

    let text = result
        .Text()
        .map_err(|e| e.message().to_string())?;
    Ok(text.to_string())
}

/// SoftwareBitmap에서 OCR 수행.
/// 영어 시도 -> 결과 없으면 한국어 시도.
pub fn recognize_text(bitmap: &SoftwareBitmap) -> Result<String, String> {
    // 영어 먼저 시도
    if let Some(en_tag) = find_matching_tag("en") {
        if let Ok(text) = recognize_with_language(bitmap, &en_tag) {
            let trimmed = text.trim().to_string();
            if !trimmed.is_empty() {
                return Ok(trimmed);
            }
        }
    }

    // 한국어 폴백
    if let Some(ko_tag) = find_matching_tag("ko") {
        if let Ok(text) = recognize_with_language(bitmap, &ko_tag) {
            let trimmed = text.trim().to_string();
            if !trimmed.is_empty() {
                return Ok(trimmed);
            }
        }
    }

    Err("No text recognized".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_installed_language_available() {
        let langs = available_languages();
        // 최소 하나의 언어에 대해 is_language_available이 true를 반환해야 함
        assert!(
            langs.iter().any(|tag| is_language_available(tag)),
            "No OCR language detected as available. Languages: {:?}",
            langs
        );
    }

    #[test]
    fn test_korean_language_available() {
        // 이 시스템에는 한국어 OCR이 설치되어 있음
        assert!(is_language_available("ko"));
    }

    #[test]
    fn test_available_languages_not_empty() {
        let langs = available_languages();
        assert!(!langs.is_empty());
    }

    #[test]
    fn test_create_bitmap_from_rgba_correct_size() {
        let width = 100;
        let height = 50;
        let data = vec![255u8; (width * height * 4) as usize];
        let result = create_bitmap_from_rgba(&data, width, height);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_bitmap_from_rgba_wrong_size() {
        let data = vec![0u8; 100]; // 너무 작음
        let result = create_bitmap_from_rgba(&data, 100, 100);
        assert!(result.is_err());
    }
}
