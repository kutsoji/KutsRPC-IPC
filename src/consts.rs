pub(crate) const IPC_PREFIX: &'static str = "discord-ipc-";

#[cfg(unix)]
pub(crate) const IPC_DIRS: [&'static str; 4] = ["XDG_RUNTIME_DIR", "TMPDIR", "TMP", "TEMP"];

#[cfg(windows)]
pub(crate) const IPC_DIR: &'static str = r"\\?\pipe\";