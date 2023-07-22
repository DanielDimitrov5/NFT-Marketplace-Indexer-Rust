use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub mod logger {
    use super::*;

    pub fn log_new_line(line: &str) -> std::io::Result<()> {
        let path = Path::new("src/logs/log.txt");
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(&path)?;

        write!(file, "{}", line)?;
        Ok(())
    }
}
