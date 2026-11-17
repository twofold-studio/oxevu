use super::process::*;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU32, Ordering},
};
use tokio::{
    process::Command,
    sync::{Notify, watch},
};

#[derive(Default)]
struct Session {
    pid: AtomicU32,
    done: AtomicBool,
    started: Notify,
    rejection: Mutex<Option<PolicyError>>,
}
impl Observer for Session {
    fn started(&self, pid: u32) -> ObserverFuture<'_, ()> {
        Box::pin(async move {
            self.pid.store(pid, Ordering::SeqCst);
            self.started.notify_one();
        })
    }
    fn finished(&self) -> ObserverFuture<'_, ()> {
        Box::pin(async move {
            self.done.store(true, Ordering::SeqCst);
        })
    }
    fn tick(&self, _: bool) -> ObserverFuture<'_, Result<(), PolicyError>> {
        Box::pin(async move { self.rejection.lock().unwrap().map_or(Ok(()), Err) })
    }
}
fn reaped(session: &Session) {
    assert!(session.done.load(Ordering::SeqCst));
    let pid = session.pid.load(Ordering::SeqCst);
    assert_ne!(pid, 0);
    // This PID belongs to the child created by this test; signal zero only checks existence.
    assert_eq!(unsafe { libc::kill(pid as i32, 0) }, -1);
    assert_eq!(
        std::io::Error::last_os_error().raw_os_error(),
        Some(libc::ESRCH)
    );
}

#[tokio::test]
async fn cancellation_kills_reaps_and_finishes_the_child() {
    let root = tempfile::tempdir().unwrap();
    let observer = Arc::new(Session::default());
    let (cancel, receiver) = watch::channel(false);
    let task = {
        let observer = observer.clone();
        let path = root.path().to_owned();
        tokio::spawn(async move {
            let mut command = Command::new("sleep");
            command.arg("60");
            run(
                command,
                receiver,
                &path,
                u64::MAX,
                &path.join("ready"),
                observer.as_ref(),
            )
            .await
        })
    };
    observer.started.notified().await;
    cancel.send(true).unwrap();
    assert!(matches!(
        tokio::time::timeout(std::time::Duration::from_secs(5), task)
            .await
            .unwrap()
            .unwrap(),
        Err(ProcessError::Stopped)
    ));
    reaped(&observer);
}

#[tokio::test]
async fn failed_encoding_is_typed_and_child_is_reaped() {
    let root = tempfile::tempdir().unwrap();
    let observer = Session::default();
    let (_sender, receiver) = watch::channel(false);
    let result = run(
        Command::new("false"),
        receiver,
        root.path(),
        u64::MAX,
        &root.path().join("ready"),
        &observer,
    )
    .await;
    assert!(matches!(result, Err(ProcessError::Encoding)));
    reaped(&observer);
}

#[tokio::test]
async fn policy_rejections_and_disk_limits_stop_children_without_encoding_fallback() {
    for policy in [
        PolicyError::Stopped,
        PolicyError::IdleExpired,
        PolicyError::PermissionRevoked,
    ] {
        let root = tempfile::tempdir().unwrap();
        let observer = Session::default();
        *observer.rejection.lock().unwrap() = Some(policy);
        let (_sender, receiver) = watch::channel(false);
        let mut command = Command::new("sleep");
        command.arg("60");
        let error = run(
            command,
            receiver,
            root.path(),
            u64::MAX,
            &root.path().join("ready"),
            &observer,
        )
        .await
        .unwrap_err();
        assert!(matches!(error, ProcessError::Policy(found) if found == policy));
        assert!(!error.retryable_encoding_failure());
        reaped(&observer);
    }
    let root = tempfile::tempdir().unwrap();
    let observer = Session::default();
    let (_sender, receiver) = watch::channel(false);
    let mut command = Command::new("sleep");
    command.arg("60");
    assert!(matches!(
        run(
            command,
            receiver,
            root.path(),
            0,
            &root.path().join("ready"),
            &observer
        )
        .await,
        Err(ProcessError::DiskLimit)
    ));
    reaped(&observer);
}

#[tokio::test(start_paused = true)]
async fn revoked_permission_wins_over_startup_timeout() {
    let root = tempfile::tempdir().unwrap();
    let observer = Arc::new(Session::default());
    let (_sender, receiver) = watch::channel(false);
    let task = {
        let observer = observer.clone();
        let path = root.path().to_owned();
        tokio::spawn(async move {
            let mut command = Command::new("sleep");
            command.arg("60");
            run(
                command,
                receiver,
                &path,
                u64::MAX,
                &path.join("ready"),
                observer.as_ref(),
            )
            .await
        })
    };
    observer.started.notified().await;
    *observer.rejection.lock().unwrap() = Some(PolicyError::PermissionRevoked);
    tokio::time::advance(std::time::Duration::from_secs(31)).await;
    assert!(matches!(
        task.await.unwrap(),
        Err(ProcessError::Policy(PolicyError::PermissionRevoked))
    ));
    reaped(&observer);
}

#[test]
fn fallback_and_cancellation_are_variant_based() {
    for error in [
        ProcessError::Start,
        ProcessError::Encoding,
        ProcessError::Timeout,
    ] {
        assert!(error.retryable_encoding_failure());
    }
    assert!(ProcessError::Stopped.stopped());
    assert!(ProcessError::Policy(PolicyError::Stopped).stopped());
    assert!(!ProcessError::Policy(PolicyError::PermissionRevoked).stopped());
}

#[tokio::test]
async fn successful_child_is_reaped_and_finishes_observation() {
    let root = tempfile::tempdir().unwrap();
    let observer = Session::default();
    let (_sender, receiver) = watch::channel(false);
    run(
        Command::new("true"),
        receiver,
        root.path(),
        u64::MAX,
        &root.path().join("ready"),
        &observer,
    )
    .await
    .unwrap();
    reaped(&observer);
}

#[tokio::test(start_paused = true)]
async fn startup_timeout_is_typed_and_reaps_the_child() {
    let root = tempfile::tempdir().unwrap();
    let observer = Arc::new(Session::default());
    let (_sender, receiver) = watch::channel(false);
    let task = {
        let observer = observer.clone();
        let path = root.path().to_owned();
        tokio::spawn(async move {
            let mut command = Command::new("sleep");
            command.arg("60");
            run(
                command,
                receiver,
                &path,
                u64::MAX,
                &path.join("ready"),
                observer.as_ref(),
            )
            .await
        })
    };
    observer.started.notified().await;
    tokio::time::advance(std::time::Duration::from_secs(31)).await;
    assert!(matches!(task.await.unwrap(), Err(ProcessError::Timeout)));
    reaped(&observer);
}
