//! A picture saved to a file: the person's Downloads folder, the name the
//! file goes under, and the write, which takes a name that no file stands at
//! and never touches one that does.
//!
//! Nothing here asks the yard or draws: the window passes the bytes it
//! already holds and the folder it was given (`Means::downloads`), and says
//! what came of it.

use std::ffi::OsString;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use crate::draft;

/// How many files of one name the write looks through, ` (2)` up to this,
/// before it gives up: a bound for a folder that answers every try with a
/// file, not a count anyone reaches.
const TRIES: usize = 9999;

/// The person's Downloads folder: `XDG_DOWNLOAD_DIR` where it is set to an
/// absolute path, else `Downloads` in their home. A variable that is empty or
/// relative names no place, since what it is relative to is the program's own
/// directory, and a home that is not absolute is none. The user-dirs file a
/// desktop keeps is not read: the variable is how another folder is said.
pub fn downloads(xdg: Option<OsString>, home: Option<PathBuf>) -> Option<PathBuf> {
    xdg.map(PathBuf::from)
        .filter(|dir| dir.is_absolute())
        .or_else(|| {
            home.filter(|home| home.is_absolute())
                .map(|home| home.join("Downloads"))
        })
}

/// This machine's Downloads folder.
#[cfg_attr(test, allow(dead_code))]
pub fn downloads_here() -> Option<PathBuf> {
    downloads(std::env::var_os("XDG_DOWNLOAD_DIR"), std::env::home_dir())
}

/// The extension a picture of the type `mime` goes under when its file has
/// no name of its own.
pub fn extension(mime: &str) -> &'static str {
    match mime {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        _ => "img",
    }
}

/// The name the picture `hash` is saved under: the name its file was sent
/// with, where that is one the yard's rule takes (draft.rs), else the hash
/// with the extension `ext` of its format. A name from a linked yard or a
/// hook is the source's own word, and one with a path in it or a control
/// character is no reason to write elsewhere or to write nothing: the hash
/// names the file instead.
pub fn file_name(given: &str, hash: &str, ext: &str) -> String {
    let hashed = format!("{hash}.{ext}");
    [given.trim(), hashed.as_str()]
        .into_iter()
        .find_map(|name| numbered(name, 1))
        .unwrap_or_else(|| format!("picture.{ext}"))
}

/// `name` as the `n`th file of its name: itself for the first, then with
/// ` (2)`, ` (3)` and so on before its extension, shortened to fit a name if
/// it is long. None where it is no name a file can have. A draft may carry a
/// file with no name (what was pasted), which the yard's rule takes; a file
/// on disk cannot.
fn numbered(name: &str, n: usize) -> Option<String> {
    if name.is_empty() {
        return None;
    }
    let (stem, ext) = draft::split_name(name);
    let suffix = match n {
        0 | 1 => String::new(),
        n => format!(" ({n})"),
    };
    draft::fit_name(stem, &suffix, ext).ok()
}

/// Writes `bytes` into the folder `dir` under `name`, or under the first of
/// `name (2)`, `name (3)` and so on that nothing stands at (no file, no
/// directory, no link), and returns the path it wrote. The folder is made
/// where it is not there. A file that stands at a name is never opened for
/// writing, nor is a link followed: each try creates its file or fails, so
/// another program's file made between two tries is passed over, not written
/// into. Half a picture is not kept.
pub fn write(dir: &Path, name: &str, bytes: &[u8]) -> io::Result<PathBuf> {
    fs::create_dir_all(dir)?;
    for n in 1..=TRIES {
        let Some(file_name) = numbered(name, n) else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{name} is no name a file can be saved under"),
            ));
        };
        let path = dir.join(file_name);
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        // Pictures can come from a yard or a hook. Keep them private like
        // the token files, even when the person's umask is wide open.
        #[cfg(unix)]
        options.mode(0o600);
        match options.open(&path) {
            Ok(mut file) => {
                return match file.write_all(bytes) {
                    Ok(()) => Ok(path),
                    Err(err) => {
                        drop(file);
                        let _ = fs::remove_file(&path);
                        Err(err)
                    }
                };
            }
            Err(err) if taken(&err, &path) => continue,
            Err(err) => return Err(err),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        format!(
            "{TRIES} files named {name} stand in {} already",
            dir.display()
        ),
    ))
}

/// Whether `err`, the refusal of the open of `path`, says its name is taken.
/// A file at the name is `AlreadyExists` everywhere, and so is a directory
/// there, except on Windows, which opens no directory for writing and answers
/// "access denied" instead: `PermissionDenied`, the same kind as a folder the
/// person may not write in. So that refusal is a taken name where something
/// stands at the name, and is returned as the folder's own where nothing does,
/// at once and not after `TRIES` numbered names. `symlink_metadata` asks
/// whether something stands there without opening it or following a link.
fn taken(err: &io::Error, path: &Path) -> bool {
    match err.kind() {
        io::ErrorKind::AlreadyExists => true,
        io::ErrorKind::PermissionDenied => fs::symlink_metadata(path).is_ok(),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    /// An absolute path on any system.
    fn here(name: &str) -> PathBuf {
        std::env::temp_dir().join(name)
    }

    #[test]
    fn the_downloads_folder_is_the_variable_or_the_homes() {
        let home = Some(here("jo"));
        let variable = |dir: PathBuf| Some(dir.into_os_string());
        // Where the variable names an absolute path, that is the folder.
        assert_eq!(
            downloads(variable(here("dl")), home.clone()),
            Some(here("dl"))
        );
        // Else Downloads in the home.
        assert_eq!(
            downloads(None, home.clone()),
            Some(here("jo").join("Downloads"))
        );
        // A variable that is empty or relative names no place.
        for bogus in ["", "dl", "./dl"] {
            assert_eq!(
                downloads(Some(bogus.into()), home.clone()),
                Some(here("jo").join("Downloads")),
                "{bogus:?}"
            );
        }
        // And a home that is no absolute path is none.
        assert_eq!(downloads(None, Some(PathBuf::from("jo"))), None);
        assert_eq!(downloads(None, None), None);
        assert_eq!(downloads(Some("dl".into()), None), None);
    }

    #[test]
    fn a_picture_goes_under_its_file_s_name_or_its_hash() {
        let hash = "ab".repeat(32);
        let hashed = format!("{hash}.png");
        for (given, want) in [
            ("fence.png", "fence.png"),
            // Spaces around a name are not part of it.
            ("  fence.png ", "fence.png"),
            ("two words.jpg", "two words.jpg"),
            ("naïve 🚂.png", "naïve 🚂.png"),
            // None, or nothing but spaces: the hash and the format's extension.
            ("", hashed.as_str()),
            ("   ", hashed.as_str()),
            // What the yard's rule would refuse is no place to write under: a
            // path, whatever way it is written, a control character, a bidi
            // one that turns the extension round, and the two dots.
            ("../../.ssh/authorized_keys", hashed.as_str()),
            ("/etc/passwd", hashed.as_str()),
            ("dir/fence.png", hashed.as_str()),
            ("C:\\Users\\jo\\fence.png", hashed.as_str()),
            ("fence\n.png", hashed.as_str()),
            ("fence\u{0}.png", hashed.as_str()),
            ("fence\u{202e}gnp.exe", hashed.as_str()),
            (".", hashed.as_str()),
            ("..", hashed.as_str()),
        ] {
            assert_eq!(file_name(given, &hash, "png"), want, "{given:?}");
        }
        // A hash that is no name either is a name of its own.
        assert_eq!(file_name("", "../x", "png"), "picture.png");
        // A long name is shortened to the 255 bytes a name holds, its
        // extension kept.
        let long = file_name(&format!("{}.png", "a".repeat(300)), &hash, "png");
        assert_eq!(long, format!("{}.png", "a".repeat(251)));
        // The extension is the format's.
        assert_eq!(extension("image/png"), "png");
        assert_eq!(extension("image/jpeg"), "jpg");
        assert_eq!(
            file_name("", &hash, extension("image/jpeg")),
            format!("{hash}.jpg")
        );
    }

    #[test]
    fn a_name_that_stands_is_never_written_over() {
        let dir = tempfile::tempdir().unwrap();
        let first = write(dir.path(), "fence.png", b"one").unwrap();
        assert_eq!(first, dir.path().join("fence.png"));
        #[cfg(unix)]
        assert_eq!(
            fs::metadata(&first).unwrap().permissions().mode() & 0o077,
            0
        );
        let second = write(dir.path(), "fence.png", b"two").unwrap();
        assert_eq!(second, dir.path().join("fence (2).png"));
        let third = write(dir.path(), "fence.png", b"three").unwrap();
        assert_eq!(third, dir.path().join("fence (3).png"));
        // The first is as it was, and each holds its own bytes.
        assert_eq!(fs::read(&first).unwrap(), b"one");
        assert_eq!(fs::read(&second).unwrap(), b"two");
        assert_eq!(fs::read(&third).unwrap(), b"three");
        // The number goes before the last extension, or at the end of a
        // name that has none.
        for (name, again) in [
            ("report.final.png", "report.final (2).png"),
            ("README", "README (2)"),
            (".env", ".env (2)"),
        ] {
            write(dir.path(), name, b"x").unwrap();
            let path = write(dir.path(), name, b"y").unwrap();
            assert_eq!(path, dir.path().join(again), "{name}");
        }
        // A directory standing at the name is a name taken as well.
        fs::create_dir(dir.path().join("pics")).unwrap();
        let path = write(dir.path(), "pics", b"z").unwrap();
        assert_eq!(path, dir.path().join("pics (2)"));
        // A taken name is refused in the system's own word: "exists", and on
        // Windows, for a directory, "access denied". The step above meets
        // only the word of the system it runs on, so this asks the rule with
        // each. "Access denied" is a taken name where something stands at the
        // name, and the folder's own refusal where nothing does.
        let stands = dir.path().join("pics");
        let nothing = dir.path().join("nothing");
        for (kind, at, want) in [
            (io::ErrorKind::AlreadyExists, &stands, true),
            (io::ErrorKind::PermissionDenied, &stands, true),
            (io::ErrorKind::PermissionDenied, &nothing, false),
            // No other refusal says a name is taken, whatever stands there.
            (io::ErrorKind::NotFound, &stands, false),
            (io::ErrorKind::InvalidInput, &stands, false),
        ] {
            let err = io::Error::from(kind);
            assert_eq!(taken(&err, at), want, "{kind:?} at {at:?}");
        }
        // A long name keeps its number and its extension whole.
        let long = format!("{}.png", "a".repeat(300));
        write(dir.path(), &long, b"x").unwrap();
        let path = write(dir.path(), &long, b"y").unwrap();
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        assert_eq!(name.len(), 255);
        assert!(name.ends_with(" (2).png"), "{name}");
    }

    #[test]
    fn the_folder_is_made_where_it_is_not_there() {
        let dir = tempfile::tempdir().unwrap();
        let downloads = dir.path().join("Downloads");
        let path = write(&downloads, "fence.png", b"x").unwrap();
        assert_eq!(path, downloads.join("fence.png"));
        assert_eq!(fs::read(path).unwrap(), b"x");
        // A file where the folder should be is why nothing is written.
        let file = dir.path().join("file");
        fs::write(&file, b"in the way").unwrap();
        assert!(write(&file, "fence.png", b"x").is_err());
        assert_eq!(fs::read(&file).unwrap(), b"in the way");
    }

    #[test]
    fn a_name_that_is_no_name_writes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        for bad in [
            "../escape.png",
            "a/b.png",
            "a\\b.png",
            "..",
            ".",
            "",
            "a\nb",
        ] {
            let err = write(dir.path(), bad, b"x").unwrap_err();
            assert_eq!(err.kind(), io::ErrorKind::InvalidInput, "{bad:?}");
        }
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
    }

    // Symlinks are a unix system's.
    #[cfg(unix)]
    #[test]
    fn a_link_standing_at_the_name_is_not_followed() {
        let dir = tempfile::tempdir().unwrap();
        let mine = dir.path().join("mine");
        fs::write(&mine, b"mine").unwrap();
        std::os::unix::fs::symlink(&mine, dir.path().join("fence.png")).unwrap();
        let path = write(dir.path(), "fence.png", b"theirs").unwrap();
        assert_eq!(path, dir.path().join("fence (2).png"));
        assert_eq!(fs::read(&mine).unwrap(), b"mine");
        // A link to nothing would be created through, were it followed.
        let nowhere = dir.path().join("nowhere");
        std::os::unix::fs::symlink(&nowhere, dir.path().join("dangling.png")).unwrap();
        let path = write(dir.path(), "dangling.png", b"theirs").unwrap();
        assert_eq!(path, dir.path().join("dangling (2).png"));
        assert!(!nowhere.exists());
    }
}
