use libhdfs3_sys::hdfs3::HdfsFs;

#[test]
fn test_all() -> anyhow::Result<()> {
    let fs = HdfsFs::new("hdfs://tr.onedigit.org:8020")?;

    let parent_path = "/user/ahmed/test";
    let _path = format!("{}/Cargo.toml", parent_path);

    // (1) write a file TODO writing is not implemented yet in `hdfspp` library
    /*
    let hdfs_file_to_write = fs.open(&path)?;
    let buf_to_write = std::fs::read("Cargo.toml")?;
    hdfs_file_to_write.write(&buf_to_write)?;
     */

    // (2) List the directory
    if let Ok(statuses) = fs.list_status(parent_path) {
        for status in statuses {
            println!("status: {:?}", status.name());
        }
    }

    // Delete the file
    // let _r = fs.delete(&path, false)?;
    // assert!(_r);

    /*
    let hdfs_file = fs.open(path)?;
    assert!(hdfs_file.available()?);

    let status = hdfs_file.get_file_status()?;
    println!("len: {}", status.len());
    println!("lastmodified: {}", status.last_modified());

    let exists = fs.exist(path);
    println!("exists: {}", exists);
     */

    Ok(())
}
