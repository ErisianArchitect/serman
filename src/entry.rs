
use std::marker::PhantomData;

use libc::{
    c_int,
};

use crate::ForkContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SignalAction {
    Ignore,
    Restart,
    Exit(u8),
    Filter(c_int),
}

impl SignalAction {
    pub fn ignore(_: c_int) -> SignalAction {
        SignalAction::Ignore
    }

    pub fn restart(_: c_int) -> SignalAction {
        SignalAction::Restart
    }

    pub fn restart_if<const SIGNAL: c_int>(signal: c_int) -> SignalAction {
        if signal == SIGNAL {
            SignalAction::Restart
        } else {
            SignalAction::Filter(signal)
        }
    }

    pub fn exit<const EXIT: u8>(_: c_int) -> SignalAction {
        SignalAction::Exit(EXIT)
    }

    pub fn exit_if<const SIGNAL: c_int, const EXIT_CODE: u8>(signal: c_int) -> SignalAction {
        if signal == SIGNAL {
            SignalAction::Exit(EXIT_CODE)
        } else {
            SignalAction::Filter(signal)
        }
    }

    pub fn exit_success(_: c_int) -> SignalAction {
        SignalAction::Exit(0)
    }

    pub fn exit_failure(_: c_int) -> SignalAction {
        SignalAction::Exit(1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExitAction {
    Restart,
    Exit(u8),
    Filter(u8),
}

impl ExitAction {
    pub fn restart(_: u8) -> ExitAction {
        ExitAction::Restart
    }

    pub fn exit<const EXIT: u8>(_: u8) -> ExitAction {
        ExitAction::Exit(EXIT)
    }

    pub fn exit_success(_: u8) -> ExitAction {
        ExitAction::Exit(0)
    }

    pub fn exit_failure(_: u8) -> ExitAction {
        ExitAction::Exit(1)
    }
}

pub trait ChildSignalHandler: Copy {
    #[inline(always)]
    fn handle(&self, signal: c_int) -> SignalAction {
        SignalAction::Filter(signal)
    }
}

pub trait ChildExitHandler: Copy {
    #[inline(always)]
    fn handle(&self, exit_code: u8) -> ExitAction {
        ExitAction::Filter(exit_code)
    }
}

impl ChildSignalHandler for () {}
impl ChildExitHandler for () {}

impl<F: Fn(c_int) -> SignalAction + Copy> ChildSignalHandler for F {
    #[inline(always)]
    fn handle(&self, signal: c_int) -> SignalAction {
        (self)(signal)
    }
}

impl<F: Fn(u8) -> ExitAction + Copy> ChildExitHandler for F {
    #[inline(always)]
    fn handle(&self, exit_code: u8) -> ExitAction {
        (self)(exit_code)
    }
}

pub trait SermanMain<R>: Copy {
    fn main(&self, ctx: ForkContext) -> R;
}

impl SermanMain<()> for () {
    fn main(&self, _: ForkContext) -> () {}
}

impl<R, F: Fn(ForkContext) -> R + Copy> SermanMain<R> for F{
    #[inline(always)]
    fn main(&self, ctx: ForkContext) -> R {
        (self)(ctx)
    }
}

pub trait DefaultValue {}
pub trait NonDefaultValue<T = ()> {}

impl DefaultValue for () {}

impl<I, O, F: Fn(I) -> O> NonDefaultValue<(I, O)> for F {}

pub struct Entry<
    R = (),
    Main: SermanMain<R> = (),
    ExitSignalHandler: ChildSignalHandler = (),
    RestartSignalHandler: ChildSignalHandler = (),
    ExitHandler: ChildExitHandler = (),
    RestartHandler: ChildExitHandler = (),
> {
    main: Main,
    exit_signal_handler: ExitSignalHandler,
    restart_signal_handler: RestartSignalHandler,
    exit_handler: ExitHandler,
    restart_handler: RestartHandler,
    _phantom: PhantomData<(R,)>,
}

impl Entry<(), (), (), (), (), ()> {
    #[must_use]
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            main: (),
            exit_signal_handler: (),
            restart_signal_handler: (),
            exit_handler: (),
            restart_handler: (),
            _phantom: PhantomData,
        }
    }
}

impl<
    ESH: ChildSignalHandler,
    RSH: ChildSignalHandler,
    EH: ChildExitHandler,
    RH: ChildExitHandler,
> Entry<(), (), ESH, RSH, EH, RH> {
    #[must_use]
    #[inline(always)]
    pub const fn main<R, F: Fn(ForkContext) -> R + Copy>(self, main: F) -> Entry<R, F, ESH, RSH, EH, RH> {
        Entry {
            main,
            exit_signal_handler: self.exit_signal_handler,
            restart_signal_handler: self.restart_signal_handler,
            exit_handler: self.exit_handler,
            restart_handler: self.restart_handler,
            _phantom: PhantomData,
        }
    }
}

// impl<
//     R: Copy,
//     F: SermanMain<R>,
//     ESH: ChildSignalHandler,
//     RSH: ChildSignalHandler,
//     EH: ChildExitHandler,
//     RH: ChildExitHandler,
// > EntryBuilder<R, F, ESH, RSH, EH, RH> {
    
// }

impl<
    R,
    Main: SermanMain<R>,
    ESH: ChildSignalHandler + DefaultValue,
    RSH: ChildSignalHandler,
    EH: ChildExitHandler,
    RH: ChildExitHandler,
> Entry<R, Main, ESH, RSH, EH, RH> {
    pub const fn exit_signal_handler<
        F: Fn(c_int) -> SignalAction + Copy
    >(self, handler: F) -> Entry<R, Main, F, RSH, EH, RH> {
        Entry {
            main: self.main,
            exit_signal_handler: handler,
            restart_signal_handler: self.restart_signal_handler,
            exit_handler: self.exit_handler,
            restart_handler: self.restart_handler,
            _phantom: PhantomData,
        }
    }
}

impl<
    R,
    Main: SermanMain<R>,
    ESH: ChildSignalHandler,
    RSH: ChildSignalHandler + DefaultValue,
    EH: ChildExitHandler,
    RH: ChildExitHandler,
> Entry<R, Main, ESH, RSH, EH, RH> {
    pub const fn restart_signal_handler<
        F: Fn(c_int) -> SignalAction + Copy
    >(self, handler: F) -> Entry<R, Main, ESH, F, EH, RH> {
        Entry {
            main: self.main,
            exit_signal_handler: self.exit_signal_handler,
            restart_signal_handler: handler,
            exit_handler: self.exit_handler,
            restart_handler: self.restart_handler,
            _phantom: PhantomData,
        }
    }
}

impl<
    R,
    Main: SermanMain<R>,
    ESH: ChildSignalHandler,
    RSH: ChildSignalHandler,
    EH: ChildExitHandler + DefaultValue,
    RH: ChildExitHandler,
> Entry<R, Main, ESH, RSH, EH, RH> {
    pub const fn exit_handler<
        F: Fn(u8) -> ExitAction + Copy
    >(self, handler: F) -> Entry<R, Main, ESH, RSH, F, RH> {
        Entry {
            main: self.main,
            exit_signal_handler: self.exit_signal_handler,
            restart_signal_handler: self.restart_signal_handler,
            exit_handler: handler,
            restart_handler: self.restart_handler,
            _phantom: PhantomData,
        }
    }
}

impl<
    R,
    Main: SermanMain<R>,
    ESH: ChildSignalHandler,
    RSH: ChildSignalHandler,
    EH: ChildExitHandler,
    RH: ChildExitHandler + DefaultValue,
> Entry<R, Main, ESH, RSH, EH, RH> {
    pub const fn restart_handler<
        F: Fn(u8) -> ExitAction + Copy
    >(self, handler: F) -> Entry<R, Main, ESH, RSH, EH, F> {
        Entry {
            main: self.main,
            exit_signal_handler: self.exit_signal_handler,
            restart_signal_handler: self.restart_signal_handler,
            exit_handler: self.exit_handler,
            restart_handler: handler,
            _phantom: PhantomData,
        }
    }
}

impl<
    R,
    Main: SermanMain<R> + NonDefaultValue<(ForkContext, R)>,
    ESH: ChildSignalHandler,
    RSH: ChildSignalHandler,
    EH: ChildExitHandler,
    RH: ChildExitHandler,
> Entry<R, Main, ESH, RSH, EH, RH> {
    pub fn run(self, ctx: ForkContext) -> () {
        // self.main.main(ctx)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sandbox() {
        let entry = Entry::new()
        .exit_handler(|exit_code| {
            if exit_code != 0 {
                ExitAction::Restart
            } else {
                ExitAction::Filter(0)
            }
        })
        .main(|ctx| {
            ctx.restart()
        });
        let result = entry.run(todo!());
    }
}
