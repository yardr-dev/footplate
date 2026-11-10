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
/// with, where that is one the yard's rule takes (draft.rs) and, on Windows,
/// one that system keeps a file under, else the hash with the extension `ext`
/// of its format. A name from a linked yard or a hook is the source's own
/// word, and one with a path in it or a control character is no reason to
/// write elsewhere or to write nothing: the hash names the file instead. So
/// does a name that Windows would read as a device (`NUL.png`), a stream
/// (`chart-12:30.png`) or a drive (`C:x.png`).
pub fn file_name(given: &str, hash: &str, ext: &str) -> String {
    let hashed = format!("{hash}.{ext}");
    [given.trim(), hashed.as_str()]
        .into_iter()
        .find_map(|name| numbered(name, 1))
        .unwrap_or_else(|| format!("picture.{ext}"))
}

/// `name` as the `n`th file of its name: itself for the first, then with
/// ` (2)`, ` (3)` and so on before its extension, shortened to fit a name if
/// it is long. None where it is no name a file can have: to the yard's rule
/// (draft.rs), and on Windows to what that system keeps a file under
/// (`windows_keeps`), so that `write` never opens what the name would reach
/// there. A draft may carry a file with no name (what was pasted), which the
/// yard's rule takes; a file on disk cannot.
fn numbered(name: &str, n: usize) -> Option<String> {
    if name.is_empty() {
        return None;
    }
    let (stem, ext) = draft::split_name(name);
    let suffix = match n {
        0 | 1 => String::new(),
        n => format!(" ({n})"),
    };
    draft::fit_name(stem, &suffix, ext)
        .ok()
        // On any other system these are names like the rest.
        .filter(|fitted| !cfg!(windows) || windows_keeps(fitted))
}

/// Whether Windows keeps a file under exactly `name`. It keeps none under:
/// - a device's name (`NUL`, `CON`, `COM1`, `LPT1` and the rest of
///   `device`'s), in any case and with any extension, which the part before
///   the first dot decides, spaces trimmed: the open reaches the device, so a
///   picture saved there is thrown away (NUL) or sent to the console or a
///   port, and the window says it was saved. Which of these the system still
///   reads so depends on its build (a bare `NUL` on all, `NUL.png` and
///   `CON` on the older), and a list for the older is no worse for the newer.
/// - a name with a `:`: what follows it names a stream of another file
///   (`chart-12:30.png` is a stream of an empty `chart-12`) or a drive
///   (`C:x.png` is a file of that drive's current directory, outside the
///   folder `Path::join` was given).
/// - a name with one of `< > " | ? *`, which the open refuses, so that the
///   picture could never be saved.
///
/// A trailing dot or space is none of these: Windows strips it and keeps the
/// file.
fn windows_keeps(name: &str) -> bool {
    if name.contains([':', '<', '>', '"', '|', '?', '*']) {
        return false;
    }
    let stem = name
        .split_once('.')
        .map_or(name, |(stem, _)| stem)
        .trim_end_matches(' ');
    !device(stem)
}

/// Whether `stem` is the name of a device of Windows, in any case: the
/// console, the printer and the serial ports, NUL, and the console's two
/// buffers. COM and LPT take a number from 1 to 9, or a superscript one to
/// three, and no more: `COM0` and `COM10` are files.
fn device(stem: &str) -> bool {
    const NAMES: [&str; 6] = ["CON", "PRN", "AUX", "NUL", "CONIN$", "CONOUT$"];
    if NAMES.iter().any(|name| stem.eq_ignore_ascii_case(name)) {
        return true;
    }
    let mut chars = stem.chars();
    let letters: String = chars.by_ref().take(3).collect();
    matches!(
        (chars.next(), chars.next()),
        (Some('1'..='9' | '¹' | '²' | '³'), None)
    ) && (letters.eq_ignore_ascii_case("COM") || letters.eq_ignore_ascii_case("LPT"))
}

/// Writes `bytes` into the folder `dir` under `name`, or under the first of
/// `name (2)`, `name (3)` and so on that nothing stands at (no file, folder
/// or link), and returns the path it wrote. The folder is made where it is not
/// there. A file that stands at a name is never opened for writing, nor is a
/// link followed: each try creates its file or fails, so another program's
/// file made between two tries is passed over, not written into. A name that
/// is no name a file can have (`numbered`) writes nothing and is
/// `InvalidInput`. Half a picture is not kept.
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
        // Whatever stands at the name is a taken name: a file, a folder, a
        // link. A folder is told here and not by the open: Windows opens no
        // folder for writing, nor a link to one (a junction or a symlink to a
        // folder, to nothing as well), and answers with "access denied",
        // which is no "exists", so the write would end on it. The check
        // follows no link, so a link is seen as itself and nothing is opened
        // through it. The open is still the only step that creates a file,
        // and numbers past one made after this look.
        if fs::symlink_metadata(&path).is_ok() {
            continue;
        }
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
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => continue,
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
        // Windows reads some names as devices, streams or drives, or refuses
        // them. Those fall back to the hash there and stay the given name on
        // other systems. Similar names and trailing dots or spaces stay as
        // files on Windows too.
        for (given, hash_on_windows) in [
            ("NUL", true),
            ("nul", true),
            ("NUL.png", true),
            ("Nul.PNG", true),
            ("NUL.tar.gz", true),
            ("NUL .png", true),
            ("NUL ", true),
            ("NUL.", true),
            ("CON", true),
            ("con.png", true),
            ("PRN.txt", true),
            ("AUX", true),
            ("CONIN$", true),
            ("CONOUT$.png", true),
            ("COM1", true),
            ("com9.jpg", true),
            ("LPT1", true),
            ("lpt9.png", true),
            ("COM¹", true),
            ("LPT³.png", true),
            ("fence.png:s", true),
            ("chart-12:30.png", true),
            ("C:x.png", true),
            ("NUL:", true),
            ("a<b", true),
            ("a>b", true),
            ("a\"b", true),
            ("a|b", true),
            ("a?b.png", true),
            ("a*b.png", true),
            ("NULL.png", false),
            ("NULx", false),
            ("xNUL.png", false),
            ("CONSOLE.png", false),
            ("COM0", false),
            ("LPT0.png", false),
            ("COM10", false),
            ("COM", false),
            ("NUL (2).png", false),
            (".NUL", false),
            ("fence.png.", false),
        ] {
            let want = if cfg!(windows) && hash_on_windows {
                hashed.as_str()
            } else {
                given.trim()
            };
            assert_eq!(file_name(given, &hash, "png"), want, "{given:?}");
        }
        // A hash that is no name either is a name of its own.
        assert_eq!(file_name("", "../x", "png"), "picture.png");
        if cfg!(windows) {
            assert_eq!(file_name("", "a:b", "png"), "picture.png");
        }
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
        // So is a link to one. Windows answers a junction or a symlink to a
        // folder, to nothing as well, "access denied" when it is opened for
        // writing, as it does a folder, so the open does not tell it taken.
        #[cfg(windows)]
        {
            let elsewhere = dir.path().join("elsewhere");
            fs::create_dir(&elsewhere).unwrap();
            // A junction wants no privilege, and std's call for one is not
            // stable yet.
            let junction = dir.path().join("junction");
            let made = std::process::Command::new("cmd")
                .args(["/C", "mklink", "/J"])
                .arg(&junction)
                .arg(&elsewhere)
                .output()
                .unwrap();
            assert!(made.status.success(), "mklink /J: {made:?}");
            // A symlink wants one that an administrator's account has and a
            // person's may lack (Developer Mode gives it): the junction
            // stands for the kind where the system refuses it.
            let mut links = vec!["junction"];
            for (name, target) in [("linked", "elsewhere"), ("dangling", "nowhere")] {
                let link = dir.path().join(name);
                if std::os::windows::fs::symlink_dir(dir.path().join(target), link).is_ok() {
                    links.push(name);
                }
            }
            for name in links {
                let path = write(dir.path(), name, b"z").unwrap();
                assert_eq!(path, dir.path().join(format!("{name} (2)")), "{name}");
            }
            // Nothing went through any of them.
            assert_eq!(fs::read_dir(&elsewhere).unwrap().count(), 0);
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
        // On Windows, too, the names that system keeps no file under: the
        // open would reach a device, a stream of another file or another
        // folder than this one.
        let windows: &[&str] = if cfg!(windows) {
            &[
                "NUL",
                "nul.png",
                "CON.png",
                "COM1.jpg",
                "LPT9",
                "chart-12:30.png",
                "C:x.png",
                "a<b",
                "a>b",
                "a\"b",
                "a|b",
                "a?b.png",
                "a*b.png",
            ]
        } else {
            &[]
        };
        for bad in [
            "../escape.png",
            "a/b.png",
            "a\\b.png",
            "..",
            ".",
            "",
            "a\nb",
        ]
        .iter()
        .chain(windows)
        {
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
