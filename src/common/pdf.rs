use std::{path::Path, process::Command};

/// Uses `pdftotext` to extract text from a PDF file
/// 
/// - this method is not portable and probably wont work outside of linux
pub fn extract_pdf_text(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("pdftotext")
        .arg("-layout")
        .arg(path)
        .arg("-")  // output to stdout
        .output()?;
        
    if !output.status.success() {
        return Err(format!(
            "pdftotext failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ).into());
    }
    
    Ok(String::from_utf8(output.stdout)?)
}