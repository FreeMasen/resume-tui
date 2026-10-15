use std::path::PathBuf;

fn main() {
    let out_path = PathBuf::from(std::env::args().nth(1).as_deref().unwrap_or("resume.pdf"));
    to_pdf::generate_pdf(out_path).unwrap();
}
