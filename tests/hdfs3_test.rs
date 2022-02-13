use libhdfs3_sys::hdfs3::HdfsFs;

#[test]
fn test_all() -> anyhow::Result<()> {
    let fs = HdfsFs::new("hdfs://tr:8020")?;

    let parent_path = "/user/ahmed/test";
    let path = format!("{}/Cargo.toml", parent_path);

    // (1) write a file 
    let hdfs_file_to_write = fs.open_for_writing(&path)?;
    assert!(fs.exist(&path));
    let buf_to_write = std::fs::read("Cargo.toml")?;
    hdfs_file_to_write.write(&buf_to_write)?;
    hdfs_file_to_write.close()?;

    // (2) Open for reading // TODO the size doesn't match 
    let hdfs_file_for_reading = fs.open(&path)?;
    let available = hdfs_file_for_reading.available()?;
    println!("avaialble: {}", available);
    
    // (3) List the directory
    if let Ok(statuses) = fs.list_status(parent_path) {
        for status in statuses {
            println!("status: {:?}, {}", status.name(), status.last_modified());
        }
    }

    // (4) Delete the file
    let _r = fs.delete(&path, false)?;
    assert!(_r);

    // (5) Verify the file does not exist
    assert!(!fs.exist(&path));

    Ok(())
}
