use tokio::process::Child;

pub(crate) struct Job {
    #[cfg(windows)]
    _handle: std::os::windows::io::OwnedHandle,
}

#[cfg(not(windows))]
pub(crate) const fn contain(_child: &Child) -> std::io::Result<Job> {
    Ok(Job {})
}

#[cfg(windows)]
pub(crate) fn contain(child: &Child) -> std::io::Result<Job> {
    use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
        SetInformationJobObject,
    };
    let process = child
        .raw_handle()
        .ok_or_else(|| std::io::Error::other("the browser process has already exited"))?;
    // SAFETY: null attributes and a null name create an anonymous job object.
    let raw = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
    if raw.is_null() {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: raw is a job handle just created and owned by nothing else.
    let handle = unsafe { OwnedHandle::from_raw_handle(raw) };
    let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
    limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    let size = u32::try_from(size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>())
        .map_err(std::io::Error::other)?;
    // SAFETY: limits is an initialised JOBOBJECT_EXTENDED_LIMIT_INFORMATION of the size passed, and the handle is open.
    let set = unsafe {
        SetInformationJobObject(
            handle.as_raw_handle(),
            JobObjectExtendedLimitInformation,
            (&raw const limits).cast(),
            size,
        )
    };
    if set == 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: both handles are open; the child handle stays valid while child is borrowed.
    if unsafe { AssignProcessToJobObject(handle.as_raw_handle(), process) } == 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(Job { _handle: handle })
}

#[cfg(all(test, windows))]
#[path = "job_tests.rs"]
mod tests;
