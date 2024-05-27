pub(crate) const IPC_PREFIX: &str = "discord-ipc-";

#[cfg(unix)]
pub(crate) const IPC_DIRS: [&str; 4] = ["XDG_RUNTIME_DIR", "TMPDIR", "TMP", "TEMP"];

#[cfg(windows)]
pub(crate) const IPC_DIR: &str = r"\\?\pipe\";
