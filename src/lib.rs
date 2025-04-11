use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Output},
};

#[cfg(target_os = "macos")]
const CLI_BYTES: &[u8] = include_bytes!("../bin/llama-gbnf-validator");

#[cfg(target_os = "macos")]
fn extract_cli_binary() -> std::io::Result<(PathBuf, tempfile::TempDir)> {
    let file_name = "llama-gbnf-validator";

    let dir = tempfile::tempdir()?;
    let path = dir.path().join(file_name);

    let mut file = fs::File::create(&path)?;
    file.write_all(CLI_BYTES)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms)?;
    }

    Ok((path, dir))
}

#[cfg(not(target_os = "macos"))]
fn extract_cli_binary() -> std::io::Result<(PathBuf, tempfile::TempDir)> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "llama-gbnf-validator is not available on this platform",
    ))
}

fn run_embedded_cli(args: &[&str]) -> std::io::Result<Output> {
    let (cli_path, _temp_dir) = extract_cli_binary()?;
    Command::new(&cli_path).args(args).output()
}

pub fn llama_gbnf_validator(
    grammar: impl AsRef<str>,
    input: impl AsRef<str>,
) -> anyhow::Result<bool> {
    let dir = tempfile::tempdir()?;
    let grammar_path = dir.path().join("grammar.txt");
    let input_path = dir.path().join("input.txt");

    fs::write(&grammar_path, grammar.as_ref())?;
    fs::write(&input_path, input.as_ref())?;

    let output = run_embedded_cli(&[
        &grammar_path.to_string_lossy(),
        &input_path.to_string_lossy(),
    ])?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(!stdout.contains("invalid"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert!(llama_gbnf_validator("root ::= \"a\"", "a").unwrap());
        assert!(!llama_gbnf_validator("root ::= \"a\"", "b").unwrap());
    }
}
