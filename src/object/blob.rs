use flate2::read::ZlibDecoder;
use std::io::Read;

pub(crate) struct Blob {
    pub(crate) raw_content: Vec<u8>,
}

impl Blob {
    pub(crate) fn from_packfile_compressed_data(data: &[u8]) -> (Self, usize) {
        let mut decompressor = ZlibDecoder::new(data);
        let mut decompressed_data = Vec::new();
        decompressor.read_to_end(&mut decompressed_data).unwrap();

        (
            Self {
                raw_content: decompressed_data,
            },
            decompressor.total_in() as usize,
        )
    }

    pub(crate) fn to_object_file_representation(&self) -> Vec<u8> {
        let header = format!("blob {}\0", self.raw_content.len());

        vec![header.into_bytes(), self.raw_content.clone()].concat()
    }
}
