use libhdfs3_sys::hdfs3::HdfsFs;

/// An integration test of the API.
///
/// Needs a local HDFS to be up and running.
/// This is fairly easy to do after downloading a recent hadoop
/// binary distribution and configuring it to run locally. For example:
///
/// `$HADOOP_HOME/sbin/start-dfs.sh`
///
#[test]
fn test_all() -> anyhow::Result<()> {
    let fs = HdfsFs::new("hdfs://localhost:8020")?;

    let parent_path = "/test";
    let path = format!("{}/Cargo.toml", parent_path);

    // (1) Create parent directory
    fs.mkdir(parent_path)?;
    assert!(fs.exist(parent_path));

    // (2) write a file 
    let hdfs_file_to_write = fs.open_for_writing(&path)?;
    assert!(fs.exist(&path));
    let buf_to_write = std::fs::read("Cargo.toml")?;
    hdfs_file_to_write.write(&buf_to_write)?;
    hdfs_file_to_write.close()?;

    // (3) Open for reading 
    let hdfs_file_for_reading = fs.open(&path)?;
    // TODO the size doesn't match 
    let _available = hdfs_file_for_reading.available()?;
    
    // (4) List the directory
    if let Ok(statuses) = fs.list_status(parent_path) {
        assert_eq!(statuses.len(), 1);
        let status = &statuses[0];
        assert_eq!(status.name(), path);
    }

    // (5) Delete the file
    assert!(fs.delete(&path, false)?);

    // (5) Verify the file does not exist
    assert!(!fs.exist(&path));

    Ok(())
}
