use std::{
  error::Error,
  future::Future,
  io::Read,
  pin::Pin,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
  task::{Context, Poll},
  thread,
  time::{Duration, Instant},
};

use liten::{net::TcpListener, task};

pub struct Sleep {
  deadline: Instant,
  // Ensure we only spawn one sleeper thread.
  waker_registered: Arc<AtomicBool>,
}

impl Sleep {
  pub fn new(duration: Duration) -> Self {
    Self {
      deadline: Instant::now() + duration,
      waker_registered: Arc::new(AtomicBool::new(false)),
    }
  }
}

impl Future for Sleep {
  type Output = ();

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
    if Instant::now() >= self.deadline {
      return Poll::Ready(());
    }

    // Spawn a thread to sleep and wake the task if we haven't already.
    if !self.waker_registered.swap(true, Ordering::SeqCst) {
      let waker = cx.waker().clone();
      let deadline = self.deadline;
      task::spawn(async move {
        let now = Instant::now();
        if deadline > now {
          thread::sleep(deadline - now);
        }
        waker.wake();
      });
    }

    Poll::Pending
  }
}

#[liten::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let tcp = TcpListener::bind("0.0.0.0:9000").unwrap();
  loop {
    let (mut stream, _) = tcp.accept().await.unwrap();

    task::spawn(async move {
      let mut vec = Vec::default();
      stream.read_to_end(&mut vec).unwrap();
      println!("{vec:?}");
    });
  }
  //task::spawn(async move {
  //  println!("nice2");
  //});
  //task::spawn(async move {
  //  async {}.await;
  //  println!("nice");
  //  async {}.await;
  //});
  //let handle_2 = task::spawn(async move { "from the await" });
  //
  //println!("2st handler {}", handle_2.await);

  //println!("3: sync print");
}
