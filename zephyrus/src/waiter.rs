use std::{future::Future, task::Poll};
use tokio::sync::oneshot::{Sender, Receiver, channel};

struct ComponentWaiter {

}
