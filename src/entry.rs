use std::marker::PhantomData;
use std::process::ExitCode;

use libc::c_int;

use crate::ForkContext;
use crate::error::Result;
use crate::util;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SignalAction {
    Exit(u8),
    Filter(c_int),
    Ignore,
    Restart,
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
    /// An exit handler that can be used to automatically restart.
    pub fn restart(_: u8) -> ExitAction {
        ExitAction::Restart
    }

    /// An exit handler that can be used to force exit.
    pub fn exit<const EXIT: u8>(_: u8) -> ExitAction {
        ExitAction::Exit(EXIT)
    }

    /// An exit handler that can be used to force exit with a success code (`0`).
    pub fn exit_success(_: u8) -> ExitAction {
        ExitAction::Exit(0)
    }

    /// An exit handler that can be used to force exit with a failure code (`1`).
    pub fn exit_failure(_: u8) -> ExitAction {
        ExitAction::Exit(1)
    }
}

pub trait ChildSignalHandler {
    #[inline(always)]
    fn handle(&self, signal: c_int) -> SignalAction {
        SignalAction::Filter(signal)
    }
}

pub trait ChildExitHandler {
    #[inline(always)]
    fn handle(&self, exit_code: u8) -> ExitAction {
        ExitAction::Filter(exit_code)
    }
}

impl ChildSignalHandler for () {}
impl ChildExitHandler for () {}

impl<F: Fn(c_int) -> SignalAction> ChildSignalHandler for F {
    #[inline(always)]
    fn handle(&self, signal: c_int) -> SignalAction {
        (self)(signal)
    }
}

impl<F: Fn(u8) -> ExitAction> ChildExitHandler for F {
    #[inline(always)]
    fn handle(&self, exit_code: u8) -> ExitAction {
        (self)(exit_code)
    }
}

pub trait SermanMain<R> {
    fn main(self, ctx: ForkContext) -> R;
}

impl SermanMain<()> for () {
    fn main(self, _: ForkContext) -> () {}
}

impl<R, F: FnOnce(ForkContext) -> R> SermanMain<R> for F {
    #[inline(always)]
    fn main(self, ctx: ForkContext) -> R {
        (self)(ctx)
    }
}

pub trait DefaultValue {}
pub trait NonDefaultMain<R> {}

impl DefaultValue for () {}

impl<R, F: FnOnce(ForkContext) -> R> NonDefaultMain<fn(ForkContext) -> R> for F {}

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

impl<ESH: ChildSignalHandler, RSH: ChildSignalHandler, EH: ChildExitHandler, RH: ChildExitHandler>
    Entry<(), (), ESH, RSH, EH, RH>
{
    /// Register the main entry point. [Entry] cannot run without a `main`.
    #[must_use]
    #[inline(always)]
    pub fn main<R, F: FnOnce(ForkContext) -> R>(self, main: F) -> Entry<R, F, ESH, RSH, EH, RH> {
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

impl<
    R,
    Main: SermanMain<R>,
    ESH: ChildSignalHandler + DefaultValue,
    RSH: ChildSignalHandler,
    EH: ChildExitHandler,
    RH: ChildExitHandler,
> Entry<R, Main, ESH, RSH, EH, RH>
{
    /// The exit signal handler is called in the case of a signal interruption in the absence
    /// of a restart request.
    pub fn exit_signal_handler<F: Fn(c_int) -> SignalAction>(
        self,
        handler: F,
    ) -> Entry<R, Main, F, RSH, EH, RH> {
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
> Entry<R, Main, ESH, RSH, EH, RH>
{
    /// The restart signal handler is called in the case of a signal interruption in the presence
    /// of a restart request.
    pub fn restart_signal_handler<F: Fn(c_int) -> SignalAction>(
        self,
        handler: F,
    ) -> Entry<R, Main, ESH, F, EH, RH> {
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
> Entry<R, Main, ESH, RSH, EH, RH>
{
    /// The exit handler tells the supervisor process what to do in the event of an exit in the
    /// absence of a restart request.
    ///
    /// It takes the exit code from the child as input and returns an [ExitAction], which tells
    /// the supervisor what to do next.
    ///
    /// This handler is also called in the case of the restart handler returning [ExitAction::Exit],
    /// or when a signal handler returns [SignalAction::Exit].
    pub fn exit_handler<F: Fn(u8) -> ExitAction>(
        self,
        handler: F,
    ) -> Entry<R, Main, ESH, RSH, F, RH> {
        Entry {
            main: self.main,
            exit_signal_handler: self.exit_signal_handler,
            restart_signal_handler: self.restart_signal_handler,
            exit_handler: handler,
            restart_handler: self.restart_handler,
            _phantom: PhantomData,
        }
    }

    /// Attaches an exit handler that will restart when the `when` function returns [true].
    pub fn restart_when<F: Fn(u8) -> bool>(self, when: F) -> Entry<R, Main, ESH, RSH, impl Fn(u8) -> ExitAction, RH> {
        self.exit_handler(move |exit_code| {
            util::select_copy(
                ExitAction::Filter(exit_code),
                ExitAction::Restart,
                when(exit_code),
            )
        })
    }

    /// Attaches an exit handler that will restart when the child exits with a failure code (`exit_code != 0`).
    #[must_use]
    #[inline(always)]
    pub fn restart_on_failure(self) -> Entry<R, Main, ESH, RSH, impl Fn(u8) -> ExitAction, RH> {
        self.restart_when(|exit_code| exit_code != 0)
    }
}

impl<
    R,
    Main: SermanMain<R>,
    ESH: ChildSignalHandler,
    RSH: ChildSignalHandler,
    EH: ChildExitHandler,
    RH: ChildExitHandler + DefaultValue,
> Entry<R, Main, ESH, RSH, EH, RH>
{
    /// The restart handler tells the supervisor process what to do in the event of a restart
    /// in the absence of a signal interrupt.
    ///
    /// It takes the exit code from the child as input and returns an [ExitAction], which tells
    /// the supervisor what to do next.
    ///
    /// This handler is also called in the case of the exit handler returning [ExitAction::Restart].
    pub fn restart_handler<F: Fn(u8) -> ExitAction>(
        self,
        handler: F,
    ) -> Entry<R, Main, ESH, RSH, EH, F> {
        Entry {
            main: self.main,
            exit_signal_handler: self.exit_signal_handler,
            restart_signal_handler: self.restart_signal_handler,
            exit_handler: self.exit_handler,
            restart_handler: handler,
            _phantom: PhantomData,
        }
    }

    /// Attaches a restart handler that will redirect to an exit when the `when` function returns [true].
    pub fn exit_when<F: Fn(u8) -> bool>(self, when: F) -> Entry<R, Main, ESH, RSH, EH, impl Fn(u8) -> ExitAction> {
        self.restart_handler(move |exit_code| {
            util::select_copy(
                ExitAction::Filter(exit_code),
                ExitAction::Exit(exit_code),
                when(exit_code),
            )
        })
    }

    /// Attaches a restart handler that will perform a regular exit when the success code is `0`.
    pub fn exit_on_success(self) -> Entry<R, Main, ESH, RSH, EH, impl Fn(u8) -> ExitAction> {
        self.exit_when(|exit_code| exit_code == 0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ForkResult<P, C> {
    Parent(P),
    Child(C),
}

impl<
    R,
    Main: SermanMain<R> + NonDefaultMain<fn(ForkContext) -> R>,
    ESH: ChildSignalHandler,
    RSH: ChildSignalHandler,
    EH: ChildExitHandler,
    RH: ChildExitHandler,
> Entry<R, Main, ESH, RSH, EH, RH>
{
    //= entry.rs::run
    pub fn run(self) -> ForkResult<Result<ExitCode>, R> {
        // TODO: Finish this function.
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sandbox() {
        let entry = Entry::new()
            .restart_on_failure()
            .exit_on_success()
            .main(|ctx| ctx.restart());
        let result = entry.run();
    }
}
