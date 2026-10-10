#[tokio::test]
async fn closing_the_job_ends_its_process() -> std::io::Result<()> {
    let mut child = tokio::process::Command::new("cmd")
        .args(["/c", "ping", "-n", "30", "127.0.0.1"])
        .stdout(std::process::Stdio::null())
        .spawn()?;
    let job = super::contain(&child)?;
    assert!(child.try_wait()?.is_none());
    drop(job);
    let status = tokio::time::timeout(std::time::Duration::from_secs(5), child.wait()).await;
    assert!(
        status.is_ok_and(|status| status.is_ok()),
        "process outlived its job"
    );
    Ok(())
}
