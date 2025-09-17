mod future;
mod http;
mod runtime;
use crate::http::Http;
use future::{Future, PollState};
use runtime::Waker;
fn main() {
    let mut executor = runtime::init();
    executor.block_on(async_main());
}
fn async_main() -> impl Future<Output = String> {
    Coroutine0::new()
}

#[derive(Default)]
struct Stack0 {
    counter: Option<usize>,
}
enum State0 {
    Start,
    Wait1(Box<dyn Future<Output = String>>),
    Wait2(Box<dyn Future<Output = String>>),
    Resolved,
}
struct Coroutine0 {
    stack: Stack0,
    state: State0,
}
impl Coroutine0 {
    fn new() -> Self {
        Self {
            state: State0::Start,
            stack: Stack0::default(),
        }
    }
}
impl Future for Coroutine0 {
    type Output = String;
    fn poll(&mut self, waker: &Waker) -> PollState<Self::Output> {
        loop {
            match self.state {
                State0::Start => {
                    self.stack.counter = Some(0);
                    println!("Program starting");
                    let fut1 = Box::new(Http::get("/600/HelloAsyncAwait"));
                    self.state = State0::Wait1(fut1);
                }
                State0::Wait1(ref mut f1) => match f1.poll(waker) {
                    PollState::Ready(txt) => {
                        let mut counter = self.stack.counter.take().unwrap();
                        println!("{txt}");
                        counter += 1;
                        let fut2 = Box::new(Http::get("/400/HelloAsyncAwait"));
                        self.state = State0::Wait2(fut2);
                        self.stack.counter = Some(counter);
                    }
                    PollState::NotReady => break PollState::NotReady,
                },
                State0::Wait2(ref mut f2) => match f2.poll(waker) {
                    PollState::Ready(txt) => {
                        let mut counter = self.stack.counter.take().unwrap();
                        println!("{txt}");
                        counter += 1;
                        println!("Received {} responses.", counter);
                        self.state = State0::Resolved;
                        break PollState::Ready(String::new());
                    }
                    PollState::NotReady => break PollState::NotReady,
                },
                State0::Resolved => panic!("Polled a resolved future"),
            }
        }
    }
}
