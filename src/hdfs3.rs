use std::collections::HashMap;
use std::ffi::CStr;
use std::fmt::Formatter;
use std::rc::Rc;

use lazy_static::lazy_static;
use libc::{c_int, c_short, c_void};
use log::*;
use std::sync::RwLock;
use std::{ffi::CString, marker::PhantomData};

use crate::err::HdfsErr;
use crate::*;

const O_RDONLY: c_int = 0;
const O_WRONLY: c_int = 1;
const O_APPEND: c_int = 1024;

/// Encapsulate Namenode connection properties
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ConnectionProperties {
    pub namenode_host: String,
    pub namenode_port: u16,
    pub namenode_user: Option<String>,
    pub kerberos_ticket_cache_path: Option<String>,
}

/// since HDFS client handles are completely thread safe, here we implement Send + Sync trait
/// for HdfsFs
unsafe impl Send for HdfsFs {}
unsafe impl Sync for HdfsFs {}

lazy_static! {
    static ref HDFS_CACHE: RwLock<HashMap<ConnectionProperties, HdfsFs>> =
        RwLock::new(HashMap::new());
}

/// Hdfs Filesystem
///
/// It is basically thread safe because the native API for hdfsFs is thread-safe.
#[derive(Clone)]
pub struct HdfsFs {
    connection_properties: ConnectionProperties,
    raw: hdfsFS,
    _marker: PhantomData<()>,
}

impl std::fmt::Debug for HdfsFs {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HdfsFs")
            .field("url", &self.connection_properties)
            .finish()
    }
}

impl HdfsFs {
    /// Create an instance of HdfsFs. A global cache is used to ensure that only one instance
    /// is created per namenode uri.
    ///
    /// * connection_properties - Namenode connection parameters
    pub fn new(connection_properties: ConnectionProperties) -> Result<HdfsFs, HdfsErr> {
        HdfsFs::new_with_hdfs_params(connection_properties, HashMap::new())
    }

    /// Create an instance of HdfsFs. A global cache is used to ensure that only one instance
    /// is created per namenode uri.
    ///
    /// * connection_properties - Namenode connection parameters
    /// * hdfs_params - optional key value pairs that need to be passed to configure
    ///   the HDFS client side.
    ///   Example: key = 'dfs.domain.socket.path', value = '/var/lib/hadoop-fs/dn_socket'
    pub fn new_with_hdfs_params(
        connection_properties: ConnectionProperties,
        hdfs_params: HashMap<String, String>,
    ) -> Result<HdfsFs, HdfsErr> {
        // Try to get from cache if an entry exists.
        {
            let cache = HDFS_CACHE
                .read()
                .expect("Could not aquire read lock on HDFS cache");
            if let Some(hdfs_fs) = cache.get(&connection_properties) {
                return Ok(hdfs_fs.clone());
            }
        }

        let mut cache = HDFS_CACHE
            .write()
            .expect("Could not aquire write lock on HDFS cache");
        let hdfsFs = cache
            .entry(connection_properties.clone())
            .or_insert_with(|| {
                let hdfs_fs = create_hdfs_fs(connection_properties.clone(), hdfs_params)
                    .expect("Could not create HDFS connection");
                HdfsFs {
                    connection_properties,
                    raw: hdfs_fs,
                    _marker: PhantomData,
                }
            });

        Ok(hdfsFs.clone())
    }

    /// Open a file for append
    pub fn append(&self, path: &str) -> Result<HdfsFile, HdfsErr> {
        if !self.exist(path) {
            return Err(HdfsErr::FileNotFound(path.to_owned()));
        }
        let file = unsafe {
            let cstr_path = CString::new(path).unwrap();
            hdfsOpenFile(self.raw, cstr_path.as_ptr(), O_APPEND, 0, 0, 0)
        };
        self.new_hdfs_file(path, file)
    }

    /// Create the given path as read-only
    #[inline]
    pub fn create(&self, path: &str) -> Result<HdfsFile, HdfsErr> {
        self.create_with_params(path, false, 0, 0, 0)
    }

    /// Create the given path as writable
    #[inline]
    pub fn create_with_overwrite(&self, path: &str, overwrite: bool) -> Result<HdfsFile, HdfsErr> {
        self.create_with_params(path, overwrite, 0, 0, 0)
    }

    /// Create the given path
    pub fn create_with_params(
        &self,
        path: &str,
        overwrite: bool,
        buf_size: i32,
        replica_num: i16,
        block_size: i64,
    ) -> Result<HdfsFile, HdfsErr> {
        if !overwrite && self.exist(path) {
            return Err(HdfsErr::FileAlreadyExists(path.to_owned()));
        }
        let file = unsafe {
            let cstr_path = CString::new(path).unwrap();
            hdfsOpenFile(
                self.raw,
                cstr_path.as_ptr(),
                O_WRONLY,
                buf_size as c_int,
                replica_num as c_short,
                block_size as tOffset,
            )
        };
        self.new_hdfs_file(path, file)
    }

    pub fn get_file_status(&self, path: &str) -> Result<FileStatus, HdfsErr> {
        let ptr = unsafe {
            let cstr_path = CString::new(path).unwrap();
            hdfsGetPathInfo(self.raw, cstr_path.as_ptr())
        };
        if ptr.is_null() {
            Err(HdfsErr::Miscellaneous(format!(
                "Could not get file status for {}",
                path
            )))
        } else {
            Ok(FileStatus::new(ptr))
        }
    }

    /// Delete the content at the given path.
    ///
    /// * path - the path on the filesystem
    /// * recursive - if true, delete the content recursively.
    pub fn delete(&self, path: &str, recursive: bool) -> Result<bool, HdfsErr> {
        let res = unsafe {
            let cstr_path = CString::new(path).unwrap();
            hdfsDelete(self.raw, cstr_path.as_ptr(), recursive as c_int)
        };
        if res == 0 {
            Ok(true)
        } else {
            Err(HdfsErr::Miscellaneous(format!(
                "Could not delete path: {}",
                path
            )))
        }
    }

    /// Check if the given path exists on the filesystem
    pub fn exist(&self, path: &str) -> bool {
        (unsafe {
            let cstr_path = CString::new(path).unwrap();
            hdfsExists(self.raw, cstr_path.as_ptr())
        } == 0)
    }

    /// Get the file status of each entry under the specified path
    /// Note that it is an error to list an empty directory.
    pub fn list_status(&self, path: &str) -> Result<Vec<FileStatus>, HdfsErr> {
        let mut entry_num: c_int = 0;
        let ptr = unsafe {
            let cstr_path = CString::new(path).unwrap();
            hdfsListDirectory(self.raw, cstr_path.as_ptr(), &mut entry_num)
        };
        if ptr.is_null() {
            Err(HdfsErr::Miscellaneous(format!(
                "Could not list content of path: {}",
                path
            )))
        } else {
            let shared_ptr = Rc::new(HdfsFileInfoPtr::new_array(ptr, entry_num));

            let list = (0..entry_num)
                .into_iter()
                .map(|idx| FileStatus::from_array(shared_ptr.clone(), idx as u32))
                .collect::<Vec<FileStatus>>();

            Ok(list)
        }
    }

    pub fn mkdir(&self, path: &str) -> Result<bool, HdfsErr> {
        let res = unsafe {
            let cstr_path = CString::new(path).unwrap();
            hdfsCreateDirectory(self.raw, cstr_path.as_ptr())
        };
        if res == 0 {
            Ok(true)
        } else {
            Err(HdfsErr::Miscellaneous(format!(
                "Could not create directory at path: {}",
                path
            )))
        }
    }

    #[inline]
    pub fn open(&self, path: &str) -> Result<HdfsFile, HdfsErr> {
        self.open_with_buf_size(path, 0)
    }

    pub fn open_with_buf_size(&self, path: &str, buf_size: i32) -> Result<HdfsFile, HdfsErr> {
        let file = unsafe {
            let cstr_path = CString::new(path).unwrap();
            hdfsOpenFile(
                self.raw,
                cstr_path.as_ptr(),
                O_RDONLY,
                buf_size as c_int,
                0,
                0,
            )
        };
        self.new_hdfs_file(path, file)
    }

    pub fn open_for_writing(&self, path: &str) -> Result<HdfsFile, HdfsErr> {
        let file = unsafe {
            let cstr_path = CString::new(path).unwrap();
            hdfsOpenFile(self.raw, cstr_path.as_ptr(), O_WRONLY, 0, 0, 0)
        };
        self.new_hdfs_file(path, file)
    }

    fn new_hdfs_file(&self, path: &str, file: hdfsFile) -> Result<HdfsFile, HdfsErr> {
        if file.is_null() {
            Err(HdfsErr::Miscellaneous(format!(
                "Could not open HDFS file at path {}",
                path
            )))
        } else {
            Ok(HdfsFile {
                fs: self.clone(),
                path: path.to_owned(),
                file,
                _market: PhantomData,
            })
        }
    }

    /// Rename a file
    ///
    /// old_path - the path to rename
    /// new_path - the new name
    ///
    /// Note that the destination directory must exist.
    pub fn rename(&self, old_path: &str, new_path: &str) -> Result<bool, HdfsErr> {
        let ret = unsafe {
            let cstr_old_path = CString::new(old_path).unwrap();
            let cstr_new_path = CString::new(new_path).unwrap();
            hdfsRename(self.raw, cstr_old_path.as_ptr(), cstr_new_path.as_ptr())
        };
        if ret == 0 {
            Ok(true)
        } else {
            Err(HdfsErr::Miscellaneous(format!(
                "Could not reanme {} to {}",
                old_path, new_path
            )))
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Safely deallocatable hdfsFileinfo pointer
struct HdfsFileInfoPtr {
    pub ptr: *const hdfsFileInfo,
    pub len: i32,
}

impl Drop for HdfsFileInfoPtr {
    fn drop(&mut self) {
        unsafe { hdfsFreeFileInfo(self.ptr as *mut hdfsFileInfo, self.len) };
    }
}

impl HdfsFileInfoPtr {
    fn new(ptr: *const hdfsFileInfo) -> HdfsFileInfoPtr {
        HdfsFileInfoPtr { ptr, len: 1 }
    }

    pub fn new_array(ptr: *const hdfsFileInfo, len: i32) -> HdfsFileInfoPtr {
        HdfsFileInfoPtr { ptr, len }
    }
}

/// Interface that represents the client side information for a file or directory.
pub struct FileStatus {
    raw: Rc<HdfsFileInfoPtr>,
    idx: u32,
    _marker: PhantomData<()>,
}

impl FileStatus {
    /// create FileStatus from *const hdfsFileInfo
    #[inline]
    fn new(ptr: *const hdfsFileInfo) -> FileStatus {
        FileStatus {
            raw: Rc::new(HdfsFileInfoPtr::new(ptr)),
            idx: 0,
            _marker: PhantomData,
        }
    }

    /// create FileStatus from *const hdfsFileInfo which points to a dynamically allocated array.
    #[inline]
    fn from_array(raw: Rc<HdfsFileInfoPtr>, idx: u32) -> FileStatus {
        FileStatus {
            raw,
            idx,
            _marker: PhantomData,
        }
    }

    /// Get the pointer to hdfsFileInfo
    #[inline]
    fn ptr(&self) -> *const hdfsFileInfo {
        unsafe { self.raw.ptr.offset(self.idx as isize) }
    }

    /// Get the name of the file
    #[inline]
    pub fn name(&self) -> &str {
        let slice = unsafe { CStr::from_ptr((*self.ptr()).mName) }.to_bytes();
        std::str::from_utf8(slice).unwrap()
    }

    /// Is this a file?
    #[inline]
    pub fn is_file(&self) -> bool {
        match unsafe { &*self.ptr() }.mKind {
            tObjectKind::kObjectKindFile => true,
            tObjectKind::kObjectKindDirectory => false,
        }
    }

    /// Is this a directory?
    #[inline]
    pub fn is_directory(&self) -> bool {
        match unsafe { &*self.ptr() }.mKind {
            tObjectKind::kObjectKindFile => false,
            tObjectKind::kObjectKindDirectory => true,
        }
    }

    /// Get the owner of the file
    #[inline]
    pub fn owner(&self) -> &str {
        let slice = unsafe { CStr::from_ptr((*self.ptr()).mOwner) }.to_bytes();
        std::str::from_utf8(slice).unwrap()
    }

    /// Get the group associated with the file
    #[inline]
    pub fn group(&self) -> &str {
        let slice = unsafe { CStr::from_ptr((*self.ptr()).mGroup) }.to_bytes();
        std::str::from_utf8(slice).unwrap()
    }

    /// Get the permissions associated with the file
    #[inline]
    pub fn permission(&self) -> i16 {
        unsafe { &*self.ptr() }.mPermissions as i16
    }

    /// Get the length of this file, in bytes.
    #[allow(clippy::len_without_is_empty)]
    #[inline]
    pub fn len(&self) -> usize {
        unsafe { &*self.ptr() }.mSize as usize
    }

    /// Get the block size of the file.
    #[inline]
    pub fn block_size(&self) -> usize {
        unsafe { &*self.ptr() }.mBlockSize as usize
    }

    /// Get the replication factor of a file.
    #[inline]
    pub fn replica_count(&self) -> i16 {
        unsafe { &*self.ptr() }.mReplication as i16
    }

    /// Get the last modification time for the file in seconds
    #[inline]
    pub fn last_modified(&self) -> time_t {
        unsafe { &*self.ptr() }.mLastMod
    }

    /// Get the last access time for the file in seconds
    #[inline]
    pub fn last_access(&self) -> time_t {
        unsafe { &*self.ptr() }.mLastAccess
    }
}

// -------------------------------------------------------------------------------------------------

/// An HDFS file
#[derive(Clone)]
pub struct HdfsFile {
    fs: HdfsFs,
    path: String,
    file: hdfsFile,
    _market: PhantomData<()>,
}
impl std::fmt::Debug for HdfsFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HdfsFile")
            .field("connection_properties", &self.fs.connection_properties)
            .field("path", &self.path)
            .finish()
    }
}

impl HdfsFile {
    #[inline]
    pub fn fs(&self) -> &HdfsFs {
        &self.fs
    }

    #[inline]
    pub fn path(&self) -> &str {
        &self.path
    }

    ///  Number of bytes that can be read from this file without blocking.
    pub fn available(&self) -> Result<i32, HdfsErr> {
        let ret = unsafe { hdfsAvailable(self.fs.raw, self.file) };
        if ret < 0 {
            Err(HdfsErr::Miscellaneous(format!(
                "Could not determine HDFS availability for {}",
                self.path
            )))
        } else {
            Ok(ret)
        }
    }

    /// Close the opened file
    pub fn close(&self) -> Result<bool, HdfsErr> {
        if unsafe { hdfsCloseFile(self.fs.raw, self.file) } == 0 {
            Ok(true)
        } else {
            Err(HdfsErr::Miscellaneous(format!(
                "Could not close {}",
                self.path
            )))
        }
    }

    /// Get the file status, including file size, last modified time, etc
    pub fn get_file_status(&self) -> Result<FileStatus, HdfsErr> {
        self.fs.get_file_status(self.path())
    }

    /// Read data from an open file
    pub fn read(&self, buf: &mut [u8]) -> Result<i32, HdfsErr> {
        let read_len = unsafe {
            hdfsRead(
                self.fs.raw,
                self.file,
                buf.as_ptr() as *mut c_void,
                buf.len() as tSize,
            )
        };
        if read_len > 0 {
            Ok(read_len as i32)
        } else {
            Err(HdfsErr::Miscellaneous(format!(
                "Failed to read from {}",
                self.path
            )))
        }
    }

    /// Seek to given offset in file.
    pub fn seek(&self, offset: u64) -> bool {
        (unsafe { hdfsSeek(self.fs.raw, self.file, offset as tOffset) }) == 0
    }

    pub fn write(&self, buf: &[u8]) -> Result<i32, HdfsErr> {
        let written_len = unsafe {
            hdfsWrite(
                self.fs.raw,
                self.file,
                buf.as_ptr() as *mut c_void,
                buf.len() as tSize,
            )
        };
        if written_len > 0 {
            Ok(written_len)
        } else {
            Err(HdfsErr::Miscellaneous(format!(
                "Failed to write to {}",
                self.path
            )))
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Create an instance of hdfsFs.
///
/// * connection_properties - Namenode connection parameters
/// * hdfs_params - optional key value pairs that need to be passed to configure
///   the HDFS client side
fn create_hdfs_fs(
    connection_properties: ConnectionProperties,
    hdfs_params: HashMap<String, String>,
) -> Result<hdfsFS, HdfsErr> {
    let hdfs_fs = unsafe {
        let hdfs_builder = hdfsNewBuilder();

        let cstr_host = CString::new(connection_properties.namenode_host.as_bytes()).unwrap();
        for (k, v) in hdfs_params {
            let cstr_k = CString::new(k).unwrap();
            let cstr_v = CString::new(v).unwrap();
            hdfsBuilderConfSetStr(hdfs_builder, cstr_k.as_ptr(), cstr_v.as_ptr());
        }
        hdfsBuilderSetNameNode(hdfs_builder, cstr_host.as_ptr());
        hdfsBuilderSetNameNodePort(hdfs_builder, connection_properties.namenode_port);

        if let Some(user) = connection_properties.namenode_user.clone() {
            let cstr_user = CString::new(user.as_bytes()).unwrap();
            hdfsBuilderSetUserName(hdfs_builder, cstr_user.as_ptr());
        }

        if let Some(kerb_ticket_cache_path) =
            connection_properties.kerberos_ticket_cache_path.clone()
        {
            let cstr_kerb_ticket_cache_path =
                CString::new(kerb_ticket_cache_path.as_bytes()).unwrap();
            hdfsBuilderSetKerbTicketCachePath(hdfs_builder, cstr_kerb_ticket_cache_path.as_ptr());
        }

        info!(
            "Connecting to Namenode, host: {}, port: {}, user: {:?}, krb_ticket_cache: {:?}",
            connection_properties.namenode_host,
            connection_properties.namenode_port,
            connection_properties.namenode_user,
            connection_properties.kerberos_ticket_cache_path
        );

        hdfsBuilderConnect(hdfs_builder)
    };

    if hdfs_fs.is_null() {
        Err(HdfsErr::CannotConnectToNameNode(format!(
            "{}:{}",
            connection_properties.namenode_host, connection_properties.namenode_port
        )))
    } else {
        Ok(hdfs_fs)
    }
}
