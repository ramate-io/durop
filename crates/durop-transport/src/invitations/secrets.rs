//! Native host credential storage; secrets never travel through a process argument.

use super::{AdminApiKey, InvitationError};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
};

/// Interchangeable host administration credential storage. Implementations must
/// protect plaintext from other OS users and never log the value. Methods are
/// synchronous and belong on a worker, since native keychain access can prompt.
pub trait SecretStore: Send + Sync {
    /// Return the saved credential, or None when none exists.
    fn load(&self) -> Result<Option<AdminApiKey>, InvitationError>;

    /// Replace the saved credential; the caller retains ownership of its input.
    fn save(&self, key: &AdminApiKey) -> Result<(), InvitationError>;

    /// Remove the saved credential; missing entries are already successfully deleted.
    fn delete(&self) -> Result<(), InvitationError>;
}

/// Prefer the macOS login Keychain via native Security APIs. If the keychain is
/// unavailable, use the explicit fallback path with a private Unix directory and
/// mode-0600 file. Other Unix platforms use that same protected-file contract.
/// Unsupported platforms fail closed rather than saving unprotected plaintext.
pub struct PlatformSecretStore {
    // Caller chooses an application-private directory outside the repository.
    fallback_path: PathBuf,
}

impl PlatformSecretStore {
    /// Select a credential file inside a dedicated application-private directory.
    /// Existing directories and loaded files must be owned by this user with exact
    /// 0700/0600 permissions. Reading rejects symbolic links; saving replaces the
    /// directory entry atomically without following an existing symbolic link.
    pub fn new(fallback_path: PathBuf) -> Self {
        Self { fallback_path }
    }

    // Provision and validate the immediate parent before opening any secret file.
    #[cfg(unix)]
    fn private_parent(&self) -> Result<&Path, InvitationError> {
        use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};
        let parent = self
            .fallback_path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .ok_or(InvitationError::Storage)?;
        if !parent.exists() {
            let mut builder = std::fs::DirBuilder::new();
            builder
                .recursive(true)
                .mode(0o700)
                .create(parent)
                .map_err(|_| InvitationError::Storage)?;
        }
        let metadata = std::fs::symlink_metadata(parent).map_err(|_| InvitationError::Storage)?;
        // SAFETY: getuid has no preconditions and does not borrow or modify memory.
        let uid = unsafe { libc::getuid() };
        if !metadata.is_dir()
            || metadata.file_type().is_symlink()
            || metadata.uid() != uid
            || metadata.permissions().mode() & 0o777 != 0o700
        {
            return Err(InvitationError::Storage);
        }
        Ok(parent)
    }

    // Open relative to a held no-follow directory FD to prevent pathname races.
    #[cfg(unix)]
    fn directory(&self) -> Result<std::fs::File, InvitationError> {
        use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
        let directory = std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_DIRECTORY)
            .open(self.private_parent()?)
            .map_err(|_| InvitationError::Storage)?;
        // Recheck the held inode, so replacement between pathname validation and
        // open cannot substitute a directory belonging to another user.
        let metadata = directory.metadata().map_err(|_| InvitationError::Storage)?;
        // SAFETY: getuid has no preconditions.
        if metadata.uid() != unsafe { libc::getuid() }
            || metadata.permissions().mode() & 0o777 != 0o700
        {
            return Err(InvitationError::Storage);
        }
        Ok(directory)
    }

    // C names are used only with openat/unlinkat; reject traversal and NUL bytes.
    #[cfg(unix)]
    fn filename(&self) -> Result<std::ffi::CString, InvitationError> {
        use std::os::unix::ffi::OsStrExt;
        let name = self
            .fallback_path
            .file_name()
            .ok_or(InvitationError::Storage)?;
        std::ffi::CString::new(name.as_bytes()).map_err(|_| InvitationError::Storage)
    }

    // File fallback is intentionally available only where mode/ownership are meaningful.
    #[cfg(unix)]
    fn file_load(&self) -> Result<Option<AdminApiKey>, InvitationError> {
        use std::os::{
            fd::{AsRawFd, FromRawFd},
            unix::fs::{MetadataExt, PermissionsExt},
        };
        let directory = self.directory()?;
        let name = self.filename()?;
        // SAFETY: live directory FD and NUL-terminated filename; returned FD is uniquely owned.
        let fd = unsafe {
            libc::openat(
                directory.as_raw_fd(),
                name.as_ptr(),
                libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC,
            )
        };
        if fd < 0 {
            return if std::io::Error::last_os_error().kind() == std::io::ErrorKind::NotFound {
                Ok(None)
            } else {
                Err(InvitationError::Storage)
            };
        }
        // SAFETY: successful openat gives this function exclusive ownership of fd.
        let file = unsafe { std::fs::File::from_raw_fd(fd) };
        let metadata = file.metadata().map_err(|_| InvitationError::Storage)?;
        // SAFETY: getuid has no preconditions.
        if !metadata.is_file()
            || metadata.uid() != unsafe { libc::getuid() }
            || metadata.permissions().mode() & 0o777 != 0o600
            || metadata.nlink() != 1
        {
            return Err(InvitationError::Storage);
        }
        let mut bytes = Vec::new();
        file.take(1025)
            .read_to_end(&mut bytes)
            .map_err(|_| InvitationError::Storage)?;
        let secret = String::from_utf8(bytes).map_err(|_| InvitationError::Storage)?;
        AdminApiKey::from_secret(secret)
            .map(Some)
            .map_err(|_| InvitationError::Storage)
    }

    // Publish a newly created private inode atomically; never truncate/follow existing paths.
    #[cfg(unix)]
    fn file_save(&self, key: &AdminApiKey) -> Result<(), InvitationError> {
        use std::os::fd::{AsRawFd, FromRawFd};
        let directory = self.directory()?;
        let name = self.filename()?;
        let temporary = std::ffi::CString::new(format!(
            ".credential-{}",
            crate::random_hex::<16>().map_err(|_| InvitationError::Storage)?
        ))
        .map_err(|_| InvitationError::Storage)?;
        // SAFETY: all pointers are valid C strings and directory remains open.
        let fd = unsafe {
            libc::openat(
                directory.as_raw_fd(),
                temporary.as_ptr(),
                libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                0o600,
            )
        };
        if fd < 0 {
            return Err(InvitationError::Storage);
        }
        // SAFETY: successful openat transfers one uniquely owned descriptor.
        let mut file = unsafe { std::fs::File::from_raw_fd(fd) };
        let result = (|| {
            file.write_all(key.0.as_bytes())
                .map_err(|_| InvitationError::Storage)?;
            file.sync_all().map_err(|_| InvitationError::Storage)?;
            // SAFETY: both names are valid and relative to our held directory FD.
            if unsafe {
                libc::renameat(
                    directory.as_raw_fd(),
                    temporary.as_ptr(),
                    directory.as_raw_fd(),
                    name.as_ptr(),
                )
            } != 0
            {
                return Err(InvitationError::Storage);
            }
            directory.sync_all().map_err(|_| InvitationError::Storage)
        })();
        if result.is_err() {
            // SAFETY: delete only the randomly named temporary entry in our directory.
            unsafe {
                libc::unlinkat(directory.as_raw_fd(), temporary.as_ptr(), 0);
            }
        }
        result
    }

    // Unlink the entry itself, never its symlink target; absence is idempotent.
    #[cfg(unix)]
    fn file_delete(&self) -> Result<(), InvitationError> {
        use std::os::fd::AsRawFd;
        let directory = self.directory()?;
        let name = self.filename()?;
        // SAFETY: directory and filename remain valid; flags forbid recursive deletion.
        let result = unsafe { libc::unlinkat(directory.as_raw_fd(), name.as_ptr(), 0) };
        if result == 0 || std::io::Error::last_os_error().kind() == std::io::ErrorKind::NotFound {
            Ok(())
        } else {
            Err(InvitationError::Storage)
        }
    }
}

impl PlatformSecretStore {
    /// Read the file fallback without native Keychain prompts.
    ///
    /// Startup and catalog browsing must not block on `login` Keychain UI; invitation
    /// credentials still call [`SecretStore::load`], which may prompt.
    pub fn load_unattended(&self) -> Result<Option<AdminApiKey>, InvitationError> {
        #[cfg(unix)]
        {
            self.file_load()
        }
        #[cfg(not(unix))]
        {
            Err(InvitationError::Storage)
        }
    }
}

impl SecretStore for PlatformSecretStore {
    fn load(&self) -> Result<Option<AdminApiKey>, InvitationError> {
        #[cfg(target_os = "macos")]
        if let Ok(Some(key)) = keychain::load() {
            return Ok(Some(key));
        }
        self.load_unattended()
    }

    fn save(&self, key: &AdminApiKey) -> Result<(), InvitationError> {
        #[cfg(target_os = "macos")]
        if keychain::save(key).is_ok() {
            // Remove any earlier fallback copy after migration to native storage.
            self.file_delete()?;
            return Ok(());
        }
        #[cfg(unix)]
        {
            self.file_save(key)
        }
        #[cfg(not(unix))]
        {
            let _ = key;
            Err(InvitationError::Storage)
        }
    }

    fn delete(&self) -> Result<(), InvitationError> {
        #[cfg(target_os = "macos")]
        let native = keychain::delete();
        #[cfg(unix)]
        let fallback = self.file_delete();
        #[cfg(target_os = "macos")]
        {
            native.and(fallback)
        }
        #[cfg(all(unix, not(target_os = "macos")))]
        {
            fallback
        }
        #[cfg(not(unix))]
        {
            Err(InvitationError::Storage)
        }
    }
}

#[cfg(target_os = "macos")]
mod keychain {
    use super::*;
    use std::{ffi::c_void, ptr};

    // Fixed application service/account names avoid cross-application lookup ambiguity.
    const SERVICE: &[u8] = b"io.ramate.durop.tailscale-admin";
    const ACCOUNT: &[u8] = b"user-api-key";
    const NOT_FOUND: i32 = -25300;

    // Native Security owns returned item/data allocations until explicitly released.
    #[link(name = "Security", kind = "framework")]
    unsafe extern "C" {
        fn SecKeychainFindGenericPassword(
            keychain: *const c_void,
            service_len: u32,
            service: *const i8,
            account_len: u32,
            account: *const i8,
            length: *mut u32,
            data: *mut *mut c_void,
            item: *mut *mut c_void,
        ) -> i32;
        fn SecKeychainAddGenericPassword(
            keychain: *const c_void,
            service_len: u32,
            service: *const i8,
            account_len: u32,
            account: *const i8,
            length: u32,
            data: *const c_void,
            item: *mut *mut c_void,
        ) -> i32;
        fn SecKeychainItemModifyAttributesAndData(
            item: *mut c_void,
            attributes: *const c_void,
            length: u32,
            data: *const c_void,
        ) -> i32;
        fn SecKeychainItemFreeContent(attributes: *mut c_void, data: *mut c_void) -> i32;
        fn SecKeychainItemDelete(item: *mut c_void) -> i32;
    }

    // Security item references follow Core Foundation ownership rules.
    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFRelease(value: *const c_void);
    }

    // RAII releases native handles on every return path, including malformed secrets.
    struct Item(*mut c_void);

    impl Drop for Item {
        fn drop(&mut self) {
            if !self.0.is_null() {
                // SAFETY: successful Security lookup gives an owned CF reference.
                unsafe {
                    CFRelease(self.0);
                }
            }
        }
    }

    // A missing entry is distinct from a locked/inaccessible keychain.
    fn find() -> Result<Option<Item>, InvitationError> {
        let mut item = ptr::null_mut();
        // SAFETY: fixed byte slices have correct explicit lengths; item is a valid out-pointer.
        let status = unsafe {
            SecKeychainFindGenericPassword(
                ptr::null(),
                SERVICE.len() as u32,
                SERVICE.as_ptr().cast(),
                ACCOUNT.len() as u32,
                ACCOUNT.as_ptr().cast(),
                ptr::null_mut(),
                ptr::null_mut(),
                &mut item,
            )
        };
        match status {
            0 => Ok(Some(Item(item))),
            NOT_FOUND => Ok(None),
            _ => Err(InvitationError::Storage),
        }
    }

    pub(super) fn load() -> Result<Option<AdminApiKey>, InvitationError> {
        let mut length = 0;
        let mut data = ptr::null_mut();
        // SAFETY: valid fixed service/account buffers and initialized writable out-pointers.
        let status = unsafe {
            SecKeychainFindGenericPassword(
                ptr::null(),
                SERVICE.len() as u32,
                SERVICE.as_ptr().cast(),
                ACCOUNT.len() as u32,
                ACCOUNT.as_ptr().cast(),
                &mut length,
                &mut data,
                ptr::null_mut(),
            )
        };
        if status == NOT_FOUND {
            return Ok(None);
        }
        if status != 0 {
            return Err(InvitationError::Storage);
        }
        let secret = if !data.is_null() && length <= 1024 {
            // SAFETY: Security supplies exactly length initialized readable bytes until FreeContent.
            String::from_utf8(
                unsafe { std::slice::from_raw_parts(data.cast::<u8>(), length as usize) }.to_vec(),
            )
            .ok()
        } else {
            None
        };
        // SAFETY: this allocation was returned by the successful Security lookup.
        unsafe {
            SecKeychainItemFreeContent(ptr::null_mut(), data);
        }
        secret.ok_or(InvitationError::Storage).and_then(|secret| {
            AdminApiKey::from_secret(secret)
                .map(Some)
                .map_err(|_| InvitationError::Storage)
        })
    }

    pub(super) fn save(key: &AdminApiKey) -> Result<(), InvitationError> {
        let existing = find()?;
        // SAFETY: native handles and buffers live throughout the call; APIs copy input data.
        let status = unsafe {
            if let Some(item) = existing {
                SecKeychainItemModifyAttributesAndData(
                    item.0,
                    ptr::null(),
                    key.0.len() as u32,
                    key.0.as_ptr().cast(),
                )
            } else {
                SecKeychainAddGenericPassword(
                    ptr::null(),
                    SERVICE.len() as u32,
                    SERVICE.as_ptr().cast(),
                    ACCOUNT.len() as u32,
                    ACCOUNT.as_ptr().cast(),
                    key.0.len() as u32,
                    key.0.as_ptr().cast(),
                    ptr::null_mut(),
                )
            }
        };
        if status == 0 {
            Ok(())
        } else {
            Err(InvitationError::Storage)
        }
    }

    pub(super) fn delete() -> Result<(), InvitationError> {
        if let Some(item) = find()? {
            // SAFETY: find returned a live owned keychain item reference.
            if unsafe { SecKeychainItemDelete(item.0) } != 0 {
                return Err(InvitationError::Storage);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(unix)]
    fn protected_file_round_trip_and_unsafe_paths_fail_closed() {
        use std::os::unix::fs::{PermissionsExt, symlink};
        let directory = std::env::temp_dir().join(format!(
            "durop-secret-{}",
            crate::random_hex::<16>().unwrap()
        ));
        let store = PlatformSecretStore::new(directory.join("api-key"));
        let key = AdminApiKey::from_secret("tskey-api-contract-test-credential".into()).unwrap();
        assert!(store.file_load().unwrap().is_none());
        store.file_save(&key).unwrap();
        assert_eq!(store.file_load().unwrap().unwrap().0, key.0);
        assert_eq!(
            std::fs::metadata(&store.fallback_path)
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        std::fs::set_permissions(&store.fallback_path, std::fs::Permissions::from_mode(0o644))
            .unwrap();
        assert!(matches!(store.file_load(), Err(InvitationError::Storage)));
        std::fs::remove_file(&store.fallback_path).unwrap();
        let outside = directory.join("outside");
        std::fs::write(&outside, b"untouched").unwrap();
        symlink(&outside, &store.fallback_path).unwrap();
        assert!(store.file_load().is_err());
        store.file_save(&key).unwrap();
        assert_eq!(std::fs::read(&outside).unwrap(), b"untouched");
        store.file_delete().unwrap();
        store.file_delete().unwrap();
        std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(store.file_save(&key).is_err());
        std::fs::remove_dir_all(directory).unwrap();
    }
}
