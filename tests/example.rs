use std::{
    fs,
    io::{self, BufRead, Read, Seek as _, Write as _},
    iter,
    path::{Path, PathBuf},
};

use anyhow::Context;
use disktree::{
    Key, Value,
    collections::{GetByKey, InsertItem, ItemSlice, ItemStore, ItemVec, KeySet},
    functional::Setter as _,
};
use rand::RngExt;
use sha2::{Digest, Sha256};
use structured_zstd::encoding::CompressionLevel;

const IMDB_DIR: &str = "tests/imdb";

#[test]
fn test_imdb() -> anyhow::Result<()> {
    let mut digest = Sha256::new();
    let mut duplicates = 0;

    // Load IMDB reviews from filesystem
    let mut dir_entries = fs::read_dir(IMDB_DIR)?.collect::<io::Result<Vec<_>>>()?;
    dir_entries.retain(|ent| ent.file_name().as_encoded_bytes().ends_with(b".txt"));
    dir_entries.sort_by_cached_key(|ent| ent.file_name());
    let mut dir_entries = dir_entries.into_iter().map(|ent| ent.path());

    // Load first 10K reviews
    let mut index = ItemVec::new();
    let mut objects = Vec::new();
    duplicates += load_store(
        dir_entries.by_ref().take(10_000),
        &mut index,
        &mut objects,
        &mut digest,
    )?;

    // This has been measured and should be stable since we sort the directory entries
    assert_eq!(index.len(), 10_000 - duplicates);
    let index = index.into_bytes();
    assert_eq!(index.len(), 479_184);
    assert_eq!(objects.len(), 7_161_727);

    // Load store from a &[u8] without copying
    // `ItemSlice` is a read-only view of the store
    let (index, overflow) = ItemSlice::from_bytes(&index);
    assert!(overflow.is_empty());

    // `ItemStore` is a read-write view of the store
    // Load 10K more rows
    let mut index = ItemStore::new(index);
    duplicates += load_store(
        dir_entries.by_ref().take(10_000),
        &mut index,
        &mut objects,
        &mut digest,
    )?;

    assert_eq!(index.len(), 20_000 - duplicates);
    let index = index.into_bytes(); // This is copying bytes from earlier store though
    assert_eq!(index.len(), 956_304);
    assert_eq!(objects.len(), 14_363_182);

    // Reload updated store, still zero-copy
    let (index, overflow) = ItemSlice::from_bytes(&index);
    assert!(overflow.is_empty());
    let mut index = ItemStore::new(index);

    // Load a subset of 10K rows
    let mut index2 = ItemVec::new();
    let mut objects2 = Vec::new();
    let duplicates2 = load_store(
        dir_entries.by_ref().take(10_000),
        &mut index2,
        &mut objects2,
        &mut digest,
    )?;

    // Append this subset to the main store
    let mut duplicates3 = 10_000;
    for (_key, value) in index.append(index2, objects.len().try_into()?) {
        objects.extend_from_slice(&objects2[value.into_range()]);
        duplicates3 -= 1;
    }

    assert!(duplicates3 >= duplicates2);
    duplicates += duplicates3;

    assert_eq!(index.len(), 30_000 - duplicates);
    let index = index.into_bytes();
    assert_eq!(index.len(), 1_429_104);
    assert_eq!(objects.len(), 21_336_179);

    // Reload store and verify it
    let (index, overflow) = ItemSlice::from_bytes(&index);
    assert!(overflow.is_empty());
    verify_store(index, &objects[..], &mut digest)?;

    // Generate an infinite serie of items sampled from index
    let mut rng = rand::rng();
    let mut index_samples = iter::repeat_with(move || {
        let n = rng.random_range(0..index.len());
        &index[n]
    });

    // Remove random items from store
    {
        let mut index = ItemStore::new(index);
        let mut removed = 0;
        while removed < 1_000 {
            let item = index_samples.next().unwrap();
            if index.remove(item.key()).is_some() {
                removed += 1;
            }
        }

        assert_eq!(index.len(), 29_000 - duplicates);
    }

    // Remove many items from store
    {
        let keys = index_samples
            .take(1_000)
            .map(|item| item.key().to_owned())
            .collect::<KeySet>();

        let mut index = ItemStore::new(index);
        for (_key, _value) in index.remove_all(&keys) {
            // item has been removed
        }

        assert_eq!(index.len(), 30_000 - duplicates - keys.len());
        assert!(keys.iter().all(|key| !index.contains(key)));
    }

    Ok(())
}

const SAMPLE_SIZE: usize = 10_000;

fn sample_index_path(path: &Path) -> PathBuf {
    let mut file_name = path.file_name().unwrap().to_owned();
    file_name.push("_index");
    path.with_file_name(file_name)
}

fn sample_objects_path(path: &Path) -> PathBuf {
    let mut file_name = path.file_name().unwrap().to_owned();
    file_name.push("_objects");
    path.with_file_name(file_name)
}

fn write_sample(path: impl AsRef<Path>) -> anyhow::Result<()> {
    let path = path.as_ref();
    let mut dir_entries = fs::read_dir(path)?.collect::<io::Result<Vec<_>>>()?;
    dir_entries.retain(|ent| ent.file_name().as_encoded_bytes().ends_with(b".txt"));

    let mut index = ItemVec::new();
    let mut objects = Vec::new();
    let mut digest = Sha256::new();
    while index.len() < SAMPLE_SIZE {
        let n = rand::random_range(0..dir_entries.len());
        let (key, object) = read_file(&dir_entries[n].path(), &mut digest)?;
        let offset = objects.len().try_into()?;
        let decompressed = object.len().try_into()?;
        let compressed =
            structured_zstd::encoding::compress_slice_to_vec(&object, CompressionLevel::Default);
        let length = compressed.len().try_into()?;
        let Some(mut value) = index.insert_mut(key) else {
            continue;
        };
        value.set(Value {
            offset,
            decompressed,
            length,
            extra: [0; _],
        });
        objects.extend_from_slice(&compressed);
    }

    let mut index_file = Vec::new();
    writeln!(&mut index_file, "{}", env!("CARGO_PKG_VERSION_MAJOR"))?;
    index_file.extend(index.into_bytes());

    fs::write(sample_index_path(path), index_file)?;
    fs::write(sample_objects_path(path), objects)?;

    Ok(())
}

fn read_sample(path: impl AsRef<Path>) -> anyhow::Result<()> {
    let path = path.as_ref();
    let index = fs::read(sample_index_path(path))
        .with_context(|| format!("{:?}", sample_index_path(path)))?;
    let objects = fs::read(sample_objects_path(path))
        .with_context(|| format!("{:?}", sample_index_path(path)))?;

    let mut index = io::Cursor::new(index);
    let mut version = String::new();
    index.read_line(&mut version)?;
    assert_eq!(version.trim_end(), env!("CARGO_PKG_VERSION_MAJOR"));
    let offset = index.position() as _;
    let index = index.into_inner();

    let (index, overflow) = ItemSlice::from_bytes(&index[offset..]);
    assert!(overflow.is_empty());
    assert_eq!(index.len(), SAMPLE_SIZE);
    verify_store(index, &objects[..], &mut Sha256::new())?;

    Ok(())
}

#[test]
#[ignore = "Please run manually"]
fn write_imdb_sample() -> anyhow::Result<()> {
    write_sample(IMDB_DIR)
}

#[test]
fn read_imdb_sample() -> anyhow::Result<()> {
    read_sample(IMDB_DIR)
}

fn load_store(
    dir_entries: impl Iterator<Item = PathBuf>,
    index: &mut impl InsertItem,
    objects: &mut Vec<u8>,
    digest: &mut Sha256,
) -> anyhow::Result<usize> {
    let mut duplicates = 0;

    for dir_entry in dir_entries {
        let (key, object) = read_file(&dir_entry, digest)?;
        let Some(mut value) = index.insert_mut(key) else {
            eprintln!("Not inserted: file={:?}", dir_entry.file_name());
            duplicates += 1;
            continue;
        };

        let offset = objects.len().try_into()?;
        let decompressed = object.len().try_into()?;

        let compressed =
            structured_zstd::encoding::compress_slice_to_vec(&object, CompressionLevel::Default);
        let length = compressed.len().try_into()?;
        drop(object);
        objects.extend(compressed);

        value.set(Value {
            offset,
            decompressed,
            length,
            extra: [0; _],
        });
    }

    Ok(duplicates)
}

fn verify_store(
    index: &(impl GetByKey + ?Sized),
    objects: &[u8],
    digest: &mut Sha256,
) -> anyhow::Result<()> {
    for key in index.keys() {
        let Some(value) = index.get(key) else {
            panic!("Key not found: {key:?}")
        };
        let object = &objects[value.into_range()];
        let key2 = verify_object(object, digest)?;
        assert_eq!(key, &key2);
    }

    Ok(())
}

fn verify_object(object: &[u8], digest: &mut Sha256) -> anyhow::Result<Key> {
    let mut zst = structured_zstd::decoding::StreamingDecoder::new(io::Cursor::new(object))?;
    let mut buf = [0u8; 0x40];
    loop {
        let read = zst.read(&mut buf)?;
        if read == 0 {
            break;
        }

        digest.update(&buf[..read]);
    }

    let key = Key::from_array(digest.finalize_reset().0);
    Ok(key)
}

fn read_file(path: &Path, digest: &mut Sha256) -> io::Result<(Key, Vec<u8>)> {
    let mut file = fs::File::open(path)?;

    let file_len: u32 = file.seek(io::SeekFrom::End(0))?.try_into().unwrap();
    file.seek(io::SeekFrom::Start(0))?;

    let contents = read_to_vec(&mut file, file_len as _)?;
    Digest::update(digest, &contents);
    let key = Key::from_array(digest.finalize_reset().0);

    Ok((key, contents))
}

fn read_to_vec(reader: &mut impl io::Read, cap: usize) -> io::Result<Vec<u8>> {
    let mut vec = Vec::with_capacity(cap);
    reader.read_to_end(&mut vec)?;
    Ok(vec)
}
