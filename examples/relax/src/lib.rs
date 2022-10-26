//! # A Concordium V1 smart contract
use concordium_std::*;
use core::fmt::Debug;

/// Your smart contract state.
#[derive(Serialize, SchemaType, Clone)]
pub struct State {
    text: String,
}

/// Your smart contract errors.
#[derive(Debug, PartialEq, Eq, Reject, Serial, SchemaType)]
enum Error {
    /// Failed parsing the parameter.
    #[from(ParseError)]
    ParseParamsError,
    InvokeFailed,
    LogFailed(u8),
}

impl From<LogError> for Error {
    fn from(e: LogError) -> Self { Error::LogFailed(e as u8) } // Workaround because LogError does not implement Serial.
}

/// Init function that creates a new smart contract.
#[init(contract = "relax")]
fn init<S: HasStateApi>(
    _ctx: &impl HasInitContext,
    _state_builder: &mut StateBuilder<S>,
) -> InitResult<State> {
    Ok(State {
        text: "Fresh".into(),
    })
}

#[receive(contract = "relax", name = "receive", parameter = "String", error = "Error", mutable)]
fn receive<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &mut impl HasHost<State, StateApiType = S>,
) -> Result<(), Error> {
    let param: String = ctx.parameter_cursor().get()?;

    host.state_mut().text = param;

    Ok(())
}

#[receive(
    contract = "relax",
    name = "receive-indirect",
    parameter = "String",
    error = "Error",
    mutable
)]
fn receive_indirect<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &mut impl HasHost<State, StateApiType = S>,
) -> Result<(), Error> {
    let param: String = ctx.parameter_cursor().get()?;

    if host
        .invoke_contract(
            &ctx.self_address(),
            &param,
            EntrypointName::new_unchecked("receive"),
            Amount::zero(),
        )
        .is_err()
    {
        return Err(Error::InvokeFailed);
    }
    Ok(())
}

#[receive(contract = "relax", name = "view", return_value = "State")]
fn view<'a, 'b, S: HasStateApi>(
    _ctx: &'a impl HasReceiveContext,
    host: &'b impl HasHost<State, StateApiType = S>,
) -> ReceiveResult<&'b State> {
    Ok(host.state())
}

#[receive(contract = "relax", name = "view-indirect", return_value = "State")]
fn view_indirect<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &impl HasHost<State, StateApiType = S>,
) -> Result<State, Error> {
    match host.invoke_contract_read_only(
        &ctx.self_address(),
        &(),
        EntrypointName::new_unchecked("view"),
        Amount::zero(),
    ) {
        Ok(res) => Ok(res.unwrap_abort().get()?),
        Err(_) => Err(Error::InvokeFailed),
    }
}

#[receive(
    contract = "relax",
    name = "return",
    parameter = "(u32, String)",
    return_value = "State",
    mutable
)]
fn set_return<'a, 'b, S: HasStateApi>(
    ctx: &'a impl HasReceiveContext,
    host: &'b mut impl HasHost<State, StateApiType = S>,
) -> Result<&'b State, Error> {
    let (n, contents): (u32, String) = ctx.parameter_cursor().get()?;
    host.state_mut().text = contents.repeat(n as usize);
    Ok(host.state())
}

#[receive(
    contract = "relax",
    name = "return-indirect",
    parameter = "(u32, String)",
    return_value = "State",
    mutable
)]
fn set_return_indirect<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &mut impl HasHost<State, StateApiType = S>,
) -> Result<State, Error> {
    let param: (u32, String) = ctx.parameter_cursor().get()?;
    match host.invoke_contract(
        &ctx.self_address(),
        &param,
        EntrypointName::new_unchecked("return"),
        Amount::zero(),
    ) {
        Ok((_, res)) => Ok(res.unwrap_abort().get()?),
        Err(_) => Err(Error::InvokeFailed),
    }
}

/// View function that returns the content of the state.
#[receive(contract = "relax", name = "log", parameter = "(u32, String)", enable_logger)]
fn log<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &impl HasHost<State, StateApiType = S>,
    logger: &mut impl HasLogger,
) -> Result<(), Error> {
    let (n, contents): (u32, String) = ctx.parameter_cursor().get()?;
    for _ in 0..n {
        logger.log_raw(contents.as_bytes())?;
    }
    // Perform invocation to reset log limit.
    let res = host.invoke_transfer(&ctx.owner(), Amount::zero());
    if res.is_err() {
        return Err(Error::InvokeFailed);
    };
    for _ in 0..n {
        logger.log_raw(contents.as_bytes())?;
    }
    Ok(())
}
