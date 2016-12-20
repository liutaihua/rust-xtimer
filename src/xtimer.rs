//extern crate time;
extern crate timer;

extern crate chrono;

use std::sync::{Arc, Mutex};

//use std::io::Timer as itime;
//use timer::timer::Timer as iTimer;
use std::time::Duration;
//use std::comm::Receiver;
use std::sync::mpsc::channel;

use std::time as std_time;

use std::marker::PhantomData;

use std::ops::Sub;


#[derive(Default)]
struct TimerData<'a> {
    Key: String,
    expire: std_time::Instant,
    index: usize,
    next: Option<Box<TimerData<'a>>>,
    _marker: PhantomData<&'a ()>,
    func: Option<&'a Fn(i32) -> i32>
}

impl<'a> TimerData<'a> {
    pub fn Delay(&self) -> std_time::Duration {
        self.expire.sub(std_time::Instant::now())
//        self.expire.sub
    }
}

#[derive(Default)]
pub struct Timer<'a> {
    lock: Arc<Mutex<i32>>,
    free: Option<Box<TimerData<'a>>>,
    timers: Vec<Box<TimerData<'a>>>,
    signal: timer::Timer,
    num: i32
}

impl<'a> Timer<'a> {
    pub fn new(num: i32) -> &'a mut Timer<'a> {
        let mut t = &Timer{num: num, ..Default::default()};
        t.init();
        &mut t
    }

    fn init(&mut self) {
        self.lock = Arc::new(Mutex::new(0));
//        let mut td = &Timer{..Default::default()};
        self.timers = Vec::new();
        self.grow();
    }

    fn grow(&mut self) {
//        let tds = vec![self.num:TimerData];
        let tds = Vec::with_capacity(self.num as usize);
        for i in 0..self.num {
            tds.push(TimerData{expire: std_time::Instant::now(), ..Default::default()})
        }

        self.free = Some(Box::new(tds[0]));
//        let td = self.free;
        let td: Box<TimerData>;
        for i in 1..self.num {
            td.next = Some(Box::new(tds[i as usize]));
            match td.next {
                Some(t) => {
                    td = t.next.unwrap();
                }
                _ => {
                    panic!("fuck");
                }
            }
        }
        td.next = None
    }

    fn get(&mut self) -> Box<TimerData<'a >> {
        let td: Box<TimerData>;
        match self.free {
            Some(t) => {
//                self.free = t.unwrap().next;
                td = self.free.unwrap();
            }
            _ => {
                self.grow();
                td = self.free.unwrap();
            }
        }
        self.free = td.next;
        td
    }

    fn put(&mut self, td: Box<TimerData<'a>>) {
        td.func = None;
        td.next = self.free;
        self.free = Some(td);
    }

    fn add(&mut self, td: Box<TimerData<'a>>) {
        let d: std_time::Duration;
        td.index = self.timers.len();
        self.timers.push(td);
        self.up(td.index as i32);
        if td.index == 0{
            d = td.Delay();
//            self.signal.reset(d);
            self.signal = timer::Timer::new();
        }
    }

    pub fn Add<'b>(&mut self, expire: std_time::Instant, func: &'a Fn(i32) -> i32) -> Box<TimerData<'a >> {
        let td = self.get();
        td.expire = expire;
        td.func = Some(func);
        self.add(td);
        td
    }

    fn del(&mut self, td: Box<TimerData>) {
        let i = td.index;
        let last = self.timers.len() - 1;
        if i <0 || i > last || Box::into_raw(self.timers[i]) != Box::into_raw(td) {
            return
        }
        if i != last {
            self.swap(i as i32, last as i32);
            self.down(i as i32, last as i32);
            self.up(i as i32);
        }
        self.timers[last].index = -1;
//        let ss = std::mem::size_of::<TimerData>();
//        let _tmp = self.timers.as_slice()[0..last];
//        self.timers = _tmp.into_vec();
        let v = Vec::new();
        for i in 0..last {
            v.push(self.timers[i]);
        }
        self.timers = v;
    }

    pub fn Set(&mut self, td: Box<TimerData<'a>>, expire: std_time::Duration) {
        self.del(td);
        td.expire = std_time::Instant::now() + expire;
        self.add(td);
    }

    fn swap(&mut self, i: i32, j:i32) {
        self.timers.swap(i as usize, j as usize);
    }

    fn up(&mut self, j: i32) {
        loop {
            let i = (j-1) / 2;
            if i == j || !self.less(j, i) {
                break
            }
            self.swap(i, j);
            j = i;
        }
    }

    fn down(&mut self, i: i32, n:i32) {
        let j1:i32;
        let j2:i32;
        let j:i32;
        loop {
            j1 = 2*i+1;
            if j1 >= n || j1 < 0 {
                break;
            }
            j = j1;
            j2 = j1+1;
            if j2 < n && !self.less(j1, j2) {
                j = j2;
            }
            if !self.less(j, i) {
                break;
            }
            self.swap(i, j);
            i = j;
        }
    }

    fn less(&self, i:i32, j:i32) -> bool {
        self.timers[i as usize].expire > self.timers[j as usize].expire
    }

    fn start(&mut self) {
        let (tx, rx) = channel();
        let _guard = self.signal.schedule_with_delay(chrono::Duration::seconds(1), move || {
            // This closure is executed on the scheduler thread,
            // so we want to move it away asap.
            self.expire();
            let _ignored = tx.send(()); // Avoid unwrapping here.
        });
    }

    fn expire(&mut self) {
        let func: Option<&Fn(i32) -> i32>;
        let td: Box<TimerData>;
        let d: std_time::Duration;
        let infiniteDuration = std_time::Duration::new((1<<63) as u64);
        loop {
            if self.timers.len() == 0 {
                d = infiniteDuration;
                break;
            }
            td = self.timers[0];
            d = td.Delay();
            if d.as_secs() > 0 {
                break;
            }
            func = td.func;
            self.del(td);
            match func {
                Some(f) => {
                    f(1);
                }
                _ => {
                    panic!("fuck22");
                }
            }
            self.signal = timer::Timer::new();
        }
    }
}


#[test]
fn test_simple() {
    let t = Timer::new(10);
}
