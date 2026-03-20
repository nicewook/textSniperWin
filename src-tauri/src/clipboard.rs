use arboard::Clipboard;

/// 텍스트를 클립보드에 복사. 빈 문자열이면 복사하지 않고 Err 반환.
pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err("Empty text, clipboard not modified".to_string());
    }

    let mut cb = Clipboard::new().map_err(|e| format!("Clipboard open failed: {}", e))?;
    cb.set_text(trimmed).map_err(|e| format!("Clipboard write failed: {}", e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_nonempty_text() {
        let result = copy_to_clipboard("hello OCR");
        assert!(result.is_ok());

        // 실제로 클립보드에 복사되었는지 확인
        let mut cb = Clipboard::new().unwrap();
        let content = cb.get_text().unwrap();
        assert_eq!(content, "hello OCR");
    }

    #[test]
    fn test_copy_empty_text_rejected() {
        let result = copy_to_clipboard("");
        assert!(result.is_err());
    }

    #[test]
    fn test_copy_whitespace_only_rejected() {
        let result = copy_to_clipboard("   \n\t  ");
        assert!(result.is_err());
    }
}
