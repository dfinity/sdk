use crossbeam::channel::Receiver;

/// Differentiate between:
///   - the process exited (Child)
///   - shutdown was requested (Receiver)
pub enum ChildOrReceiver<T> {
    Child,
    Receiver(T),
}

/// Function that waits for a child or a receiver to stop. This encapsulate the polling so
/// it is easier to maintain.
pub fn wait_for_child_or_receiver<T>(
    child: &mut std::process::Child,
    receiver: &Receiver<T>,
) -> ChildOrReceiver<T> {
    loop {
        // Check if either the child exited or a shutdown has been requested.
        // These can happen in either order in response to Ctrl-C, so increase the chance
        // to notice a shutdown request even if the replica exited quickly.
        let child_try_wait = child.try_wait();
        let receiver_signalled = receiver.recv_timeout(std::time::Duration::from_millis(100));

        match (receiver_signalled, child_try_wait) {
            (Ok(t), _) => {
                // Prefer to indicate the shutdown request
                return ChildOrReceiver::Receiver(t);
            }
            (Err(_), Ok(Some(_))) => {
                return ChildOrReceiver::Child;
            }
            _ => {}
        };
    }
}
