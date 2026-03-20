use windows::Win32::Foundation::{HANDLE, CloseHandle, ERROR_ALREADY_EXISTS, GetLastError};
use windows::Win32::System::Threading::{CreateMutexW, ReleaseMutex};
use windows::core::w;

pub struct SingleInstance {
    handle: HANDLE,
}

impl SingleInstance {
    /// Named Mutex를 생성하여 단일 인스턴스 확인.
    /// 이미 실행 중이면 Err 반환.
    pub fn acquire() -> Result<Self, ()> {
        unsafe {
            let handle = CreateMutexW(
                None,
                true,
                w!("Global\\TextSniperWin_SingleInstance"),
            ).map_err(|_| ())?;

            let last_error = GetLastError();
            if last_error == ERROR_ALREADY_EXISTS {
                let _ = CloseHandle(handle);
                return Err(());
            }

            Ok(Self { handle })
        }
    }
}

impl Drop for SingleInstance {
    fn drop(&mut self) {
        unsafe {
            let _ = ReleaseMutex(self.handle);
            let _ = CloseHandle(self.handle);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acquire_succeeds_first_time() {
        let instance = SingleInstance::acquire();
        assert!(instance.is_ok());
        // drop releases mutex
    }

    #[test]
    fn test_acquire_fails_second_time() {
        let _first = SingleInstance::acquire().unwrap();
        let second = SingleInstance::acquire();
        assert!(second.is_err());
    }
}
