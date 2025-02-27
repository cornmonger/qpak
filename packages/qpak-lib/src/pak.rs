// SPDX-License-Identifier: MIT
//! Quake PAK archive manipulation.
use std::{fs::File, io::{BufReader, BufWriter, Read, Seek, Write}, path::Path};
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};
use std::path::PathBuf;
use tokio::{fs::File as AsyncFile, io::{AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt, AsyncWrite, AsyncWriteExt, BufReader as AsyncBufReader, BufWriter as AsyncBufWriter,SeekFrom}};
use crate::{Error, Result};

/// The chunk size to use when reading and writing files.
const IO_BYTE_SIZE: usize = 8388608; // 8MiB

/// The header of a PAK archive describes where the table is located in the file and its size.
#[derive(Debug)]
pub struct Header {
    /// Where the table is stored. Stored in little-endian.
    table_offset: u32,
    /// The size of the table, including entries. Stored in little-endian.
    table_size: u32,
}

impl Header {
    pub(crate) const PAK_FILE_DESIGNATOR: [u8; 4] = [b'P', b'A', b'C', b'K'];

    pub fn new(table_offset: u32, table_size: u32) -> Self {
        Header { table_offset, table_size }
    }

    /// Reads a header from a (buffered) reader.
    pub async fn read<R>(reader: &mut R) -> Result<Self>
    where
        R: AsyncRead + AsyncSeek + Unpin,
    {
        let mut magic = [0u8; 4];
        reader.read(&mut magic).await?;
        if magic != Self::PAK_FILE_DESIGNATOR {
            return Err(Error::InvalidMagicNumber(magic));
        }

        let table_offset = reader.read_u32_le().await?;
        let table_size = reader.read_u32_le().await?;

        Ok(Header::new(table_offset, table_size))
    }

    /// Reads a header from a (buffered) reader.
    pub fn read_sync<R>(reader: &mut R) -> Result<Self>
    where
        R: Read + Seek
    {
        let mut magic = [0u8; 4];
        reader.read(&mut magic)?;
        if magic != Self::PAK_FILE_DESIGNATOR {
            return Err(Error::InvalidMagicNumber(magic));
        }

        let table_offset = reader.read_u32::<LittleEndian>()?;
        let table_size = reader.read_u32::<LittleEndian>()?;

        Ok(Header::new(table_offset, table_size))
    }

    /// Writes the header to a (buffered) writer.
    pub async fn write<R>(&self, writer: &mut R) -> Result<()>
    where
        R: AsyncWrite + Unpin
    {
        writer.write_all(&Self::PAK_FILE_DESIGNATOR).await?;
        writer.write_u32_le(self.table_offset).await?;
        writer.write_u32_le(self.table_size).await?;
        Ok(())
    }

    /// Writes the header to a (buffered) writer.
    pub fn write_sync<R>(&self, writer: &mut R) -> Result<()>
    where
        R: Write
    {
        writer.write_all(&Self::PAK_FILE_DESIGNATOR)?;
        writer.write_u32::<LittleEndian>(self.table_offset)?;
        writer.write_u32::<LittleEndian>(self.table_size)?;
        Ok(())
    }

    /// Where the table is stored in the PAK file
    pub fn table_offset(&self) -> u32 {
        self.table_offset
    }

    /// The size of the table, including entries
    pub fn table_size(&self) -> u32 {
        self.table_size
    }
}

/// PAK tables act as an index of all files in the archive.
/// Each [TableEntry] points to its corresponding file data.
#[derive(Debug)]
pub struct Table {
    entries: Vec<TableEntry>,
}

impl Table {
    pub fn new(entries: Vec<TableEntry>) -> Self {
        Table { entries }
    }

    /// Reads a table and its entries from a (buffered) reader.
    pub async fn read<R>(reader: &mut R, header: &Header) -> Result<Self>
    where
        R: AsyncRead + AsyncSeek + Unpin
    {
        reader.seek(SeekFrom::Start(header.table_offset as u64)).await?;

        let mut entries = Vec::new();
        for _ in 0..header.table_size / TableEntry::SIZE as u32 {
            let entry = TableEntry::read(reader).await?;
            entries.push(entry);
        }

        Ok(Table::new(entries))
    }

    /// Reads a table and its entries from a (buffered) reader.
    pub fn read_sync<R>(reader: &mut R, header: &Header) -> Result<Self>
    where
        R: Read + Seek
    {
        reader.seek(SeekFrom::Start(header.table_offset as u64))?;

        let mut entries = Vec::new();
        for _ in 0..header.table_size / TableEntry::SIZE as u32 {
            let entry = TableEntry::read_sync(reader)?;
            entries.push(entry);
        }

        Ok(Table::new(entries))
    }

    /// Retrieves the [TableEntry] items
    pub fn entries(&self) -> &Vec<TableEntry> {
        &self.entries
    }

    /// TRUE if the table contains an entry with the given path
    pub fn contains<P: AsRef<Path>>(&self, path: P) -> bool {
        let path = path.as_ref().to_string_lossy();
        self.entries.iter().any(|entry| entry.path == path)
    }

    /// Writes the table and its entries to a (buffered) writer.
    pub async fn write<W>(&self, writer: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        for entry in &self.entries {
            entry.write(writer).await?;
        }

        Ok(())
    }

    /// Writes the table and its entries to a (buffered) writer.
    pub fn write_sync<W>(&self, writer: &mut W) -> Result<()>
    where
        W: Write,
    {
        for entry in &self.entries {
            entry.write_sync(writer)?;
        }

        Ok(())
    }
}

/// A single entry in a PAK [Table].
/// Each entry is [TableEntry::PATH_SIZE] (64 bytes) in size.
#[derive(Debug, Clone)]
pub struct TableEntry {
    /// Stored as a null-terminated [TableEntry::PATH_SIZE] UTF8 string. Paths do not use '/' for root.
    path: String,
    /// Where the item file is stored in the PAK file. Stored in little-endian.
    offset: u32,
    /// The size of the item file in bytes. Stored in little-endian.
    size: u32,
}

impl TableEntry {
    /// The fixed size of [TableEntry::path] (56 bytes)
    const PATH_SIZE: usize = 56;
    /// The fixed size of a table entry (64 bytes)
    const SIZE: usize = Self::PATH_SIZE + size_of::<u32>() + size_of::<u32>();

    /// Will throw an [Error::FileNameTooLong] if the path is greater than [TableEntry::PATH_SIZE]
    pub fn new(path: String, offset: u32, size: u32) -> Result<Self> {
        if path.len() > Self::PATH_SIZE {
            return Err(Error::FilenameTooLong(path));
        }

        Ok(TableEntry { path, offset, size })
    }

    /// Reads a table entry from a (buffered) reader.
    pub async fn read<R>(reader: &mut R) -> Result<Self>
    where
        R: AsyncRead + Unpin
    {
        let mut path = [0u8; Self::PATH_SIZE];
        reader.read(&mut path).await?;
        let path_end = path.iter()
            .position(|&b| b == 0)
            .ok_or(Error::FilenameTooLong(String::from_utf8_lossy(&path).into_owned()))?;
        let path = String::from_utf8(path[0..path_end].to_vec())
            .map_err(|e| Error::NonUtf8Filename(e))?;

        let offset = reader.read_u32_le().await?;
        let size = reader.read_u32_le().await?;

        Ok(TableEntry::new(path, offset, size)?)
    }

    /// Reads a table entry from a (buffered) reader.
    pub fn read_sync<R>(reader: &mut R) -> Result<Self>
    where
        R: Read
    {
        let mut path = [0u8; Self::PATH_SIZE];
        reader.read(&mut path)?;
        let path_end = path.iter()
            .position(|b| *b == 0)
            .ok_or(Error::FilenameTooLong(String::from_utf8_lossy(&path).into_owned()))?;
        let path = String::from_utf8(path[0..path_end].to_vec())
            .map_err(|e| Error::NonUtf8Filename(e))?;

        let offset = reader.read_u32::<LittleEndian>()?;
        let size = reader.read_u32::<LittleEndian>()?;

        Ok(TableEntry::new(path, offset, size)?)
    }

    /// Retrieves the path of a table entry as a String.
    /// Its stored format is a fixed 56 byte null-terminated UTF-8 string.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Where the table entry file data is located in the file.
    pub fn offset(&self) -> u32 {
        self.offset
    }

    /// The size of the table entry file's data.
    pub fn size(&self) -> u32 {
        self.size
    }

    /// Writes the table entry to a (buffered) writer.
    pub async fn write<W>(&self, writer: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin
    {

        let mut path = [0u8; Self::PATH_SIZE];
        for (i, byte) in self.path.as_bytes().iter().enumerate() {
            path[i] = *byte;
        }

        writer.write_all(&path).await?;
        writer.write_u32_le(self.offset).await?;
        writer.write_u32_le(self.size).await?;
        Ok(())
    }

    /// Writes the table entry to a (buffered) writer.
    pub fn write_sync<W>(&self, writer: &mut W) -> Result<()>
    where
        W: Write,
    {
        let mut path = [0u8; Self::PATH_SIZE];
        for (i, byte) in self.path.as_bytes().iter().enumerate() {
            path[i] = *byte;
        }

        writer.write_all(&path)?;
        writer.write_u32::<LittleEndian>(self.offset)?;
        writer.write_u32::<LittleEndian>(self.size)?;
        Ok(())
    }
}

/// Represents the header, table, and table directory of a PAK file.
#[derive(Debug)]
pub struct PakManifest {
    header: Header,
    table: Table,
}

impl PakManifest {
    /// The default offset for the table directory when stored at the end of the PAK file as is typical (and standard in this lib).
    const ITEMS_OFFSET: u32 = ((Header::PAK_FILE_DESIGNATOR.len() * size_of::<u8>()) + size_of::<u32>() + size_of::<u32>()) as u32;

    pub fn new(header: Header, table: Table) -> Self {
        PakManifest { header, table }
    }

    /// Reads a PAK manifest from an (buffered) reader.
    pub async fn read<R>(reader: &mut R) -> Result<Self>
    where
        R: AsyncRead + AsyncSeek + Unpin
    {
        let header = Header::read(reader).await?;
        let table = Table::read(reader, &header).await?;
        Ok(Self::new(header, table))
    }

    /// Reads a PAK manifest from a (buffered) reader.
    pub fn read_sync<R>(reader: &mut R) -> Result<Self>
    where
        R: Read + Seek
    {
        let header = Header::read_sync(reader)?;
        let table = Table::read_sync(reader, &header)?;
        Ok(Self::new(header, table))
    }

    /// Generates a PAK manifest from a directory.
    /// Throws [Error::NonUtf8Path], [Error::FilenameTooLong]
    pub fn from_dir_sync<P: AsRef<Path>>(input_dir: P) -> Result<Self> {
        let files = walk_pak_dir_sync(input_dir)?;
        Self::from_walk_results(files)
    }

    /// Generates a PAK manifest from a directory.
    /// Throws [Error::NonUtf8Path], [Error::FilenameTooLong]
    pub async fn from_dir<P: AsRef<Path>>(input_dir: P) -> Result<Self> {
        let files = walk_pak_dir(input_dir).await?;
        Self::from_walk_results(files)
    }

    pub fn from_walk_results(files: Vec<(PathBuf, u64)>) -> Result<Self> {
        let mut total_size = 0;
        for (_, size) in &files {
            total_size += size;
        }

        let table_size = (files.len() * TableEntry::SIZE) as u32;
        let header = Header::new(Self::ITEMS_OFFSET + total_size as u32, table_size);

        let mut table_entries = Vec::with_capacity(files.len());
        let mut entry_offset = Self::ITEMS_OFFSET;
        for (path, size) in files {
            let path = path.to_str()
                .ok_or_else(|| Error::NonUtf8Path(path.clone()))?
                .to_string();
            let size = size as u32;
            let entry = TableEntry::new(path, entry_offset, size)?;
            table_entries.push(entry);
            entry_offset += size;
        }

        let table = Table::new(table_entries);
        Ok(Self::new(header, table))
    }

    // Returns the header of a PAK, containing offsets.
    pub fn header(&self) -> &Header {
        &self.header
    }

    /// Returns the table (index) of a PAK
    pub fn table(&self) -> &Table {
        &self.table
    }

    /// Returns the table entries of a PAK, including filepath, size, and offset of each item.
    pub fn table_entries(&self) -> &Vec<TableEntry> {
        &self.table.entries
    }
}

/// Represents a PAK file by filepath and [PakManifest]
/// Provides methods to iterate over file contents.
#[derive(Debug)]
pub struct PakFile {
    /// The filepath of the PAK file
    filepath: PathBuf,
    /// The manifest of the PAK file, including header, table, and table entries.
    manifest: PakManifest
}

impl PakFile {
    pub fn new(filepath: PathBuf, manifest: PakManifest) -> Self {
        PakFile { filepath, manifest }
    }

    /// Constructs from a PAK file.
    /// Throws [Error::OpenPak], [Error::ReadPak]
    pub async fn from_file<P>(filepath: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let filepath = PathBuf::from(filepath.as_ref());
        let file = AsyncFile::open(&filepath).await
            .map_err(|e| Error::OpenPak(e))?;

        let mut reader = AsyncBufReader::new(file);
        let manifest = PakManifest::read(&mut reader).await
            .map_err(|e| Error::ReadPak(filepath.to_path_buf(), e.to_string()))?;

        Ok(Self::new(filepath, manifest))
    }

    /// Constructs from a PAK file.
    /// Throws [Error::OpenPak], [Error::ReadPak]
    pub fn from_file_sync<P>(filepath: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let filepath = PathBuf::from(filepath.as_ref());
        let file = File::open(&filepath)
            .map_err(|e| Error::OpenPak(e))?;

        let mut reader = BufReader::new(file);
        let manifest = PakManifest::read_sync(&mut reader)
            .map_err(|e| Error::ReadPak(filepath.to_path_buf(), e.to_string()))?;

        Ok(Self::new(filepath, manifest))
    }

    /// Creates a PAK file by copying files from a directory. The manifest for the directory must already have been generated using [PakManifest::from_dir].
    pub async fn create_from_dir<P>(input_dir: P, manifest: PakManifest, output_filepath: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let out_file = AsyncFile::create(output_filepath.as_ref()).await?;
        let mut writer = AsyncBufWriter::new(out_file);

        manifest.header.write(&mut writer).await?;

        for entry in manifest.table.entries.iter() {
            let input_path = input_dir.as_ref().join(&entry.path);
            let in_file = AsyncFile::open(&input_path).await?;
            let mut reader = AsyncBufReader::new(in_file);

            let mut size_remaining = entry.size as usize;
            while size_remaining > 0 {
                let chunk_size = std::cmp::min(size_remaining, IO_BYTE_SIZE);
                let mut chunk = vec![0; chunk_size];
                reader.read_exact(&mut chunk).await?;
                writer.write_all(&chunk).await?;
                size_remaining -= chunk_size;
            }
        }

        manifest.table.write(&mut writer).await?;
        writer.flush().await?;

        Ok(Self::new(PathBuf::from(output_filepath.as_ref()), manifest))
    }

    /// Creates (writes) a PAK file by copying files from a directory. The manifest for the directory must already have been generated using [PakManifest::from_dir_sync].
    pub fn create_from_dir_sync<P>(input_dir: P, manifest: PakManifest, output_filepath: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let out_file = File::create(output_filepath.as_ref())?;
        let mut writer = BufWriter::new(out_file);

        manifest.header.write_sync(&mut writer)?;

        for entry in manifest.table.entries.iter() {
            let input_path = input_dir.as_ref().join(&entry.path);
            let input_file = File::open(&input_path)?;
            let mut reader = BufReader::new(input_file);

            let mut size_remaining = entry.size as usize;
            while size_remaining > 0 {
                let chunk_size = std::cmp::min(size_remaining, IO_BYTE_SIZE);
                let mut chunk = vec![0; chunk_size];
                reader.read_exact(&mut chunk)?;
                writer.write_all(&chunk)?;
                size_remaining -= chunk_size;
            }
        }

        manifest.table.write_sync(&mut writer)?;
        writer.flush()?;

        Ok(Self::new(PathBuf::from(output_filepath.as_ref()), manifest))
    }

    /// Extracts the contents of the PAK file to the specified directory.
    /// Throws [Error::CreateDirectory], [Error::OpenPak], [Error::WritePak]
    pub fn extract_sync<P: AsRef<Path>>(&self, dest_dir: P) -> Result<()> {
        for pak_item in self.read_items_sync()? {
            let pak_item = pak_item?;

            let mut path = std::path::PathBuf::from(dest_dir.as_ref());
            path.push(&pak_item.table_entry().path());

            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| Error::CreateDirectory(e))?;
            }

            let file = File::create(&path)
                .map_err(|e| Error::OpenPak(e))?;

            let mut writer = BufWriter::new(file);
            writer.write_all(pak_item.data().as_ref())
                .map_err(|e| Error::WritePak(e))?;

            writer.flush()?;
        }

        Ok(())
    }

    /// Extracts the contents of the PAK file to the specified directory.
    /// Throws [Error::CreateDirectory], [Error::OpenPak], [Error::WritePak]
    pub async fn extract<P: AsRef<Path>>(&self, dest_dir: P) -> Result<()> {
        use tokio_stream::StreamExt;
        let mut items = std::pin::pin!(self.read_items());
        while let Some(pak_item) = items.try_next().await? {
            let mut path = std::path::PathBuf::from(dest_dir.as_ref());
            path.push(&pak_item.table_entry().path());

            if let Some(parent) = path.parent() {
                tokio::fs::create_dir_all(parent).await
                    .map_err(|e| Error::CreateDirectory(e))?;
            }

            let file = AsyncFile::create(&path).await
                .map_err(|e| Error::OpenPak(e))?;

            let mut writer = AsyncBufWriter::new(file);
            writer.write_all(pak_item.data().as_ref()).await
                .map_err(|e| Error::WritePak(e))?;

            writer.flush().await
                .map_err(|e| Error::WritePak(e))?;
        }

        Ok(())
    }

    /// Returns an iterator over each file item in the PAK, including data.
    /// Throws [Error::OpenPak], [Error::ReadPak]
    pub fn read_items<'p>(&'p self) -> impl tokio_stream::Stream<Item = Result<PakItem<'p>>> {
        async_stream::try_stream!{
            let table_entries = &self.manifest.table.entries;
            let file = AsyncFile::open(&self.filepath).await
                .map_err(|e| Error::OpenPak(e))?;
            let mut reader = AsyncBufReader::new(file);

            for i in 0..table_entries.len() {
                let table_entry = table_entries.get(i).unwrap();
                reader.seek(SeekFrom::Start(table_entry.offset as u64)).await
                    .map_err(|e| Error::ReadPak(self.filepath.to_path_buf(), e.to_string()))?;

                let mut data: Vec<u8> = Vec::with_capacity(table_entry.size as usize);
                (&mut reader)
                    .take(table_entry.size as u64)
                    .read_to_end(&mut data).await
                        .map_err(|e| Error::ReadPak(self.filepath.to_path_buf(), e.to_string()))?;

                let item = PakItem { table_entry, data };
                yield item;
            }
        }
    }

    /// Returns an iterator over each file item in the PAK, including data.
    /// Throws [Error::OpenPak], [Error::ReadPak]
    pub fn read_items_sync<'p>(&'p self) -> Result<impl Iterator<Item = Result<PakItem<'p>>>> {
        let table_entries = &self.manifest.table.entries;
        let file = File::open(&self.filepath)
            .map_err(|e| Error::OpenPak(e))?;
        let mut reader = BufReader::new(file);

        //todo: make sure this is actually lazy in the way it's used in the cli cmd
        let map = table_entries.iter().map(move |table_entry| {
            reader.seek(SeekFrom::Start(table_entry.offset as u64))
                .map_err(|e| Error::ReadPak(self.filepath.to_path_buf(), e.to_string()))?;

            let mut data: Vec<u8> = Vec::with_capacity(table_entry.size as usize);
            (&mut reader)
                .take(table_entry.size as u64)
                .read_to_end(&mut data)
                    .map_err(|e| Error::ReadPak(self.filepath.to_path_buf(), e.to_string()))?;

            Ok(PakItem { table_entry, data })
        });

        Ok(map)
    }

    // Returns the header, table, and table entries of the PAK file.
    pub fn manifest(&self) -> &PakManifest {
        &self.manifest
    }
}

/// An iterator item representing a file in a PAK archive.
#[derive(Debug)]
pub struct PakItem<'t> {
    /// The table entry for this item
    table_entry: &'t TableEntry,
    /// The data for this item's file
    data: Vec<u8>
}

impl PakItem<'_> {
    /// The table entry for this item
    pub fn table_entry(&self) -> &TableEntry {
        self.table_entry
    }

    /// The data for this item's file
    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }
}

/// Walks a directory and returns a vector of tuples containing the path and size of each file.
/// Sorts based on heirarchy and file name.
/// This sort order should be maintained in each PAK.
/// Returns: (path: PathBuf, size: u64)
/// Throws [Error::ReadDirectory]
pub fn walk_pak_dir_sync<P: AsRef<Path>>(dir: P) -> Result<Vec<(PathBuf, u64)>> {
    let mut entries = walkdir::WalkDir::new(&dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| !entry.file_name().to_string_lossy().starts_with('.'))
        .filter(|entry| entry.metadata().is_ok_and(|metadata| metadata.is_file()))
        .map(|entry| {
            let metadata = entry.metadata()
                .map_err(|e| Error::ReadDirectory(entry.path().to_path_buf(), e.to_string()))?;
            let path = entry.path().strip_prefix(&dir)
                .map_err(|e| Error::ReadDirectory(entry.path().to_path_buf(), e.to_string()))?
                .to_path_buf();
            let len = metadata.len();
            Ok::<(PathBuf, u64), Error>((path, len))
        })
        .collect::<Vec<_>>()
        .into_iter()
        .collect::<Result<Vec<_>>>()?;

    sort_walk_paths(&mut entries);
    Ok(entries)
}

/// Sorts based on heirarchy and file name
pub fn sort_walk_paths(entries: &mut Vec<(PathBuf, u64)>) {
    entries.sort_by(|a, b| {
        match a.0.parent().cmp(&b.0.parent()) {
            std::cmp::Ordering::Equal => a.0.file_name().cmp(&b.0.file_name()),
            other => other,
        }
    });
}

/// Walks a directory and returns a vector of tuples containing the path and size of each file.
/// Sorts based on heirarchy and file name.
/// This sort order should be maintained in each PAK.
/// Returns: (path: PathBuf, size: u64)
/// Throws [Error::ReadDirectory]
pub async fn walk_pak_dir<P: AsRef<Path>>(dir: P) -> Result<Vec<(PathBuf, u64)>> {
    let dir = dir.as_ref().to_path_buf();
    tokio::task::spawn(async move {
        walk_pak_dir_sync(dir)
    }).await.expect("Expected task to join properly")
}
