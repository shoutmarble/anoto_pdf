pub mod make_plots;
pub mod persist_json;
pub mod anoto_matrix;
pub mod pdf_dotpaper;
pub mod decode_utils;
pub mod codec;
pub mod controls;
pub mod fonts;

pub use anoto_matrix::{gen_matrix, gen_matrix_from_json, generate_matrix_only, save_generated_matrix, load_matrix_from_json, load_matrix_from_txt, save_matrix_from_json, extract_6x6_section};
pub use decode_utils::decode_position;
pub use codec::anoto_6x6_a4_fixed;
