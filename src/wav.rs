use std::collections::HashMap;


#[derive(Debug)]
pub struct WavFile {
    pub sub_chunks: HashMap<SubChunkIds,SubChunk>
}

#[derive(Debug)]
pub struct SubChunk {
    pub id: SubChunkIds,
    pub size: u32,
    pub data: SubChunkData,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum SubChunkIds {
    Riff,
    Fmt,
    Fact,
    Info,
    List,
    Data,
}

#[derive(Debug)]
pub enum SubChunkData {
    Riff(String),
    Fmt(FmtChunkData),
    Fact(u32),
    Info(Vec<u8>),
    List(ListChunkData),
    Data(Vec<u8>),
}

#[derive(Debug)]
pub struct  FmtChunkData {
    pub format_tag:FmtFormatTag,
    pub channels: u16,
    pub sampling_rate: u32,
    pub ave_byte_per_sec: u32,
    pub block_align: u16,
    pub bit_per_sample: u16,
    pub arg_param_1: Option<i32>,
    pub arg_param_2: Option<i32>,
}

#[derive(Debug)]
pub enum FmtFormatTag {
    LinerPCM,  // 1
    ALaw,      // 3
    MuLaw,     // 7
}

#[derive(Debug)]
pub struct ListChunkData {
    pub list_type_id: ListTypeId,
    pub data: ListData,
}

#[derive(Debug)]
pub enum ListTypeId {
    INFO,
}

#[derive(Debug)]
pub enum ListData {
    InfoData,
}

impl WavFile {
    pub fn new(d: Vec<u8>)
        -> Result<WavFile, &'static str>
    {
        let id:[u8;4] = d[0..4].try_into().unwrap();
        let id = String::from_utf8_lossy(&id).to_string();
        assert_eq!("RIFF".to_string(), id);
        WavFile::_parse_wav_file(d)
    }

    fn _parse_wav_file(d: Vec<u8>)
        -> Result<WavFile, &'static str>
    {
        let mut sub_chunks = HashMap::new();
        //for mut offset in 0..d.len() {
        let mut offset = 0;
        while offset < d.len() {
            let id: [u8;4] = d[offset..offset+4].try_into().unwrap();
            offset += 4;
            match std::str::from_utf8(&id).unwrap() {
                "RIFF" => {
                    let size: [u8;4] = d[offset..offset+4].try_into().unwrap();

                    // little endian
                    let size = ((size[3] as u32) << 24) + ((size[2] as u32) << 16) + ((size[1] as u32) << 8) + ((size[0]) as u32 );

                    offset += 4;
                    let data: [u8;4] = d[offset..offset+4].try_into().unwrap();
                    let data = String::from_utf8_lossy(&data).to_string();
                    offset += 4;
                    let riff_chunk = SubChunk {
                        id: SubChunkIds::Riff,
                        size,
                        data: SubChunkData::Riff(data)
                    };
                    sub_chunks.insert(SubChunkIds::Riff, riff_chunk);
                }
                "fmt " => {
                    let size: [u8;4] = d[offset..offset+4].try_into().unwrap();

                    // little endian
                    let size = ((size[3] as u32) << 24) + ((size[2] as u32) << 16) + ((size[1] as u32) << 8) + ((size[0]) as u32 );
                    offset+=4;

                    let fmt_format = ((d[offset+1] as u16) << 8) + (d[offset] as u16);
                    let format_tag = match fmt_format {
                        1 => FmtFormatTag::LinerPCM,
                        3 => FmtFormatTag::ALaw,
                        7 => FmtFormatTag::MuLaw,
                        _ => unimplemented!(),
                    };
                    offset += 2;

                    let channels = (d[offset+1]) as u16;
                    offset += 2; // 0x0100 or 0x0200 二つ目のバイトはみなくていい

                    let sampling_rate= ((d[offset+3] as u32) << 24) + ((d[offset+2] as u32) << 16) + ((d[offset+1] as u32) << 8) + ((d[0]) as u32 );
                    offset += 4;

                    let ave_byte_per_sec = ((d[offset+3] as u32) << 24) + ((d[offset+2] as u32) << 16) + ((d[offset+1] as u32) << 8) + ((d[0]) as u32 );
                    offset += 4;

                    let block_align = ((d[offset+1] as u16) << 8) + ((d[0]) as u16 );
                    offset += 2;

                    let bit_per_sample = ((d[offset+1] as u16) << 8) + ((d[0]) as u16 );
                    offset += 2;

                    if size > 16 {
                        offset+=4;
                    }

                    let fmt_chunk_data = FmtChunkData{
                        format_tag,
                        channels,
                        sampling_rate,
                        ave_byte_per_sec,
                        block_align,
                        bit_per_sample,
                        arg_param_1: None,
                        arg_param_2: None,
                    };

                    let fmt_chunk = SubChunk {
                        id: SubChunkIds::Fmt,
                        size,
                        data: SubChunkData::Fmt(fmt_chunk_data)
                    };
                    sub_chunks.insert(SubChunkIds::Fmt, fmt_chunk);
                }
                "fact" => {
                    let size = ((d[offset+3] as u32) << 24) + ((d[offset+2] as u32) << 16) + ((d[offset+1] as u32) << 8) + ((d[offset]) as u32 );
                    offset+=4;
                    let dw_sample_length = ((d[offset+3] as u32) << 24) + ((d[offset+2] as u32) << 16) + ((d[offset+1] as u32) << 8) + ((d[offset]) as u32 );
                    offset+=4;
                    let fact_chunk = SubChunk {
                        id:SubChunkIds::Fact,
                        size,
                        data: SubChunkData::Fact(dw_sample_length)
                    };
                    sub_chunks.insert(SubChunkIds::Fact, fact_chunk);
                }
                "LIST" => {
                    let size = ((d[offset+3] as u32) << 24) + ((d[offset+2] as u32) << 16) + ((d[offset+1] as u32) << 8) + ((d[offset]) as u32 );
                    offset+=4;

                    let b:[u8;4] = d[offset..offset+4].try_into().unwrap();
                    let list_type_id = match std::str::from_utf8(&b).unwrap() {
                        "INFO" => ListTypeId::INFO,
                        _ => unimplemented!(),
                    };
                    offset+=4;

                    let b = ListData::InfoData{};
                    offset+= size as usize - 4;

                    let list_chunk_data = ListChunkData{
                        list_type_id,
                        data: b
                    };

                    let list_chunk = SubChunk {
                        id: SubChunkIds::List,
                        size,
                        data: SubChunkData::List(list_chunk_data)
                    };
                    sub_chunks.insert(SubChunkIds::List, list_chunk);
                }
                "INFO" => {
                    let size = ((d[offset+3] as u32) << 24) + ((d[offset+2] as u32) << 16) + ((d[offset+1] as u32) << 8) + ((d[offset]) as u32 );
                    offset+=4;
                    let data : Vec<u8> = d[offset..offset+size as usize].try_into().unwrap();
                    offset += size as usize;
                    let info_chunk = SubChunk {
                        id: SubChunkIds::Info,
                        size,
                        data: SubChunkData::Info(data)
                    };
                    sub_chunks.insert(SubChunkIds::Info, info_chunk);
                }
                "data" => {
                    let size = ((d[offset+3] as u32) << 24) + ((d[offset+2] as u32) << 16) + ((d[offset+1] as u32) << 8) + ((d[offset]) as u32 );
                    offset+=4;
                    let data : Vec<u8> = d[offset..d.len()].try_into().unwrap();
                    let data_chunk = SubChunk {
                        id: SubChunkIds::Data,
                        size,
                        data: SubChunkData::Data(data)
                    };
                    sub_chunks.insert(SubChunkIds::Data, data_chunk);
                    break;
                }
                _ => {
                    // ignore unkown chunks
                    eprintln!("error ignore unkown chunks");
                    let size = ((d[offset+3] as u32) << 24) + ((d[offset+2] as u32) << 16) + ((d[offset+1] as u32) << 8) + ((d[offset]) as u32 );
                    offset += size as usize;
                },
            };
        }
        Ok(WavFile {
            sub_chunks
        })
    }

    fn _determine_id(){}
}


#[cfg(test)]
mod wav_file_test {

}