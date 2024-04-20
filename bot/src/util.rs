use std::{convert::Infallible, ops::ControlFlow, sync::Arc};

use dptree::{di::Injectable, from_fn_with_description, Handler, HandlerDescription};

pub struct PrettyChat<'a>(pub &'a teloxide::types::Chat);

impl<'a> std::fmt::Debug for PrettyChat<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_group() || self.0.is_supergroup() {
            write!(f, "GroupChat")?;
            self.0.title().map(|x| write!(f, " title: {x}"));
            self.0.description().map(|x| write!(f, " description: {x}"));
        } else if self.0.is_private() {
            write!(f, "PrivateChat")?;
            self.0.username().map(|x| write!(f, " username: @{x}"));
            self.0.first_name().map(|x| write!(f, " first_name: {x}"));
            self.0.last_name().map(|x| write!(f, " last_name: {x}"));
            self.0.bio().map(|x| write!(f, " bio: {x}"));
        } else if self.0.is_channel() {
            write!(f, "Channel")?;
            self.0.username().map(|x| write!(f, " username: @{x}"));
            self.0.title().map(|x| write!(f, " title: {x}"));
            self.0
                .description()
                .map(|x| write!(f, ", description: {x}"));
        }
        Ok(())
    }
}

pub fn wrap_endpoint<'a, F, Input, Output, FnArgs, Descr>(
    f: F,
) -> Handler<'a, Input, Result<Output, Infallible>, Descr>
where
    Input: Send + Sync + 'a,
    Output: Send + Sync + 'a,
    Descr: HandlerDescription,
    F: Injectable<Input, ControlFlow<Output>, FnArgs> + Send + Sync + 'a,
{
    let f = Arc::new(f);

    from_fn_with_description(Descr::endpoint(), move |event, _cont| {
        let f = Arc::clone(&f);
        async move {
            let f = f.inject(&event);
            let cf = f().await;
            drop(f);

            match cf {
                ControlFlow::Continue(_) => ControlFlow::Continue(event),
                ControlFlow::Break(out) => ControlFlow::Break(Ok(out)),
            }
        }
    })
}

#[macro_export]
macro_rules! ok_or_break {
    ($e: expr) => {
        match $e {
            Ok(r) => r,
            Err(_) => {
                return ControlFlow::Break(());
            }
        }
    };
}
