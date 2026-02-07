use std::{
    fmt::write,
    io::{ErrorKind, Result},
    path::{Path, PathBuf},
    vec,
};

use backend_commons::store::{StateBuilder, Store};
use tokio::{
    fs::{File, OpenOptions, create_dir_all, read_dir, try_exists},
    io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
};
use tracing::info;

pub struct Wal {
    writer: BufWriter<File>,
    size: u64,
}

pub struct EventStore {
    wal: Wal,
    path: PathBuf,
    log_index: u64,
    version: u64,
}

fn parse_log_index(file_name: &str) -> Option<u64> {
    file_name.strip_suffix(".log")?.parse().ok()
}

fn parse_snapshot_index(file_name: &str) -> Option<u64> {
    file_name.strip_suffix(".snapshot")?.parse().ok()
}

async fn last_snapshot_index(path: &Path) -> Result<Option<u64>> {
    let snapshot_dir_path = path.join("snapshot");
    create_dir_all(&snapshot_dir_path).await?;
    let mut snapshot_dir = read_dir(&snapshot_dir_path).await?;
    let mut last_index: Option<u64> = None;
    while let Some(entry) = snapshot_dir.next_entry().await? {
        let idx = entry
            .path()
            .file_name()
            .and_then(|filename| filename.to_str())
            .and_then(|filename_str| parse_snapshot_index(filename_str));
        if let Some(idx) = idx {
            if last_index.is_none() || idx > last_index.unwrap() {
                last_index = Some(idx);
            }
        }
    }
    Ok(last_index)
}

async fn last_log_index(path: &Path) -> Result<Option<u64>> {
    let wal_dir_path = path.join("wal");
    create_dir_all(&wal_dir_path).await?;
    let mut wal_dir = read_dir(&wal_dir_path).await?;
    let mut last_index: Option<u64> = None;
    while let Some(entry) = wal_dir.next_entry().await? {
        let idx = entry
            .path()
            .file_name()
            .and_then(|filename| filename.to_str())
            .and_then(|filename_str| parse_log_index(filename_str));
        if let Some(idx) = idx {
            if last_index.is_none() || idx > last_index.unwrap() {
                last_index = Some(idx);
            }
        }
    }
    Ok(last_index)
}

impl EventStore {
    async fn read_state(
        path: &Path,
        snapshot_idx: u64,
        state: &mut impl StateBuilder,
    ) -> Result<()> {
        let snapshot_file_path = path
            .join("snapshot")
            .join(format!("{snapshot_idx:020}.snapshot"));
        let file = File::open(snapshot_file_path).await?;
        let mut reader = BufReader::new(file);
        let version = reader.read_u64_le().await?;
        let mut content = Vec::new();
        reader.read_to_end(&mut content).await?;
        state.load_state(version, content)
    }

    async fn apply_events(state: &mut impl StateBuilder, path: &Path, log_idx: u64) -> Result<()> {
        let log_file_path = path.join("wal").join(format!("{log_idx:020}.log"));
        let log_file = File::open(log_file_path).await?;
        let mut reader = BufReader::new(log_file);
        let version = reader.read_u64_le().await?;
        loop {
            let len = match reader.read_u32_le().await {
                Ok(len) => len,
                Err(error) => {
                    if error.kind() == ErrorKind::UnexpectedEof {
                        break;
                    } else {
                        return Err(error);
                    }
                }
            };
            let mut event_buf = vec![0u8; len as usize];
            reader.read_exact(&mut event_buf).await?;
            state.load_event(version, event_buf)?;
        }
        Ok(())
    }

    async fn get_log_version(path: &Path, id: u64) -> Result<u64> {
        let log_file_path = path.join("wal").join(format!("{id:020}.log"));
        let log_file = File::open(log_file_path).await?;
        let mut reader = BufReader::new(log_file);
        reader.read_u64_le().await
    }

    pub async fn open(
        path: &Path,
        current_version: u64,
        state: &mut impl StateBuilder,
    ) -> Result<Self> {
        let snapshot_index = last_snapshot_index(path).await?;
        let log_index = last_log_index(path).await?;

        if let Some(snapshot_idx) = snapshot_index {
            Self::read_state(path, snapshot_idx, state).await?;
        }

        if let Some(log_index) = log_index {
            for i in snapshot_index.unwrap_or(0)..=log_index {
                Self::apply_events(state, path, i).await?;
            }
        }

        let log_index = match log_index {
            Some(idx) => {
                let log_version = Self::get_log_version(path, idx).await?;
                if log_version == current_version {
                    idx
                } else {
                    idx + 1
                }
            }
            None => 0,
        };

        let wal = Wal::open(
            path.join("wal")
                .join(format!("{log_index:020}.log"))
                .to_str()
                .unwrap(),
            current_version,
        )
        .await?;

        Ok(Self {
            log_index,
            wal,
            path: path.to_path_buf(),
            version: current_version,
        })
    }

    pub async fn flush(&mut self) -> Result<()> {
        self.wal.flush().await
    }

    pub async fn append(&mut self, event: &[u8]) -> Result<()> {
        self.wal.append(event).await
    }

    pub async fn next_log(&mut self) -> Result<()> {
        self.flush().await?;
        self.log_index += 1;
        self.wal = Wal::open(
            self.path
                .join("wal")
                .join(format!("{:020}.log", self.log_index))
                .to_str()
                .unwrap(),
            self.version,
        )
        .await?;
        Ok(())
    }

    pub fn get_current_log_size(&self) -> u64 {
        self.wal.get_size()
    }
}

impl Wal {
    pub async fn open(path: &str, version: u64) -> Result<Self> {
        let write_version = !try_exists(path).await?;
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .write(true)
            .open(path)
            .await?;

        let mut writer = BufWriter::new(file);
        if write_version {
            writer.write_u64_le(version).await?;
            writer.flush().await?;
            writer.get_ref().sync_all().await?;
        }
        let metadata = writer.get_ref().metadata().await?;

        Ok(Self {
            writer,
            size: metadata.len(),
        })
    }

    pub async fn append(&mut self, event: &[u8]) -> Result<()> {
        let len = event.len() as u32;
        self.writer.write_all(&len.to_le_bytes()).await?;
        self.writer.write_all(event).await?;
        self.size += len as u64 + 4;
        Ok(())
    }

    pub async fn flush(&mut self) -> Result<()> {
        self.writer.flush().await?;
        self.writer.get_ref().sync_all().await?;
        Ok(())
    }

    fn get_size(&self) -> u64 {
        self.size
    }
}

impl Store for EventStore {
    async fn apply_event(&mut self, data: &[u8]) -> Result<()> {
        info!("Applying event of size {} bytes", data.len());
        self.append(data).await
    }

    async fn snapshot(&mut self, data: &[u8]) -> Result<()> {
        let snapshot_index = self.log_index;
        self.next_log().await?;

        let tmp_snapshot_file_path = self
            .path
            .join("snapshot")
            .join(format!("{:020}.snapshot.tmp", snapshot_index));

        let snapshot_file_path = self
            .path
            .join("snapshot")
            .join(format!("{:020}.snapshot", snapshot_index));

        let mut tmp_file = File::create(&tmp_snapshot_file_path).await?;
        tmp_file.write_u64_le(self.version).await?;
        tmp_file.write_all(data).await?;
        tmp_file.flush().await?;
        tmp_file.sync_all().await?;
        tokio::fs::rename(tmp_snapshot_file_path, snapshot_file_path).await?;
        Ok(())
    }
}
