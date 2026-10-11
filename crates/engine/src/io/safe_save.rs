//! Sibling temporary files owned by one save. The destination changes only after preparation
//! succeeds; concurrent saves have separate files and the last successful replacement wins.
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

pub(super) fn write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let permissions = match fs::metadata(path) {
        Ok(meta) if meta.is_file() => Some(meta.permissions()),
        Ok(_) => None,
        Err(e) if e.kind() == io::ErrorKind::NotFound => None,
        Err(e) => return Err(e),
    };
    replace_with(path, |file| {
        file.write_all(bytes)?;
        if let Some(permissions) = permissions {
            file.set_permissions(permissions)?;
        }
        // Synchronize the prepared file before replacing the destination. Directory durability
        // and network-filesystem/power-loss guarantees are outside this operation's contract.
        file.sync_all()
    })
}

fn replace_with(path: &Path, prepare: impl FnOnce(&mut File) -> io::Result<()>) -> io::Result<()> {
    let mut temp = Temporary::create(path)?;
    prepare(temp.file()?)?;
    temp.commit(path)
}

struct Temporary {
    path: PathBuf,
    file: Option<File>,
    committed: bool,
}

impl Temporary {
    fn create(destination: &Path) -> io::Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        if destination.file_name().is_none() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "save destination needs a file name"));
        }
        // A short name also works beside a destination already at the filename-length limit.
        for _ in 0..128 {
            let id = NEXT.fetch_add(1, Ordering::Relaxed);
            let path = destination.with_file_name(format!(".wordcraft-save-{}-{id}.tmp", std::process::id()));
            let mut options = OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            match options.open(&path) {
                Ok(file) => return Ok(Self { path, file: Some(file), committed: false }),
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e),
            }
        }
        Err(io::Error::new(io::ErrorKind::AlreadyExists, "could not create a save temporary file"))
    }

    fn file(&mut self) -> io::Result<&mut File> {
        self.file.as_mut().ok_or_else(|| io::Error::other("save temporary file is closed"))
    }

    fn commit(mut self, destination: &Path) -> io::Result<()> {
        // Close before replacement, including on Windows.
        drop(self.file.take());
        fs::rename(&self.path, destination)?;
        self.committed = true;
        Ok(())
    }
}

impl Drop for Temporary {
    fn drop(&mut self) {
        drop(self.file.take());
        if !self.committed {
            // If filesystem permissions change, cleanup can fail too. Never remove a neighbour
            // or the destination to compensate for a failed save.
            let _ = fs::remove_file(&self.path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Scratch(PathBuf);
    impl Scratch {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let path =
                std::env::temp_dir().join(format!("wordcraft-save-owner-test-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn target(&self) -> PathBuf {
            self.0.join("original.docx")
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn partial_and_complete_preparation_failures_leave_original_and_no_owned_temp() {
        for partial in [true, false] {
            let d = Scratch::new();
            let path = d.target();
            fs::write(&path, b"Original bytes.").unwrap();
            let err = replace_with(&path, |file| {
                file.write_all(if partial { b"Partial" } else { b"Complete replacement bytes." })?;
                Err(io::Error::other("injected preparation failure"))
            })
            .unwrap_err();
            assert!(err.to_string().contains("injected"));
            assert_eq!(fs::read(&path).unwrap(), b"Original bytes.");
            assert_eq!(fs::read_dir(&d.0).unwrap().count(), 1);
        }
    }

    #[test]
    fn overlapping_prepared_saves_keep_separate_bytes_and_commit_complete_files() {
        let d = Scratch::new();
        let path = d.target();
        fs::write(&path, b"Original bytes.").unwrap();
        let mut first = Temporary::create(&path).unwrap();
        let mut second = Temporary::create(&path).unwrap();
        assert_ne!(first.path, second.path);
        first.file().unwrap().write_all(b"First complete save.").unwrap();
        second.file().unwrap().write_all(b"Second complete save.").unwrap();
        first.file().unwrap().sync_all().unwrap();
        second.file().unwrap().sync_all().unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"Original bytes.");
        first.commit(&path).unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"First complete save.");
        second.commit(&path).unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"Second complete save.");
        assert_eq!(fs::read_dir(&d.0).unwrap().count(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn new_temporary_files_are_private_until_prepared() {
        use std::os::unix::fs::PermissionsExt;
        let d = Scratch::new();
        let temp = Temporary::create(&d.target()).unwrap();
        assert_eq!(fs::metadata(&temp.path).unwrap().permissions().mode() & 0o777, 0o600);
    }
}
