pub mod io {
    use encoding_rs::*;
    use encoding_rs_io::DecodeReaderBytesBuilder;
    use std::collections::HashMap;
    use std::fs::File;
    use std::env;
    use std::io;
    use std::io::prelude::*;
    use std::io::BufReader;
    fn get_encoding<'a>(encoding : &str) -> Option<&'a Encoding> {
        match encoding {
            "WINDOWS_1252" => {
                Some(WINDOWS_1252)
            }
            _ => {
                Some(UTF_8)
            }
        }
    }
    fn get_path_plus_fname(file_name: &str) -> String {
        let cd = env::current_dir().unwrap();
        let val = format!("{}\\{}", cd.display(), file_name);
        val
    }
    pub fn read_file(file_name: &str, encoding: &str) -> io::Result<Vec<u8>> {
        let mut buffer = Vec::new();
        let mut _x = &File::open(file_name)?;
        let mut _reader = BufReader::new(_x);
        let mut transcoded = DecodeReaderBytesBuilder::new()
            .encoding(get_encoding(encoding))
            .build(_reader);
        transcoded.read_to_end(&mut buffer);
        Ok(buffer)
    }

    /**
     * This will append contents to file.
     * */
    pub fn write_file(file_name: &str, contents: &mut String, encoding : &str) -> io::Result<()> {
        match read_file(file_name, encoding) {
            Ok(prior_content) => {
                let mut file = File::create(file_name).ok().expect(format!("Could not open file in write mode: {}.", file_name).as_str());
                let mut buffer_slice = contents.as_bytes();
                let mut read_buffer_slice = prior_content.as_slice();
                file.write_all(read_buffer_slice)?;
                file.write_all(buffer_slice)?;
            }
            Err(e) => {
                println!("Warning... file could not be read: {:?}", e);
                let mut file = File::create(file_name).ok().expect(format!("Could not create or open file in write mode: {}.", file_name).as_str());
                let mut buffer_slice = contents.as_bytes();
                file.write_all(buffer_slice)?;
            }
        }
        Ok(())
    }
}