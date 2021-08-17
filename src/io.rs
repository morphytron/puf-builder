pub mod io {
    use encoding_rs::*;
    use encoding_rs_io::DecodeReaderBytesBuilder;
    use std::collections::HashMap;
    use std::fs::File;
    use std::io;
    use std::io::prelude::*;
    use std::io::BufReader;
    use crate::EncodingTypes;
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
    pub fn read_file(file_name: &str, encoding: &str) -> io::Result<Vec<u8>> {
        let mut buffer = Vec::new();
        let mut _x = &File::open(file_name)?;
        let mut _reader = BufReader::new(_x);
        let transcoded = DecodeReaderBytesBuilder::new()
            .encoding(get_encoding(encoding))
            .build(_x);
        _reader.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    /**
     * This will append contents to file.
     * */
    pub fn write_file(file_name: &str, contents: &mut String, encoding : &str) -> io::Result<()> {
        File::create(file_name)?;
        let mut file_string = read_file(file_name, encoding).unwrap();
        let mut file = File::create(file_name)?;
        //let array = *contents.as_mut_vec();
        unsafe {
            file_string.extend(contents.as_mut_vec().iter());
            file.write_all(file_string.as_slice())?;
        }
        Ok(())
    }
}