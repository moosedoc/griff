
type DataResult = Result<ChunkData, ChunkError>;
type ChunkResult = Result<Chunk, ChunkError>;
type FourCC<'a> = &'a [u8;4];
type ByteStream = Vec<u8>;
type DataExtract<'a> = (ChunkId, usize, &'a [u8]);

#[derive(PartialEq, Debug, Clone, Default)]
pub enum ChunkId {
    #[default]
    NoId,

    Meta,
    Stri,
    Symb,
    Refs,
    Rela,
    Srcs,
    Cmdl,
    Riff,
    CdIx,
}
impl ChunkId {
    pub fn match_id<'a>(id: FourCC) -> ChunkId {
        match id {
            META => ChunkId::Meta,
            STRI => ChunkId::Stri,
            SYMB => ChunkId::Symb,
            REFS => ChunkId::Refs,
            RELA => ChunkId::Rela,
            SRCS => ChunkId::Srcs,
            CMDL => ChunkId::Cmdl,
            RIFF => ChunkId::Riff,
            CDIX => ChunkId::CdIx,

            _ => panic!("Unknown FourCC!")
        }
    }
}

const RIFF: FourCC = b"RIFF";
const META: FourCC = b"meta";
const STRI: FourCC = b"stri";
const SYMB: FourCC = b"symb";
const REFS: FourCC = b"refs";
const RELA: FourCC = b"rela";
const SRCS: FourCC = b"srcs";
const CMDL: FourCC = b"cmdl";

pub const CDIX: FourCC = b"CdIx";

#[derive(Debug, PartialEq)]
pub enum ChunkError {
    BadHeader,
    CorruptId,
    NotRiffFile,
    IncompatibleFile,
}

#[derive(Debug, Clone, Default)]
pub struct Chunk {
    pub id: ChunkId,
    pub data: ChunkData,
}

#[derive(Debug, Clone, Default)]
pub struct ChunkRiff {
    pub file_type: [u8; 4],
    pub data: Vec<Chunk>,
}

#[derive(Debug, Clone)]
pub struct ChunkStream {
    pub data: ByteStream,
}

#[derive(Debug, Clone, Default)]
pub enum ChunkData {
    #[default]
    NoData,

    RiffData(ChunkRiff),
    StreamData(ChunkStream)
}

pub enum CDFromError {
    NoDataError
}

#[derive(Debug, Clone)]
pub struct ChunkMeta {
    pub version: [u8; 4],
}

#[derive(Debug)]
pub struct Riff {
    pub chunk: Option<Chunk>,
}

impl Riff {
    pub async fn parse<'a>(data: &'a [u8]) -> Result<Riff, ChunkError> {
        let mut riff: Riff = Riff{ chunk: None };
        let c = Riff::parse_chunk(data);
        if c.is_err() {
            return Err(c.unwrap_err());
        }
        riff.chunk = Some(c.unwrap());
        Ok(riff)
    }

    fn parse_chunk<'a>(data: &'a [u8]) -> ChunkResult {
        let mut chunk: Chunk = Default::default();
        let result = Riff::extract_data(data);
        if result.is_err() {
            return Err(result.unwrap_err());
        }
        let (id, _, cdata) = result.unwrap();
        chunk.id = id;

        let dr = match chunk.id {
            ChunkId::Riff => Riff::parse_riff(&cdata),
            _ => todo!(),
        };
        if dr.is_err() {
            return Err(dr.unwrap_err());
        }
        chunk.data = dr.unwrap();

        Ok(chunk)
    }

    fn parse_riff<'a>(data: &'a [u8]) -> DataResult {
        let mut c_riff: ChunkRiff = Default::default();
        c_riff.file_type = data.get(0..4).unwrap().try_into().unwrap();
        if c_riff.file_type != *CDIX {
            return Err(ChunkError::IncompatibleFile);
        }
        
        let mut cursor: usize = 4;
        loop {
            let mut chunk: Chunk = Default::default();
            let result = Riff::extract_data(data.get(cursor..).unwrap());
            if result.is_err() {
                return Err(result.unwrap_err());
            }
            let (id, mut len, cdata) = result.unwrap();
            chunk.id = id;
            let dr = match chunk.id {
                _ => Riff::parse_generic(cdata, len),
            };

            if len % 2 == 1 {
                len += 1;
            }
            cursor += len + 8;
            chunk.data = dr?;
            c_riff.data.push(chunk);
            if cursor >= data.len() {
                break;
            }
        }
        Ok(ChunkData::RiffData(c_riff))
    }

    fn parse_generic<'a>(data: &'a [u8], len: usize) -> DataResult {
        let cd: ChunkData = ChunkData::StreamData(
                                            ChunkStream {
                                                data: data.get(..len).unwrap().to_vec()
                                            });
        Ok(cd)
    }

    fn extract_data<'a>(data: &'a [u8]) -> Result<DataExtract, ChunkError> {
        let _id = data.get(0..4).clone();
        if _id.is_none() {
            return Err(ChunkError::CorruptId);
        }
        let id = ChunkId::match_id(_id.unwrap().try_into().unwrap());
        let _len = data.get(4..8);
        if _len.is_none() {
            return Err(ChunkError::BadHeader);
        }
        let len = Riff::get_usize(_len.unwrap().try_into().unwrap());

        Ok((id, len, data.get(8..).unwrap()))
    }

    pub fn get_usize(d: [u8;4]) -> usize {
        let us: usize = ((d[3] as u32) << 24 | (d[2] as u32) << 16 | (d[1] as u32) << 8 | d[0] as u32) as usize;
        us
    }
}
