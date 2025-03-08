use std::fs;
use std::path::PathBuf;
use zed_extension_api::serde_json;
use zed_extension_api::settings::LspSettings;
use zed_extension_api::{self as zed, LanguageServerId, Result};

const SYSTEMD_LANGUAGE_SERVER_NAME: &'static str = "systemd";

struct UnitFileExtension {
    cached_executable_path: Option<String>,
}

impl UnitFileExtension {
    fn language_server_binary_path(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<String> {
        if let Some(path) = worktree.which(SYSTEMD_LANGUAGE_SERVER_NAME) {
            return Ok(path);
        }

        if let Some(path) = &self.cached_executable_path {
            if fs::metadata(path).map_or(false, |meta| meta.is_file()) {
                return Ok(path.clone());
            }
        }

        zed::set_language_server_installation_status(
            &language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let release = zed::latest_github_release(
            "10fish/systemd-language-server-rs",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (platform, arch) = zed::current_platform();
        let asset_name = format!(
            "systemd-language-server-{os}-{arch}{extension}",
            os = match platform {
                zed::Os::Mac => "macos",
                zed::Os::Linux => "linux",
                zed::Os::Windows => "windows",
            },
            arch = match arch {
                zed::Architecture::Aarch64 => "arm64",
                zed::Architecture::X8664 => "amd64",
                zed_extension_api::Architecture::X86 => "x86",
            },
            extension = match platform {
                zed::Os::Mac | zed::Os::Linux => ".tar.gz",
                zed::Os::Windows => ".zip",
            },
        );

        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| format!("no asset found matching {:?}", asset_name))?;

        let download_path = "language-server";
        let version_dir = format!("systemd-language-server-{}", release.version);
        let binary_path = PathBuf::new()
            .join(&download_path)
            .join(&version_dir)
            .join(format!(
                "systemd-language-server{extension}",
                extension = match platform {
                    zed::Os::Windows => ".exe",
                    _ => "",
                },
            ));

        if !fs::metadata(&binary_path).map_or(false, |stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                &language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            zed::download_file(
                &asset.download_url,
                &download_path,
                match platform {
                    zed_extension_api::Os::Windows => zed::DownloadedFileType::Zip,
                    _ => zed::DownloadedFileType::GzipTar,
                },
            )
            .map_err(|e| format!("failed to download file: {e}"))?;
            zed::make_file_executable(&binary_path.to_str().unwrap())?;

            let entries = fs::read_dir(&download_path)
                .map_err(|e| format!("failed to list working directory {e}"))?;
            for entry in entries {
                let entry = entry.map_err(|e| format!("failed to load directory entry {e}"))?;
                if entry.file_name().to_str() != Some(&version_dir) {
                    fs::remove_dir_all(&entry.path()).ok();
                }
            }
        }

        let path = binary_path.to_str().unwrap().to_string();
        self.cached_executable_path = Some(path.clone());

        Ok(path)
    }
}

impl zed::Extension for UnitFileExtension {
    fn new() -> Self
    where
        Self: Sized,
    {
        UnitFileExtension {
            cached_executable_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let binary_path = self.language_server_binary_path(language_server_id, worktree)?;
        let mut env: Vec<(String, String)> = Default::default();
        env.push(("PATH".to_string(), binary_path.clone()));
        Ok(zed::Command {
            command: binary_path,
            args: vec![],
            env,
        })
    }

    fn language_server_initialization_options(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        Ok(Some(serde_json::json!({
            "in_daemon": true,
        })))
    }

    fn language_server_workspace_configuration(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        let settings = LspSettings::for_worktree(SYSTEMD_LANGUAGE_SERVER_NAME, worktree)
            .ok()
            .and_then(|lsp| lsp.settings.clone())
            .unwrap_or_default();
        Ok(Some(settings))
    }

    fn complete_slash_command_argument(
        &self,
        _command: zed::SlashCommand,
        _args: Vec<String>,
    ) -> Result<Vec<zed::SlashCommandArgumentCompletion>, String> {
        let commands = Vec::new();
        Ok(commands)
    }

    fn run_slash_command(
        &self,
        _command: zed::SlashCommand,
        _args: Vec<String>,
        _worktree: Option<&zed::Worktree>,
    ) -> Result<zed::SlashCommandOutput, String> {
        Err("`run_slash_command` not implemented".to_string())
    }

    fn suggest_docs_packages(&self, _provider: String) -> Result<Vec<String>, String> {
        Ok(Vec::new())
    }

    fn index_docs(
        &self,
        _provider: String,
        _package: String,
        _database: &zed::KeyValueStore,
    ) -> Result<(), String> {
        Err("`index_docs` not implemented".to_string())
    }
    // ...
}

zed::register_extension!(UnitFileExtension);
