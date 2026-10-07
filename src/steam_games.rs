use std::path::PathBuf;


fn manifests_dir(steam_dir: &PathBuf) -> PathBuf {
    steam_dir.join("steamapps")
}