use std::str;

use zed_extension_api::process::Command as ProcessCommand;
use zed_extension_api as zed;

const REPO: &str = "xyndra/xy_build";

struct XyBuildExtension;

impl zed::Extension for XyBuildExtension {
    fn new() -> Self {
        XyBuildExtension
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        let (os, arch) = zed::current_platform();
        let ext = if os == zed::Os::Windows { ".exe" } else { "" };

        // determine where to find a local binary
        let env = worktree.shell_env();
        let local_path = env
            .iter()
            .find(|(k, _)| k == "XY_BUILD_LSP_PATH")
            .map(|(_, v)| v.clone())
            .or_else(|| worktree.which("xy-build-lsp"));

        if let Some(path) = local_path {
            // get local version via --version
            let local_ver = ProcessCommand::new(&path)
                .arg("--version")
                .output()
                .ok()
                .and_then(|out| {
                    if out.status != Some(0) {
                        return None;
                    }
                    let s = str::from_utf8(&out.stdout).ok()?;
                    Some(s.trim().to_string())
                });

            // get latest release version
            let release_ver = zed::latest_github_release(
                REPO,
                zed::GithubReleaseOptions {
                    require_assets: true,
                    pre_release: false,
                },
            )
            .ok()
            .map(|r| r.version.trim_start_matches('v').to_string());

            match (local_ver, release_ver) {
                (Some(ref lv), Some(ref rv)) if lv == rv => {
                    return Ok(zed::Command {
                        command: path,
                        args: vec![],
                        env: vec![],
                    });
                }
                (_, Some(rv)) => match download_lsp(language_server_id, os, arch, ext, &rv) {
                    Ok(cmd) => return Ok(cmd),
                    Err(_) => {
                        zed::set_language_server_installation_status(
                            language_server_id,
                            &zed::LanguageServerInstallationStatus::Failed(
                                format!(
                                    "local version doesn't match latest release v{rv}, \
                                     and download failed. falling back to local binary."
                                ),
                            ),
                        );
                        return Ok(zed::Command {
                            command: path,
                            args: vec![],
                            env: vec![],
                        });
                    }
                },
                (_, None) => {
                    return Ok(zed::Command {
                        command: path,
                        args: vec![],
                        env: vec![],
                    });
                }
            }
        }

        // no local binary found — download or error
        let release = zed::latest_github_release(
            REPO,
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )
        .map_err(|e| {
            format!(
                "failed to fetch latest release ({e}). \
                 build xy-build-lsp from source: `cargo build -p xy-build-lsp` \
                 and place it on $PATH, or set $XY_BUILD_LSP_PATH"
            )
        })?;

        let version = release.version.trim_start_matches('v');
        download_lsp(language_server_id, os, arch, ext, version)
    }
}

fn download_lsp(
    language_server_id: &zed::LanguageServerId,
    os: zed::Os,
    arch: zed::Architecture,
    ext: &str,
    version: &str,
) -> zed::Result<zed::Command> {
    let asset_suffix = match (os, arch) {
        (zed::Os::Linux, zed::Architecture::X8664) => "x86_64-linux",
        (zed::Os::Linux, zed::Architecture::Aarch64) => "aarch64-linux",
        (zed::Os::Mac, zed::Architecture::X8664) => "x86_64-macos",
        (zed::Os::Mac, zed::Architecture::Aarch64) => "aarch64-macos",
        (zed::Os::Windows, zed::Architecture::X8664) => "x86_64-windows",
        (zed::Os::Windows, _) => "aarch64-windows",
        _ => {
            return Err(format!(
                "unsupported platform ({:?}, {:?}). \
                 build xy-build-lsp from source: `cargo build -p xy-build-lsp`",
                os, arch,
            ));
        }
    };

    let asset_name = format!("xy-build-lsp-{}{}", asset_suffix, ext);
    let cached_name = format!("xy-build-lsp-v{}{}", version, ext);

    if zed::make_file_executable(&cached_name).is_ok() {
        return Ok(zed::Command {
            command: cached_name,
            args: vec![],
            env: vec![],
        });
    }

    zed::set_language_server_installation_status(
        language_server_id,
        &zed::LanguageServerInstallationStatus::Downloading,
    );

    let release = zed::latest_github_release(
        REPO,
        zed::GithubReleaseOptions {
            require_assets: true,
            pre_release: false,
        },
    )
    .map_err(|e| {
        format!(
            "failed to fetch latest release ({e}). \
             build xy-build-lsp from source: `cargo build -p xy-build-lsp` \
             and place it on $PATH, or set $XY_BUILD_LSP_PATH"
        )
    })?;

    let asset = release
        .assets
        .iter()
        .find(|a| a.name == asset_name)
        .ok_or_else(|| {
            format!(
                "no {} asset in release {}. \
                 build xy-build-lsp from source: `cargo build -p xy-build-lsp`",
                asset_name, release.version,
            )
        })?;

    zed::download_file(
        &asset.download_url,
        &cached_name,
        zed::DownloadedFileType::Uncompressed,
    )
    .map_err(|e| {
        format!(
            "failed to download LSP ({e}). \
             build xy-build-lsp from source: `cargo build -p xy-build-lsp`"
        )
    })?;

    zed::make_file_executable(&cached_name)
        .map_err(|e| format!("failed to make LSP executable: {e}"))?;

    Ok(zed::Command {
        command: cached_name,
        args: vec![],
        env: vec![],
    })
}

zed::register_extension!(XyBuildExtension);
