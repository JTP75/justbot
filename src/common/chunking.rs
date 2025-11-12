pub fn chunk_files_configured(texts: &mut Vec<String>, paths: &mut Vec<String>) -> () {
    chunk_fixed(texts, paths)
}

fn chunk_fixed(texts: &mut Vec<String>, paths: &mut Vec<String>) -> () {
    let mut i=0;
    let thresh = 10000; // ~2500 tokens

    while i < texts.len() {
        if texts[i].len() > thresh {
            let text = texts.remove(i);
            let path = paths.remove(i);

            let chunks: Vec<String> = text.as_bytes()
                .chunks(thresh)
                .map(|chunk| String::from_utf8_lossy(chunk).into())
                .collect();
            let count = chunks.len();

            for (offset,chunk) in chunks.into_iter().enumerate() {
                texts.insert(i+offset, chunk);
                paths.insert(i+offset, format!("{}#{}", path, offset))
            }

            i += count;
        } else {
            i += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::*;

    // 41000 lines
    const BOOK_PATH: &str = "/home/pacel/misc/rust/rustbot/.ignore/book6.txt";

    #[test]
    fn test_chunk_a_big_ahh_file_fixed() {
        let text = fs::read_to_string(PathBuf::from(BOOK_PATH))
            .expect("failed to read file");
        println!("len = {}", text.len());

        let mut texts = vec![text];
        let mut paths = vec![BOOK_PATH.to_owned()];

        chunk_fixed(&mut texts, &mut paths);

        println!("chunks = {}", texts.len());
        println!("paths:\n\t{}", paths.join("\n\t"));
    }
}