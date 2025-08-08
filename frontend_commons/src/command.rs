use std::{cell::RefCell, marker::PhantomData, ops::DerefMut, rc::Rc};

pub trait Command<State> {
    fn apply(self, state: &mut State);
}

pub trait CommandTransformer<Cmd1, Cmd2> {
    fn transform(cmd: Cmd1) -> Cmd2;
}

pub struct CommandConsumer<State, CmdOuter, CmdInner>
where
    CmdInner: Command<State>,
{
    state: Rc<RefCell<Option<State>>>,
    transformer: Box<dyn Fn(CmdOuter) -> CmdInner>,
    cmd_outer: PhantomData<CmdOuter>,
    cmd_inner: PhantomData<CmdInner>,
}

pub struct CommandConsumerGenerator<State, Cmd>
where
    Cmd: Command<State>,
{
    state: Rc<RefCell<Option<State>>>,
    cmd: PhantomData<Cmd>,
}

impl<State, Cmd> CommandConsumerGenerator<State, Cmd>
where
    Cmd: Command<State>,
{
    pub fn new() -> Self {
        Self {
            state: Rc::new(RefCell::new(None)),
            cmd: PhantomData,
        }
    }

    pub fn activate(&mut self, state: State) {
        self.state.replace(Some(state));
    }

    pub fn make_custom_consumer<CmdOuter, F>(
        &mut self,
        transformer: F,
    ) -> CommandConsumer<State, CmdOuter, Cmd>
    where
        F: Fn(CmdOuter) -> Cmd + 'static,
    {
        CommandConsumer {
            state: self.state.clone(),
            transformer: Box::new(transformer),
            cmd_inner: PhantomData,
            cmd_outer: PhantomData,
        }
    }

    pub fn make_consumer<CmdOuter>(&mut self) -> CommandConsumer<State, CmdOuter, Cmd>
    where
        Cmd: From<CmdOuter>,
    {
        self.make_custom_consumer(move |cmd| Cmd::from(cmd))
    }
}

impl<State, Cmd, CmdInner> CommandConsumer<State, Cmd, CmdInner>
where
    CmdInner: Command<State>,
{
    pub fn consume(&mut self, cmd: Cmd) {
        let cmd = (self.transformer)(cmd);
        let mut state = self.state.borrow_mut();
        if let Some(state) = state.deref_mut() {
            cmd.apply(state);
        }
    }
}
