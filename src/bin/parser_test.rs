//! Debug: call parse_conversation_file directly and report
use star::language_model::train::parse_conversation_file;
use std::path::Path;

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| "personal_training.txt".to_string());
    let seqs = parse_conversation_file(Path::new(&path));
    println!("file: {}", path);
    println!("sequences: {}", seqs.len());
    if !seqs.is_empty() {
        println!("first seq length: {}", seqs[0].len());
        println!("first seq preview: {:.200}", seqs[0]);
        if seqs.len() > 1 {
            println!("second seq length: {}", seqs[1].len());
            println!("second seq preview: {:.200}", seqs[1]);
        }
        // find any sequence shorter than 200 chars -- suspicious
        let shorts: Vec<usize> = (0..seqs.len()).filter(|&i| seqs[i].len() < 200).collect();
        println!("sequences shorter than 200 chars: {} (indices: {:?})", shorts.len(), &shorts[..shorts.len().min(10)]);
        // total chars
        let total: usize = seqs.iter().map(|s| s.len()).sum();
        println!("total chars across all sequences: {}", total);
    }
}